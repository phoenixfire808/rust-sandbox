//! Single-player physgun controls. Native ownership and solver parity remain open.
use super::*;
#[derive(Resource)]
pub(crate) struct State {
    pub config: sandbox_catalog::effects::EffectsConfig,
    pub color: [f32; 3],
    pub claws: f32,
    last_reload: Option<f32>,
    beam: bool,
    pub blocked_until_release: bool,
    rotation: Option<(Entity, Quat)>,
}
impl Default for State {
    fn default() -> Self {
        let config = crate::compiled_effects_config();
        let color = std::fs::read(project_root().join("local/physgun-color.json"))
            .ok()
            .and_then(|b| serde_json::from_slice::<[f32; 3]>(&b).ok())
            .filter(|v| v.iter().all(|x| x.is_finite() && (0. ..=1.).contains(x)))
            .unwrap_or([config.red, config.green, config.blue]);
        Self {
            config,
            color,
            claws: 0.,
            last_reload: None,
            beam: false,
            blocked_until_release: true,
            rotation: None,
        }
    }
}
pub(crate) fn color(world: &mut World, axis: usize, delta: f32) {
    let mut s = world.resource_mut::<State>();
    s.color[axis] = (s.color[axis] + delta).clamp(0., 1.);
    let value = s.color;
    let root = world.resource::<PlayState>().storage_root.clone();
    let result = (|| -> Result<()> {
        std::fs::create_dir_all(&root)?;
        std::fs::write(root.join("physgun-color.json"), serde_json::to_vec(&value)?)?;
        Ok(())
    })();
    if let Err(e) = result {
        world.resource_mut::<PlayState>().status = format!("Physgun color save failed: {e}");
    }
}
pub(crate) fn reset(world: &mut World) {
    let mut s = world.resource_mut::<State>();
    s.last_reload = None;
    s.beam = false;
    s.claws = 0.;
    s.rotation = None;
    s.blocked_until_release = true;
}
pub(crate) fn reload(world: &mut World, target: Option<Entity>) {
    let now = world.resource::<Time>().elapsed_secs();
    let all = {
        let mut s = world.resource_mut::<State>();
        let all = s
            .last_reload
            .is_some_and(|t| now - t <= s.config.reload_double_seconds);
        s.last_reload = if all { None } else { Some(now) };
        all
    };
    let ids: Vec<_> = world
        .query::<(Entity, &RigidBody, &SpawnedProp)>()
        .iter(world)
        .filter(|(e, b, _)| {
            **b == RigidBody::Fixed
                && (all || Some(*e) == target)
                && world.get::<npcs::NpcBody>(*e).is_none()
        })
        .map(|(e, _, _)| e)
        .collect();
    if ids.is_empty() {
        return;
    }
    remember(world);
    for e in &ids {
        world
            .entity_mut(*e)
            .insert((RigidBody::Dynamic, Sleeping::default()));
    }
    world.resource_mut::<PlayState>().status = format!("Unfroze {} object(s)", ids.len());
    audio::emit(world, "physgun.unfreeze", None, 1.);
}
pub(crate) fn rotate(world: &mut World, e: Entity, cam: Transform, delta: Vec2, snap: bool) {
    let current = world.get::<Transform>(e).unwrap().rotation;
    let raw = {
        let mut s = world.resource_mut::<State>();
        let prior = s
            .rotation
            .filter(|(id, _)| *id == e)
            .map_or(current, |(_, q)| q);
        let raw = (Quat::from_axis_angle(*cam.right(), -delta.y)
            * Quat::from_rotation_y(-delta.x)
            * prior)
            .normalize();
        s.rotation = Some((e, raw));
        raw
    };
    let rotation = if snap {
        let step = world.resource::<State>().config.snap_degrees.to_radians();
        let (y, x, z) = raw.to_euler(EulerRot::YXZ);
        Quat::from_euler(
            EulerRot::YXZ,
            (y / step).round() * step,
            (x / step).round() * step,
            (z / step).round() * step,
        )
    } else {
        raw
    };
    let anchor = world.resource::<PlayState>().held_anchor;
    let mut t = world.get_mut::<Transform>(e).unwrap();
    let pivot = t.translation + t.rotation * anchor;
    t.rotation = rotation;
    t.translation = pivot - rotation * anchor;
}
pub(crate) fn update(world: &mut World) {
    let p = world.resource::<PlayState>();
    let blocked = crate::source_frontend::active(world)
        || p.menu_open
        || !p.physgun
        || p.tools.input_blocked
        || npcs::dead(world)
        || world.resource::<vehicles::Occupancy>().vehicle.is_some()
        || tools::devices::viewing(world);
    if blocked {
        reset(world);
        return;
    }
    let beam = p.beam_active;
    let rotating = p.held.is_some()
        && world
            .resource::<ButtonInput<KeyCode>>()
            .pressed(KeyCode::KeyE);
    let dt = world.resource::<Time>().delta_secs();
    let mut s = world.resource_mut::<State>();
    let prior = s.beam;
    s.beam = beam;
    if !rotating {
        s.rotation = None;
    }
    let target = if beam { 1. } else { 0. };
    s.claws += (target - s.claws).clamp(-s.config.claw_rate * dt, s.config.claw_rate * dt);
    if prior != beam {
        audio::emit(
            world,
            if beam { "physgun.on" } else { "physgun.off" },
            None,
            1.,
        );
    }
}
