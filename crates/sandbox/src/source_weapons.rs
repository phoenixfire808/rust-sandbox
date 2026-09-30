//! Authored primary weapon behavior. Native secondary actions and damage parity remain explicit gaps.
use super::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Ammo {
    pub loaded: u32,
    pub reserve: u32,
}
#[derive(Resource, Default)]
pub struct WeaponState {
    pub ammo: BTreeMap<String, Ammo>,
    pub next_fire: f32,
    pub fired_at: Option<f32>,
    pub reload: Option<(String, f32, f32)>,
    pub equip_blocked: bool,
    serial: u64,
    projectiles: Vec<Projectile>,
    traces: Vec<(Vec3, Vec3, f32)>,
}
struct Projectile {
    point: Vec3,
    velocity: Vec3,
    damage: f32,
    blast: f32,
    gravity: f32,
    until: f32,
    grenade: bool,
}
#[derive(Component, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Health(pub f32);

pub fn equip(world: &mut World, id: &str) {
    let Some(w) = world
        .resource::<PlayState>()
        .spawn_catalog
        .weapons
        .iter()
        .find(|w| w.id == id)
        .cloned()
    else {
        return;
    };
    if w.kind == "disabled" {
        world.resource_mut::<PlayState>().status = format!("Not implemented: {}", w.remaining);
        return;
    }
    if world
        .get_resource::<vehicles::Occupancy>()
        .is_some_and(|s| s.vehicle.is_some())
    {
        return;
    }
    let now = world.resource::<Time>().elapsed_secs();
    let tuning = world.resource::<PlayState>().spawn_catalog.runtime.clone();
    {
        let mut state = world.resource_mut::<WeaponState>();
        state.ammo.entry(id.into()).or_insert(Ammo {
            loaded: w.clip,
            reserve: w.reserve,
        });
        state.reload = None;
        state.fired_at = None;
        state.next_fire = now + tuning.equip_delay;
        state.equip_blocked = true; // Mouse-up after clicking the menu must precede gameplay fire.
    }
    let mut p = world.resource_mut::<PlayState>();
    p.active_weapon = id.into();
    p.physgun = w.kind == "physgun";
    p.held = None;
    p.beam_active = false;
    p.tools.stage = None;
    p.tools.shot = None;
    p.menu_open = false;
    p.search_focus = false;
    p.dirty = true;
    p.status = format!(
        "Equipped {} | {}",
        w.id.trim_start_matches("weapon_"),
        w.scope
    );
}
pub fn input(world: &mut World) {
    let now = world.resource::<Time>().elapsed_secs();
    let tuning = world.resource::<PlayState>().spawn_catalog.runtime.clone();
    let mouse = world.resource::<ButtonInput<MouseButton>>().clone();
    if !mouse.pressed(MouseButton::Left) {
        world.resource_mut::<WeaponState>().equip_blocked = false;
    }
    let p = world.resource::<PlayState>();
    if crate::source_frontend::active(world)
        || npcs::dead(world)
        || p.menu_open
        || p.tools.input_blocked
        || world.resource::<vehicles::Occupancy>().vehicle.is_some()
    {
        return;
    }
    let Some(w) = p
        .spawn_catalog
        .weapons
        .iter()
        .find(|w| w.id == p.active_weapon)
        .cloned()
    else {
        return;
    };
    if ["disabled", "physgun", "toolgun"].contains(&w.kind.as_str()) {
        return;
    }
    world
        .resource_mut::<WeaponState>()
        .ammo
        .entry(w.id.clone())
        .or_insert(Ammo {
            loaded: w.clip,
            reserve: w.reserve,
        });
    if let Some((id, _, end)) = world.resource::<WeaponState>().reload.clone() {
        if now >= end {
            let mut s = world.resource_mut::<WeaponState>();
            if let Some(ammo) = s.ammo.get_mut(&id) {
                let amount = w.clip.saturating_sub(ammo.loaded).min(ammo.reserve);
                ammo.loaded += amount;
                ammo.reserve -= amount;
            }
            s.reload = None;
        } else {
            return;
        }
    }
    let reload = world
        .resource::<ButtonInput<KeyCode>>()
        .just_pressed(KeyCode::KeyR);
    let state = world.resource::<WeaponState>();
    let ammo = state.ammo.get(&w.id).cloned().unwrap_or(Ammo {
        loaded: w.clip,
        reserve: w.reserve,
    });
    if w.clip > 0 && ammo.loaded < w.clip && ammo.reserve > 0 && (reload || ammo.loaded == 0) {
        world.resource_mut::<WeaponState>().reload =
            Some((w.id.clone(), now, now + w.reload_seconds));
        world.resource_mut::<PlayState>().status = "Reloading...".into();
        return;
    }
    if state.equip_blocked || now < state.next_fire || (w.clip > 0 && ammo.loaded == 0) {
        return;
    }
    if !(if w.automatic {
        mouse.pressed(MouseButton::Left)
    } else {
        mouse.just_pressed(MouseButton::Left)
    }) {
        return;
    }
    if (w.kind == "projectile" || w.kind == "grenade")
        && state.projectiles.len() >= tuning.projectile_limit
    {
        return;
    }
    let Some(eye) = camera(world) else { return };
    {
        let mut s = world.resource_mut::<WeaponState>();
        if w.clip > 0 {
            if let Some(ammo) = s.ammo.get_mut(&w.id) {
                ammo.loaded = ammo.loaded.saturating_sub(1);
            }
        }
        s.next_fire = now + w.interval;
        s.fired_at = Some(now);
        s.serial = s.serial.wrapping_add(1);
    }
    if w.kind == "projectile" || w.kind == "grenade" {
        let mut s = world.resource_mut::<WeaponState>();
        if s.projectiles.len() < tuning.projectile_limit {
            s.projectiles.push(Projectile {
                point: eye.translation,
                velocity: *eye.forward() * w.speed,
                damage: w.damage,
                blast: w.blast,
                gravity: w.gravity,
                until: now
                    + if w.kind == "grenade" {
                        tuning.grenade_fuse
                    } else {
                        (w.range / w.speed.max(1.)).min(tuning.projectile_lifetime)
                    },
                grenade: w.kind == "grenade",
            });
        }
    } else {
        let serial = world.resource::<WeaponState>().serial;
        for pellet in 0..w.pellets {
            // Reproducible disc distribution, not a claim of Source RNG/spread parity.
            let angle =
                (serial as f32 * 1.618 + pellet as f32 * 2.39996).rem_euclid(std::f32::consts::TAU);
            let radius = w.spread * ((pellet + 1) as f32 / w.pellets as f32).sqrt();
            let direction = (*eye.forward()
                + *eye.right() * angle.cos() * radius
                + *eye.up() * angle.sin() * radius)
                .normalize();
            let aim = Transform::from_translation(eye.translation).looking_to(direction, Vec3::Y);
            let hit = world_ray(world, aim, w.range);
            let end = eye.translation + direction * hit.map(|(_, d)| d).unwrap_or(w.range);
            if let Some((target, _)) = hit {
                damage(world, target, w.damage, direction);
            }
            world.resource_mut::<WeaponState>().traces.push((
                eye.translation + direction * 0.3,
                end,
                now + tuning.trace_seconds,
            ));
        }
    }
    let ammo = &world.resource::<WeaponState>().ammo[&w.id];
    let status = format!(
        "{} | {} / {} | R: reload",
        w.id.trim_start_matches("weapon_"),
        ammo.loaded,
        ammo.reserve
    );
    world.resource_mut::<PlayState>().status = status;
}
pub fn damage(world: &mut World, target: Entity, amount: f32, direction: Vec3) {
    if world.get::<SpawnedProp>(target).is_none() {
        return;
    }
    // Vehicles retain their authored health policy for now; weapon damage affects ordinary props only.
    if world.get::<vehicles::VehicleBody>(target).is_some() {
        return;
    }
    let tuning = world.resource::<PlayState>().spawn_catalog.runtime.clone();
    let health = world
        .get::<Health>(target)
        .map(|h| h.0)
        .unwrap_or(tuning.prop_health)
        - amount;
    if health <= 0. {
        tools::constraints::remove(world, target, None, false);
        if world.resource::<PlayState>().held == Some(target) {
            world.resource_mut::<PlayState>().held = None;
        }
        world.despawn(target);
    } else {
        world.entity_mut(target).insert(Health(health));
        if world.get::<RigidBody>(target) == Some(&RigidBody::Dynamic) {
            if let Some(mut velocity) = world.get_mut::<Velocity>(target) {
                velocity.linvel += direction * amount * tuning.damage_impulse;
            }
        }
    }
}
fn explode(world: &mut World, point: Vec3, radius: f32, amount: f32) {
    let targets: Vec<_> = world
        .query::<(Entity, &Transform, &SpawnedProp)>()
        .iter(world)
        .filter_map(|(e, t, _)| {
            (t.translation.distance(point) <= radius).then_some((e, t.translation))
        })
        .collect();
    for (e, p) in targets {
        let delta = p - point;
        let distance = delta.length();
        if distance < 0.001 {
            damage(world, e, amount, Vec3::Y);
            continue;
        }
        let direction = delta / distance;
        let aim =
            Transform::from_translation(point + direction * 0.02).looking_to(direction, Vec3::Y);
        if world_ray(world, aim, distance).is_none_or(|(hit, _)| hit == e) {
            damage(
                world,
                e,
                amount * (1. - distance / radius).max(0.),
                direction,
            );
        }
    }
}
pub fn simulate(world: &mut World) {
    if crate::source_frontend::active(world) {
        return;
    }
    let now = world.resource::<Time>().elapsed_secs();
    let dt = world.resource::<Time>().delta_secs().min(0.05);
    let mut projectiles = std::mem::take(&mut world.resource_mut::<WeaponState>().projectiles);
    projectiles.retain_mut(|p| {
        if now >= p.until {
            if p.grenade {
                explode(world, p.point, p.blast, p.damage);
            }
            return false;
        }
        p.velocity.y -= p.gravity * dt;
        let step = p.velocity * dt;
        if step.length_squared() < 1e-8 {
            return true;
        }
        let direction = step.normalize();
        let aim = Transform::from_translation(p.point).looking_to(direction, Vec3::Y);
        if let Some((target, d)) = world_ray(world, aim, step.length()) {
            let contact = p.point + direction * d;
            if p.grenade {
                p.point = contact - direction * 0.03;
                p.gravity = 0.; // This prototype rests at contact until its fuse, rather than creeping through geometry.
                p.velocity = Vec3::ZERO; // Contact stop is deliberate until native bounce is implemented.
            } else {
                if p.blast > 0. {
                    explode(world, contact - direction * 0.03, p.blast, p.damage);
                } else {
                    damage(world, target, p.damage, direction);
                }
                return false;
            }
        } else {
            p.point += step;
        }
        true
    });
    let mut state = world.resource_mut::<WeaponState>();
    state.projectiles = projectiles;
    state.traces.retain(|(_, _, until)| *until > now);
}
pub fn draw(mut gizmos: Gizmos, state: Res<WeaponState>) {
    for (a, b, _) in &state.traces {
        gizmos.line(*a, *b, Color::srgb(1., 0.8, 0.2));
    }
    for p in &state.projectiles {
        gizmos.sphere(
            p.point,
            if p.grenade { 0.07 } else { 0.035 },
            Color::srgb(1., 0.45, 0.05),
        );
    }
}
pub fn clear_transients(world: &mut World) {
    let mut s = world.resource_mut::<WeaponState>();
    s.projectiles.clear();
    s.traces.clear();
    s.reload = None;
    s.fired_at = None;
}
