use clap::Parser;
use log4rs::config::Deserializers;
use no_cap::{
    blockchain::init::Blockchain,
    net::chat::{handle_connection, ConnectionPool},
    p2p::P2PProtocol,
    server::handler::Server,
    types::{args::Args, blockchain::Transaction},
    utils::reqwest::get_external_ip,
};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

use lazy_static::lazy_static;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match log4rs::init_file(&args.log_config, Deserializers::new()) {
        Ok(_) => log::info!("Logger successfully initialized for no_cap!"),
        Err(e) => log::error!("Error with logger initialization: {:?}", e),
    }
    log::info!("NoCap initialized successfully!");

    if let Err(e) = dotenv::from_path(&args.dotenv) {
        log::info!("Failed to log dotenv file: {:?}", e);
    }

    get_external_ip().await.unwrap();

    let blockchain = Arc::new(Mutex::new(Blockchain::init()));
    let pool = Arc::new(Mutex::new(ConnectionPool::init()));

    let server = Arc::new(Mutex::new(Server {
        blockchain: blockchain.clone(),
        connection_pool: pool.clone(),
        p2p_protocol: None, // temporarily None
    }));

    let p2p = Arc::new(Mutex::new(P2PProtocol::new(server.clone()).await));

    server.lock().await.p2p_protocol = Some(p2p.clone());

    let listener = TcpListener::bind("0.0.0.0:2373").await.unwrap();
    log::info!("Listening on 127.0.0.0:2373");

    loop {
        let (stream, addr) = match listener.accept().await {
            Ok((s, a)) => {
                log::info!("Stream successfully obtained!");
                (s, a)
            }
            Err(e) => {
                log::error!("Error while obtaining stream: {:?}", e);
                continue;
            }
        };
        log::info!("Connection from {}", addr);
        let pool_clone = pool.clone();

        let value = server.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(pool_clone, stream, value).await {
                log::error!("Error handling connection: {}", e);
            }
        });
    }
}

// use sha2::{Digest, Sha256};
// use sodiumoxide::crypto::box_;
//
// fn sha256_hash(hash: String) -> String {
//     let mut hasher = Sha256::new();
//     hasher.update(hash.as_bytes());
//     let result = hasher.finalize();
//     hex::encode(result)
// }
//
// fn main() {
//     sodiumoxide::init().unwrap();
//
//     let (alice_pk, alice_sk) = box_::gen_keypair();
//     let (bob_pk, bob_sk) = box_::gen_keypair();
//
//     let message = "I am Satoshi!";
//
//     let nonce = box_::gen_nonce();
//     let encrypted = box_::seal(message, &nonce, &alice_pk, &bob_sk);
//
//     println!("Encrypted message: {:?}", encrypted);
//
//     let decrypted = box_::open(&encrypted, &nonce, &bob_pk, &alice_sk).expect("Decryption failed!");
//
//     println!(
//         "Decrypted message: {:?}",
//         String::from_utf8(decrypted).unwrap()
//     );
// }
