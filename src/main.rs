use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use futures::{SinkExt, StreamExt};
use log4rs::config::Deserializers;
use no_cap::{
    blockchain::init::Blockchain,
    net::chat::{handle_connection, ConnectionPool},
    p2p::P2PProtocol,
    server::handler::Server as HandlerServer,
    types::{args::Args, blockchain::TransactionMessage},
    utils::reqwest::get_external_ip,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::AsyncWriteExt,
    net::TcpListener,
    sync::{mpsc, Mutex},
};
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    pool: Arc<Mutex<ConnectionPool>>,
    p2p: Arc<Mutex<P2PProtocol>>,
    ws_peers: Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    log4rs::init_file(&args.log_config, Deserializers::new()).unwrap();
    dotenv::from_path(&args.dotenv).ok();
    get_external_ip().await.unwrap();

    // ---- Core shared state ----
    let blockchain = Arc::new(Mutex::new(Blockchain::init()));
    let pool = Arc::new(Mutex::new(ConnectionPool::init()));
    let ws_peers = Arc::new(Mutex::new(Vec::new()));

    let server = Arc::new(Mutex::new(HandlerServer {
        blockchain,
        connection_pool: pool.clone(),
        p2p_protocol: None,
    }));

    let p2p = Arc::new(Mutex::new(P2PProtocol::new(server.clone()).await));
    server.lock().await.p2p_protocol = Some(p2p.clone());

    let app_state = AppState {
        pool: pool.clone(),
        p2p,
        ws_peers: ws_peers.clone(),
    };

    // ---- CORS ----
    let cors = CorsLayer::new()
        .allow_origin(Any) // during dev
        .allow_methods(Any)
        .allow_headers(Any);

    // ---- Axum Router ----
    let app = Router::new()
        .route("/transaction", post(submit_transaction))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(app_state);

    // ---- HTTP + WS Server ----
    let http_addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    tokio::spawn(async move {
        axum::serve(TcpListener::bind(http_addr).await.unwrap(), app)
            .await
            .unwrap();
    });

    // ---- TCP P2P Server ----
    let tcp_listener = TcpListener::bind("0.0.0.0:2373").await.unwrap();
    log::info!("P2P TCP listening on 2373");

    loop {
        let (stream, addr) = tcp_listener.accept().await.unwrap();
        log::info!("P2P connection from {}", addr);

        let pool_clone = pool.clone();
        let server_clone = server.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(pool_clone, stream, server_clone).await {
                log::error!("TCP error: {:?}", e);
            }
        });
    }
}

// ---------------- HTTP ----------------

async fn submit_transaction(
    State(state): State<AppState>,
    Json(tx_msg): Json<TransactionMessage>,
) -> impl IntoResponse {
    let msg_str = serde_json::to_string(&tx_msg).unwrap();

    // ---- Core consensus logic ----
    state
        .p2p
        .lock()
        .await
        .handle_transaction(&msg_str, None, Some(state.ws_peers.clone()))
        .await;

    // ---- WebSocket broadcast ----
    let peers = state.ws_peers.lock().await;
    for peer in peers.iter() {
        let _ = peer.send(Message::Text(msg_str.clone().into()));
    }

    // ---- TCP broadcast ----
    let pool = state.pool.lock().await;
    let clients = pool.clients.lock().await;
    for client in clients.iter() {
        let mut writer = client.writer.lock().await;
        let _ = writer.write_all(msg_str.as_bytes()).await;
    }

    (axum::http::StatusCode::OK, "Transaction accepted")
}

// ---------------- WebSocket ----------------

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    state.ws_peers.lock().await.push(tx);

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let _ = sender.send(msg).await;
        }
    });

    while let Some(Ok(_)) = receiver.next().await {}

    send_task.abort();
}
