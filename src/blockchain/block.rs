use crate::{
    types::blockchain::Transaction,
    utils::hasher::{block_hasher, transactions_hasher},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub index: u32,

    pub prev_hash: String,

    pub hash: Option<String>,

    pub timestamp: DateTime<Utc>,

    pub transactions: Vec<Transaction>,

    pub merkle_root: Option<String>,
}

impl Block {
    pub fn init() -> Self {
        let mut genesis_block = Block {
            index: 0,
            prev_hash: "0".to_string(),
            hash: None,
            timestamp: Utc::now(),
            transactions: Vec::new(),
            merkle_root: None,
        };
        genesis_block.hash = Some(block_hasher(&genesis_block));
        genesis_block.merkle_root = Some(transactions_hasher(&genesis_block.transactions));

        genesis_block
    }

    pub fn new(index: u32, prev_hash: String, current_transaction: Vec<Transaction>) -> Self {
        Self {
            index,
            prev_hash,
            hash: None,
            timestamp: Utc::now(),
            transactions: current_transaction,
            merkle_root: None,
        }
    }

    // pub async fn get_block_after_timestamp(&self, timestamp: Datetime<Utc>) -> Block {
    //
    // }
}
