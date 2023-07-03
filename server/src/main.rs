use std::{env, io::Error};

use futures_util::{future, StreamExt, TryStreamExt};
use log::info;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = env_logger::try_init();
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

use common::Payload;

async fn accept_connection(stream: TcpStream) {
    let addr = stream.peer_addr().expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let (write, read) = ws_stream.split();
    // We should not forward messages other than text or binary.
    let msgs = read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()));
    let msgs = msgs.map(|msg| {
        let msg = msg.expect("Invalid message");
        let data = msg.into_data();
        let str : String = String::from_utf8(data).expect("Illegal message, invalid UTF-8");
        let payload = Payload { addr: addr.to_string(), message: str.clone()};
        let serialized = serde_json::to_string(&payload).unwrap();
        info!("Got message {}", str);
        info!("Sending back {}", serialized);
        let answer = Message::text(serialized);
        Ok(answer) });
    msgs.forward(write)
        .await
        .expect("Failed to forward messages")
}
