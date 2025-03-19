use std::net::SocketAddr;
use tokio::net::TcpStream;

use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::Receiver;

use crate::types::{ClientReply, RouterMessage};

pub(crate) fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    router_tx_clone: Sender<RouterMessage>,
    user_rx: Receiver<ClientReply>,
) -> Result<(), std::io::Error> {
    todo!();

    Ok(())
}
