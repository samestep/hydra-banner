use std::{env, net::SocketAddr, sync::Arc};

use axum::{routing::get, Router};
use reqwest::Client;
use tower_http::trace::TraceLayer;
use tracing::info;

mod format;
mod handlers;
mod hydra;
mod render;
mod status;

#[derive(Debug, Clone)]
pub struct AppState {
    pub client: Client,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hydra_banner=debug,tower_http=debug".into()),
        )
        .init();

    let client = Client::builder()
        .user_agent("hydra-banner/0.1.0")
        .build()
        .expect("reqwest client should initialize");

    let state = Arc::new(AppState { client });
    let app = Router::new()
        .route("/job/:id", get(handlers::job_banner))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("tcp listener should bind");

    axum::serve(listener, app)
        .await
        .expect("axum server should run");
}
