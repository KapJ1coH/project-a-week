#![allow(warnings)]
use ::chrono::{DateTime, Utc};
use futures::stream::{ SplitStream, SplitSink };
use futures::{SinkExt, StreamExt};
use log::{error, info, log};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message, WebSocketStream};

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use uuid::{Bytes, Uuid};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Db = Arc<Mutex<HashMap<u128, Vec<String>>>>;

pub type ArcWithHashofVector<T> = Arc<Mutex<HashMap<u128, Vec<T>>>>;
pub type ArcWithHashof<T> = Arc<Mutex<HashMap<u128, T>>>;
pub type Time = DateTime<Utc>;

pub type UserMessagesType = Arc<Mutex<BTreeMap<Time, UserMessage>>>;
pub type OptionMessages = Option<BTreeMap<Time, UserMessage>>;

pub type Responder<T> = oneshot::Sender<Result<T, Error>>;

// pub type DbObjResponse = Responder<Option<Bytes>>;
pub type DbObjResponse<T> = tokio::sync::oneshot::Sender<T>;

#[derive(Debug)]
enum DatabaseMessage {
    FindUser {
        id: u128,
        resp: DbObjResponse<User>,
    },
    AddUser {
        user: User,
    },
    GetChatRoomInfo {
        id: usize,
        resp: DbObjResponse<DatabaseMessage>,
    },
    GetAllMsgRoom {
        id: usize,
        resp: DbObjResponse<OptionMessages>,
    },
    GetMsgAfter {
        id: usize,
        time: Time,
        resp: DbObjResponse<OptionMessages>,
    },
    AddMsg {
        msg: UserMessage,
    },
    ResponseText {
        text: String,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum MessageType {
    Login,
    ExitChannel,
    Message,
    ExitApp,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserMessage {
    time: Time,
    text: String,
    from: u128,
    username: String,
    type_msg: MessageType,
}

impl UserMessage {
    pub fn new(
        time: Time,
        text: String,
        from: u128,
        username: String,
        type_msg: MessageType,
    ) -> UserMessage {
        UserMessage {
            time,
            text,
            from,
            username,
            type_msg,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Room {
    name: String,
    number: u32,
    capacity: u32,
    messages: UserMessagesType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    name: String,
    id: u128,
    username: String,
}

#[derive(Debug)]
pub struct Database {
    chat_rooms: Vec<Room>,
    users: ArcWithHashof<User>,
}

impl Database {
    pub fn new() -> Self {
        let user_messages: UserMessagesType = Arc::new(Mutex::new(BTreeMap::new()));
        let now = Utc::now();
        let message = UserMessage {
            time: now,
            text: "Hello, world!".to_string(),
            from: 123456,
            username: "Alice".to_string(),
            type_msg: MessageType::Message,
        };

        {
            user_messages.lock().unwrap().insert(now, message);
        }

        let chat_rooms = vec![Room {
            name: "Room1".to_string(),
            number: 1,
            capacity: 10,
            messages: user_messages,
        }];


        let users = Arc::new(Mutex::new(HashMap::new()));

        let me_user = User {
            name: "Tim".to_string(),
            id: 40093918_u128,
            username: "KapJ1coH".to_string(),
        };

        {
            users.lock().unwrap().insert(40093918_u128, me_user);
        }

        Database { chat_rooms, users }
    }

    async fn manager(self: &mut Database, mut receiver: Receiver<DatabaseMessage>) {
        while let Some(cmd) = receiver.recv().await {
            match cmd {
                DatabaseMessage::GetChatRoomInfo { id, resp } => {
                    // todo make this better
                    let response = format!(
                        "name: {}, number: {}, capacity: {}",
                        self.chat_rooms[id].name,
                        self.chat_rooms[id].number,
                        self.chat_rooms[id].capacity
                    );
                    if resp
                        .send(DatabaseMessage::ResponseText { text: response })
                        .is_err()
                    {
                        println!("The receiver dropped");
                    }
                }
                DatabaseMessage::FindUser { id, resp } => {
                    let users = &self.users.lock().unwrap();
                    if let Some(user) = users.get(&id) {
                        if resp.send(user.clone()).is_err() {
                            println!("The receiver dropped");
                        }
                    }
                }
                DatabaseMessage::GetAllMsgRoom { id, resp } => {
                    let messages = self.chat_rooms[id].messages.lock().unwrap();

                    if resp.send(Some(messages.clone())).is_err() {
                        println!("The receiver dropped");
                    }

                }
                DatabaseMessage::GetMsgAfter { id, time, resp } => {
                    let messages = self.chat_rooms[id].messages.lock().unwrap();
                    
                    let messages_after = messages.clone().split_off(&time);

                    if resp.send(Some(messages_after)).is_err() {
                        println!("The receiver dropped");
                    }
                }

                _ => (),
            }
        }
    }
}

impl Default for Database {
    fn default() -> Self {
        Database::new()
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:3030".to_string());
    let addr: SocketAddr = addr.parse().expect("Invalid address");

    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    let (sender, mut receiver) = mpsc::channel::<DatabaseMessage>(64);

    let mut database = Database::new();
    tokio::spawn(async move {
        database.manager(receiver).await;
    });

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

async fn get_user_id(mut receiver: &mut SplitStream<WebSocketStream<TcpStream>>) -> u128 {
    let Some(message_received) = receiver.next().await else {
        todo!()
    };
    match message_received {
        Ok(Message::Text(text)) => {
            let message: UserMessage = serde_json::from_str(&text).unwrap();
            if message.type_msg != MessageType::Login {
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

async fn connected_user(stream: TcpStream, database_messenger: Sender<DatabaseMessage>) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Error during the websocket handshake: {}", e);
            return;
        }
    };
    let (mut sender, mut receiver) = ws_stream.split();

    let user_id: u128 = get_user_id(&mut receiver).await;

    let user_exists = check_user_exists(user_id).await;

    if !user_exists {
        panic!();
    }

    let room_number = user_select_room().await;

    let (room_info_tx, room_info_rx) = oneshot::channel::<DatabaseMessage>();
    database_messenger
        .send(DatabaseMessage::GetChatRoomInfo {
            id: room_number as usize,
            resp: room_info_tx,
        })
        .await
        .unwrap();

    let room_info = room_info_rx.await;
    println!("GOT = {:?}", room_info);

    let (get_user_tx, get_user_rx) = oneshot::channel::<User>();
    database_messenger
        .send(DatabaseMessage::FindUser {
            id: 40093918_u128,
            resp: get_user_tx,
        })
        .await
        .unwrap();

    let user: User = get_user_rx.await.unwrap();

    println!("GOT = {:?}", user);


    enter_room(room_number as usize, database_messenger, sender, receiver).await;
}

async fn enter_room(
    room_number: usize,
    database_messenger: Sender<DatabaseMessage>,
    mut sender: SplitSink<WebSocketStream<TcpStream>, Message>,
    mut receiver: SplitStream<WebSocketStream<TcpStream>>,
) {
    println!("Entered the room {}", room_number);

    let (get_all_messages_tx, get_all_messages_rx) = oneshot::channel::<OptionMessages>();
    database_messenger
        .send(DatabaseMessage::GetAllMsgRoom {
            id: room_number,
            resp: get_all_messages_tx,
        })
        .await
        .unwrap();

    let messages = get_all_messages_rx.await.unwrap().unwrap();

    println!("GOT all msgs = {:?}", messages);

    let (last_message_time_received, _) = messages.last_key_value().unwrap();
    println!("GOT last message = {:?}", last_message_time_received);


    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let message: UserMessage = serde_json::from_str(&text).unwrap();
                let message_type = &message.type_msg;

                // #[derive(Serialize, Deserialize, Debug)]
                // pub struct UserMessage {
                //     time: Time,
                //     text: String,
                //     from: u128,
                //     username: String,
                //     type_msg: MessageType,
                // }

                match message_type {
                    MessageType::Message => {
                        println!(
                            "New message recieved from: {} on channel: {}",
                            message.from, room_number
                        );
                        database_messenger
                            .send(DatabaseMessage::AddMsg { msg: message })
                            .await
                            .unwrap();
                    }
                    MessageType::ExitChannel => {
                        todo!();
                    }
                    MessageType::ExitApp => {
                        todo!();
                    }
                    MessageType::Login => {
                        panic!("Wrong command in this context");
                    }
                };
            }
            Ok(_) => panic!(),
            Err(_) => panic!(),
        };
    }

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
