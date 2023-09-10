use env_logger;
use log::info;

mod constants;
mod client_connection;
use client_connection::ClientConnection;
mod structs;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("TWS Interface v.{}", constants::TWS_INTERFACE_VERSION);

    let address = format!("{}:{}", constants::IP_ADDRESS, constants::PORT);
    let mut client = ClientConnection::new(address);
    
    loop {
        // info!("state: {:?}", client.state);
        match client.next() {
            Some(message) => { info!("{:?}", message) }
            None => { info!("exited successfully"); return }
        }
    }
}