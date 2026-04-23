use std::time::{SystemTime, UNIX_EPOCH};

use askama::Template;

use crate::{
    format::format_timestamp,
    hydra::{HydraBuild, HydraError},
    status::{icon_svg, status_info},
};

const DASH: &str = "—";

fn ts_or_dash(ts: Option<i64>) -> String {
    ts.map(format_timestamp).unwrap_or_else(|| DASH.into())
}

#[derive(Template)]
#[template(path = "banner.svg", escape = "html")]
pub struct BannerTemplate {
    pub id: u64,
    pub job: String,
    pub color: &'static str,
    pub status_label: &'static str,
    pub icon: &'static str,
    pub headline_size: u32,
    pub project: String,
    pub jobset: String,
    pub system: String,
    pub evaluation: String,
    pub started: String,
    pub finished: String,
    pub queued_at: String,
    pub generated_at: String,
}

#[derive(Template)]
#[template(path = "error_banner.svg", escape = "html")]
pub struct ErrorBannerTemplate {
    pub line1: String,
    pub line2: String,
}

pub fn render_svg(build: &HydraBuild) -> BannerTemplate {
    let status = status_info(build);
    let evaluation = build
        .jobsetevals
        .as_ref()
        .and_then(|evals| evals.first().copied())
        .map_or_else(|| DASH.into(), |eval| eval.to_string());
    let generated_at = ts_or_dash(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_secs() as i64),
    );
    BannerTemplate {
        id: build.id,
        job: build.job.clone(),
        color: status.color,
        status_label: status.label,
        icon: icon_svg(status.icon),
        headline_size: headline_font_size(&build.job),
        project: build.project.clone(),
        jobset: build.jobset.clone(),
        system: build.system.clone(),
        evaluation,
        started: ts_or_dash(build.starttime),
        finished: ts_or_dash(build.stoptime),
        queued_at: format_timestamp(build.timestamp),
        generated_at,
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect::<String>() + "…"
    }
}

pub fn render_error_svg(err: &HydraError) -> ErrorBannerTemplate {
    match err {
        HydraError::NotFound(id) => ErrorBannerTemplate {
            line1: format!("Build #{id} does not exist. "),
            line2: format!("Visit https://hydra.nixos.org/build/{id} to confirm."),
        },
        HydraError::InvalidId(raw) => ErrorBannerTemplate {
            line1: format!("Invalid build ID: {:?}", truncate(raw, 24)),
            line2: "Build IDs must be positive i32 integers".to_string(),
        },
        HydraError::Other(msg) => ErrorBannerTemplate {
            line1: truncate(msg, 48),
            line2: "Visit https://hydra.nixos.org for more information.".to_string(),
        },
    }
}

fn headline_font_size(s: &str) -> u32 {
    let ideal = 1140.0 / (s.len() as f64 * 0.50);
    (ideal.floor() as u32).clamp(20, 46)
}

