use crate::{
    blockchain::init::Blockchain,
    net::chat::{Client, ConnectionPool},
    server::handler::Server,
    types::blockchain::{ActionType, Ping, Transaction, TransactionMessage},
    utils::message::create_block_message,
};
use axum::extract::ws::Message;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::AsyncWriteExt,
    net::tcp::OwnedWriteHalf,
    sync::{mpsc, Mutex},
};

static PROPOSAL_OWNERS: Lazy<Arc<Mutex<HashMap<String, String>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub static CURRENT_TRANSACTIONS: once_cell::sync::Lazy<Arc<tokio::sync::Mutex<Vec<Transaction>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(tokio::sync::Mutex::new(Vec::new())));

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

    fn validate_proposal(tx: &Transaction) -> Result<(), String> {
        if tx.agent_id.trim().is_empty() {
            return Err("Agent ID is empty".to_string());
        }

        if tx.reasoning_hash.trim().is_empty() {
            return Err("Reasoning hash is empty".to_string());
        }

        if tx.action_type != crate::types::blockchain::ActionType::ProposeUpdate {
            return Err("Transaction is not a ProposeUpdate".to_string());
        }

        if tx.payload.description.trim().is_empty() {
            return Err("Proposal description is empty".to_string());
        } else {
            Ok(())
        }
    }

    pub async fn handle_transaction(
        &self,
        msg: &str,
        writer: Option<Arc<Mutex<OwnedWriteHalf>>>,
        ws_peers: Option<Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>>>,
    ) {
        match serde_json::from_str::<TransactionMessage>(msg) {
            Ok(tx_msg) => {
                log::info!(
                    "Received transaction from agent: {}\n",
                    tx_msg.payload.agent_id
                );
                log::info!("Action type: {:?}", tx_msg.payload.action_type);
                log::info!("Reasoning hash: {}", tx_msg.payload.reasoning_hash);
                log::info!("Description: {}", tx_msg.payload.payload.description);

                match tx_msg.payload.action_type {
                    crate::types::blockchain::ActionType::ProposeUpdate => {
                        log::info!("ProposeUpdate: {:?}", tx_msg);

                        if let Err(err) = Self::validate_proposal(&tx_msg.payload) {
                            log::warn!("Invalid proposal: {:?}", err);
                            return;
                        }

                        PROPOSAL_OWNERS.lock().await.insert(
                            tx_msg.payload.reasoning_hash.clone(),
                            tx_msg.payload.agent_id.clone(),
                        );

                        CURRENT_TRANSACTIONS
                            .lock()
                            .await
                            .push(tx_msg.payload.clone());
                        let pool = self.connection_pool.lock().await;
                        for client in pool.clients.lock().await.iter() {
                            let mut writer = client.writer.lock().await;
                            let message = serde_json::to_string(&tx_msg).unwrap();
                            let _ = writer.write_all(message.as_bytes()).await;
                        }

                        log::info!("Proposal broadcasted to peers.");
                    }

                    crate::types::blockchain::ActionType::VoteAccept
                    | crate::types::blockchain::ActionType::VoteReject => {
                        let owners = PROPOSAL_OWNERS.lock().await;
                        if let Some(owner) = owners.get(&tx_msg.payload.reasoning_hash) {
                            if owner == &tx_msg.payload.agent_id {
                                log::warn!(
                                    "Agent {} attempted to vote on its own proposal {}. Ignoring.",
                                    tx_msg.payload.agent_id,
                                    tx_msg.payload.reasoning_hash
                                );
                                return;
                            }
                        }
                        log::info!("VoteAccept: {:?}", tx_msg);

                        CURRENT_TRANSACTIONS.lock().await.push(tx_msg.payload);

                        let verdict = self
                            .blockchain
                            .lock()
                            .await
                            .proof_of_work(
                                self.connection_pool.lock().await.clients.lock().await.len() as u32,
                            )
                            .await;

                        log::warn!("Verdict: {:?}", verdict);

                        if verdict == Some(ActionType::VoteAccept) {
                            log::info!("Adding new block!\n");
                            let (block, _blockchain) =
                                self.blockchain.lock().await.add_new_block().await;
                            let message = serde_json::to_string_pretty(&block).unwrap();
                            let pool = self.connection_pool.lock().await;
                            for client in pool.clients.lock().await.iter() {
                                let mut writer = client.writer.lock().await;
                                let _ = writer.write_all(message.as_bytes()).await;
                            }

                            if let Some(ws) = ws_peers {
                                let msg = serde_json::to_string_pretty(&block).unwrap();
                                let peers = ws.lock().await;
                                for peer in peers.iter() {
                                    let _ = peer.send(Message::Text(msg.clone().into()));
                                }
                            }
                        } else if verdict == Some(ActionType::VoteReject) {
                            log::info!("Block has been rejected!\n");
                        } else {
                            log::warn!("Consensus not reached yet!\n");
                        }
                    }
                    crate::types::blockchain::ActionType::FlagMalicious => {
                        log::info!("FlagMalicious: {:?}\n", tx_msg);
                    }
                    crate::types::blockchain::ActionType::FinalizeBlock => {
                        log::info!("FinalizeBlock: {:?}\n", tx_msg);
                    }
                    _ => {}
                }
            }

            Err(e) => {}
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
