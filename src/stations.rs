use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    /// State/region (e.g. "New York NY") or country fallback
    #[serde(default)]
    pub city: String,
    /// Country name (kept for debugging / future use)
    #[allow(dead_code)]
    #[serde(default)]
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    pub stream_url: String,
}

pub fn load() -> anyhow::Result<Vec<Station>> {
    let raw = include_str!("../stations.json");
    Ok(serde_json::from_str(raw)?)
}
