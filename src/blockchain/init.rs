use super::block::Block;
use crate::{
    p2p::CURRENT_TRANSACTIONS,
    types::blockchain::{ActionType, Transaction},
    utils::hasher::{block_hasher, transactions_hasher},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::LinkedList,
    sync::{Arc, Mutex},
    thread::AccessError,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Blockchain {
    pub blocks: LinkedList<Block>,

    pub current_transactions: Vec<Transaction>,

    pub archieved_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn init() -> Blockchain {
        let genesis_block = Block::init();

        let mut blocks = LinkedList::new();
        blocks.push_back(genesis_block);

        Blockchain {
            blocks,
            current_transactions: Vec::new(),
            archieved_transactions: Vec::new(),
        }
    }

    pub async fn add_new_block(&mut self) -> (Block, Blockchain) {
        println!("\nBlockchain: {:?}\n", self);
        let index = self.get_last_block().map(|block| block.index).unwrap_or(1) + 1;
        println!("Last Block: {:?}", self.get_last_block());

        let prev_hash = self
            .get_last_block()
            .and_then(|block| block.hash.clone())
            .unwrap_or_else(|| "0".to_string());
        let mut global_tx = CURRENT_TRANSACTIONS.lock().await;
        let mut block = Block::new(index, prev_hash, global_tx.to_vec());
        global_tx.clear();

        block.hash = Some(block_hasher(&block));
        block.merkle_root = Some(transactions_hasher(&block.transactions));

        self.blocks.push_back(block.clone());
        self.current_transactions = Vec::new();
        self.archieved_transactions
            .append(&mut self.current_transactions);
        (block, self.clone())
    }

    pub fn get_last_block(&self) -> Option<Block> {
        self.blocks.back().cloned()
    }

    pub fn get_blockchain_after_timestamp(&self, timestamp: DateTime<Utc>) -> Vec<Block> {
        self.blocks
            .iter()
            .filter(|block| block.timestamp > timestamp)
            .cloned()
            .collect()
    }

    pub async fn proof_of_work(&self, connection_len: u32) -> Option<ActionType> {
        let transactions = CURRENT_TRANSACTIONS.lock().await;

        let accept_votes = transactions
            .iter()
            .filter(|tx| tx.action_type == ActionType::VoteAccept)
            .count();
        println!("Accept: {:?}", accept_votes);
        let reject_votes = transactions
            .iter()
            .filter(|tx| tx.action_type == ActionType::VoteReject)
            .count();
        println!("Reject: {:?}", reject_votes);
        let connection_len = connection_len - 1;

        let accept_ratio = accept_votes as f32 / connection_len as f32;
        let reject_ratio = reject_votes as f32 / connection_len as f32;

        log::info!("Votes: Accept={} Reject={}", accept_votes, reject_votes);
        log::info!(
            "Consensus ratio: Accept={:.2} Reject={:.2}",
            accept_ratio,
            reject_ratio
        );

        if accept_ratio >= 2.0 / 3.0 {
            Some(ActionType::VoteAccept)
        } else if reject_ratio >= 2.0 / 3.0 {
            Some(ActionType::VoteReject)
        } else {
            None
        }
    }
}
