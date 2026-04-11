use std::collections::HashMap;

use axum::http::header;
use reqwest::Client;
use serde::Deserialize;

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

pub async fn fetch_build(client: &Client, id: u64) -> Result<HydraBuild, String> {
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
