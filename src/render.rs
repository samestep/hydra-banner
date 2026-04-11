use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use askama::Template;

use crate::{
    format::{format_duration, format_size, format_timestamp},
    hydra::{BuildProduct, HydraBuild},
    status::{icon_svg, status_info},
};

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
    pub priority: String,
    pub nix_name: String,
    pub started: String,
    pub finished: String,
    pub duration: String,
    pub queued_at: String,
    pub product: String,
    pub size: String,
    pub generated_at: String,
}

#[derive(Template)]
#[template(path = "error_banner.svg", escape = "html")]
pub struct ErrorBannerTemplate {
    pub message: String,
}

pub fn render_svg(build: &HydraBuild) -> BannerTemplate {
    let status = status_info(build);
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
    let nix_name = build.nixname.clone().unwrap_or_else(|| "—".to_string());
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
    let generated_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| format_timestamp(duration.as_secs() as i64))
        .unwrap_or_else(|| "—".to_string());
    let (product, size) = build
        .buildproducts
        .as_ref()
        .map(best_product)
        .unwrap_or_else(|| ("—".to_string(), "—".to_string()));

    BannerTemplate {
        id: build.id,
        job: build.job.clone(),
        color: status.color,
        status_label: status.label,
        icon: icon_svg(status.icon_variant),
        headline_size: headline_font_size(&build.job),
        project: build.project.clone(),
        jobset: build.jobset.clone(),
        system: build.system.clone(),
        evaluation,
        priority,
        nix_name,
        started,
        finished,
        duration,
        queued_at,
        product,
        size,
        generated_at,
    }
}

pub fn render_error_svg(message: &str) -> ErrorBannerTemplate {
    ErrorBannerTemplate {
        message: message.to_string(),
    }
}

pub fn headline_font_size(s: &str) -> u32 {
    let ideal = 1140.0 / (s.len() as f64 * 0.50);
    (ideal.floor() as u32).clamp(20, 46)
}

pub fn best_product(products: &HashMap<String, BuildProduct>) -> (String, String) {
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
