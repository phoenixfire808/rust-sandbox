use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceConfig {
    pub spawn_queue_limit: usize,
    pub report_seconds: f32,
    pub frame_budget_ms: f32,
}
pub fn load(path: &Path) -> Result<PerformanceConfig> {
    let mut rows: Vec<PerformanceConfig> =
        csv::Reader::from_path(path.join("source_performance.csv"))?
            .deserialize()
            .collect::<std::result::Result<_, _>>()?;
    if rows.len() != 1 {
        return Err("source_performance.csv requires one row".into());
    }
    let c = rows.remove(0);
    if !(1..=64).contains(&c.spawn_queue_limit)
        || !c.report_seconds.is_finite()
        || !(1.0..=60.0).contains(&c.report_seconds)
        || !c.frame_budget_ms.is_finite()
        || !(1.0..=1000.0).contains(&c.frame_budget_ms)
    {
        return Err("invalid performance settings".into());
    }
    Ok(c)
}
pub fn generate(c: &PerformanceConfig) -> String {
    format!("pub fn compiled_performance_config()->sandbox_catalog::performance::PerformanceConfig {{ serde_json::from_str({:?}).expect(\"build-validated performance settings\") }}\n", serde_json::to_string(c).unwrap())
}
