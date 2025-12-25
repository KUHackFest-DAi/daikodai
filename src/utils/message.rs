use crate::blockchain::block::Block;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
    version: String,

    address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockMessage {
    pub meta: Meta,

    pub payload: Block,
}

fn meta(external_ip: String, port: usize) -> Meta {
    Meta {
        version: String::from("1.0.0"),
        address: format!("{}:{}", external_ip, port),
    }
}

pub async fn create_block_message(ip: String, port: usize, block: Block) -> String {
    let meta = meta(ip, port);
    let message = BlockMessage {
        meta,
        payload: block,
    };

    serde_json::to_string(&message).unwrap()
}
