use std::{io::prelude::*, collections::VecDeque};
use std::net::TcpStream;

use log::{info, error};

use crate::{constants, structs::EMessage};

#[derive(Debug)]
pub enum TwsError {
    Disconnected,
    EmptyBuffer,
    SocketFault
}

#[derive(Debug)]
pub struct TwsSocket {
    address: String,
    socket: TcpStream,
    client_version: u32,
    receive_buffer: VecDeque<String>,
    server_version: i32,
    server_time: String,
}

impl TwsSocket {
    pub fn new(address: String) -> TwsSocket {
        info!("Connecting to {}", address);
        let tcpsocket = TwsSocket::create_socket(&address);
        
        let mut twssocket = TwsSocket {
            address,
            socket: tcpsocket,
            client_version: constants::CLIENT_VERSION,
            receive_buffer: VecDeque::new(),
            server_version: 0,
            server_time: String::new(),
        };

        twssocket.connect();
        twssocket
    }

    fn create_socket(address: &str) -> TcpStream {
        match TcpStream::connect(&address) {
            Err(e) => {
                error!("Failed to connect: {e}");
                panic!("Stopping program due to not establishing connection");
            }
            Ok(socket) => {
                // socket.set_read_timeout(Some(Duration::new(5, 0))).unwrap();
                socket
            }
        }
    }

    fn connect(&mut self) {
        info!("Initiating TWS Message Exchange");
        let mut msg = EMessage::new();
        msg.push(constants::CLIENT_VERSION.to_string());
        self.send(msg);
        info!("Client version: {}", self.client_version);

        self.server_version = match self.next_string() {
            Ok(s) => s.parse().unwrap(),
            _ => panic!("Unhandled case: read error during handshake")
        };
        if self.server_version != 76 {
            panic!("Redirect or New Server Version {}", self.server_version)
        }
        info!("Server version: {}", self.server_version);

        self.server_time = match self.next_string() {
            Ok(s) => s,
            _ => panic!("Unhandled case: read error during handshake")
        };
        info!("Server time: {}", self.server_time);
        info!("Local time: {:?}", std::time::SystemTime::now());
        info!("Connected to TWS");

        info!("Starting api");
        let mut msg = EMessage::new();
        msg.push(71.to_string()); // StartAPI outgoing message id 
        msg.push(2.to_string()); // VERSION 
        msg.push(1337.to_string()); // client id
        if self.server_version > 72 {       // MinServerVer.OPTIONAL_CAPABILITIES
            msg.push("".to_string());   // optional capabilities, since 76>72
        }
        self.send(msg);

        // Before making requests The socket Needs to receive:
        //   ManagedAccounts
        //   NextValidId
        //   Error(-1) Notifications of:
        //      2104    Market data farm connection is OK
        //      2106    HMDS data farm connection is OK
        //
        // And invalidate making requests when it's not OK?
        // See https://interactivebrokers.github.io/tws-api/message_codes.html
        // for all error/warning/system/etc codes

        
        
    }

    pub fn reconnect(&mut self) {
        self.socket.shutdown(std::net::Shutdown::Both).unwrap();
        info!("Reconnecting to {}", self.address);
        self.socket = TwsSocket::create_socket(&self.address);
        self.receive_buffer = VecDeque::new();
        self.connect();
    }

    fn send(&mut self, message: EMessage) {
        match self.socket.write(&message.consume()) {
            Err(e) => panic!("Failed to send message {}", e),
            _ => {}
        }
    }

    fn receive(&mut self) -> Result<(), TwsError>{
        if self.receive_buffer.is_empty() {
            let mut buffer: Vec<u8> = vec![0; constants::MAX_MSG_SIZE];
            match self.socket.read(&mut buffer) {
                Ok(0) => Err(TwsError::Disconnected),
                Ok(size) => {
                    if buffer[size-1] != 0 {
                        panic!("Expect to finish message(s) with 0")
                    }
                    self.receive_buffer = buffer[0..size]
                        .split(|c| *c==0)
                        .map(|str| {
                            match String::from_utf8(str.to_vec()) {
                                Ok(str) => str,
                                Err(e) => {
                                    error!("Failed to convert message to UTF8 string. {e}");
                                    panic!("This should never happen");
                                }
                            }
                        })
                        .filter(|str| str.len() > 0)
                        .collect();
                    if self.receive_buffer.is_empty() {
                        Err(TwsError::EmptyBuffer)
                    } else {
                        Ok(())
                    }
                },
                Err(_) => Err(TwsError::SocketFault)
            }
        } else {
            Ok(())
        }
    }

    pub fn next_string(&mut self) -> Result<String, TwsError> {
        match self.receive() {
            Err(e) => {
                Err(e)
            }
            _ => {
                match self.receive_buffer.pop_front() {
                    Some(str) => Ok(str),
                    None => panic!("Should not have NONE elements in buffer after receiving messages")
                }
            }
        }        
    }
}