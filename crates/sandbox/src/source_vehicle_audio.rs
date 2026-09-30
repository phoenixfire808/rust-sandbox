//! Occupant-local vehicle loop state adapter for the shared mounted-audio system.
use super::*;
use sandbox_catalog::spawn::VehicleFeedback;

fn feedback<'a>(world: &'a World, vehicle: &str) -> Option<&'a VehicleFeedback> {
    world
        .resource::<PlayState>()
        .spawn_catalog
        .vehicle_feedback
        .iter()
        .find(|row| row.vehicle == vehicle)
}

pub(super) fn steering_response(world: &World, vehicle: &str) -> f32 {
    feedback(world, vehicle)
        .map(|row| row.steering_response)
        .filter(|rate| rate.is_finite() && *rate > 0.)
        .unwrap_or(8.)
}

/// Event id and bounded playback rate consumed by source_audio's single owned loop.
pub(super) fn engine_loop(world: &World) -> Option<(String, f32)> {
    let occupancy = world.get_resource::<Occupancy>()?;
    let vehicle = occupancy.vehicle?;
    if occupancy.weight < 1. || occupancy.exit_to.is_some() {
        return None;
    }
    let id = world.get::<VehicleBody>(vehicle)?.id.as_str();
    let definition = super::definition(world, id)?;
    if definition.kind != "wheels" {
        return None;
    }
    let rule = feedback(world, id)?;
    let velocity = world.get::<Velocity>(vehicle)?.linvel;
    let transform = world.get::<Transform>(vehicle)?;
    let forward =
        transform.rotation * (Quat::from_rotation_y(definition.forward_yaw) * Vec3::NEG_Z);
    let speed = velocity.dot(forward).abs();
    if !speed.is_finite()
        || !occupancy.throttle.is_finite()
        || !definition.max_speed.is_finite()
        || definition.max_speed <= 0.
        || !rule.engine_min_pitch.is_finite()
        || !rule.engine_max_pitch.is_finite()
        || rule.engine_min_pitch > rule.engine_max_pitch
    {
        return None;
    }
    let speed_fraction = (speed / definition.max_speed.max(0.01)).clamp(0., 1.);
    let throttle_fraction = occupancy.throttle.abs().clamp(0., 1.);
    let activity = (speed_fraction * 0.8 + throttle_fraction * 0.2).clamp(0., 1.);
    let pitch = rule.engine_min_pitch + (rule.engine_max_pitch - rule.engine_min_pitch) * activity;
    if !pitch.is_finite() {
        return None;
    }
    let pitch = pitch.clamp(rule.engine_min_pitch, rule.engine_max_pitch);
    pitch
        .is_finite()
        .then(|| (rule.engine_event.clone(), pitch))
}
