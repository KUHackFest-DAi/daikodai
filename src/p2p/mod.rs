use std::sync::Arc;

use crate::{
    blockchain::init::Blockchain,
    net::chat::{Client, ConnectionPool},
    server::handler::Server,
    types::blockchain::Ping,
    utils::message::create_block_message,
};
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf, sync::Mutex};

pub struct P2PProtocol {
    pub server: Arc<Mutex<Server>>,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub connection_pool: Arc<Mutex<ConnectionPool>>,
}

impl P2PProtocol {
    pub async fn new(server: Arc<Mutex<Server>>) -> Self {
        let server_clone = server.clone();
        let server_lock = server_clone.lock().await;

        Self {
            server,
            blockchain: server_lock.blockchain.clone(),
            connection_pool: server_lock.connection_pool.clone(),
        }
    }

    pub async fn send_message(writer: &mut OwnedWriteHalf, message: String) {
        let _ = writer
            .write_all(format!("\n{}\n", message).as_bytes())
            .await;
    }

    // pub async fn handle_message(&self, writer: &mut OwnedWriteHalf, message: String) {
    //     let message = serde_json::from_str(&message).unwrap();
    //
    //     match message {
    //         MessageType::BlockMessage(block) => {
    //             let message = serde_json::to_string(&block).unwrap();
    //             self.handle_block(writer, message).await;
    //         }
    //
    //         MessageType::PeersMessage(peer) => {
    //             let message = serde_json::to_string(&peer).unwrap();
    //             self.handle_block(writer, message).await;
    //         }
    //
    //         MessageType::PingMessage(ping) => {
    //             let message = serde_json::to_string(&ping).unwrap();
    //             self.handle_ping(writer, message).await;
    //         }
    //
    //         MessageType::TransactionMessage(transaction) => {
    //             let message = serde_json::to_string(&transaction).unwrap();
    //             self.handle_transaction(writer, message).await;
    //         }
    //     }
    // }
    //
    // pub async fn handle_ping(&self, writer: &mut OwnedWriteHalf, message: String) {
    //     let peers_message = {
    //         let pool = self.connection_pool.lock().await;
    //         let message: Ping = serde_json::from_str(&message).unwrap();
    //         let alive_peers = pool.get_most_alive_peers(message.peer_count).await;
    //         serde_json::to_string(&alive_peers).unwrap()
    //     };
    //
    //     P2PProtocol::send_message(writer, peers_message);
    //
    //     let last_height = self.blockchain.lock().await.blocks.len();
    //     let pool = self.connection_pool.lock().await;
    //     let message: Ping = serde_json::from_str(&message).unwrap();
    //     if message.block_height < last_height {
    //         for block in &self
    //             .blockchain
    //             .lock()
    //             .await
    //             .blocks
    //             .iter()
    //             .skip(message.block_height + 1)
    //         {
    //             let block_message = create_block_message(ip, port, block).await;
    //
    //             P2PProtocol::send_message(writer, block_message);
    //         }
    //     } else {
    //         None
    //     }
    // }
    //
    // pub async fn handle_block(&self, writer: &mut OwnedWriteHalf, message: String) {
    //     todo!()
    // }
    //
    // pub async fn handle_transaction(&self, writer: &mut OwnedWriteHalf, message: String) {
    //     todo!()
    // }
    //
    // pub async fn handle_peers(&self, writer: &mut OwnedWriteHalf, message: String) {
    //     todo!()
    // }
}
