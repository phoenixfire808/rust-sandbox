use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Independent sandbox tuning for the Source-inspired gravity gun subset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GravityConfig {
    pub punt_range: f32,
    pub pull_range: f32,
    pub hold_range: f32,
    pub hold_distance: f32,
    pub punt_mass_limit: f32,
    pub hold_mass_limit: f32,
    pub punt_impulse: f32,
    pub pull_speed: f32,
    pub hold_stiffness: f32,
    pub hold_damping: f32,
    pub launch_impulse: f32,
    pub action_cooldown: f32,
}

impl GravityConfig {
    pub fn validate(&self) -> Result<()> {
        for (value, low, high) in [
            (self.punt_range, 1., 1000.),
            (self.pull_range, 1., 1000.),
            (self.hold_range, 1., 500.),
            (self.hold_distance, 0.5, 20.),
            (self.punt_mass_limit, 0.1, 10000.),
            (self.hold_mass_limit, 0.1, 10000.),
            (self.punt_impulse, 0.1, 100000.),
            (self.pull_speed, 0.1, 1000.),
            (self.hold_stiffness, 0.1, 10000.),
            (self.hold_damping, 0., 1000.),
            (self.launch_impulse, 0.1, 100000.),
            (self.action_cooldown, 0.05, 5.),
        ] {
            if !value.is_finite() || !(low..=high).contains(&value) {
                return Err("invalid gravity weapon parameter".into());
            }
        }
        if self.hold_mass_limit > self.punt_mass_limit {
            return Err("gravity hold mass limit must not exceed punt mass limit".into());
        }
        if self.hold_distance > self.hold_range || self.hold_distance > self.pull_range {
            return Err("gravity hold distance must fit within pull and hold ranges".into());
        }
        Ok(())
    }
}

pub fn load(path: &Path) -> Result<GravityConfig> {
    let mut rows: Vec<GravityConfig> = csv::Reader::from_path(path.join("source_gravity.csv"))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?;
    if rows.len() != 1 {
        return Err("source_gravity.csv requires one row".into());
    }
    let config = rows.remove(0);
    config.validate()?;
    Ok(config)
}

pub fn generate(config: &GravityConfig) -> String {
    format!(
        "pub fn compiled_gravity_config()->sandbox_catalog::gravity::GravityConfig {{ serde_json::from_str({:?}).expect(\"build-validated gravity sheet\") }}\n",
        serde_json::to_string(config).unwrap()
    )
}
