//! Fixed-step load-limited contact forces. Independent aggregate rider/board model.
use super::*;
fn contact(
    world: &mut World,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    board: Entity,
) -> Option<(Entity, f32, Vec3)> {
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
        .cast_ray_and_get_normal(
            origin,
            direction,
            length,
            true,
            QueryFilter::default()
                .exclude_rigid_body(board)
                .exclude_sensors(),
        )
        .map(|(e, h)| (e, h.time_of_impact, h.normal))
}
pub(crate) fn step(world: &mut World) {
    let dt = world.resource::<Time<Fixed>>().delta_secs();
    if !dt.is_finite() || dt <= 0. || dt > 0.1 {
        return;
    }
    let now = world.resource::<Time<Fixed>>().elapsed_secs();
    let catalog = world.resource::<Catalog>().0.clone();
    let b = &catalog.board;
    let occupied = world.resource::<vehicles::Occupancy>().vehicle;
    let focused = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .iter(world)
        .any(|w| w.focused);
    let blocked = !focused
        || npcs::dead(world)
        || crate::source_frontend::active(world)
        || world.resource::<PlayState>().menu_open
        || tools::devices::viewing(world)
        || world.resource::<Controls>().was_blocked;
    let (steer, push, brake, slide, manual, flip, spin, catch, grind, pop, nollie, goofy) = {
        let mut c = world.resource_mut::<Controls>();
        let pop = c.pop.take();
        if blocked {
            (
                0., false, true, false, 0., 0., 0., false, false, None, false, c.goofy,
            )
        } else {
            (
                c.steer, c.push, c.brake, c.slide, c.manual, c.flip, c.spin, c.catch, c.grind, pop,
                c.nollie, c.goofy,
            )
        }
    };
    let bodies: Vec<_> = world
        .query_filtered::<(
            Entity,
            &Transform,
            &Velocity,
            &RigidBody,
            &vehicles::VehicleBody,
        ), With<Board>>()
        .iter(world)
        .map(|(e, t, v, r, id)| (e, *t, *v, *r, id.id.clone()))
        .collect();
    let mut bail = None;
    for (e, t, mut v, rigid, id) in bodies {
        let Some(p) = profile(&catalog, &id) else {
            continue;
        };
        if rigid != RigidBody::Dynamic {
            world.entity_mut(e).insert(ExternalForce::default());
            continue;
        }
        let ridden = occupied == Some(e);
        let active = ridden && !blocked && world.resource::<vehicles::Occupancy>().weight >= 1.;
        let mass = b.mass + if ridden { b.rider_mass } else { 0. };
        let mut state = world.entity_mut(e).take::<Board>().unwrap_or_default();
        if state.was_ridden != ridden {
            state.was_ridden = ridden;
            state.caught = true;
            state.trick = false;
            state.airtime = 0.;
        }
        state.steering += ((if active { steer } else { 0. }) - state.steering)
            * (1. - (-b.steering_response * dt).exp());
        let up = t.rotation * Vec3::Y;
        let forward = t.rotation * Vec3::NEG_Z;
        let right = t.rotation * Vec3::X;
        let speed = v.linvel.dot(forward);
        let gravity_on = world.get::<tools::Properties>(e).is_none_or(|p| p.gravity);
        let mut force = if gravity_on {
            Vec3::NEG_Y * mass * p.gravity
        } else {
            Vec3::ZERO
        };
        let mut torque = Vec3::ZERO;
        let rest = b.wheel_radius * 2. + b.suspension;
        let mut contacts = Vec::new();
        if up.y > 0.1 && now >= state.pop_ignore {
            for x in [-b.track * 0.5, b.track * 0.5] {
                for z in [-b.wheelbase * 0.5, b.wheelbase * 0.5] {
                    // Weight-shift approximation: only the balanced truck supports a manual.
                    if active && manual.abs() > 0.1 && (z < 0.) == (manual > 0.) {
                        continue;
                    }
                    let offset = t.rotation * Vec3::new(x, 0., z);
                    if let Some((_, distance, normal)) =
                        contact(world, t.translation + offset, -up, rest, e)
                    {
                        if normal.dot(up) <= 0.1 {
                            continue;
                        }
                        let point_velocity = v.linvel + v.angvel.cross(offset);
                        let mass_scale = mass / (b.mass + b.rider_mass);
                        let load = ((rest - distance) * b.spring
                            - point_velocity.dot(normal) * b.damping)
                            .max(0.)
                            * mass_scale;
                        let load = load.min(mass * p.gravity * b.max_load * 0.25);
                        contacts.push((offset, normal, point_velocity, load, z < 0.));
                    }
                }
            }
        }
        let grounded = !contacts.is_empty();
        let was_airborne = !state.grounded && state.airtime > dt * 2.;
        if active && grounded && was_airborne {
            let landing_normal = contacts
                .iter()
                .map(|c| c.1)
                .sum::<Vec3>()
                .normalize_or_zero();
            let impact = -state.last_velocity.dot(landing_normal);
            if up.dot(landing_normal) < p.landing_cos
                || impact > p.landing_speed
                || (state.trick && !state.caught)
            {
                bail = Some((e, "Bailed: landing angle / impact / uncaught trick"));
            } else {
                world.resource_mut::<Controls>().status = "Landed".into();
            }
            state.trick = false;
            state.caught = true;
        }
        state.grounded = grounded;
        state.airtime = if grounded { 0. } else { state.airtime + dt };
        let total_load = contacts.iter().map(|c| c.3).sum::<f32>();
        for (offset, normal, point_velocity, load, front) in &contacts {
            let angle = state.steering * b.steering_angle * if *front { 1. } else { -1. };
            let truck_forward = Quat::from_axis_angle(*normal, angle) * forward;
            let tangent =
                (truck_forward - *normal * truck_forward.dot(*normal)).normalize_or_zero();
            let lateral = tangent.cross(*normal).normalize_or_zero();
            let share = *load / total_load.max(0.001);
            let grip = if active && slide {
                p.slide_grip
            } else {
                p.grip
            };
            let lateral_force = -point_velocity.dot(lateral) * mass * share / dt;
            let lateral_force = lateral_force.clamp(-load * grip, load * grip);
            let longitudinal = point_velocity.dot(tangent);
            let stop = if ridden && brake {
                b.brake_accel * mass * share
            } else {
                b.rolling_resistance * load
            };
            let rolling_force =
                -longitudinal.signum() * stop.min(longitudinal.abs() * mass * share / dt);
            let friction =
                (lateral * lateral_force + tangent * rolling_force).clamp_length_max(load * grip);
            let f = *normal * *load + friction;
            force += f;
            torque += offset.cross(f);
        }
        if active && grounded {
            if push && manual.abs() < 0.1 && now >= state.next_push && speed < p.push_limit {
                v.linvel += forward * p.push_delta.min(p.push_limit - speed);
                state.next_push = now + p.push_interval;
            }
            if manual.abs() > 0.1 {
                // Rider weight shift, not a forced rotation. Low-assist profiles can tip out.
                let pitch = forward.dot(Vec3::Y).asin();
                let target = manual * b.manual_angle;
                let correction = (target - pitch) * b.balance_torque
                    - v.angvel.dot(right) * b.balance_damping * p.manual_assist;
                let weight_shift = if gravity_on {
                    mass * p.gravity * b.wheelbase * 0.5 * manual
                } else {
                    0.
                };
                torque +=
                    right * (weight_shift + correction.clamp(-b.balance_torque, b.balance_torque));
                world.resource_mut::<Controls>().status = if manual > 0. {
                    "Manual: balance rear truck"
                } else {
                    "Nose manual: balance front truck"
                }
                .into();
            }
        }
        state.grind = false;
        if active && grind && now >= state.pop_ignore {
            // Explicit authored practice rail only. No false arbitrary-world rail detection.
            if let Some((rail, distance, normal)) =
                contact(world, t.translation, Vec3::NEG_Y, rest, e)
            {
                if normal.y > 0.7
                    && world
                        .get::<SpawnedProp>(rail)
                        .is_some_and(|r| r.model == RAIL)
                {
                    let rt = *world.get::<Transform>(rail).unwrap();
                    let tangent = rt.rotation * Vec3::NEG_Z;
                    let alignment = forward.dot(tangent).abs();
                    if alignment > (0.95 - p.grind_assist * 0.3) && up.y > p.landing_cos {
                        let lateral = rt.rotation * Vec3::X;
                        force += normal
                            * ((rest - distance) * b.spring - v.linvel.y * b.damping)
                                .clamp(0., mass * p.gravity * b.max_load);
                        force -= lateral
                            * (v.linvel.dot(lateral) * mass / dt)
                                .clamp(-mass * p.gravity, mass * p.gravity);
                        force -= tangent
                            * v.linvel.dot(tangent).signum()
                            * b.rolling_resistance
                            * mass
                            * p.gravity;
                        state.grind = true;
                        state.caught = true;
                        world.resource_mut::<Controls>().status =
                            "Practice-rail grind | pop to leave".into();
                    }
                }
            }
        }
        if active && (grounded || state.grind) && now >= state.next_pop {
            if let Some(charge) = pop {
                let jump = p.pop_min + (p.pop_max - p.pop_min) * charge.clamp(0., 1.);
                v.linvel.y = v.linvel.y.max(0.) + jump;
                v.angvel += right * if nollie { -0.8 } else { 0.8 };
                state.next_pop = now + b.pop_cooldown;
                state.pop_ignore = now + dt * 3.;
                state.grounded = false;
                state.airtime = 0.;
                state.caught = true;
                world.resource_mut::<Controls>().status =
                    if nollie { "Nollie" } else { "Ollie" }.into();
            }
        }
        if active && !grounded && !state.grind {
            if flip.abs() > 0.1 || spin.abs() > 0.1 {
                state.trick = true;
                state.caught = false;
                let handedness = if goofy { -1. } else { 1. };
                v.angvel += forward
                    * (flip * handedness * p.flip_rate - v.angvel.dot(forward))
                    * (1. - (-b.steering_response * dt).exp());
                v.angvel += Vec3::Y
                    * (spin * p.air_turn - v.angvel.y)
                    * (1. - (-b.steering_response * dt).exp());
            }
            let assist_catch = p.catch_assist > 0. && v.linvel.y < 0. && up.y > p.landing_cos;
            if (catch || assist_catch) && up.y > p.landing_cos {
                state.caught = true;
                let gain = if catch { 1. } else { p.catch_assist };
                torque += up.cross(Vec3::Y) * b.balance_torque * gain
                    - v.angvel * b.balance_damping * gain;
            }
            if !state.trick {
                // Level the pop pitch while retaining forward momentum; no extra midair jumps.
                torque += up.cross(Vec3::Y) * b.balance_torque * p.catch_assist
                    - right * v.angvel.dot(right) * b.balance_damping * p.catch_assist;
            }
            v.angvel.y += steer * p.air_turn * dt;
            world.resource_mut::<Controls>().status = if state.caught {
                "Air: caught"
            } else {
                "Air: catch with C / RB"
            }
            .into();
        }
        // Wheel rays deliberately reject inverted boards, so use an independent
        // world-down probe rather than an unreachable inverted-and-grounded check.
        if active
            && up.y < 0.1
            && state.airtime > dt * 2.
            && contact(
                world,
                t.translation + Vec3::Y * b.thickness,
                Vec3::NEG_Y,
                rest + b.thickness,
                e,
            )
            .is_some()
        {
            bail = Some((e, "Bailed: inverted or sideways board"));
        }
        if !t.translation.is_finite()
            || !v.linvel.is_finite()
            || !v.angvel.is_finite()
            || !force.is_finite()
            || !torque.is_finite()
        {
            force = Vec3::ZERO;
            torque = Vec3::ZERO;
            v = Velocity::zero();
            if ridden {
                bail = Some((e, "Skateboard stopped: invalid physics state"));
            }
        }
        v.linvel = v.linvel.clamp_length_max(b.max_speed);
        v.angvel = v.angvel.clamp_length_max(p.flip_rate * 2.);
        state.last_velocity = v.linvel;
        world.entity_mut(e).insert((
            state,
            v,
            ColliderMassProperties::Mass(mass),
            GravityScale(0.),
            ExternalForce { force, torque },
        ));
    }
    if let Some((e, reason)) = bail {
        if world.resource::<vehicles::Occupancy>().vehicle == Some(e) {
            let position = definition(world, &world.resource::<vehicles::Occupancy>().definition)
                .and_then(|c| vehicles::exit_position(world, e, &c));
            vehicles::release(world, position);
            world.resource_mut::<PlayState>().status = reason.into();
            let goofy = world.resource::<Controls>().goofy;
            *world.resource_mut::<Controls>() = Controls {
                goofy,
                was_blocked: true,
                ..default()
            };
        }
    }
}
