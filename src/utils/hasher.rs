use crate::{
    blockchain::block::Block,
    types::{blockchain::Transaction, error::ErrorTypes},
};
use sha2::{Digest, Sha256};

fn transaction_serialize(transactions: &Vec<Transaction>) -> Result<String, ErrorTypes> {
    match serde_json::to_string(transactions) {
        Ok(transation) => {
            log::info!("Transaction successfully serialized!");
            Ok(transation)
        }
        Err(e) => {
            log::error!("Error while serializing transactions");
            return Err(ErrorTypes::TransactionSerializeError(format!(
                "Error while seriazling transactions: {:?}",
                e
            )));
        }
    }
}

fn hasher(input: String) -> String {
    let mut hasher = Sha256::new();

    hasher.update(input.as_bytes());

    let hex_digest = hasher.finalize();
    hex::encode(hex_digest)
}

pub fn block_hasher(block: &Block) -> String {
    let transaction = transaction_serialize(&block.transactions).unwrap();
    let input = format!(
        "{:?}{:?}{:?}{:?}{:?}",
        block.index, block.prev_hash, block.hash, transaction, block.timestamp
    );
    hasher(input)
}

pub fn transactions_hasher(transactions: &Vec<Transaction>) -> String {
    let transaction = transaction_serialize(&transactions).unwrap();
    hasher(transaction)
}
