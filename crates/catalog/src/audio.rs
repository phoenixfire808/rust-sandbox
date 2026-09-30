//! Authored audio event references and bounded independent mixer policy.
use crate::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub master: f32,
    pub max_voices: usize,
    pub max_asset_bytes: usize,
    pub max_cache_bytes: usize,
    pub max_samples: usize,
    pub voice_seconds: f32,
    pub impact_force: f32,
    pub impact_speed: f32,
    pub impact_cooldown: f32,
    pub shake_gain: f32,
    pub shake_limit: f32,
    pub shake_decay: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub id: String,
    pub paths: String,
    pub volume: f32,
    pub range: f32,
    pub cooldown: f32,
    pub evidence: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Catalog {
    pub config: Config,
    pub events: Vec<Event>,
}
pub fn load(dir: &Path) -> Result<Catalog> {
    let mut rows: Vec<Config> = csv::Reader::from_path(dir.join("source_audio.csv"))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?;
    if rows.len() != 1 {
        return Err("source_audio.csv requires one row".into());
    }
    let config = rows.remove(0);
    for (v, min, max) in [
        (config.master, 0., 1.),
        (config.voice_seconds, 0.1, 60.),
        (config.impact_force, 1., 1000000.),
        (config.impact_speed, 0.01, 100.),
        (config.impact_cooldown, 0.05, 10.),
        (config.shake_gain, 0., 0.1),
        (config.shake_limit, 0., 0.05),
        (config.shake_decay, 1., 40.),
    ] {
        if !v.is_finite() || !(min..=max).contains(&v) {
            return Err("invalid audio configuration".into());
        }
    }
    if !(1..=128).contains(&config.max_voices)
        || !(1024..=16_777_216).contains(&config.max_asset_bytes)
        || !(1024..=10_000_000).contains(&config.max_samples)
        || !(config.max_asset_bytes..=268_435_456).contains(&config.max_cache_bytes)
    {
        return Err("invalid audio budgets".into());
    }
    let events: Vec<Event> = csv::Reader::from_path(dir.join("source_audio_events.csv"))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?;
    if events.is_empty() || events.len() > 256 {
        return Err("invalid audio event count".into());
    }
    let mut ids = BTreeSet::new();
    for e in &events {
        if e.id.is_empty()
            || !ids.insert(&e.id)
            || e.evidence.is_empty()
            || e.paths.split('|').count() > 8
        {
            return Err("invalid audio event identity".into());
        }
        for path in e.paths.split('|') {
            if !path.starts_with("sound/")
                || !(path.ends_with(".wav") || path.ends_with(".mp3"))
                || path.contains(':')
                || path.contains('\\')
                || path
                    .split('/')
                    .any(|p| p.is_empty() || p == ".." || p == ".")
            {
                return Err(format!("invalid audio asset {path}").into());
            }
        }
        for (v, min, max) in [
            (e.volume, 0., 1.),
            (e.range, 0.1, 1000.),
            (e.cooldown, 0., 10.),
        ] {
            if !v.is_finite() || !(min..=max).contains(&v) {
                return Err(format!("invalid audio event {}", e.id).into());
            }
        }
    }
    Ok(Catalog { config, events })
}
pub fn generate(c: &Catalog) -> String {
    format!("pub fn compiled_audio_catalog()->sandbox_catalog::audio::Catalog {{ serde_json::from_str({:?}).expect(\"build-validated audio sheets\") }}\n",serde_json::to_string(c).unwrap())
}
