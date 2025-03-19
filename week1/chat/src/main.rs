use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot};

use log::{info, error};
use types::{ClientReply, RouterMessage};
mod router;
mod types;
mod session_handler;
mod connection_manager;
mod room_manager;
mod user_manager;


const ADDRESS: &str = "127.0.0.1:3030";

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr: SocketAddr = ADDRESS.to_string().parse().expect("Invalid address");
    
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    info!("Server listening on {}", addr);

    let (router_tx, mut router_rx) = mpsc::channel::<RouterMessage>(100);;

    let (user_tx, mut user_rx) = oneshot::channel::<ClientReply>();

    tokio::spawn(router::start(router_rx, user_tx));

    while let Ok((stream, addr)) = listener.accept().await {
        let router_tx_clone = router_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = connection_manager::handle_connection(stream, addr, router_tx_clone, user_rx) {
                error!("Error handling a connection from {}: {:?}", addr, e);
            }
        });
    }
}
