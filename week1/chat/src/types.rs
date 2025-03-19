use std::collections::BTreeMap;

use serde::{Serialize, Deserialize};
use ::chrono::{DateTime, Utc};

pub type Time = DateTime<Utc>;

pub type Messages = BTreeMap<Time, UserTextMessage>;



#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    Login { user_id: u128 },
    JoinRoom { room_id: u128 },
    SendMessage { room_id: u128, message: UserTextMessage},
    LeaveRoom,
}


#[derive(Serialize, Deserialize)]
pub enum RouterMessage {
    Login { user_id: u128, session_id: u128 },
    JoinRoom { room_id: u128, session_id: u128 },
    SendMessage { room_id: u128, message: String, session_id: u128},
    LeaveRoom,
}


#[derive(Serialize, Deserialize)]
pub enum ClientReply {
    Err { message: Option<String>, user_id: u128 },
    Messages { messages: Messages, user_id: u128 },
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserTextMessage {
    time: Time,
    text: String,
    from: u128,
    username: String,
}

impl UserTextMessage {
    pub fn new(
        time: Time,
        text: String,
        from: u128,
        username: String,
    ) -> UserTextMessage {
        UserTextMessage {
            time,
            text,
            from,
            username,
        }
    }
}
