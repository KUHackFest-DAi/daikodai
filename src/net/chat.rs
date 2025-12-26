use crate::{
    server::handler::Server,
    types::{
        blockchain::{PeerAddr, TransactionMessage},
        chat::{ChatError, ChatResponse},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{tcp::OwnedWriteHalf, TcpListener, TcpStream},
    sync::{broadcast, Mutex},
};

#[derive(Clone, Debug)]
pub struct Client {
    pub nickname: String,
    pub writer: Arc<Mutex<OwnedWriteHalf>>,
}

impl Client {
    pub fn new(nickname: String, writer: Arc<Mutex<OwnedWriteHalf>>) -> Client {
        let last_seen = SystemTime::now();
        Self { nickname, writer }
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionPool {
    pub clients: Arc<Mutex<Vec<Client>>>,
}

impl ConnectionPool {
    pub fn init() -> ConnectionPool {
        ConnectionPool {
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn is_empty(&self) -> bool {
        println!("{:?}", self.clients.lock().await.len());
        true
    }

    pub async fn add_peer(&self, nickname: String, writer: Arc<Mutex<OwnedWriteHalf>>) -> Client {
        let peer = Client::new(nickname, writer);

        self.clients.lock().await.push(peer.clone());

        self.send_welcome_message(&peer).await.unwrap();
        self.broadcast_join(&peer).await;

        peer
    }

    pub async fn remove_peer(&self, writer: Arc<Mutex<OwnedWriteHalf>>) -> ConnectionPool {
        let mut clients = self.clients.lock().await;

        if let Some(position) = clients
            .iter()
            .position(|peer| Arc::ptr_eq(&peer.writer, &writer))
        {
            clients.remove(position);
            log::info!("Client has been removed from the pool!");
        } else {
            log::warn!("Client not found in connection pool");
        }

        self.clone()
    }

    pub async fn send_welcome_message(&self, client: &Client) -> Result<ChatResponse, ChatError> {
        let count = {
            let client = self.clients.lock().await;
            println!("Clients: {:?}", client);
            client.len() - 1
        };

        let mut writer = client.writer.lock().await;
        let message = format!("===\nWelcome {}!\n\nThere are {} user(s) besides you\n\nHelp:\nType anything to chat\n- /list will list all the connected users\n- /quit will disconnect you\n===", client.nickname, count);

        match writer.write_all(message.as_bytes()).await {
            Ok(_) => Ok(ChatResponse::WelcomeMessage(format!(
                "Successfully sent welcome message to {:?}",
                client.nickname
            ))),
            Err(e) => Err(ChatError::WelcomeMessage(format!(
                "Error while sending welcome message to {:?}: {:?}",
                client.nickname, e
            ))),
        }
    }

    pub async fn broadcast(&self, writer: Arc<Mutex<OwnedWriteHalf>>, message: String) {
        let clients = self.clients.lock().await;
        for client in clients.iter() {
            if Arc::ptr_eq(&client.writer, &writer) {
                continue;
            }

            let mut writer = client.writer.lock().await;
            match writer.write_all(message.as_bytes()).await {
                Ok(_) => {
                    log::info!("Broadcast successfully done!\n");
                }
                Err(e) => {
                    log::error!("Broadcast error: {:?}\n", e);
                }
            }
        }
    }

    pub async fn broadcast_join(&self, client: &Client) {
        self.broadcast(
            client.clone().writer,
            format!("{} has just joined!\n", client.nickname),
        )
        .await;
    }

    pub async fn broadcast_quit(&self, client: &Client) {
        self.broadcast(
            client.clone().writer,
            format!("{} has just quit!\n", client.nickname),
        )
        .await;
    }

    pub async fn broadcast_new_message(&self, client: &Client, message: String) {
        self.broadcast(
            client.clone().writer,
            format!("\n[{}]: {}\n", client.nickname, message),
        )
        .await;
    }

    pub async fn list_clients(&self, writer: Arc<Mutex<OwnedWriteHalf>>) {
        let mut message = String::from("===\nCurrently connected users:");
        let clients = self.clients.lock().await;

        for user in clients.iter() {
            if Arc::ptr_eq(&user.writer, &writer) {
                message.push_str(&format!("\n - {} (you)", user.nickname));
            } else {
                message.push_str(&format!("\n - {}", user.nickname));
            }
        }
        message.push_str("\n===");
        let mut writer = writer.lock().await;

        let _ = writer.write_all(message.as_bytes()).await;
        let _ = writer.write_all(b"\n").await;
    }
}
//     pub async fn get_most_alive_clients(&self, count: usize) -> Vec<Client> {
//         let max_idle = Duration::from_secs(180);

//         let now = SystemTime::now();

//         let mut alive_clients: Vec<Client> = self
//             .clients
//             .lock()
//             .await
//             .iter()
//             .filter(|peer| match now.duration_since(peer.last_seen) {
//                 Ok(duration) => duration <= max_idle,
//                 Err(_) => false,
//             })
//             .cloned()
//             .collect();

//         alive_clients.sort_by(|a, b| b.last_seen.cmp(&a.last_seen));

//         alive_clients.into_iter().take(count).collect()
//     }
// }

pub async fn handle_connection(
    pool: Arc<Mutex<ConnectionPool>>,
    stream: TcpStream,
    server: Arc<Mutex<Server>>,
) -> Result<(), Box<dyn Error>> {
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let writer = Arc::new(Mutex::new(writer));

    // Ask for nickname
    writer
        .lock()
        .await
        .write_all(b"Choose your nickname: ")
        .await?;

    let mut nickname_line = String::new();
    reader.read_line(&mut nickname_line).await?;
    let nickname = nickname_line.trim().to_string();

    let client = pool
        .lock()
        .await
        .add_peer(nickname.clone(), writer.clone())
        .await;

    log::info!("Your nickname is: {:?}", nickname);

    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        let msg = line.trim();
        if msg.is_empty() {
            continue;
        }

        println!("Message: {:?}", msg);

        match msg {
            "/quit" => {
                pool.lock().await.broadcast_quit(&client).await;
                log::error!("Client is disconnected!");
                break;
            }
            "/list" => {
                pool.lock().await.list_clients(writer.clone()).await;
            }
            _ => {
                // P2P handling
                let p2p_arc = {
                    let server_guard = server.lock().await;
                    server_guard.p2p_protocol.as_ref().unwrap().clone()
                };
                let p2p = p2p_arc.lock().await;
                pool.lock()
                    .await
                    .broadcast_new_message(&client, msg.to_string())
                    .await;

                p2p.handle_transaction(msg, Some(writer.clone()), None)
                    .await;

                // Broadcast to all clients
            }
        }
    }

    Ok(())
}
