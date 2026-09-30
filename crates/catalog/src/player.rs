use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerConfig {
    pub model: String,
    pub hands: String,
    pub animations: String,
    pub world_physgun: String,
    pub world_physgun_skin: i32,
    pub world_toolgun: String,
    pub physgun_idle: String,
    pub toolgun_idle: String,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_speed: f32,
    pub height: f32,
    pub crouch_height: f32,
    pub crouch_eye: f32,
    pub crouch_speed: f32,
    pub duck_seconds: f32,
    pub radius: f32,
    pub eye_height: f32,
    pub step_height: f32,
    pub camera_distance: f32,
    pub view_fov: f32,
    pub ground_acceleration: f32,
    pub air_acceleration: f32,
    pub ground_friction: f32,
    pub stop_speed: f32,
    pub air_speed_cap: f32,
    pub jump_boost: f32,
    pub minimum_move_speed: f32,
    pub ground_probe: f32,
    pub walkable_normal: f32,
}
pub fn load(path: &Path) -> Result<PlayerConfig> {
    let rows: Vec<PlayerConfig> = csv::Reader::from_path(path.join("source_player.csv"))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?;
    if rows.len() != 1 {
        return Err("source_player.csv requires one row".into());
    }
    let c = rows.into_iter().next().unwrap();
    for p in [
        &c.model,
        &c.hands,
        &c.animations,
        &c.world_physgun,
        &c.world_toolgun,
    ] {
        if !p.starts_with("models/")
            || !p.ends_with(".mdl")
            || p.contains("..")
            || p.contains(':')
            || p.contains('\\')
        {
            return Err("invalid player model reference".into());
        }
    }
    for (v, lo, hi) in [
        (c.walk_speed, 0.1, 20.),
        (c.run_speed, 0.1, 30.),
        (c.jump_speed, 0.1, 20.),
        (c.height, 0.5, 3.),
        (c.crouch_height, 0.5, 3.),
        (c.crouch_eye, 0.1, 3.),
        (c.crouch_speed, 0.01, 1.),
        (c.duck_seconds, 0.01, 2.),
        (c.radius, 0.05, 0.6),
        (c.eye_height, 0.3, 3.),
        (c.step_height, 0.01, 0.6),
        (c.camera_distance, 1., 10.),
        (c.view_fov, 30., 100.),
        (c.ground_acceleration, 0.1, 1000.),
        (c.air_acceleration, 0.1, 1000.),
        (c.ground_friction, 0., 100.),
        (c.stop_speed, 0.01, 30.),
        (c.air_speed_cap, 0.01, 30.),
        (c.jump_boost, 0., 1.),
        (c.minimum_move_speed, 0.001, 0.1),
        (c.ground_probe, 0.001, 0.1),
        (c.walkable_normal, 0.1, 1.),
    ] {
        if !v.is_finite() || !(lo..=hi).contains(&v) {
            return Err("invalid player parameter".into());
        }
    }
    if !(0..=255).contains(&c.world_physgun_skin) {
        return Err("invalid world weapon skin".into());
    }
    if c.radius * 2. >= c.crouch_height
        || c.crouch_height >= c.height
        || c.crouch_eye > c.crouch_height
        || c.eye_height > c.height
        || c.run_speed < c.walk_speed
    {
        return Err("inconsistent player dimensions/speeds".into());
    }
    Ok(c)
}
pub fn generate(c: &PlayerConfig) -> String {
    format!("pub fn compiled_player_config()->sandbox_catalog::player::PlayerConfig {{ serde_json::from_str({:?}).expect(\"build-validated player sheet\") }}\n",serde_json::to_string(c).unwrap())
}
