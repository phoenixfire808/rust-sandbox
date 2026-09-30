//! Surface-dependent grounded footsteps using the existing one-shot audio backend.
//! Unknown collider metadata is intentionally silent.
use crate::{
    source_assets::SurfaceMaterial,
    source_play::{self, PlayState},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use sandbox_catalog::footsteps::FootstepCatalog;

#[derive(Resource)]
struct ActiveFootsteps(FootstepCatalog);
#[derive(Resource, Default)]
struct Cadence {
    last_position: Option<Vec3>,
    elapsed: f32,
    left: bool,
}
struct EmitFootstep(String, Vec3, f32);
impl Command for EmitFootstep {
    fn apply(self, world: &mut World) {
        source_play::audio::emit(world, &self.0, Some(self.1), self.2);
    }
}
pub struct SourceFootstepsPlugin;
impl Plugin for SourceFootstepsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActiveFootsteps(crate::compiled_footstep_catalog()))
            .init_resource::<Cadence>()
            .add_systems(
                FixedUpdate,
                footsteps.after(super::source_player::finish_walk),
            );
    }
}
fn footsteps(
    time: Res<Time<Fixed>>,
    play: Res<PlayState>,
    player: Option<Res<super::source_player::PlayerState>>,
    frontend: Option<Res<crate::source_frontend::Frontend>>,
    life: Option<Res<source_play::npcs::PlayerLife>>,
    remote: Option<Res<source_play::tools::devices::Remote>>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    body: Query<&Transform, With<super::source_player::PlayerBody>>,
    surfaces: Query<&SurfaceMaterial>,
    rapier: ReadRapierContext,
    catalog: Res<ActiveFootsteps>,
    mut cadence: ResMut<Cadence>,
    mut commands: Commands,
) {
    let Some(player) = player else { return };
    let Ok(transform) = body.get(player.entity) else {
        return;
    };
    let now = transform.translation;
    let dt = time.delta_secs();
    let focused = window.single().is_ok_and(|w| w.focused);
    let eligible = focused
        && player.grounded
        && !player.noclip
        && player.vehicle.is_none()
        // No validated wading surface behavior exists yet, so suppress all wet steps.
        && player.water_level == 0
        && !play.menu_open
        && !play.search_focus
        && !remote.is_some_and(|r| r.0.is_some())
        && !frontend.as_ref().is_some_and(|f| f.is_open())
        && !life.is_some_and(|l| l.health <= 0.0);
    let Some(previous) = cadence.last_position.replace(now) else {
        cadence.elapsed = 0.0;
        return;
    };
    if !eligible || !dt.is_finite() || dt <= 0.0 || dt > 0.25 {
        cadence.elapsed = 0.0;
        return;
    }
    let planar_distance = Vec2::new(now.x - previous.x, now.z - previous.z).length();
    let speed = planar_distance / dt.max(0.001);
    let hull_height = if player.crouched {
        player.config.crouch_height
    } else {
        player.config.height
    };
    let Some(surface) = catalog.0.surfaces.get(&player_surface(
        &rapier,
        &surfaces,
        player.entity,
        now,
        hull_height,
        player.config.step_height,
    )) else {
        cadence.elapsed = 0.0;
        return;
    };
    if speed < surface.min_speed {
        cadence.elapsed = 0.0;
        return;
    }
    // A discontinuous transform change (respawn, teleport, set-pos) is not a stride.
    if speed > surface.run_speed * 1.5 {
        cadence.elapsed = 0.0;
        return;
    }
    cadence.elapsed += dt;
    let mut interval = if speed >= surface.run_speed {
        surface.run_interval
    } else {
        surface.walk_interval
    };
    if player.crouched {
        interval *= surface.crouch_interval_multiplier;
    }
    if cadence.elapsed < interval {
        return;
    }
    cadence.elapsed %= interval;
    let id = if cadence.left {
        &surface.left_event
    } else {
        &surface.right_event
    };
    cadence.left = !cadence.left;
    commands.queue(EmitFootstep(
        id.clone(),
        now - Vec3::Y * (hull_height * 0.5),
        surface.gain,
    ));
}
fn player_surface(
    rapier: &ReadRapierContext,
    surfaces: &Query<&SurfaceMaterial>,
    player: Entity,
    position: Vec3,
    hull_height: f32,
    step_height: f32,
) -> String {
    let Ok(context) = rapier.single() else {
        return String::new();
    };
    let origin = position - Vec3::Y * (hull_height * 0.5) + Vec3::Y * 0.05;
    let filter = QueryFilter::default()
        .exclude_collider(player)
        .exclude_sensors();
    let Some((entity, _)) = context.cast_ray(
        origin,
        Vec3::NEG_Y,
        step_height.max(0.1) + 0.1,
        true,
        filter,
    ) else {
        return String::new();
    };
    surfaces
        .get(entity)
        .map(|m| m.surface_prop.trim().to_ascii_lowercase())
        .unwrap_or_default()
}
