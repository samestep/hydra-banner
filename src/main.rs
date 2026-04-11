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

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
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
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client should initialize");

    let state = Arc::new(AppState { client });
    let app = Router::new()
        .route("/", get(handlers::home))
        .route("/health", get(handlers::health))
        .route("/build/:id", get(handlers::build_banner))
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
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("axum server should run");
}
