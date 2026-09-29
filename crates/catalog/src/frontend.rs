use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrontendConfig {
    pub margin: f32,
    pub sidebar_width: f32,
    pub settings_width: f32,
    pub map_icon: f32,
    pub font_size: f32,
    pub row_height: f32,
    pub feedback_limit: usize,
    pub spawn_clearance: f32,
}
pub fn load(path: &Path) -> Result<FrontendConfig> {
    let rows: Vec<FrontendConfig> = csv::Reader::from_path(path.join("source_frontend.csv"))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?;
    if rows.len() != 1 {
        return Err("source_frontend needs exactly one row".into());
    }
    let c = rows.into_iter().next().unwrap();
    if [
        c.margin,
        c.sidebar_width,
        c.settings_width,
        c.map_icon,
        c.font_size,
        c.row_height,
        c.spawn_clearance,
    ]
    .iter()
    .any(|v| !v.is_finite() || *v <= 0. || *v > 1024.)
        || !(256..=32768).contains(&c.feedback_limit)
    {
        return Err("invalid frontend dimensions or feedback limit".into());
    }
    Ok(c)
}
pub fn generate(c: &FrontendConfig) -> String {
    format!("pub fn compiled_frontend_config()->sandbox_catalog::frontend::FrontendConfig {{ serde_json::from_str({:?}).expect(\"build-validated frontend\") }}\n",serde_json::to_string(c).unwrap())
}
