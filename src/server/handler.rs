use crate::{blockchain::init::Blockchain, net::chat::ConnectionPool, p2p::P2PProtocol};
use std::{error::Error, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

pub struct Server {
    pub node_id: String,

    pub blockchain: Arc<Mutex<Blockchain>>,

    pub connection_pool: Arc<Mutex<ConnectionPool>>,

    pub p2p_protocol: Arc<Mutex<P2PProtocol>>,
}

impl Server {
    pub async fn new(
        &self,
        node_id: String,
        blockchain: Arc<Mutex<Blockchain>>,
        connection_pool: Arc<Mutex<ConnectionPool>>,
        p2p_protocol: Arc<Mutex<P2PProtocol>>,
    ) -> Server {
        Server {
            node_id,
            blockchain,
            connection_pool,
            p2p_protocol,
        }
    }

    pub async fn get_external_ip(&self) {
        todo!()
    }

    pub async fn handle_connection(
        pool: Arc<ConnectionPool>,
        stream: TcpStream,
    ) -> Result<(), Box<dyn Error>> {
        let (mut reader, mut writer) = stream.into_split();

        match writer.write_all(b"Choose your nickname: ").await {
            Ok(()) => log::info!("Nickname write successful: "),
            Err(e) => log::error!("Nickname write error: {:?}", e),
        }

        let mut buf = [0u8; 1024];
        let n = match reader.read(&mut buf).await {
            Ok(n) => {
                log::info!("Nickname successfully read!");
                n
            }
            Err(e) => {
                log::error!("Error while reading nickname: {:?}", e);
                return Err(Box::new(e));
            }
        };
        let nickname = String::from_utf8_lossy(&buf[..n]).trim().to_string();

        let writer = Arc::new(Mutex::new(writer));
        let client = pool.add_peer(nickname.clone(), writer.clone()).await;

        log::info!("Your nickname is: {:?}", nickname);

        loop {
            buf.fill(0);
            let bytes_read = reader.read(&mut buf).await;

            match bytes_read {
                Ok(0) => {
                    pool.broadcast_quit(&client).await;
                    log::error!("EOF: Client is disconnected!");
                    break;
                }

                Ok(_) => {
                    let message = String::from_utf8_lossy(&buf[..]);
                    let msg = message.trim_end_matches('\0').trim();
                    println!("Message: {:?}", msg);

                    if msg == "/quit" {
                        pool.broadcast_quit(&client).await;
                        log::error!("Client is disconnected!");
                        break;
                    } else if msg == "/list" {
                        pool.list_clients(writer.clone()).await;
                    } else {
                        pool.broadcast_new_message(&client, msg.to_string()).await;
                    }
                }
                Err(e) => {
                    log::error!("Error occured while reading the message: {:?}", e);
                }
            }
        }

        Ok(())
    }
}
