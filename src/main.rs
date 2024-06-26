mod config;
mod lexer;
mod save;

use anyhow::Context;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use clap::Parser;
use log::Level;
use serde::Deserialize;

use std::sync::{Arc, Mutex};
use std::{collections::HashMap, net::SocketAddr};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use tracing as log;

use axum::extract::connect_info::ConnectInfo;

#[derive(Clone)]
struct ServerState {
    master_hashmap: HashMap<String, String>,
    password: String,
    persist: bool,
}

#[derive(Debug, Deserialize)]
struct Params {
    password: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::Configuration::parse();

    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(Level::INFO)
            .finish(),
    )?;

    let state = Arc::new(Mutex::new(ServerState {
        master_hashmap: HashMap::new(),
        password: config.password.clone(),
        persist: config.persist,
    }));

    if config.persist {
        let commands = save::load();

        if let Ok(commands) = commands {
            for command in commands {
                handle_command(command, state.clone(), true);
            }
        }
    }

    let app = Router::new()
        .route("/", get(ws_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", config.port))
        .await
        .unwrap();

    log::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();

    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<Mutex<ServerState>>>,
    Query(params): Query<Params>,
) -> impl IntoResponse {
    if let Some(password) = params.password {
        if password != state.lock().unwrap().password {
            return (StatusCode::UNAUTHORIZED, "Invalid password").into_response();
        }
    } else {
        return (StatusCode::BAD_REQUEST, "Password required").into_response();
    }

    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    log::info!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| async move {
        let _ = tokio::spawn(handle_socket(socket, addr, state)).await;
    })
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr, state: Arc<Mutex<ServerState>>) {
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        log::info!("Pinged {who}...");
    } else {
        log::error!("Could not send ping {who}");
        return;
    }

    loop {
        if let Some(msg) = socket.recv().await {
            if let Ok(msg) = msg {
                log::info!("Received message from {who}: {:?}", msg);
                match msg {
                    Message::Text(text) => {
                        let msg = handle_command(text, state.clone(), false);

                        if let Some(msg) = msg {
                            if socket.send(Message::Text(msg)).await.is_err() {
                                log::error!("Could not send message to {who}");
                                return;
                            }
                        }
                    }
                    Message::Pong(vec) => {
                        log::info!("Recieved pong ({:?}) from {who}", vec)
                    }
                    _ => {
                        log::warn!("Received non-text message from {who}");
                    }
                }
            }
        } else {
            log::info!("Client {who} abruptly disconnected");
            return;
        }
    }
}

fn handle_command(
    unparsed_command: String,
    state: Arc<Mutex<ServerState>>,
    load_state: bool,
) -> Option<String> {
    let parsed = lexer::Lexer::parse(unparsed_command.clone());

    if let Err(_) = parsed {
        return Some("Invalid command".to_string());
    } else {
        let (command, key, value) = parsed.unwrap();

        match command {
            lexer::Command::Add => {
                let mut state = state.lock().unwrap();
                let _ = state.master_hashmap.insert(key.clone(), value);

                if state.persist && !load_state {
                    save::save(unparsed_command)
                        .context(format!("Failed to append ADD {} to savefile", key))
                        .unwrap();
                }
            }
            lexer::Command::Get => {
                let value = {
                    let state = state.lock().unwrap();
                    state.master_hashmap.get(&key).cloned()
                };
                match value {
                    Some(value) => {
                        return Some(value);
                    }
                    None => {
                        return Some("Key not found".to_string());
                    }
                }
            }
            lexer::Command::Delete => {
                let mut state = state.lock().unwrap();
                let _ = state.master_hashmap.remove(&key);

                if state.persist && !load_state {
                    save::save(unparsed_command)
                        .context(format!("Failed to append DELETE {key} to savefile"))
                        .unwrap();
                }
            }
            lexer::Command::Invalid => {
                return Some("Invalid command".to_string());
            }
        };
    }

    return None;
}
