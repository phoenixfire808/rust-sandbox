use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EffectsConfig {
    pub glow_texture: String,
    pub beam_texture: String,
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub core_size: f32,
    pub tip_size: f32,
    pub endpoint_size: f32,
    pub beam_width: f32,
    pub beam_repeat: f32,
    pub beam_scroll: f32,
    pub pulse_hz: f32,
    pub pulse_depth: f32,
    pub reload_double_seconds: f32,
    pub snap_degrees: f32,
    pub claw_rate: f32,
    pub distance_speed: f32,
    pub prongs_shut: String,
    pub prongs_open: String,
}
impl EffectsConfig {
    pub fn validate(&self) -> Result<()> {
        for path in [&self.glow_texture, &self.beam_texture] {
            if !path.starts_with("sprites/")
                || path.contains("..")
                || path.contains(':')
                || path.contains('\\')
                || path.ends_with('/')
            {
                return Err("invalid effect texture path".into());
            }
        }
        for (value, low, high) in [
            (self.red, 0., 1.),
            (self.green, 0., 1.),
            (self.blue, 0., 1.),
            (self.core_size, 0.001, 2.),
            (self.tip_size, 0.001, 2.),
            (self.endpoint_size, 0.001, 2.),
            (self.beam_width, 0.001, 0.5),
            (self.beam_repeat, 0.01, 20.),
            (self.beam_scroll, -20., 20.),
            (self.pulse_hz, 0., 20.),
            (self.pulse_depth, 0., 0.9),
            (self.reload_double_seconds, 0.1, 1.),
            (self.snap_degrees, 1., 90.),
            (self.claw_rate, 0.1, 30.),
            (self.distance_speed, 0.1, 30.),
        ] {
            if !value.is_finite() || !(low..=high).contains(&value) {
                return Err("invalid effect parameter".into());
            }
        }
        if self.prongs_shut.is_empty() || self.prongs_open.is_empty() || self.prongs_shut.len() > 128 || self.prongs_open.len() > 128 {
            return Err("invalid physgun prong clip names".into());
        }
        Ok(())
    }
}
pub fn load(path: &Path) -> Result<EffectsConfig> {
    let mut rows: Vec<EffectsConfig> = csv::Reader::from_path(path.join("source_effects.csv"))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?;
    if rows.len() != 1 {
        return Err("source_effects.csv requires one row".into());
    }
    let config = rows.remove(0);
    config.validate()?;
    Ok(config)
}
pub fn generate(config: &EffectsConfig) -> String {
    format!("pub fn compiled_effects_config()->sandbox_catalog::effects::EffectsConfig {{ serde_json::from_str({:?}).expect(\"build-validated effects sheet\") }}\n", serde_json::to_string(config).unwrap())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_effects_validate_and_reject_invalid_parameters() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets");
        let original = load(&path).unwrap();
        assert!(generate(&original).contains("compiled_effects_config"));
        for width in [0., -1., f32::NAN, f32::INFINITY] {
            let mut c = original.clone();
            c.beam_width = width;
            assert!(c.validate().is_err());
        }
        let mut c = original;
        c.glow_texture = "sprites/../private".into();
        assert!(c.validate().is_err());
    }
}
