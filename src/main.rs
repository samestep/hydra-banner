use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use reqwest::Client;
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

#[derive(Debug, Clone)]
struct AppState {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct HydraBuild {
    id: u64,
    #[serde(default)]
    job: String,
    #[serde(default)]
    project: String,
    #[serde(default)]
    jobset: String,
    #[serde(default)]
    system: String,
    #[serde(default)]
    finished: u8,
    #[serde(default)]
    buildstatus: Option<i32>,
    #[serde(default)]
    timestamp: i64,
    #[serde(default)]
    starttime: Option<i64>,
    #[serde(default)]
    stoptime: Option<i64>,
    #[serde(default)]
    nixname: Option<String>,
    #[serde(default)]
    priority: Option<i64>,
    #[serde(default)]
    jobsetevals: Option<Vec<u64>>,
    #[serde(default)]
    buildproducts: Option<HashMap<String, BuildProduct>>,
}

#[derive(Debug, Deserialize)]
struct BuildProduct {
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    subtype: Option<String>,
    #[serde(default)]
    filesize: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
struct StatusInfo {
    icon_variant: &'static str,
    color: &'static str,
    label: &'static str,
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
        .route("/job/:id", get(job_banner))
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

async fn job_banner(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Response {
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

async fn fetch_build(client: &Client, id: u64) -> Result<HydraBuild, String> {
    let url = format!("https://hydra.nixos.org/build/{id}");
    let response = client
        .get(url)
        .header(header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|err| format!("request to Hydra failed: {err}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("Hydra returned HTTP {}", status.as_u16()));
    }

    response
        .json::<HydraBuild>()
        .await
        .map_err(|err| format!("invalid Hydra JSON: {err}"))
}

fn status_info(build: &HydraBuild) -> StatusInfo {
    if build.finished == 0 {
        return StatusInfo {
            icon_variant: "question",
            color: "#A6AEB0",
            label: "BUILDING...",
        };
    }

    match build.buildstatus.unwrap_or(1) {
        0 => StatusInfo {
            icon_variant: "checkmark",
            color: "#61B329",
            label: "SUCCESS",
        },
        1 => StatusInfo {
            icon_variant: "red-x",
            color: "#FF5A79",
            label: "FAILED",
        },
        2 => StatusInfo {
            icon_variant: "gray-x",
            color: "#4D5357",
            label: "DEPENDENCY FAILED",
        },
        3 => StatusInfo {
            icon_variant: "stopsign",
            color: "#ED4C5C",
            label: "ABORTED",
        },
        4 => StatusInfo {
            icon_variant: "stopsign",
            color: "#ED4C5C",
            label: "CANCELLED",
        },
        6 => StatusInfo {
            icon_variant: "red-x",
            color: "#FF5A79",
            label: "FAILED WITH OUTPUT",
        },
        7 => StatusInfo {
            icon_variant: "stopsign",
            color: "#ED4C5C",
            label: "TIMED OUT",
        },
        9 => StatusInfo {
            icon_variant: "stopsign",
            color: "#ED4C5C",
            label: "UNSUPPORTED SYSTEM",
        },
        10 => StatusInfo {
            icon_variant: "stopsign",
            color: "#ED4C5C",
            label: "LOG LIMIT EXCEEDED",
        },
        11 => StatusInfo {
            icon_variant: "red-x",
            color: "#FF5A79",
            label: "OUTPUT LIMIT EXCEEDED",
        },
        12 => StatusInfo {
            icon_variant: "red-x",
            color: "#FF5A79",
            label: "NON-DETERMINISTIC",
        },
        _ => StatusInfo {
            icon_variant: "red-x",
            color: "#FF5A79",
            label: "FAILED",
        },
    }
}

fn icon_svg(icon_variant: &str) -> &'static str {
    match icon_variant {
        "checkmark" => {
            r##"<svg width="132" height="132" viewBox="0 0 64 64" aria-hidden="true"><path fill="#61B329" d="M55.999 2L18.8 42.909 8 34.729H2L18.8 62 62 2z"/></svg>"##
        }
        "gray-x" => {
            r##"<svg width="132" height="132" viewBox="0 0 64 64" aria-hidden="true"><path fill="#4D5357" d="M62 10.571L53.429 2 32 23.429 10.571 2 2 10.571 23.429 32 2 53.429 10.571 62 32 40.571 53.429 62 62 53.429 40.571 32z"/></svg>"##
        }
        "stopsign" => {
            r##"<svg width="132" height="132" viewBox="0 0 64 64" aria-hidden="true"><path fill="#E9EDF2" d="M64 45.254L45.254 64H18.747L0 45.254V18.747L18.747 0h26.507L64 18.747z"/><path fill="#ED4C5C" d="M58 42.768L42.769 58H21.231L6 42.768V21.231L21.231 6h21.538L58 21.231z"/></svg>"##
        }
        "question" => {
            r##"<svg width="132" height="132" viewBox="0 0 64 64" aria-hidden="true"><g fill-rule="evenodd" clip-rule="evenodd" fill="#A6AEB0"><path d="M30.249 2.065C18.612 2.789 12.531 9.379 12 21.296h11.739c.147-4.128 2.451-7.214 6.741-7.669 4.211-.447 8.206.556 9.416 3.435 1.307 3.11-1.627 6.724-3.022 8.241-2.582 2.813-6.776 4.865-8.95 7.9-2.131 2.974-2.51 6.887-2.674 11.676h10.346c.145-3.062.349-5.995 1.742-7.898 2.266-3.092 5.65-4.541 8.486-6.983 2.709-2.334 5.559-5.147 6.043-9.501C53.319 7.466 42.683 1.289 30.249 2.065z"/><ellipse cx="30.515" cy="55.567" rx="6.532" ry="6.433"/></g></svg>"##
        }
        _ => {
            r##"<svg width="132" height="132" viewBox="0 0 64 64" aria-hidden="true"><path fill="#FF5A79" d="M62 10.571L53.429 2 32 23.429 10.571 2 2 10.571 23.429 32 2 53.429 10.571 62 32 40.571 53.429 62 62 53.429 40.571 32z"/></svg>"##
        }
    }
}

fn xml_escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn format_timestamp(ts: i64) -> String {
    let secs = ts % 86400;
    let days = ts / 86400;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let j = days + 2440588;
    let f = j + 1401 + (((4 * j + 274277) / 146097) * 3) / 4 - 38;
    let e = 4 * f + 3;
    let g = (e % 1461) / 4;
    let dg = 5 * g + 2;
    let day = (dg % 153) / 5 + 1;
    let month = (dg / 153 + 2) % 12 + 1;
    let year = e / 1461 - 4716 + (14 - month) / 12;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        year, month, day, h, m, s
    )
}

fn format_duration(secs: i64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else {
        format!("{}m {}s", secs / 60, secs % 60)
    }
}

fn format_size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * 1024;
    const GIB: u64 = 1024 * 1024 * 1024;

    if bytes >= GIB {
        format!("{:.1}GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1}MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        if bytes % KIB == 0 {
            format!("{}KiB", bytes / KIB)
        } else {
            format!("{:.1}KiB", bytes as f64 / KIB as f64)
        }
    } else {
        format!("{bytes}B")
    }
}

fn best_product(products: &HashMap<String, BuildProduct>) -> (String, String) {
    let Some(product) = products
        .values()
        .max_by_key(|product| product.filesize.unwrap_or(0))
    else {
        return ("—".to_string(), "—".to_string());
    };

    let type_label = match &product.subtype {
        Some(subtype) if !subtype.is_empty() => format!("{} / {}", product.kind, subtype),
        _ if !product.kind.is_empty() => product.kind.clone(),
        _ => "—".to_string(),
    };
    let size_label = product
        .filesize
        .map(format_size)
        .unwrap_or_else(|| "—".to_string());

    (type_label, size_label)
}

fn headline_font_size(s: &str) -> u32 {
    // ~880px available (separator at x=1088 minus section start at x=196)
    // Helvetica avg char width ≈ 0.55 × font_size
    let ideal = 880.0 / (s.len() as f64 * 0.55);
    (ideal.floor() as u32).clamp(14, 38)
}

fn render_svg(build: &HydraBuild) -> String {
    let status = status_info(build);
    let icon = icon_svg(status.icon_variant);
    let headline_size = headline_font_size(&build.job);
    let evaluation = build
        .jobsetevals
        .as_ref()
        .and_then(|evals| evals.first().copied())
        .map(|eval| eval.to_string())
        .unwrap_or_else(|| "—".to_string());
    let priority = build
        .priority
        .map(|value| value.to_string())
        .unwrap_or_else(|| "—".to_string());
    let nix_name = build
        .nixname
        .clone()
        .unwrap_or_else(|| "—".to_string());
    let started = build
        .starttime
        .map(format_timestamp)
        .unwrap_or_else(|| "—".to_string());
    let finished = build
        .stoptime
        .map(format_timestamp)
        .unwrap_or_else(|| "—".to_string());
    let duration = match (build.starttime, build.stoptime) {
        (Some(start), Some(stop)) if stop >= start => format_duration(stop - start),
        _ => "—".to_string(),
    };
    let queued_at = format_timestamp(build.timestamp);
    let (product, size) = build
        .buildproducts
        .as_ref()
        .map(best_product)
        .unwrap_or_else(|| ("—".to_string(), "—".to_string()));

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1800" height="300" viewBox="0 0 1800 300" role="img" aria-label="Hydra build #{id} banner">
<style>
text {{ font-family: Helvetica, Arial, sans-serif; fill: #222; }}
.muted {{ fill: #777; }}
.label {{ font-size: 22px; font-weight: 700; }}
.value {{ font-size: 22px; }}
.mono {{ font-family: "DejaVu Sans Mono", "SFMono-Regular", Consolas, monospace; font-size: 18px; }}
.headline {{ font-size: 38px; font-weight: 700; }}
.build-id {{ font-size: 24px; font-weight: 700; }}
</style>
<rect width="1800" height="300" fill="#f5f5f5"/>
<rect x="10" y="10" width="1780" height="280" rx="10" fill="#fff" stroke="#d9d9d9"/>
<line x1="1088" y1="44" x2="1088" y2="256" stroke="#e1e1e1"/>
<line x1="1422" y1="44" x2="1422" y2="256" stroke="#e1e1e1"/>
<g transform="translate(34,72)">
  {icon}
  <text x="66" y="172" text-anchor="middle" class="build-id">#{id}</text>
</g>
<g transform="translate(196,54)">
  <text x="0" y="0" style="font-size:{headline_size}px;font-weight:700">{job}</text>
  <text x="0" y="42" style="font-size:30px;font-weight:700;fill:{color}">{status_label}</text>
  <text x="0" y="92" class="label muted">Project / Jobset</text>
  <text x="0" y="122" class="value">{project} / {jobset}</text>
  <text x="360" y="92" class="label muted">System</text>
  <text x="360" y="122" class="value">{system}</text>
  <text x="590" y="92" class="label muted">Evaluation</text>
  <text x="590" y="122" class="value">{evaluation}</text>
  <text x="770" y="92" class="label muted">Priority</text>
  <text x="770" y="122" class="value">{priority}</text>
  <text x="0" y="168" class="label muted">Nix Name</text>
  <text x="0" y="196" class="mono">{nix_name}</text>
</g>
<g transform="translate(1118,58)">
  <text x="0" y="0" class="label muted">Started</text>
  <text x="0" y="30" class="value">{started}</text>
  <text x="0" y="78" class="label muted">Finished</text>
  <text x="0" y="108" class="value">{finished}</text>
  <text x="0" y="156" class="label muted">Duration</text>
  <text x="0" y="186" class="value">{duration}</text>
</g>
<g transform="translate(1450,58)">
  <text x="0" y="0" class="label muted">Queued At</text>
  <text x="0" y="30" class="value">{queued_at}</text>
  <text x="0" y="78" class="label muted">Product</text>
  <text x="0" y="108" class="value">{product}</text>
  <text x="0" y="156" class="label muted">Size</text>
  <text x="0" y="186" class="value">{size}</text>
</g>
</svg>"##,
        id = build.id,
        icon = icon,
        headline_size = headline_size,
        job = xml_escape(&build.job),
        color = status.color,
        status_label = status.label,
        project = xml_escape(&build.project),
        jobset = xml_escape(&build.jobset),
        system = xml_escape(&build.system),
        evaluation = xml_escape(&evaluation),
        priority = xml_escape(&priority),
        nix_name = xml_escape(&nix_name),
        started = xml_escape(&started),
        finished = xml_escape(&finished),
        duration = xml_escape(&duration),
        queued_at = xml_escape(&queued_at),
        product = xml_escape(&product),
        size = xml_escape(&size),
    )
}

fn render_error_svg(message: &str) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="60" viewBox="0 0 400 60" role="img" aria-label="Hydra banner error"><rect width="400" height="60" fill="#fff5f5"/><text x="12" y="36" font-family="Helvetica, Arial, sans-serif" font-size="20" fill="#c62828">Error: {message}</text></svg>"##,
        message = xml_escape(message)
    )
}
