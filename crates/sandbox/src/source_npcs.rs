//! Bounded ground NPC subset. Native schedules, equipment and ragdolls remain separate work.
use super::*;
use crate::source_player::{self, Actor, PlayerState};
use sandbox_catalog::spawn::Npc;
use std::sync::Arc;

#[derive(Component)]
pub struct NpcBody {
    pub id: String,
    definition: Npc,
    actor: Arc<Actor>,
    vertical: f32,
    phase: &'static str,
    phase_time: f32,
    cooldown: f32,
}
#[derive(Resource, Default)]
pub struct NpcSettings {
    pub disabled: bool,
    pub ignore_players: bool,
}
#[derive(Resource)]
pub struct PlayerLife {
    pub health: f32,
    pub dead_for: f32,
    spawn: Transform,
}
pub fn dead(world: &World) -> bool {
    world
        .get_resource::<PlayerLife>()
        .is_some_and(|s| s.health <= 0.)
}
pub fn definition(world: &World, id: &str) -> Option<Npc> {
    world
        .resource::<PlayState>()
        .spawn_catalog
        .npcs
        .iter()
        .find(|n| n.id == id && n.kind != "disabled")
        .cloned()
}
pub fn spawn(world: &mut World, id: &str) {
    let Some(n) = definition(world, id) else {
        return;
    };
    if dead(world) {
        return;
    }
    let Some(eye) = camera(world) else { return };
    let Some(hit) = tools::trace(world, eye, None) else {
        world.resource_mut::<PlayState>().status = "Aim at a clear floor to spawn an NPC".into();
        return;
    };
    if hit.normal.y < 0.7 {
        world.resource_mut::<PlayState>().status =
            "This ground NPC requires an upward-facing floor".into();
        return;
    }
    let yaw = world.resource::<PlayerState>().yaw;
    spawn::enqueue_npc(
        world,
        n.model,
        hit.point,
        Quat::from_rotation_y(yaw - n.forward_yaw),
        id.into(),
    );
    world.resource_mut::<weapons::WeaponState>().equip_blocked = true;
    let mut p = world.resource_mut::<PlayState>();
    p.menu_open = false;
    p.search_focus = false;
    p.dirty = true;
}

#[derive(Resource, Default)]
struct NpcClips(BTreeMap<String, BTreeMap<String, crate::source_animation::Clip>>);

pub fn prepare(world: &mut World, id: &str) -> Result<()> {
    if world
        .get_resource::<NpcClips>()
        .is_some_and(|cache| cache.0.contains_key(id))
    {
        return Ok(());
    }
    let n = definition(world, id).ok_or("unsupported NPC identity")?;
    gpu_model(world, &n.model)?;
    let source = world.resource::<MountedSource>();
    let skeleton =
        crate::source_animation::Skeleton::read(&source.mounts.read(&source.bsp, &n.model)?)?;
    let mut clips = BTreeMap::new();
    for (key, candidates) in [("idle", &n.idle), ("walk", &n.walk), ("attack", &n.attack)] {
        let mut found = None;
        for candidate in candidates.split('|').filter(|s| !s.is_empty()) {
            let (path, name) = candidate.split_once("::").unwrap_or((&n.model, candidate));
            if let Ok(clip) = crate::source_animation::load_clip(
                &source.mounts,
                &source.bsp,
                path,
                name,
                &skeleton,
            ) {
                found = Some(clip);
                break;
            }
        }
        clips.insert(
            key.into(),
            found.ok_or_else(|| format!("{} has no supported {key} clip", n.id))?,
        );
    }
    if !world.contains_resource::<NpcClips>() {
        world.insert_resource(NpcClips::default());
    }
    world.resource_mut::<NpcClips>().0.insert(id.into(), clips);
    Ok(())
}

pub fn clear_at(world: &mut World, id: &str, feet: Vec3) -> bool {
    let Some(n) = definition(world, id) else {
        return false;
    };
    let shape = Collider::capsule_y(n.height * 0.5 - n.radius, n.radius);
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
        feet + Vec3::Y * n.height * 0.5,
        Quat::IDENTITY,
        &shape,
        QueryFilter::default()
            .groups(CollisionGroups::new(Group::GROUP_4, Group::ALL))
            .exclude_sensors(),
        |_| {
            blocked = true;
            false
        },
    );
    !blocked
}

pub fn attach(world: &mut World, entity: Entity, id: &str) -> Result<()> {
    let n = definition(world, id).ok_or("unsupported NPC identity")?;
    let rules = world
        .resource::<PlayState>()
        .spawn_catalog
        .npc_rules
        .clone();
    if world.query::<&NpcBody>().iter(world).count() >= rules.limit {
        return Err("NPC limit reached".into());
    }
    prepare(world, id)?;
    let mut actor = source_player::actor(world, &n.model, 0, Some(entity))?;
    actor.clips = world.resource::<NpcClips>().0[id].clone();
    let children: Vec<_> = world
        .get::<Children>(entity)
        .map(|c| c.iter().collect())
        .unwrap_or_default();
    for child in children {
        if child != actor.root {
            world.despawn(child);
        }
    }
    // Model origin stays at feet. The local collider is centered half a hull above it.
    let shape = Collider::compound(vec![(
        Vec3::Y * n.height * 0.5,
        Quat::IDENTITY,
        Collider::capsule_y(n.height * 0.5 - n.radius, n.radius),
    )]);
    world.entity_mut(entity).insert((
        RigidBody::KinematicPositionBased,
        shape,
        CollisionGroups::new(Group::GROUP_4, Group::ALL),
        KinematicCharacterController {
            filter_groups: Some(CollisionGroups::new(Group::GROUP_4, Group::ALL)),
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(rules.step_height),
                min_width: CharacterLength::Absolute(0.1),
                include_dynamic_bodies: false,
            }),
            snap_to_ground: Some(CharacterLength::Absolute(rules.step_height)),
            ..default()
        },
        weapons::Health(n.health),
        NpcBody {
            id: id.into(),
            definition: n,
            actor: Arc::new(actor),
            vertical: 0.,
            phase: "idle",
            phase_time: 0.,
            cooldown: 0.,
        },
    ));
    Ok(())
}

fn line_hit(world: &mut World, from: Vec3, to: Vec3, exclude: Entity) -> Option<Entity> {
    let delta = to - from;
    let distance = delta.length();
    if distance < 0.001 {
        return None;
    }
    let mut q = world.query::<(
        &RapierContextSimulation,
        &RapierContextColliders,
        &RapierRigidBodySet,
        &RapierContextJoints,
        &RapierQueryPipeline,
    )>();
    let Ok((simulation, colliders, rigidbody_set, joints, query_pipeline)) = q.single(world) else {
        return Some(exclude);
    };
    let context = bevy_rapier3d::plugin::RapierContext {
        simulation,
        colliders,
        rigidbody_set,
        joints,
        query_pipeline,
    };
    context
        .cast_ray(
            from,
            delta / distance,
            distance,
            true,
            QueryFilter::default()
                .exclude_collider(exclude)
                .exclude_sensors(),
        )
        .map(|(e, _)| e)
}
fn hostile(a: &str, b: &str) -> bool {
    a != b && a != "neutral" && b != "neutral"
}

pub fn think(world: &mut World) {
    if !world.contains_resource::<PlayerState>() {
        return;
    }
    let player_id = world.resource::<PlayerState>().entity;
    let Some(player_transform) = world.get::<Transform>(player_id).copied() else {
        return;
    };
    let rules = world
        .resource::<PlayState>()
        .spawn_catalog
        .npc_rules
        .clone();
    if !world.contains_resource::<PlayerLife>() {
        world.insert_resource(PlayerLife {
            health: rules.player_health,
            dead_for: 0.,
            spawn: player_transform,
        });
    }
    let dt = world.resource::<Time<Fixed>>().delta_secs();
    let paused = crate::source_frontend::active(world)
        || !world
            .query_filtered::<&Window, With<PrimaryWindow>>()
            .iter(world)
            .any(|w| w.focused);
    let disabled = paused || world.resource::<NpcSettings>().disabled;
    if !paused && dead(world) {
        world.resource_mut::<PlayerLife>().dead_for += dt;
        if world.resource::<PlayerLife>().dead_for >= rules.respawn_seconds {
            let transform = world.resource::<PlayerLife>().spawn;
            if vehicles::clear_hull(world, transform.translation) {
                vehicles::release(world, None);
                weapons::clear_transients(world);
                world.entity_mut(player_id).insert(transform);
                if let Some(mut controller) =
                    world.get_mut::<KinematicCharacterController>(player_id)
                {
                    controller.translation = None;
                }
                let mut p = world.resource_mut::<PlayerState>();
                p.vertical = 0.;
                p.horizontal_velocity = Vec3::ZERO;
                p.direction = Vec3::ZERO;
                p.jump = false;
                p.grounded = false;
                let mut life = world.resource_mut::<PlayerLife>();
                life.health = rules.player_health;
                life.dead_for = 0.;
                world.resource_mut::<PlayState>().held = None;
                world.resource_mut::<PlayState>().beam_active = false;
                world.resource_mut::<PlayState>().tools.stage = None;
                world.resource_mut::<PlayState>().status =
                    "Respawned | NPC combat is an explicitly scoped prototype".into();
            } else {
                world.resource_mut::<PlayState>().status =
                    "Respawn point blocked. Load a clear scene from the menu to recover.".into();
            }
        }
    }
    let actors: Vec<_> = world
        .query::<(Entity, &Transform, &NpcBody)>()
        .iter(world)
        .map(|(e, t, n)| {
            (
                e,
                t.translation + Vec3::Y * n.definition.height * 0.5,
                n.definition.faction.clone(),
            )
        })
        .collect();
    let ids: Vec<_> = actors.iter().map(|a| a.0).collect();
    let player_position = world.get::<Transform>(player_id).unwrap().translation;
    for entity in ids {
        if disabled {
            if let Some(mut controller) = world.get_mut::<KinematicCharacterController>(entity) {
                controller.translation = None;
            }
            continue;
        }
        let Some(n) = world.get::<NpcBody>(entity) else {
            continue;
        };
        let c = n.definition.clone();
        let mut vertical = n.vertical;
        let cooldown = (n.cooldown - dt).max(0.);
        let from = world.get::<Transform>(entity).unwrap().translation + Vec3::Y * c.height * 0.5;
        let mut candidates: Vec<_> = actors
            .iter()
            .filter(|(e, _, f)| {
                *e != entity && hostile(&c.faction, f) && world.get::<NpcBody>(*e).is_some()
            })
            .map(|(e, p, _)| (*e, *p))
            .collect();
        if !world.resource::<NpcSettings>().ignore_players
            && !dead(world)
            && !world.resource::<PlayerState>().noclip
            && hostile(&c.faction, "resistance")
        {
            candidates.push((player_id, player_position));
        }
        candidates.sort_by(|a, b| {
            from.distance_squared(a.1)
                .total_cmp(&from.distance_squared(b.1))
        });
        let target = if c.kind == "passive" {
            None
        } else {
            candidates.into_iter().find(|(e, p)| {
                from.distance(*p) <= c.sight
                    && line_hit(world, from, *p, entity).is_none_or(|hit| hit == *e)
            })
        };
        let mut movement = Vec3::ZERO;
        let mut phase = "idle";
        let mut attacked = false;
        if let Some((target, point)) = target {
            let delta = point - from;
            let flat = Vec3::new(delta.x, 0., delta.z).normalize_or_zero();
            if flat.length_squared() > 0. {
                world.get_mut::<Transform>(entity).unwrap().rotation =
                    Quat::from_rotation_y((-flat.x).atan2(-flat.z) - c.forward_yaw);
            }
            if delta.length() > c.range {
                movement = flat * c.speed * dt;
                phase = "walk";
            } else {
                phase = "attack";
                if cooldown <= 0. {
                    if target == player_id {
                        let mut life = world.resource_mut::<PlayerLife>();
                        life.health = (life.health - c.damage).max(0.);
                        let health = life.health;
                        world.resource_mut::<PlayState>().status = if health <= 0. {
                            "You died. Respawning...".into()
                        } else {
                            format!("Health: {health:.0}")
                        };
                    } else {
                        weapons::damage(world, target, c.damage, flat);
                    }
                    attacked = true;
                }
            }
        }
        if world
            .get::<KinematicCharacterControllerOutput>(entity)
            .is_some_and(|o| o.grounded)
        {
            vertical = 0.;
        }
        vertical -= rules.gravity * dt;
        movement.y = vertical * dt;
        if let Some(mut controller) = world.get_mut::<KinematicCharacterController>(entity) {
            controller.translation = Some(movement);
        }
        if let Some(mut n) = world.get_mut::<NpcBody>(entity) {
            n.vertical = vertical;
            n.cooldown = if attacked { c.interval } else { cooldown };
            if n.phase != phase || attacked {
                n.phase = phase;
                n.phase_time = 0.;
            } else {
                n.phase_time += dt;
            }
        }
    }
}
pub fn animate(world: &mut World) {
    let actors: Vec<_> = world
        .query::<&NpcBody>()
        .iter(world)
        .map(|n| (n.actor.clone(), n.phase, n.phase_time))
        .collect();
    for (actor, phase, time) in actors {
        if let Some(clip) = actor.clips.get(phase) {
            let pose = clip.sample_mode(time, phase != "attack");
            let globals = actor.skeleton.globals(&pose);
            source_player::skin(world, &actor, &globals);
        }
    }
}
