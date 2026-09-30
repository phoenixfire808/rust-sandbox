//! Typed vehicle occupancy and an explicitly approximate ray-supported chassis controller.
use super::*;
use crate::source_player::PlayerState;
use sandbox_catalog::spawn::Vehicle;
#[path = "source_vehicle_visuals.rs"]
pub(super) mod visuals;
use visuals::CrashCooldown;

#[derive(Component, Clone)]
pub struct VehicleBody {
    pub id: String,
}
#[derive(Resource, Default)]
pub struct Occupancy {
    pub vehicle: Option<Entity>,
    pub definition: String,
    pub pose: String,
    pub weight: f32,
    pub body_rotation: Quat,
    entry_from: Vec3,
    exit_to: Option<Vec3>,
    elapsed: f32,
    previous_yaw: f32,
    throttle: f32,
    steering: f32,
    brake: bool,
}
pub fn definition(world: &World, id: &str) -> Option<Vehicle> {
    world
        .resource::<PlayState>()
        .spawn_catalog
        .vehicles
        .iter()
        .find(|v| v.id == id)
        .cloned()
}
pub fn attach(world: &mut World, entity: Entity, id: &str) -> Result<()> {
    let c = definition(world, id).ok_or("unknown vehicle identity")?;
    let tuning = world.resource::<PlayState>().spawn_catalog.runtime.clone();
    world.entity_mut(entity).insert((
        VehicleBody { id: id.into() },
        ColliderMassProperties::Mass(c.mass),
        ExternalForce::default(),
        ActiveEvents::CONTACT_FORCE_EVENTS,
        ContactForceEventThreshold(c.mass * tuning.crash_force_per_mass),
        CrashCooldown::default(),
        Damping {
            linear_damping: tuning.vehicle_linear_damping,
            angular_damping: tuning.vehicle_angular_damping,
        },
    ));
    Ok(())
}
pub fn spawn(world: &mut World, id: &str) {
    let p = world.resource::<PlayState>();
    let Some(entry) = p
        .spawn_catalog
        .entries
        .iter()
        .find(|e| e.id == id && e.kind == "vehicle")
        .cloned()
    else {
        return;
    };
    let Some(definition) = definition(world, id) else {
        return;
    };
    let Some(eye) = camera(world) else { return };
    let distance = world
        .resource::<PlayState>()
        .spawn_catalog
        .runtime
        .vehicle_spawn_distance;
    let hit = tools::trace(world, eye, None);
    let (position, normal) = hit
        .map(|h| (h.point, h.normal))
        .unwrap_or((eye.translation + *eye.forward() * distance, Vec3::Y));
    let yaw = world
        .get_resource::<PlayerState>()
        .map(|p| p.yaw)
        .unwrap_or(0.);
    spawn::enqueue_vehicle(
        world,
        entry.model,
        position,
        normal,
        Quat::from_rotation_y(yaw - definition.forward_yaw),
        id.into(),
    );
    let mut p = world.resource_mut::<PlayState>();
    p.menu_open = false;
    p.search_focus = false;
    p.dirty = true;
}
fn ray(
    world: &mut World,
    origin: Vec3,
    direction: Vec3,
    distance: f32,
    exclude: Entity,
) -> Option<f32> {
    let mut q = world.query::<(
        &RapierContextSimulation,
        &RapierContextColliders,
        &RapierRigidBodySet,
        &RapierContextJoints,
        &RapierQueryPipeline,
    )>();
    let (simulation, colliders, rigidbody_set, joints, query_pipeline) = q.single(world).ok()?;
    let context = bevy_rapier3d::plugin::RapierContext {
        simulation,
        colliders,
        rigidbody_set,
        joints,
        query_pipeline,
    };
    context
        .cast_ray(
            origin,
            direction,
            distance,
            true,
            QueryFilter::default()
                .exclude_collider(exclude)
                .exclude_sensors(),
        )
        .map(|(_, d)| d)
}
pub(crate) fn clear_hull(world: &mut World, center: Vec3) -> bool {
    let Some(player) = world.get_resource::<PlayerState>() else {
        return false;
    };
    let shape = Collider::capsule_y(
        player.config.height * 0.5 - player.config.radius,
        player.config.radius,
    );
    let entity = player.entity;
    let mut q = world.query::<(
        &RapierContextSimulation,
        &RapierContextColliders,
        &RapierRigidBodySet,
        &RapierContextJoints,
        &RapierQueryPipeline,
    )>();
    let Ok((simulation, colliders, rigidbody_set, joints, query_pipeline)) = q.single(world) else {
        return false;
    };
    let context = bevy_rapier3d::plugin::RapierContext {
        simulation,
        colliders,
        rigidbody_set,
        joints,
        query_pipeline,
    };
    let mut blocked = false;
    context.intersections_with_shape(
        center,
        Quat::IDENTITY,
        &shape,
        QueryFilter::default()
            .groups(CollisionGroups::new(Group::GROUP_3, Group::ALL))
            .exclude_collider(entity)
            .exclude_sensors(),
        |_| {
            blocked = true;
            false
        },
    );
    !blocked
}
fn exit_position(world: &mut World, vehicle: Entity, c: &Vehicle) -> Option<Vec3> {
    let transform = *world.get::<Transform>(vehicle)?;
    let player = world.resource::<PlayerState>();
    let half = player.config.height * 0.5;
    let player_id = player.entity;
    let from = world.get::<Transform>(player_id)?.translation;
    for side in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z] {
        let candidate =
            transform.translation + transform.rotation * side * c.exit_distance + Vec3::Y * 1.5;
        let Some(down) = ray(world, candidate, Vec3::NEG_Y, 5., player_id) else {
            continue;
        };
        let center = candidate - Vec3::Y * down + Vec3::Y * (half + 0.04);
        if clear_hull(world, center) {
            // A body-fit endpoint is mandatory. The transition itself is visual-only, not a swept pedestrian walk.
            if center.distance(from) < c.exit_distance + 4. {
                return Some(center);
            }
        }
    }
    None
}
pub fn release(world: &mut World, center: Option<Vec3>) {
    let Some(player) = world.get_resource::<PlayerState>() else {
        *world.resource_mut::<Occupancy>() = Occupancy::default();
        return;
    };
    let entity = player.entity;
    let center = center.or_else(|| {
        world
            .resource::<Occupancy>()
            .vehicle
            .map(|_| world.resource::<Occupancy>().entry_from)
    });
    if let Some(center) = center {
        if let Some(mut t) = world.get_mut::<Transform>(entity) {
            t.translation = center;
        }
    }
    world.entity_mut(entity).remove::<ColliderDisabled>();
    let mut player = world.resource_mut::<PlayerState>();
    player.vehicle = None;
    player.horizontal_velocity = Vec3::ZERO;
    player.vertical = 0.;
    player.local_velocity = Vec3::ZERO;
    player.grounded = false;
    *world.resource_mut::<Occupancy>() = Occupancy::default();
}
pub fn update(world: &mut World) {
    let Some(player) = world.get_resource::<PlayerState>() else {
        return;
    };
    let player_id = player.entity;
    let now_dt = world.resource::<Time>().delta_secs().min(0.1);
    let occupied = world.resource::<Occupancy>().vehicle;
    if let Some(vehicle) = occupied {
        if world.get::<VehicleBody>(vehicle).is_none() {
            let fallback = world.resource::<Occupancy>().entry_from;
            release(world, Some(fallback));
            return;
        }
    }
    let focused = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .iter(world)
        .any(|w| w.focused);
    let blocked = npcs::dead(world)
        || crate::source_frontend::active(world)
        || world.resource::<PlayState>().menu_open
        || !focused;
    let keys = world.resource::<ButtonInput<KeyCode>>().clone();
    if occupied.is_none()
        && !blocked
        && keys.just_pressed(KeyCode::KeyE)
        && world.resource::<PlayState>().held.is_none()
    {
        if let Some((vehicle, distance)) =
            aimed_prop(world).filter(|(e, _)| world.get::<VehicleBody>(*e).is_some())
        {
            if distance
                <= world
                    .resource::<PlayState>()
                    .spawn_catalog
                    .runtime
                    .enter_range
            {
                let id = world.get::<VehicleBody>(vehicle).unwrap().id.clone();
                let c = definition(world, &id).unwrap();
                let p = world.resource::<PlayerState>();
                let center = world.get::<Transform>(player_id).unwrap().translation
                    + Vec3::Y
                        * if p.crouched {
                            (p.config.height - p.config.crouch_height) * 0.5
                        } else {
                            0.
                        };
                let yaw = world
                    .get::<Transform>(vehicle)
                    .unwrap()
                    .rotation
                    .to_euler(EulerRot::YXZ)
                    .0;
                *world.resource_mut::<Occupancy>() = Occupancy {
                    vehicle: Some(vehicle),
                    definition: id,
                    pose: c.pose,
                    entry_from: center,
                    previous_yaw: yaw,
                    ..default()
                };
                crate::source_player::reset_crouch(world);
                world.entity_mut(player_id).insert(ColliderDisabled);
                let mut player = world.resource_mut::<PlayerState>();
                player.vehicle = Some(vehicle);
                player.yaw = yaw + c.forward_yaw;
                player.noclip = false;
                player.horizontal_velocity = Vec3::ZERO;
                player.vertical = 0.;
                let mut play = world.resource_mut::<PlayState>();
                play.held = None;
                play.beam_active = false;
                play.tools.stage = None;
                play.tools.shot = None;
                play.status =
                    "Entering vehicle | WASD drive | Space brake | E exit | F4 view".into();
            }
        }
    }
    let Some(vehicle) = world.resource::<Occupancy>().vehicle else {
        return;
    };
    let c = definition(world, &world.resource::<Occupancy>().definition).unwrap();
    let transform = *world.get::<Transform>(vehicle).unwrap();
    let exiting = world.resource::<Occupancy>().exit_to.is_some();
    if !blocked
        && !exiting
        && keys.just_pressed(KeyCode::KeyE)
        && occupied.is_some()
        && world.resource::<Occupancy>().weight >= 1.
    {
        if let Some(center) = exit_position(world, vehicle, &c) {
            let mut s = world.resource_mut::<Occupancy>();
            s.exit_to = Some(center);
            s.elapsed = 0.;
        } else {
            world.resource_mut::<PlayState>().status =
                "Exit blocked. Move the vehicle to clear space.".into();
        }
    }
    let player_height = world.resource::<PlayerState>().config.height;
    let seat = transform.transform_point(Vec3::new(c.seat_x, c.seat_y, c.seat_z));
    let yaw = transform.rotation.to_euler(EulerRot::YXZ).0;
    let (center, finished, delta_yaw) = {
        let mut s = world.resource_mut::<Occupancy>();
        s.elapsed += now_dt;
        let delta_yaw = (yaw - s.previous_yaw + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        s.previous_yaw = yaw;
        s.body_rotation = transform.rotation * Quat::from_rotation_y(c.seat_yaw);
        let seated_center = seat + Vec3::Y * (player_height * 0.5);
        if let Some(exit) = s.exit_to {
            let t = (s.elapsed / c.exit_seconds).clamp(0., 1.);
            s.weight = 1. - t;
            s.throttle = 0.;
            s.steering = 0.;
            s.brake = true;
            (seated_center.lerp(exit, t), t >= 1., delta_yaw)
        } else {
            let t = (s.elapsed / c.entry_seconds).clamp(0., 1.);
            s.weight = t;
            let drive = !blocked && t >= 1. && c.kind != "seat";
            s.throttle = if drive {
                (keys.pressed(KeyCode::KeyW) as i32 - keys.pressed(KeyCode::KeyS) as i32) as f32
            } else {
                0.
            };
            s.steering = if drive {
                (keys.pressed(KeyCode::KeyA) as i32 - keys.pressed(KeyCode::KeyD) as i32) as f32
            } else {
                0.
            };
            s.brake = !drive || keys.pressed(KeyCode::Space);
            (s.entry_from.lerp(seated_center, t), false, delta_yaw)
        }
    };
    if finished {
        if clear_hull(world, center) {
            release(world, Some(center));
        } else {
            let mut s = world.resource_mut::<Occupancy>();
            s.exit_to = None;
            s.elapsed = c.entry_seconds;
        }
        return;
    }
    if let Some(mut t) = world.get_mut::<Transform>(player_id) {
        t.translation = center;
    }
    if let Some(mut controller) = world.get_mut::<KinematicCharacterController>(player_id) {
        controller.translation = None;
    }
    let mut player = world.resource_mut::<PlayerState>();
    player.yaw += delta_yaw;
    player.local_velocity = Vec3::ZERO;
    player.vertical = 0.;
    player.grounded = true;
}
pub fn drive(world: &mut World) {
    let dt = world.resource::<Time<Fixed>>().delta_secs();
    let tuning = world.resource::<PlayState>().spawn_catalog.runtime.clone();
    let occupied = world.resource::<Occupancy>().vehicle;
    let focused = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .iter(world)
        .any(|w| w.focused);
    let blocked = npcs::dead(world)
        || crate::source_frontend::active(world)
        || world.resource::<PlayState>().menu_open
        || !focused;
    let (throttle, steer, brake) = {
        let s = world.resource::<Occupancy>();
        if blocked {
            (0., 0., true)
        } else {
            (s.throttle, s.steering, s.brake)
        }
    };
    let rows: Vec<_> = world
        .query::<(
            Entity,
            &VehicleBody,
            &Transform,
            &Velocity,
            &SpawnedProp,
            &RigidBody,
        )>()
        .iter(world)
        .map(|(e, v, t, vel, p, b)| (e, v.id.clone(), *t, *vel, p.center, p.half, *b))
        .collect();
    for (e, id, t, velocity, center, half, body) in rows {
        let Some(c) = definition(world, &id) else {
            continue;
        };
        if c.kind == "seat" || body != RigidBody::Dynamic {
            world.entity_mut(e).insert(ExternalForce::default());
            continue;
        }
        let up = *t.up();
        let frame = Quat::from_rotation_y(c.forward_yaw);
        let local_forward = frame * Vec3::NEG_Z;
        let local_right = frame * Vec3::X;
        let forward = t.rotation * local_forward;
        let right = t.rotation * local_right;
        let width = local_right.abs().dot(half);
        let length = local_forward.abs().dot(half);
        let com = t.transform_point(center);
        let mut force = Vec3::ZERO;
        let mut torque = Vec3::ZERO;
        let mut contacts = 0;
        for x in [-tuning.wheel_span_x, tuning.wheel_span_x] {
            for z in [-tuning.wheel_span_z, tuning.wheel_span_z] {
                let local = center
                    + local_right * (width * x)
                    + local_forward * (length * z)
                    + Vec3::Y * (-half.y + c.suspension * 0.5);
                let origin = t.transform_point(local);
                let ground = ray(world, origin, -up, c.suspension, e);
                let water = if c.kind == "airboat" && up.y > tuning.min_up {
                    world
                        .get_resource::<crate::source_assets::WaterSurfaces>()
                        .and_then(|w| w.height(origin))
                        .map(|height| (origin.y - height) / up.y)
                        .filter(|distance| *distance >= -c.suspension && *distance <= c.suspension)
                } else {
                    None
                };
                let contact = match (ground, water) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (a, b) => a.or(b),
                };
                if let Some(distance) = contact {
                    let offset = origin - com;
                    let speed = (velocity.linvel + velocity.angvel.cross(offset)).dot(up);
                    let magnitude = ((c.suspension - distance) * c.spring - speed * c.damping)
                        .clamp(0., c.mass * tuning.spring_force_limit);
                    let f = up * magnitude;
                    force += f;
                    torque += offset.cross(f);
                    contacts += 1;
                }
            }
        }
        if contacts > 0 && up.y > tuning.min_up {
            let active = occupied == Some(e);
            let gas = if active { throttle } else { 0. };
            let speed = velocity.linvel.dot(forward);
            if speed.abs() < c.max_speed || gas.signum() != speed.signum() {
                force += forward * gas * c.engine_accel * c.mass;
            }
            force -= right * velocity.linvel.dot(right) * c.grip * c.mass;
            if (active && brake) || !active {
                force -= forward
                    * speed.signum()
                    * (c.brake_accel * c.mass).min(speed.abs() * c.mass / dt.max(0.001));
            }
            if active {
                let desired = steer * c.steer_rate * (speed / tuning.steer_speed).clamp(-1., 1.);
                torque += up * (desired - velocity.angvel.dot(up)) * c.mass * half.length_squared();
            }
        }
        world.entity_mut(e).insert(ExternalForce { force, torque });
    }
}
