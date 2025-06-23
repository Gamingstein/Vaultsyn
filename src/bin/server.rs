use futures::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx =
    futures::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, Message>;
type Clients = Arc<Mutex<Vec<Tx>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));
    println!("🧪 Vaultsyn WebSocket Server running at ws://localhost:9001");

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().unwrap();
        let ws_stream = accept_async(stream).await.unwrap();
        let (tx, mut rx) = ws_stream.split();

        let client_list = clients.clone();
        client_list.lock().unwrap().push(tx);

        let broadcast_clients = clients.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = rx.next().await {
                if let Message::Text(txt) = msg {
                    // Remove all sinks from the clients list, send, then put them back
                    let mut sinks = {
                        let mut clients = broadcast_clients.lock().unwrap();
                        std::mem::take(&mut *clients)
                    };
                    for client in sinks.iter_mut() {
                        let _ = client.send(Message::Text(txt.clone())).await;
                    }
                    // Put the sinks back into the clients list
                    let mut clients = broadcast_clients.lock().unwrap();
                    clients.extend(sinks);
                }
            }
        });

        println!("🔗 New connection from {}", peer);
    }
}
