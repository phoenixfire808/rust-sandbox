//! Independent skateboard presets, not extracted commercial-game constants.
use crate::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardConfig {
    pub length: f32,
    pub width: f32,
    pub thickness: f32,
    pub wheelbase: f32,
    pub track: f32,
    pub wheel_radius: f32,
    pub mass: f32,
    pub rider_mass: f32,
    pub suspension: f32,
    pub spring: f32,
    pub damping: f32,
    pub max_load: f32,
    pub steering_angle: f32,
    pub steering_response: f32,
    pub rolling_resistance: f32,
    pub brake_accel: f32,
    pub max_speed: f32,
    pub charge_seconds: f32,
    pub pop_cooldown: f32,
    pub deadzone: f32,
    pub flick_threshold: f32,
    pub manual_angle: f32,
    pub balance_torque: f32,
    pub balance_damping: f32,
    pub rail_length: f32,
    pub rail_width: f32,
    pub rail_height: f32,
    pub bank_length: f32,
    pub bank_width: f32,
    pub bank_angle: f32,
    pub spawn_distance: f32,
    pub enter_seconds: f32,
    pub exit_seconds: f32,
    pub exit_distance: f32,
    pub eye_height: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub label: String,
    pub controls: String,
    pub gravity: f32,
    pub grip: f32,
    pub slide_grip: f32,
    pub push_delta: f32,
    pub push_interval: f32,
    pub push_limit: f32,
    pub pop_min: f32,
    pub pop_max: f32,
    pub air_turn: f32,
    pub flip_rate: f32,
    pub catch_assist: f32,
    pub landing_cos: f32,
    pub landing_speed: f32,
    pub manual_assist: f32,
    pub grind_assist: f32,
    pub scope: String,
    pub remaining: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkateCatalog {
    pub board: BoardConfig,
    pub profiles: Vec<Profile>,
}
fn rows<T: serde::de::DeserializeOwned>(dir: &Path, name: &str) -> Result<Vec<T>> {
    Ok(csv::Reader::from_path(dir.join(name))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?)
}
pub fn load(dir: &Path) -> Result<SkateCatalog> {
    let mut configs: Vec<BoardConfig> = rows(dir, "source_skateboard.csv")?;
    if configs.len() != 1 {
        return Err("one skateboard geometry row required".into());
    }
    let board = configs.remove(0);
    for (name, value) in serde_json::to_value(&board)?.as_object().unwrap() {
        let v = value.as_f64().ok_or("numeric skateboard field required")?;
        if !v.is_finite() || v <= 0. || v > 100000. {
            return Err(format!("invalid skateboard field {name}").into());
        }
    }
    if !(0.5..=1.3).contains(&board.length)
        || !(0.15..=0.4).contains(&board.width)
        || !(0.01..=0.05).contains(&board.thickness)
        || !(0.02..=0.07).contains(&board.wheel_radius)
        || board.wheelbase >= board.length
        || board.wheelbase < board.length * 0.4
        || board.track > board.width * 1.5
        || board.track < board.width * 0.5
        || !(1.0..=8.0).contains(&board.mass)
        || !(30.0..=150.0).contains(&board.rider_mass)
        || board.suspension > 0.2
        || board.steering_angle > 0.8
        || board.deadzone >= 0.5
        || !(0.5..=0.95).contains(&board.flick_threshold)
        || board.manual_angle > 0.6
        || board.bank_angle > 0.7
        || board.max_speed > 60.
        || board.rolling_resistance > 0.2
        || board.eye_height > 2.
        || board.rail_width > 0.5
        || board.rail_length > 30.
    {
        return Err("inconsistent skateboard geometry/tuning".into());
    }
    let profiles: Vec<Profile> = rows(dir, "source_skate_profiles.csv")?;
    let mut ids = BTreeSet::new();
    for p in &profiles {
        if !ids.insert(p.id.clone())
            || !["realism", "skate", "session", "sandbox"].contains(&p.id.as_str())
            || !["direct", "flick", "dual"].contains(&p.controls.as_str())
            || p.label.is_empty()
            || p.label.len() > 80
            || p.scope.is_empty()
            || p.remaining.is_empty()
        {
            return Err("invalid skateboard profile identity".into());
        }
        for (v, low, high) in [
            (p.gravity, 5., 15.),
            (p.grip, 0.1, 3.),
            (p.slide_grip, 0.01, 1.),
            (p.push_delta, 0.1, 4.),
            (p.push_interval, 0.2, 2.),
            (p.push_limit, 1., board.max_speed),
            (p.pop_min, 0.1, 6.),
            (p.pop_max, p.pop_min, 8.),
            (p.air_turn, 0., 8.),
            (p.flip_rate, 1., 20.),
            (p.catch_assist, 0., 1.),
            (p.landing_cos, 0.1, 0.99),
            (p.landing_speed, 2., 20.),
            (p.manual_assist, 0., 1.),
            (p.grind_assist, 0., 1.),
        ] {
            if !v.is_finite() || v < low || v > high {
                return Err(format!("invalid skateboard profile {}", p.id).into());
            }
        }
        if p.slide_grip > p.grip {
            return Err("slide grip exceeds rolling grip".into());
        }
    }
    if ids.len() != 4 {
        return Err("all four skateboard presets required".into());
    }
    Ok(SkateCatalog { board, profiles })
}
pub fn generate(c: &SkateCatalog) -> String {
    format!(
        "pub fn compiled_skate_catalog()->sandbox_catalog::skateboard::SkateCatalog {{ serde_json::from_str({:?}).expect(\"build-validated skateboard sheets\") }}\n",
        serde_json::to_string(c).unwrap()
    )
}
