use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub stream_url: String,
}

pub fn load() -> anyhow::Result<Vec<Station>> {
    let raw = include_str!("../stations.json");
    Ok(serde_json::from_str(raw)?)
}
