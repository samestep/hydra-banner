use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::error;

use crate::{
    hydra::fetch_build,
    render::{render_error_svg, render_svg},
    AppState,
};

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn build_banner(State(state): State<Arc<AppState>>, Path(id): Path<u64>) -> Response {
    match fetch_build(&state.client, id).await {
        Ok(build) => (
            [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
            render_svg(&build),
        )
            .into_response(),
        Err(message) => {
            error!("failed to fetch build {}: {}", id, message);
            (
                StatusCode::BAD_GATEWAY,
                [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
                render_error_svg(&message),
            )
                .into_response()
        }
    }
}
