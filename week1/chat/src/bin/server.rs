use crate::mpsc::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use futures::stream::SplitStream;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::protocol::Message};
use::chrono::{DateTime, Utc};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::{Bytes, Uuid};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Db = Arc<Mutex<HashMap<u128, Vec<String>>>>;
pub type Responder<T> = oneshot::Sender<Result<T, Error>>;

#[derive(Debug)]
enum DbCommand {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: String,
        resp: Responder<()>,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MessageType {
    Login,
    ChangeChannel,
    ExitChannel,
    Message,
    ExitApp,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct UserMessage {
    time: DateTime<Utc>,
    text: String,
    from: u128,
    username: String,
    type_msg: MessageType,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:3030".to_string());
    let addr: SocketAddr = addr.parse().expect("Invalid address");

    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    let (sender, mut receiver) = mpsc::channel::<DbCommand>(64);

    tokio::spawn(async move {
        database(receiver).await;
    });

    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        // let db = Arc::clone(&db);
        // tokio::spawn(handle_connection(stream, db));
        let sender = sender.clone();
        tokio::spawn(async move {
            connected_user(stream, sender).await;
        });
    }
}

async fn database(mut receiver: Receiver<DbCommand>) {}


async fn get_user_id(mut receiver: SplitStream<WebSocketStream<TcpStream>>) -> u128 {
    let Some(message_received) = receiver.next().await else { todo!() };
    match message_received {
        Ok(Message::Text(text)) => {
            let message: UserMessage = serde_json::from_str(&text).unwrap();
            if message.type_msg != MessageType::Login{
                panic!("Not implemented yet");
            }
            let user_id: u128 = message.from;
            return user_id;
        }
        Ok(_) => panic!(),
        Err(_) => panic!(),
    }

}

async fn check_user_exists(user_id: u128) -> bool {
    // TODO implement proper check

    if user_id == 40093918_u128 {
        return true;
    }
    false
}

async fn user_select_room() -> u32 {
    // TODO implement this
    return 0;
} 

async fn connected_user(stream: TcpStream, sender: Sender<DbCommand>) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Error during the websocket handshake: {}", e);
            return;
        }
    };
    let (mut sender, mut receiver) = ws_stream.split();

    let user_id: u128 = get_user_id(receiver).await;

    let user_exists = check_user_exists(user_id).await;

    if !user_exists {
        panic!();
    } 

    let room_number = user_select_room().await;

    // load room, kinda like a state machine? 
    // when joined, start a loop where user send msgs and gets updates




    // while let Some(msg) = receiver.next().await {
    //     match msg {
    //         Ok(Message::Text(text)) => {
    //             let reversed = text.chars().rev().collect::<String>();
    //             info!("Recieved from user: {}, message: {}", id, text);

    //             let message = format!("Recieved from user: {}, message: {}", id, text);

    //             {
    //                 let mut db_temp = db.lock().unwrap();
    //                 db_temp
    //                     .entry(id)
    //                     .or_insert(vec!["Connected".to_string()])
    //                     .push(message);
    //             }
    //             if let Err(e) = sender.send(Message::Text(reversed.clone().into())).await {
    //                 error!("Error sending message: {}", e);
    //             }

    //             let message = format!("Sent to user: {}, message: {}", id, reversed);
    //             {
    //                 let mut db = db.lock().unwrap();
    //                 db.entry(id)
    //                     .or_insert(vec!["Connected".to_string()])
    //                     .push(message);
    //             }
    //         }
    //         Ok(Message::Close(text)) => {
    //             info!("Connection closed: {:?}", text);
    //             {
    //                 let db_temp = db.lock().unwrap();
    //                 for (key, val) in db_temp.clone().into_iter() {
    //                     info!("{} / {:?}", key, val);
    //                 }
    //             }
    //             break;
    //         }
    //         Ok(_) => (),
    //         Err(e) => {
    //             error!("Error processing message: {}", e);
    //             break;
    //         }
    //     }
    // }
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
                {
                    let db_temp = db.lock().unwrap();
                    for (key, val) in db_temp.clone().into_iter() {
                        info!("{} / {:?}", key, val);
                    }
                }
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
