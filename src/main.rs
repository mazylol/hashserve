use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;

use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum::extract::connect_info::ConnectInfo;

#[derive(Clone)]
struct ServerState {
    master_vec: Vec<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(Mutex::new(ServerState {
        master_vec: Vec::new(),
    }));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<Mutex<ServerState>>>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr, state: Arc<Mutex<ServerState>>) {
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        println!("Pinged {who}...");
    } else {
        println!("Could not send ping {who}");
        return;
    }

    loop {
        if let Some(msg) = socket.recv().await {
            if let Ok(msg) = msg {
                println!("Received message from {who}: {:?}", msg);
                match msg {
                    Message::Text(text) => {
                        state.lock().unwrap().master_vec.push(text.clone());
                    },
                    _ => {
                        println!("Received non-text message from {who}");
                    }
                };

                println!("master_vec: {:?}", state.lock().unwrap().master_vec);
                socket.send(Message::Text(String::from("uwu"))).await.unwrap();
            }
        } else {
            println!("client {who} abruptly disconnected");
            return;
        }
    }
}
