use super::block::Block;
use crate::{
    types::blockchain::Transaction,
    utils::hasher::{block_hasher, transactions_hasher},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;

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

    pub fn add_new_block(&mut self) -> Blockchain {
        let index = self.get_last_block().map(|block| block.index).unwrap_or(1) + 1;
        let prev_hash = self
            .get_last_block()
            .map(|block| block.hash)
            .unwrap()
            .unwrap();
        let mut block = Block::new(index, prev_hash, self.current_transactions.clone());

        block.hash = Some(block_hasher(&block));
        block.merkle_root = Some(transactions_hasher(&block.transactions));

        self.blocks.push_back(block);
        self.current_transactions = Vec::new();
        self.archieved_transactions
            .append(&mut self.current_transactions);
        self.clone()
    }

    pub fn get_last_block(&self) -> Option<Block> {
        if self.blocks.is_empty() {
            self.blocks.back().cloned()
        } else {
            None
        }
    }

    pub fn get_blockchain_after_timestamp(&self, timestamp: DateTime<Utc>) -> Vec<Block> {
        self.blocks
            .iter()
            .filter(|block| block.timestamp > timestamp)
            .cloned()
            .collect()
    }

    pub fn proof_of_work(&self) -> Block {
        todo!()
    }
}
