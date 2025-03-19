
use chrono::Utc;
use futures::SinkExt;
use futures::StreamExt;
use server::MessageType;
use server::UserMessage;
// use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest, Message}};
// use uuid::{Uuid};
mod server;



#[tokio::main]
async fn main() {
    let my_id = 40093918_u128;

    let addr = "ws://127.0.0.1:3030".to_string();
    let request = addr.into_client_request().unwrap();

    let (ws_stream, response ) = connect_async(request).await.unwrap();

    
    println!("Connected to the server");
    println!("Response: {:?}", response);
    
    let (mut sender, mut receiver) = ws_stream.split();

    let usr_msg = UserMessage::new(
        Utc::now(),
        "Login".to_string(),
        my_id,
        "Tim".to_string(),
        MessageType::Login,
    );

    sender.send(Message::Text(serde_json::to_string(&usr_msg).unwrap().into())).await.unwrap();

    // let msg = receiver.next().await.unwrap();
    // println!("{:?}", msg);

    let usr_msg = UserMessage::new(
        Utc::now(),
        "New Message".to_string(),
        my_id,
        "Tim".to_string(),
        MessageType::Message,
    );




    sender.send(Message::Text(serde_json::to_string(&usr_msg).unwrap().into())).await.unwrap();

    sender.send(Message::Close(None)).await.unwrap();

    println!("Message sent!");
}
