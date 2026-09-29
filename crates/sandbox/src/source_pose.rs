//! Original clip selection driven by measured movement, with bounded pose crossfades.
use crate::source_animation::Clip;
use bevy::prelude::*;
use sandbox_catalog::presentation::{AnimationState, LayoutConfig};
use std::collections::BTreeMap;

pub const DIRECTIONS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
#[derive(Default)]
pub struct PosePlayback {
    key: String,
    elapsed: f32,
    phase: f32,
    transition: f32,
    from: Vec<Transform>,
    pose: Vec<Transform>,
}
impl PosePlayback {
    #[allow(clippy::too_many_arguments)]
    pub fn sample(
        &mut self,
        clips: &BTreeMap<String, Clip>,
        mapping: &AnimationState,
        layout: &LayoutConfig,
        velocity: Vec3,
        airborne: bool,
        noclip: bool,
        dt: f32,
    ) -> Vec<Transform> {
        let speed = Vec2::new(velocity.x, velocity.z).length();
        let moving = speed > layout.move_threshold && !noclip;
        let running = speed > layout.run_threshold;
        let sector = (velocity.x.atan2(-velocity.z) / std::f32::consts::FRAC_PI_4).round() as i32;
        let direction = DIRECTIONS[sector.rem_euclid(8) as usize];
        let jumping = airborne && !noclip;
        let name = if jumping {
            mapping.jump.clone()
        } else if moving {
            (if running { &mapping.run } else { &mapping.walk }).replace("{direction}", direction)
        } else {
            mapping.idle.clone()
        };
        if self.key != name {
            self.from = self.pose.clone();
            self.transition = 0.;
            self.elapsed = 0.;
            self.key = name.clone();
        }
        let clip = &clips[&name];
        self.elapsed += dt;
        self.transition += dt;
        let seconds = if moving && !jumping {
            let nominal = if running {
                mapping.run_speed
            } else {
                mapping.walk_speed
            };
            self.phase = (self.phase + dt * (speed / nominal).clamp(0., 2.) / clip.duration())
                .rem_euclid(1.);
            self.phase * clip.duration()
        } else {
            self.elapsed
        };
        let mut pose = clip.sample_mode(seconds, !jumping);
        let weight = (self.transition / layout.blend_seconds).clamp(0., 1.);
        if self.from.len() == pose.len() && weight < 1. {
            for (p, old) in pose.iter_mut().zip(&self.from) {
                p.translation = old.translation.lerp(p.translation, weight);
                p.rotation = old.rotation.slerp(p.rotation, weight);
            }
        }
        self.pose = pose.clone();
        pose
    }
}
