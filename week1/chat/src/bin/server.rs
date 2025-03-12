use futures::{SinkExt, StreamExt};
use log::{error, info};
use std::env;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

type Db = Arc<Mutex<HashMap<u128, Vec<String>>>>;

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:3030".to_string());
    let addr: SocketAddr = addr.parse().expect("Invalid address");

    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let db = Arc::clone(&db);
        tokio::spawn(handle_connection(stream, db));
    }

    {
        let db_temp = db.lock().unwrap();
        for (key, val) in db_temp.clone().into_iter() {
            print!("{} / {:?}", key, val);
        }
    }
}

async fn handle_connection(stream: TcpStream, db: Db) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Error during the websocket handshake: {}", e);
            return;
        }
    };

    let id = Uuid::new_v4().to_u128_le();

    {
        let mut db_temp = db.lock().unwrap();
        db_temp
            .entry(id)
            .or_insert(vec!["Connected".to_string()])
            .push("Connected again".to_string());
    }

    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let reversed = text.chars().rev().collect::<String>();
                info!("Recieved from user: {}, message: {}", id, text);

                let message = format!("Recieved from user: {}, message: {}", id, text);

                {
                    let mut db_temp = db.lock().unwrap();
                    db_temp
                        .entry(id)
                        .or_insert(vec!["Connected".to_string()])
                        .push(message);
                }
                if let Err(e) = sender.send(Message::Text(reversed.clone().into())).await {
                    error!("Error sending message: {}", e);
                }

                let message = format!("Sent to user: {}, message: {}", id, reversed);
                {
                    let mut db = db.lock().unwrap();
                    db.entry(id)
                        .or_insert(vec!["Connected".to_string()])
                        .push(message);
                }
            }
            Ok(Message::Close(text)) => {
                info!("Connection closed: {:?}", text);
                break;
            }
            Ok(_) => (),
            Err(e) => {
                error!("Error processing message: {}", e);
                break;
            }
        }
    }
}
