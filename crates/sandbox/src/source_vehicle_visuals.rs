//! Original wheel attachment driven skinning, not native suspension/pose-parameter parity.
use super::*;
use crate::source_player::{self, Actor};
use std::sync::Arc;

#[derive(Component)]
struct WheelVisuals {
    actor: Arc<Actor>,
    wheels: Vec<(usize, sandbox_catalog::spawn::Wheel, f32)>,
}
#[derive(Component)]
struct WheelAttempted;
#[derive(Component, Default)]
pub(crate) struct CrashCooldown(f32);

fn initialize(world: &mut World, entity: Entity, id: &str) -> Result<()> {
    let rules: Vec<_> = world
        .resource::<PlayState>()
        .spawn_catalog
        .wheels
        .iter()
        .filter(|w| w.vehicle == id)
        .cloned()
        .collect();
    if rules.is_empty() {
        return Ok(());
    }
    let path = world
        .get::<SpawnedProp>(entity)
        .ok_or("vehicle model missing")?
        .model
        .clone();
    let old: Vec<_> = world
        .get::<Children>(entity)
        .map(|c| c.iter().collect())
        .unwrap_or_default();
    let mut actor = source_player::actor(world, &path, 0, Some(entity))?;
    let mut wheels = Vec::new();
    for rule in rules {
        let Some((_, bone, _)) = actor
            .skeleton
            .attachments
            .iter()
            .find(|(name, _, _)| name == &rule.attachment)
        else {
            world.despawn(actor.root);
            return Err(format!("Missing wheel attachment {}", rule.attachment).into());
        };
        wheels.push((*bone, rule, 0.));
    }
    // Keep rendered parts directly on the prop so existing colour/material tools still reach them.
    let parts: Vec<_> = world
        .get::<Children>(actor.root)
        .map(|c| c.iter().collect())
        .unwrap_or_default();
    for child in parts {
        world.entity_mut(child).insert(ChildOf(entity));
    }
    world.despawn(actor.root);
    actor.root = entity;
    for child in old {
        if world.get::<Mesh3d>(child).is_some() {
            world.despawn(child);
        }
    }
    if let Some(properties) = world.get::<tools::Properties>(entity).cloned() {
        tools::apply_visual_properties(world, entity, &properties)?;
    }
    world.entity_mut(entity).insert(WheelVisuals {
        actor: Arc::new(actor),
        wheels,
    });
    Ok(())
}
pub fn animate(world: &mut World) {
    if !world.contains_resource::<PlayerState>() {
        return;
    }
    let new: Vec<_> = world
        .query_filtered::<(Entity, &VehicleBody), Without<WheelAttempted>>()
        .iter(world)
        .map(|(e, v)| (e, v.id.clone()))
        .collect();
    for (e, id) in new {
        world.entity_mut(e).insert(WheelAttempted);
        if let Err(error) = initialize(world, e, &id) {
            world.resource_mut::<PlayState>().status =
                format!("Vehicle wheel visuals unavailable: {error}");
            eprintln!("VEHICLE_WHEELS_MISSING {id}: {error}");
        }
    }
    let dt = world.resource::<Time>().delta_secs();
    let eye = world.resource::<PlayerState>().eye.translation;
    let distance = world
        .resource::<PlayState>()
        .spawn_catalog
        .runtime
        .wheel_visual_distance;
    let (occupied, steer) = {
        let s = world.resource::<Occupancy>();
        (s.vehicle, s.steering)
    };
    let rows: Vec<_> = world
        .query::<(Entity, &Transform, &Velocity, &VehicleBody)>()
        .iter(world)
        .filter(|(_, t, _, _)| t.translation.distance_squared(eye) <= distance * distance)
        .map(|(e, t, v, b)| (e, *t, v.linvel, b.id.clone()))
        .collect();
    for (e, transform, velocity, id) in rows {
        let Some(c) = definition(world, &id) else {
            continue;
        };
        let frame = Quat::from_rotation_y(c.forward_yaw);
        let speed = velocity.dot(transform.rotation * (frame * Vec3::NEG_Z));
        let axle = -(frame * Vec3::X);
        // source_position maps Source (x,y,z) to Bevy (-y,z,-x).
        // Invert that exact basis, not the unrelated (x,z,-y) convention.
        let source_axle = Vec3::new(-axle.z, -axle.x, axle.y).normalize();
        let steering = if occupied == Some(e) { steer } else { 0. };
        let Some(mut visual) = world.get_mut::<WheelVisuals>(e) else {
            continue;
        };
        for (_, rule, angle) in &mut visual.wheels {
            *angle = (*angle + speed * dt / rule.radius).rem_euclid(std::f32::consts::TAU);
        }
        let actor = visual.actor.clone();
        let wheels = visual.wheels.clone();
        drop(visual);
        let mut pose = actor.skeleton.bind_pose();
        let bind = actor.skeleton.globals(&pose);
        for (bone, rule, spin) in wheels {
            let pivot = bind[bone].transform_point3(Vec3::ZERO);
            let turn = Quat::from_axis_angle(Vec3::Z, steering * rule.steer_angle);
            let roll = Quat::from_axis_angle(source_axle, spin);
            let global = Mat4::from_translation(pivot)
                * Mat4::from_quat(turn * roll)
                * Mat4::from_translation(-pivot)
                * bind[bone];
            let parent = actor.skeleton.bones[bone].parent;
            pose[bone] = Transform::from_matrix(if parent < 0 {
                global
            } else {
                bind[parent as usize].inverse() * global
            });
        }
        source_player::skin(world, &actor, &actor.skeleton.globals(&pose));
    }
}
pub fn crashes(
    mut events: EventReader<ContactForceEvent>,
    time: Res<Time>,
    play: Res<PlayState>,
    context: ReadRapierContext,
    mut cooldowns: Query<&mut CrashCooldown>,
    mut effects: ResMut<impacts::Impacts>,
) {
    let Ok(context) = context.single() else {
        events.clear();
        return;
    };
    let now = time.elapsed_secs();
    let c = &play.spawn_catalog.runtime;
    for event in events.read() {
        let vehicle = if cooldowns.contains(event.collider1) {
            event.collider1
        } else {
            event.collider2
        };
        let Ok(mut cooldown) = cooldowns.get_mut(vehicle) else {
            continue;
        };
        if now < cooldown.0 {
            continue;
        }
        let Some(pair) = context.contact_pair(event.collider1, event.collider2) else {
            continue;
        };
        let contact = pair
            .manifolds()
            .find_map(|m| m.solver_contact(0).map(|p| (p.point(), m.normal())));
        let Some((point, normal)) = contact else {
            continue;
        };
        cooldown.0 = now + c.crash_cooldown;
        effects.push_burst(point, normal, now, c.impact_limit);
    }
}
