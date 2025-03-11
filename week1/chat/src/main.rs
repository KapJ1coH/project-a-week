use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use futures::{StreamExt, SinkExt};
use std::env;
use std::net::SocketAddr;
use log::{info, error};


#[tokio::main]
async fn main() {
    env_logger::init();

    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:3030".to_string());
    let addr: SocketAddr = addr.parse().expect("Invalid address");

    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}


async fn handle_connection(stream: TcpStream) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Error during the websocket handshake: {}", e);
            return;
        }
    };

    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let reversed = text.chars().rev().collect::<String>();
                if let Err(e) = sender.send(Message::Text(reversed.into())).await {
                    error!("Error sending message: {}", e);
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => (),
            Err(e) => {
                error!("Error processing message: {}", e);
                break;
            }
        }
    }
}
