use crate::{
    blockchain::init::Blockchain, net::chat::ConnectionPool, p2p::P2PProtocol,
    types::blockchain::TransactionMessage,
};
use std::{error::Error, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

pub struct Server {
    pub blockchain: Arc<Mutex<Blockchain>>,

    pub connection_pool: Arc<Mutex<ConnectionPool>>,

    pub p2p_protocol: Option<Arc<Mutex<P2PProtocol>>>,
}

impl Server {
    pub async fn new(
        blockchain: Arc<Mutex<Blockchain>>,
        connection_pool: Arc<Mutex<ConnectionPool>>,
    ) -> Server {
        Server {
            blockchain,
            connection_pool,
            p2p_protocol: None,
        }
    }

    pub async fn get_external_ip(&self) {
        todo!()
    }
}
