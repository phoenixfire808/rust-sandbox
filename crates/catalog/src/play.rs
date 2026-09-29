use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlayConfig {
    pub id: String,
    pub physgun_model: String,
    pub toolgun_model: String,
    pub default_prop: String,
    pub spawn_distance: f32,
    pub hold_gain: f32,
    pub hold_max_speed: f32,
    pub max_props: usize,
    pub page_size: usize,
    pub sensitivity: f32,
    pub gravity: f32,
    pub prop_mass: f32,
    pub view_x: f32,
    pub view_y: f32,
    pub view_z: f32,
}
pub fn load(path: &Path) -> Result<PlayConfig> {
    let mut rows: Vec<PlayConfig> = csv::Reader::from_path(path.join("source_play.csv"))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?;
    if rows.len() != 1 {
        return Err("source_play.csv requires one configuration".into());
    }
    let c = rows.remove(0);
    for p in [&c.physgun_model, &c.toolgun_model, &c.default_prop] {
        if !p.starts_with("models/")
            || !p.ends_with(".mdl")
            || p.contains("..")
            || p.contains('\\')
            || p.contains(':')
        {
            return Err("invalid model reference".into());
        }
    }
    if !(1..=1000).contains(&c.max_props) || !(1..=256).contains(&c.page_size) {
        return Err("invalid prop/page limits".into());
    }
    for (v, min, max) in [
        (c.spawn_distance, 1., 30.),
        (c.hold_gain, 1., 60.),
        (c.hold_max_speed, 1., 100.),
        (c.sensitivity, 0.0001, 0.05),
        (c.gravity, -50., 0.),
        (c.prop_mass, 0.1, 10000.),
        (c.view_x, -2., 2.),
        (c.view_y, -2., 2.),
        (c.view_z, -2., 0.),
    ] {
        if !v.is_finite() || !(min..=max).contains(&v) {
            return Err("invalid source_play numeric value".into());
        }
    }
    Ok(c)
}
pub fn generate(c: &PlayConfig) -> String {
    format!("pub fn compiled_play_config()->sandbox_catalog::play::PlayConfig {{ sandbox_catalog::play::PlayConfig {{id:{:?}.into(),physgun_model:{:?}.into(),toolgun_model:{:?}.into(),default_prop:{:?}.into(),spawn_distance:{:?},hold_gain:{:?},hold_max_speed:{:?},max_props:{},page_size:{},sensitivity:{:?},gravity:{:?},prop_mass:{:?},view_x:{:?},view_y:{:?},view_z:{:?}}} }}\n",c.id,c.physgun_model,c.toolgun_model,c.default_prop,c.spawn_distance,c.hold_gain,c.hold_max_speed,c.max_props,c.page_size,c.sensitivity,c.gravity,c.prop_mass,c.view_x,c.view_y,c.view_z)
}
