use crate::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FootstepSurface {
    pub surface_prop: String,
    pub left_event: String,
    pub right_event: String,
    pub walk_interval: f32,
    pub run_interval: f32,
    pub min_speed: f32,
    pub run_speed: f32,
    pub gain: f32,
    pub crouch_interval_multiplier: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FootstepCatalog {
    pub surfaces: BTreeMap<String, FootstepSurface>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    surface_prop: String,
    left_event: String,
    right_event: String,
    walk_interval: f32,
    run_interval: f32,
    min_speed: f32,
    run_speed: f32,
    gain: f32,
    crouch_interval_multiplier: f32,
}
pub fn load(dir: &Path) -> Result<FootstepCatalog> {
    let rows: Vec<Row> = csv::Reader::from_path(dir.join("source_footsteps.csv"))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?;
    if rows.is_empty() || rows.len() > 64 {
        return Err("source_footsteps.csv requires 1..=64 surface rows".into());
    }
    let mut surfaces = BTreeMap::new();
    for row in rows {
        let key = row.surface_prop.trim().to_ascii_lowercase();
        let surface = FootstepSurface {
            surface_prop: key.clone(),
            left_event: row.left_event,
            right_event: row.right_event,
            walk_interval: row.walk_interval,
            run_interval: row.run_interval,
            min_speed: row.min_speed,
            run_speed: row.run_speed,
            gain: row.gain,
            crouch_interval_multiplier: row.crouch_interval_multiplier,
        };
        if key.is_empty()
            || key.len() > 64
            || !key
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            || surface.left_event.is_empty()
            || surface.right_event.is_empty()
        {
            return Err("invalid footstep surface".into());
        }
        for (value, low, high) in [
            (surface.walk_interval, 0.15, 2.0),
            (surface.run_interval, 0.15, 2.0),
            (surface.min_speed, 0.01, 20.0),
            (surface.run_speed, 0.01, 30.0),
            (surface.gain, 0.0, 1.0),
            (surface.crouch_interval_multiplier, 1.0, 2.0),
        ] {
            if !value.is_finite() || !(low..=high).contains(&value) {
                return Err("invalid footstep tuning".into());
            }
        }
        if surface.run_interval > surface.walk_interval
            || surface.run_speed < surface.min_speed
            || surfaces.insert(key, surface).is_some()
        {
            return Err("duplicate or inconsistent footstep surface".into());
        }
    }
    let audio = crate::audio::load(dir)?;
    for surface in surfaces.values() {
        if !audio
            .events
            .iter()
            .any(|event| event.id == surface.left_event)
            || !audio
                .events
                .iter()
                .any(|event| event.id == surface.right_event)
        {
            return Err(format!(
                "footstep event missing from source_audio_events.csv for {}",
                surface.surface_prop
            )
            .into());
        }
    }
    Ok(FootstepCatalog { surfaces })
}
pub fn generate(catalog: &FootstepCatalog) -> String {
    format!(
        "pub fn compiled_footstep_catalog()->sandbox_catalog::footsteps::FootstepCatalog {{ serde_json::from_str({:?}).expect(\"build-validated footstep sheet\") }}\n",
        serde_json::to_string(catalog).unwrap()
    )
}
