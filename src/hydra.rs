use std::collections::HashMap;

use axum::http::header;
use reqwest::Client;
use serde::Deserialize;

// Please set a proper User-Agent when scraping hydra.nixos.org, so we can reach out to you, when you are overloading the machine! --Hexa
pub const HYDRA_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/MiniHarinn/hydra-banner)"
);

#[derive(Debug)]
pub enum HydraError {
    NotFound(u64),
    InvalidId(String),
    Other(String),
}

#[derive(Debug, Deserialize)]
pub struct HydraBuild {
    pub id: u64,
    #[serde(default)]
    pub job: String,
    #[serde(default)]
    pub project: String,
    #[serde(default)]
    pub jobset: String,
    #[serde(default)]
    pub system: String,
    #[serde(default)]
    pub finished: u8,
    #[serde(default)]
    pub buildstatus: Option<i32>,
    #[serde(default)]
    pub timestamp: i64,
    #[serde(default)]
    pub starttime: Option<i64>,
    #[serde(default)]
    pub stoptime: Option<i64>,
    #[serde(default)]
    pub nixname: Option<String>,
    #[serde(default)]
    pub priority: Option<i64>,
    #[serde(default)]
    pub jobsetevals: Option<Vec<u64>>,
    #[serde(default)]
    pub buildproducts: Option<HashMap<String, BuildProduct>>,
}

#[derive(Debug, Deserialize)]
pub struct BuildProduct {
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub subtype: Option<String>,
    #[serde(default)]
    pub filesize: Option<u64>,
}

pub async fn fetch_build(client: &Client, id: u64) -> Result<HydraBuild, HydraError> {
    // Hydra's build IDs are PostgreSQL `serial` (i32). Any id beyond i32::MAX
    // causes a DB overflow on Hydra's side and returns 500 instead of 404.
    if id > i32::MAX as u64 {
        return Err(HydraError::NotFound(id));
    }

    let url = format!("https://hydra.nixos.org/build/{id}");
    let response = client
        .get(url)
        .header(header::ACCEPT, "application/json")
        .header(header::USER_AGENT, HYDRA_USER_AGENT)
        .send()
        .await
        .map_err(|err| HydraError::Other(format!("request to Hydra failed: {err}")))?;

    let status = response.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(HydraError::NotFound(id));
    }

    if !status.is_success() {
        return Err(HydraError::Other(format!(
            "Hydra returned HTTP {}",
            status.as_u16()
        )));
    }

    response
        .json::<HydraBuild>()
        .await
        .map_err(|err| HydraError::Other(format!("invalid Hydra JSON: {err}")))
}
