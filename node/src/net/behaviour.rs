use crate::chain;
use crate::chain::block::Block;
use libp2p::swarm::NetworkBehaviour;
use libp2p::{gossipsub, mdns, request_response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceRequest {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceResponse {
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxResponse {
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRequest {
    pub idx: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponse {
    pub block: Option<Block>,
}

#[derive(NetworkBehaviour)]
pub struct ClientBehaviour {
    pub nonce: request_response::json::Behaviour<NonceRequest, NonceResponse>,
    pub tx: request_response::json::Behaviour<chain::tx::Tx, TxResponse>,
    pub find_block: request_response::json::Behaviour<BlockRequest, BlockResponse>,
}

#[derive(NetworkBehaviour)]
pub struct NodeBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub nonce: request_response::json::Behaviour<NonceRequest, NonceResponse>,
    pub tx: request_response::json::Behaviour<chain::tx::Tx, TxResponse>,
    pub find_block: request_response::json::Behaviour<BlockRequest, BlockResponse>,
}
