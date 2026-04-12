use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use tracing::error;

use crate::{
    hydra::{fetch_build, HydraError},
    render::{render_error_svg, render_svg, ErrorBannerTemplate},
    AppState,
};

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn home() -> Redirect {
    Redirect::permanent("https://github.com/MiniHarinn/hydra-banner")
}

pub async fn build_banner(State(state): State<Arc<AppState>>, Path(raw): Path<String>) -> Response {
    let svg_response = |status: StatusCode, svg: ErrorBannerTemplate| {
        (
            status,
            [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
            svg,
        )
            .into_response()
    };

    let id = match raw.parse::<u64>() {
        Ok(id) => id,
        Err(_) => {
            let err = HydraError::InvalidId(raw.clone());
            error!("invalid build ID: {:?}", raw);
            return svg_response(StatusCode::BAD_REQUEST, render_error_svg(&err));
        }
    };

    match fetch_build(&state.client, id).await {
        Ok(build) => (
            [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
            render_svg(&build),
        )
            .into_response(),
        Err(err) => {
            error!("failed to fetch build {}: {:?}", id, err);
            let status = match &err {
                HydraError::NotFound(_) => StatusCode::NOT_FOUND,
                HydraError::InvalidId(_) => StatusCode::BAD_REQUEST,
                HydraError::Other(_) => StatusCode::BAD_GATEWAY,
            };
            svg_response(status, render_error_svg(&err))
        }
    }
}
