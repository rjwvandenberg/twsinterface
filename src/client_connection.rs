use log::{error, info};

use crate::{structs::{EMessage, TwsSocket, Decimal, TwsError, IncomingMessageId}, constants};

#[derive(Debug)]
pub enum ClientConnectionState {
    Connected,
    Disconnected,
}

#[derive(Debug)]
pub struct ClientConnection {
    pub state: ClientConnectionState,
    socket: TwsSocket
}

impl ClientConnection {
    pub fn new(tws_address: String) -> ClientConnection {
        ClientConnection {
            state: ClientConnectionState::Connected,
            socket: TwsSocket::new(tws_address)
        }
    }

    pub fn next(&mut self) -> Option<String> {
        let mut msg = None;
        self.state = match &self.state {
            ClientConnectionState::Connected => {
                match self.read_message() {
                    Ok(s) => {
                        msg = Some(s);
                        ClientConnectionState::Connected
                    }
                    Err(s) => {
                        msg = Some(s);
                        ClientConnectionState::Disconnected
                    }
                }   
            }

            ClientConnectionState::Disconnected => {
                msg = Some(format!("Attempted to reconnect"));
                self.socket.reconnect();
                ClientConnectionState::Connected
            }

            _ => { panic!("Invalid state for next()"); }
        };

        msg
    }

    fn read_message(&mut self) -> Result<String, String> {
        match self.read_int() {
            Some(v) => {
                match IncomingMessageId::try_from(v) {
                    Ok(id) => self.decode(id),
                    Err(e) => Err(e),
                }
            },
            None => Err(String::from("Reverting, because expected message did not arrive."))
        }
    }

    fn decode(&mut self, id: IncomingMessageId) -> Result<String, String> {
        match id {
            IncomingMessageId::Error => {
                // https://interactivebrokers.github.io/tws-api/message_codes.html
                let version = self.read_int().unwrap();
                let msg = if version < 2  {
                    format!("{}", self.read_string().unwrap())
                } else { // needs additional work unescaping ascii, serverversion checks, 
                    // -1 is notification
                    format!("{} {} {} ", self.read_int().unwrap(), self.read_int().unwrap(), self.read_string().unwrap())
                };
                Ok(format!("{:?} {} {}", id, version, msg))
            }
            IncomingMessageId::NextValidId => {
                Ok(format!("{:?} {} {}", id, self.read_int().unwrap(), self.read_int().unwrap()))
            }
            IncomingMessageId::ManagedAccounts => {
                // comma seperated list of accounts. 
                // panic on more than one account
                Ok(format!("{:?} {} {}", id, self.read_int().unwrap(), self.read_string().unwrap()))
            }
            _ => panic!("{:?} message not implemented.", id)
        }
    }

    fn read_string(&mut self) -> Option<String> {
        match self.socket.next_string() {
            Ok(s) => Some(s),
            Err(e) => {
                error!("Encountered error '{:?}' in TwsSocket.", e);
                None
            },
        }
    }
    
    fn read_double(&mut self) -> Option<f64> {
        self.read_string().and_then(|s| Some(s.parse().unwrap()))
    }

    fn read_double_max(&mut self) -> Option<f64> {
        self.read_string().and_then(|s| match s.cmp(&constants::INFINITY.to_string()) {
            std::cmp::Ordering::Equal => Some(f64::MAX),
            _ => None
        })
    }

    fn read_decimal(&mut self) -> Option<Decimal> {
        self.read_string().and_then(|s| Some(Decimal{ number: s }) )
    }

    fn read_long(&mut self) -> Option<i64> {
        self.read_string().and_then(|s| Some(s.parse().unwrap()))
    }

    fn read_int(&mut self) -> Option<i32> {
        self.read_string().and_then(|s| Some(s.parse().unwrap()))
    }

    fn read_int_max(&mut self) -> Option<i32> {
        self.read_string().and_then(|s| {
            let max = s.parse().unwrap();
            if max != i32::MAX {
                panic!("IsNullOrEmpty pattern for reading max value, see EDecoder.cs")
            }
            Some(max)
        })
    }

    fn read_bool_from_int(&mut self) -> Option<bool> {
        self.read_int().and_then(|i| Some(i != 0))
    }

    fn read_char(&mut self) -> Option<char> {
        self.read_string().and_then(|s| {
            if s.len() > 1 {
                panic!("Expected char, got '{s}'");
            }
            Some(s.chars().next().unwrap())
        })
    } 
}