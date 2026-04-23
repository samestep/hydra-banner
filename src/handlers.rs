use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use tracing::error;

use crate::{
    hydra::{fetch_build, HydraError},
    render::{render_error_svg, render_svg},
    AppState,
};

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn home() -> Redirect {
    Redirect::permanent("https://github.com/MiniHarinn/hydra-banner")
}

fn svg(status: StatusCode, body: impl IntoResponse) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        body,
    )
        .into_response()
}

pub async fn build_banner(State(state): State<AppState>, Path(raw): Path<String>) -> Response {
    let id = match raw.parse::<u64>() {
        Ok(id) => id,
        Err(_) => {
            let err = HydraError::InvalidId(raw.clone());
            error!("invalid build ID: {:?}", raw);
            return svg(StatusCode::BAD_REQUEST, render_error_svg(&err));
        }
    };

    match fetch_build(&state.client, id).await {
        Ok(build) => {
            let status = if build.finished == 0 {
                StatusCode::ACCEPTED
            } else {
                StatusCode::OK
            };
            svg(status, render_svg(&build))
        }
        Err(err) => {
            error!("failed to fetch build {}: {:?}", id, err);
            let status = match &err {
                HydraError::NotFound(_) => StatusCode::NOT_FOUND,
                HydraError::InvalidId(_) => StatusCode::BAD_REQUEST,
                HydraError::Other(_) => StatusCode::BAD_GATEWAY,
            };
            svg(status, render_error_svg(&err))
        }
    }
}
