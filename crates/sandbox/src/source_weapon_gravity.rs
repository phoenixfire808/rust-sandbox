//! Source-inspired physcannon subset. Eligibility uses Rapier-authored/computed mass when ready.
use super::*;

#[derive(Resource)]
pub(super) struct State {
    config: sandbox_catalog::gravity::GravityConfig,
    holding: Option<Entity>,
    pulling: Option<Entity>,
    blocked: bool,
    next_action: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            config: crate::compiled_gravity_config(),
            holding: None,
            pulling: None,
            blocked: true,
            next_action: 0.,
        }
    }
}

pub(super) fn reset(world: &mut World) {
    if let Some(mut state) = world.get_resource_mut::<State>() {
        state.holding = None;
        state.pulling = None;
        state.blocked = true;
    }
}

fn eye(world: &mut World) -> Option<Transform> {
    crate::source_player::aim_eye(world).or_else(|| super::camera(world))
}

fn target(world: &mut World, range: f32) -> Option<(Entity, Vec3)> {
    let view = eye(world)?;
    let hit = super::world_hit(world, view, range)?;
    Some((hit.0, hit.1.point))
}

/// Prefer synchronized total body mass. Explicit mass components are a bounded
/// fallback while Rapier's computed component has not been produced yet.
fn body_mass(world: &World, entity: Entity) -> Option<f32> {
    if let Some(mass) = world
        .get::<ReadMassProperties>(entity)
        .map(|p| p.mass)
        .filter(|mass| mass.is_finite() && *mass > 0.)
    {
        return Some(mass);
    }
    let collider = world
        .get::<ColliderMassProperties>(entity)
        .and_then(|p| match p {
            ColliderMassProperties::Mass(mass) => Some(*mass),
            ColliderMassProperties::MassProperties(properties) => Some(properties.mass),
            ColliderMassProperties::Density(_) => None,
        });
    let additional = world
        .get::<AdditionalMassProperties>(entity)
        .and_then(|p| match p {
            AdditionalMassProperties::Mass(mass) => Some(*mass),
            AdditionalMassProperties::MassProperties(properties) => Some(properties.mass),
        });
    let mass = match (collider, additional) {
        (Some(collider), Some(additional)) => collider + additional,
        (Some(mass), None) => mass,
        _ => return None,
    };
    (mass.is_finite() && mass > 0.).then_some(mass)
}

fn eligible(world: &World, entity: Entity, max_mass: f32) -> std::result::Result<bool, ()> {
    if world.get::<SpawnedProp>(entity).is_none()
        || world.get::<RigidBody>(entity) != Some(&RigidBody::Dynamic)
        || world.get::<npcs::NpcBody>(entity).is_some()
        || world.get::<vehicles::VehicleBody>(entity).is_some()
        || world.get::<tools::devices::Device>(entity).is_some()
        || world.get::<tools::posers::State>(entity).is_some()
    {
        return Ok(false);
    }
    let Some(mass) = body_mass(world, entity) else {
        return Err(());
    };
    Ok(mass <= max_mass)
}

fn focused(world: &mut World) -> bool {
    world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .iter(world)
        .any(|window| window.focused && window.cursor_options.grab_mode != CursorGrabMode::None)
}

fn cooldown(
    world: &mut World,
    now: f32,
    seconds: f32,
    apply_weapon_cooldown: bool,
    fire_animation: bool,
) {
    world.resource_mut::<State>().next_action = now + seconds;
    if apply_weapon_cooldown || fire_animation {
        let mut weapon = world.resource_mut::<WeaponState>();
        if apply_weapon_cooldown {
            weapon.next_fire = now + seconds;
        }
        if fire_animation {
            weapon.fired_at = Some(now);
        }
    }
}

fn denied(world: &mut World, now: f32, seconds: f32, reason: &str) {
    world.resource_mut::<PlayState>().status = reason.into();
    audio::emit(world, "weapon_weapon_physcannon.denied", None, 1.);
    cooldown(world, now, seconds, false, false);
}

fn acquire(
    world: &mut World,
    entity: Entity,
    config: &sandbox_catalog::gravity::GravityConfig,
    now: f32,
) {
    world.resource_mut::<State>().holding = Some(entity);
    world.resource_mut::<PlayState>().status =
        "Gravity gun holding object | LMB launch | RMB drop".into();
    audio::emit(world, "weapon_weapon_physcannon.pull", None, 1.);
    cooldown(world, now, config.action_cooldown, true, false);
}

fn hold_point(eye: Transform, half: Vec3, configured_distance: f32) -> Vec3 {
    let clearance = half.abs().length() + 0.75;
    eye.translation + *eye.forward() * configured_distance.max(clearance).max(2.5)
}

fn wake(world: &mut World, entity: Entity) {
    if world.get::<Sleeping>(entity).is_some() {
        world.entity_mut(entity).insert(Sleeping::default());
    }
}

fn tick_held(
    world: &mut World,
    entity: Entity,
    config: &sandbox_catalog::gravity::GravityConfig,
    now: f32,
) {
    if !matches!(eligible(world, entity, config.hold_mass_limit), Ok(true)) {
        world.resource_mut::<State>().holding = None;
        world.resource_mut::<PlayState>().status = "Gravity gun released invalid target".into();
        return;
    }
    let Some(eye) = eye(world) else { return };
    let Some(half) = world.get::<SpawnedProp>(entity).map(|prop| prop.half) else {
        return;
    };
    let desired = hold_point(eye, half, config.hold_distance);
    let Some(position) = world.get::<Transform>(entity).map(|t| t.translation) else {
        world.resource_mut::<State>().holding = None;
        return;
    };
    if eye.translation.distance(position) > config.hold_range {
        world.resource_mut::<State>().holding = None;
        world.resource_mut::<PlayState>().status = "Gravity gun released: out of range".into();
        return;
    }
    // Refuse to pull through a blocker. The held prop itself must remain the first ray hit.
    let distance = eye.translation.distance(position) + half.abs().length();
    if super::world_hit(world, eye, distance.max(0.1)).is_some_and(|(hit, _)| hit != entity) {
        world.resource_mut::<State>().holding = None;
        world.resource_mut::<PlayState>().status = "Gravity gun released: obstructed".into();
        return;
    }
    let dt = world.resource::<Time>().delta_secs().min(0.05);
    if let Some(mut velocity) = world.get_mut::<Velocity>(entity) {
        let current = velocity.linvel;
        velocity.linvel = current
            + ((desired - position) * config.hold_stiffness - current * config.hold_damping) * dt;
        let max_speed = config.pull_speed * 3.;
        let current = velocity.linvel;
        if current.length() > max_speed {
            velocity.linvel = current.normalize() * max_speed;
        }
    }
    wake(world, entity);
    let mouse = world.resource::<ButtonInput<MouseButton>>().clone();
    if mouse.just_pressed(MouseButton::Left) && now >= world.resource::<State>().next_action {
        if let Some(mut velocity) = world.get_mut::<Velocity>(entity) {
            velocity.linvel += *eye.forward() * config.launch_impulse;
        }
        wake(world, entity);
        world.resource_mut::<State>().holding = None;
        world.resource_mut::<PlayState>().status = "Gravity gun launched object".into();
        audio::emit(world, "weapon_weapon_physcannon.fire", None, 1.);
        cooldown(world, now, config.action_cooldown, true, true);
    } else if mouse.just_pressed(MouseButton::Right) && now >= world.resource::<State>().next_action
    {
        world.resource_mut::<State>().holding = None;
        world.resource_mut::<PlayState>().status = "Gravity gun dropped object".into();
        wake(world, entity);
        audio::emit(world, "weapon_weapon_physcannon.drop", None, 1.);
        cooldown(world, now, config.action_cooldown, true, false);
    }
}

fn tick_pull(
    world: &mut World,
    entity: Entity,
    config: &sandbox_catalog::gravity::GravityConfig,
    now: f32,
) {
    match eligible(world, entity, config.hold_mass_limit) {
        Ok(true) => {}
        Err(()) => {
            world.resource_mut::<State>().pulling = None;
            world.resource_mut::<PlayState>().status = "Gravity gun mass not ready".into();
            return;
        }
        Ok(false) => {
            world.resource_mut::<State>().pulling = None;
            denied(
                world,
                now,
                config.action_cooldown,
                "Gravity gun target is ineligible",
            );
            return;
        }
    }
    let Some(eye) = eye(world) else { return };
    let Some(half) = world.get::<SpawnedProp>(entity).map(|prop| prop.half) else {
        return;
    };
    let Some(position) = world.get::<Transform>(entity).map(|t| t.translation) else {
        world.resource_mut::<State>().pulling = None;
        return;
    };
    let distance = eye.translation.distance(position);
    let hold_distance = config.hold_distance.max(half.abs().length() + 0.75);
    if hold_distance > config.hold_range {
        world.resource_mut::<State>().pulling = None;
        denied(
            world,
            now,
            config.action_cooldown,
            "Gravity gun target too large to hold",
        );
        return;
    }
    if distance > config.pull_range + half.abs().length() {
        world.resource_mut::<State>().pulling = None;
        world.resource_mut::<PlayState>().status = "Gravity gun pull canceled: out of range".into();
        return;
    }
    let ray_limit = distance + half.abs().length() + 0.5;
    if !super::world_hit(world, eye, ray_limit).is_some_and(|(hit, _)| hit == entity) {
        world.resource_mut::<State>().pulling = None;
        world.resource_mut::<PlayState>().status = "Gravity gun pull blocked".into();
        return;
    }
    let desired = hold_point(eye, half, config.hold_distance);
    let dt = world.resource::<Time>().delta_secs().min(0.05);
    if let Some(mut velocity) = world.get_mut::<Velocity>(entity) {
        let current = velocity.linvel;
        velocity.linvel = current
            + ((desired - position) * config.hold_stiffness - current * config.hold_damping) * dt;
        let max_speed = config.pull_speed;
        let current = velocity.linvel;
        if current.length() > max_speed {
            velocity.linvel = current.normalize() * max_speed;
        }
    }
    wake(world, entity);
    // Attach when the prop reaches its actual commanded hold point. hold_point
    // may enforce a minimum clearance beyond the configured distance.
    if position.distance(desired) <= 0.3 {
        world.resource_mut::<State>().pulling = None;
        acquire(world, entity, config, now);
    }
}

pub(super) fn input(world: &mut World) {
    if !world.contains_resource::<State>() {
        world.init_resource::<State>();
    }
    let active = world.resource::<PlayState>().active_weapon == "weapon_weapon_physcannon";
    let blocked = !active
        || !focused(world)
        || crate::source_frontend::active(world)
        || world.resource::<PlayState>().menu_open
        || world.resource::<PlayState>().tools.input_blocked
        || npcs::dead(world)
        || world.resource::<vehicles::Occupancy>().vehicle.is_some()
        || tools::devices::viewing(world);
    let mouse = world.resource::<ButtonInput<MouseButton>>().clone();
    if blocked {
        reset(world);
        return;
    }
    let now = world.resource::<Time>().elapsed_secs();
    let (holding, pulling, config, was_blocked, next_action) = {
        let state = world.resource::<State>();
        (
            state.holding,
            state.pulling,
            state.config.clone(),
            state.blocked,
            state.next_action,
        )
    };
    if was_blocked && (mouse.pressed(MouseButton::Left) || mouse.pressed(MouseButton::Right)) {
        return;
    }
    world.resource_mut::<State>().blocked = false;
    if let Some(entity) = holding {
        tick_held(world, entity, &config, now);
        return;
    }
    if let Some(entity) = pulling {
        if mouse.just_pressed(MouseButton::Right) && now >= next_action {
            world.resource_mut::<State>().pulling = None;
            world.resource_mut::<PlayState>().status = "Gravity gun canceled pull".into();
            cooldown(world, now, config.action_cooldown, false, false);
            return;
        }
        tick_pull(world, entity, &config, now);
        return;
    }
    if now < next_action {
        return;
    }

    if mouse.just_pressed(MouseButton::Right) {
        if let Some((entity, _)) = target(world, config.pull_range) {
            match eligible(world, entity, config.hold_mass_limit) {
                Ok(true) => {
                    world.resource_mut::<State>().pulling = Some(entity);
                    world.resource_mut::<PlayState>().status = "Gravity gun pulling object".into();
                    cooldown(world, now, config.action_cooldown, false, false);
                    tick_pull(world, entity, &config, now);
                }
                Err(()) => denied(
                    world,
                    now,
                    config.action_cooldown,
                    "Gravity gun mass not ready",
                ),
                Ok(false) => denied(
                    world,
                    now,
                    config.action_cooldown,
                    "Gravity gun target is ineligible or too heavy",
                ),
            }
        }
    } else if mouse.just_pressed(MouseButton::Left) {
        if let Some((entity, _)) = target(world, config.punt_range) {
            match eligible(world, entity, config.punt_mass_limit) {
                Ok(true) => {
                    if let Some(eye) = eye(world) {
                        if let Some(mut velocity) = world.get_mut::<Velocity>(entity) {
                            velocity.linvel += *eye.forward() * config.punt_impulse;
                        }
                        wake(world, entity);
                        world.resource_mut::<PlayState>().status = "Gravity gun punt".into();
                        audio::emit(world, "weapon_weapon_physcannon.fire", None, 1.);
                        cooldown(world, now, config.action_cooldown, true, true);
                    }
                }
                Err(()) => denied(
                    world,
                    now,
                    config.action_cooldown,
                    "Gravity gun mass not ready",
                ),
                Ok(false) => denied(
                    world,
                    now,
                    config.action_cooldown,
                    "Gravity gun target is ineligible or too heavy",
                ),
            }
        }
    }
}
