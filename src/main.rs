use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use clap::Parser;
use serde::Deserialize;
use tracing::Level;

use std::net::SocketAddr;

use http_body_util::{combinators::BoxBody, BodyExt};
use http_body_util::{Empty, Full};
use hyper::body::{Body, Bytes};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

mod config;
mod lexer;
mod save;

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
async fn main() -> anyhow::Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        let state = state.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(move |req| handler(req, state.clone())))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
            {}
        });
    }
}

async fn handler(
    req: Request<hyper::body::Incoming>,
    state: Arc<Mutex<ServerState>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // parse query params
    let params = req.uri().query().map(|q| {
        serde_urlencoded::from_str::<Params>(q)
            .map_err(|e| {
                tracing::error!("Failed to parse query params: {:?}", e);
            })
            .ok()
    });

    if let Some(Some(params)) = params {
        if let Some(password) = params.password {
            let state = state.lock().unwrap();
            if state.password != password {
                let mut resp = Response::new(full("Unauthorized"));
                *resp.status_mut() = hyper::StatusCode::UNAUTHORIZED;
                return Ok(resp);
            }
        } else {
            let mut resp = Response::new(full("Unauthorized"));
            *resp.status_mut() = hyper::StatusCode::UNAUTHORIZED;
            return Ok(resp);
        }
    } else {
        let mut resp = Response::new(full("Unauthorized"));
        *resp.status_mut() = hyper::StatusCode::UNAUTHORIZED;
        return Ok(resp);
    }

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let state = state.lock().unwrap();
            Ok(Response::new(full(format!("{:?}", state.master_hashmap))))
        }
        (&Method::POST, "/") => {
            let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
            if upper > 1024 * 64 {
                let mut resp = Response::new(full("Body too big"));
                *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(resp);
            }

            let whole_body_frames = req.collect().await?.to_bytes();

            let whole_body_vec = whole_body_frames.iter().cloned().collect::<Vec<u8>>();

            let whole_body = String::from_utf8(whole_body_vec).unwrap();

            let resp = handle_command(whole_body, state.clone(), false);

            if let Some(resp) = resp {
                Ok(Response::new(full(resp)))
            } else {
                Ok(Response::new(full("OK")))
            }
        }

        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
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
