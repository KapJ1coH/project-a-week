
use futures::SinkExt;
use futures::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest, Message}};




#[tokio::main]
async fn main() {
    let addr = "ws://127.0.0.1:3030".to_string();
    let request = addr.into_client_request().unwrap();

    let (ws_stream, response ) = connect_async(request).await.unwrap();

    
    println!("Connected to the server");
    println!("Response: {:?}", response);
    
    let (mut sender, mut receiver) = ws_stream.split();

    sender.send(Message::Text("Test".into())).await.unwrap();

    let msg = receiver.next().await.unwrap();
    println!("{:?}", msg);

    sender.send(Message::Close(None)).await.unwrap();

    println!("Message sent!");
}
