use std::time::SystemTime;

use crate::{blockchain::block::Block, net::chat::Client};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum VoteVerdict {
    Accept,

    Reject,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EvaluationVote {
    ValidateBlock,

    FlagConflict,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModelModification {
    pub model_hash: String,

    pub cid: String,

    pub description: String,

    // ref to IPFS, images/videos
    pub validation_proof: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModelParameters {
    pub update_id: String,

    pub confidence: f32,

    pub score: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EvaluationResult {
    pub vote: Option<VoteVerdict>,

    pub evaluation: Option<EvaluationVote>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PayloadData {
    pub model_modification: Option<ModelModification>,

    pub model_parameters: Option<ModelParameters>,

    pub evaluation_result: Option<EvaluationResult>,

    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ActionType {
    ProposeUpdate,

    EvaluateUpdate,

    VoteAccept,

    VoteReject,

    FlagMalicious,

    FinalizeBlock,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub agent_id: String,

    pub signature: String,

    // if the change is questioned later on, it should be shown and will be verified if the
    // reasoning is post-hoc rationalizzation
    pub reasoning_hash: String,

    pub action_type: ActionType,

    pub payload: PayloadData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ping {
    pub block_height: u32,

    pub peer_count: usize,

    pub is_miner: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeerAddr {
    ip: String,

    port: u32,
}
//
// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub enum MessageType {
//     BlockMessage(Block),
//
//     PeersMessage(Peer),
//
//     PingMessage(Ping),
//
//     TransactionMessage(Transaction),
// }
