//! Bounded FIFO for interactive spawns. Only immutable asset data leaves the World.
use super::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use std::{collections::VecDeque, time::Instant};

#[derive(Default)]
pub(super) struct FrameSamples {
    values: Vec<f32>,
    elapsed: f32,
    config: Option<sandbox_catalog::performance::PerformanceConfig>,
}
pub(super) fn report(
    time: Res<Time<bevy::time::Real>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    props: Query<(), With<SpawnedProp>>,
    queue: Res<PendingSpawns>,
    mut samples: Local<FrameSamples>,
) {
    if !windows.iter().any(|w| w.focused) {
        samples.values.clear();
        samples.elapsed = 0.;
        return;
    }
    let dt = time.delta_secs();
    if dt <= 0. {
        return;
    }
    let config = samples
        .config
        .get_or_insert_with(crate::compiled_performance_config);
    let (interval, budget) = (config.report_seconds, config.frame_budget_ms);
    samples.values.push(dt * 1000.);
    samples.elapsed += dt;
    if samples.elapsed < interval {
        return;
    }
    samples.values.sort_unstable_by(f32::total_cmp);
    let n = samples.values.len();
    let over = samples.values.iter().filter(|v| **v > budget).count();
    println!(
        "FRAME_TIMING frames={n} median_ms={:.2} p95_ms={:.2} max_ms={:.2} budget_ms={budget:.3} over_budget={over} props={} pending={}",
        samples.values[n / 2],
        samples.values[((n as f32 * 0.95).ceil() as usize).saturating_sub(1)],
        samples.values[n - 1],
        props.iter().count(),
        queue.requests.len()
    );
    samples.values.clear();
    samples.elapsed = 0.;
}

struct Request {
    vehicle: Option<String>,
    npc: Option<String>,
    rotation: Quat,
    path: String,
    position: Vec3,
    normal: Vec3,
    submitted: Instant,
}
#[derive(Resource)]
pub(super) struct PendingSpawns {
    requests: VecDeque<Request>,
    active: Option<(String, Task<Result<PreparedModel>>)>,
    limit: usize,
}
impl Default for PendingSpawns {
    fn default() -> Self {
        Self {
            requests: VecDeque::new(),
            active: None,
            limit: crate::compiled_performance_config().spawn_queue_limit,
        }
    }
}
pub(super) fn enqueue(world: &mut World, path: String, position: Vec3, normal: Vec3) {
    enqueue_typed(world, path, position, normal, Quat::IDENTITY, None, None);
}
pub(super) fn enqueue_vehicle(
    world: &mut World,
    path: String,
    position: Vec3,
    normal: Vec3,
    rotation: Quat,
    id: String,
) {
    enqueue_typed(world, path, position, normal, rotation, Some(id), None);
}
pub(super) fn enqueue_npc(
    world: &mut World,
    path: String,
    position: Vec3,
    rotation: Quat,
    id: String,
) {
    enqueue_typed(world, path, position, Vec3::Y, rotation, None, Some(id));
}
fn enqueue_typed(
    world: &mut World,
    path: String,
    position: Vec3,
    normal: Vec3,
    rotation: Quat,
    vehicle: Option<String>,
    npc: Option<String>,
) {
    let count = world.query::<&SpawnedProp>().iter(world).count();
    let npcs = world.query::<&npcs::NpcBody>().iter(world).count();
    let queue = world.resource::<PendingSpawns>();
    if npc.is_some()
        && npcs + queue.requests.iter().filter(|r| r.npc.is_some()).count()
            >= world.resource::<PlayState>().spawn_catalog.npc_rules.limit
    {
        world.resource_mut::<PlayState>().status = "NPC limit reached".into();
        return;
    }
    if queue.requests.len() >= queue.limit
        || count + queue.requests.len() >= world.resource::<PlayState>().config.max_props
    {
        world.resource_mut::<PlayState>().status = "Spawn queue or prop limit reached".into();
        return;
    }
    if let Err(e) = crate::source_assets::virtual_path(&path) {
        world.resource_mut::<PlayState>().status = format!("Cannot spawn: {e}");
        return;
    }
    world.resource_mut::<PlayState>().status = format!("Loading {path} (Z cancels pending spawn)");
    if vehicle.is_none() && npc.is_none() {
        world.resource_mut::<PlayState>().selected = path.clone();
    }
    world
        .resource_mut::<PendingSpawns>()
        .requests
        .push_back(Request {
            vehicle,
            npc,
            rotation,
            path,
            position,
            normal,
            submitted: Instant::now(),
        });
}
pub(super) fn cancel_latest(world: &mut World) -> bool {
    let cancelled = world
        .resource_mut::<PendingSpawns>()
        .requests
        .pop_back()
        .is_some();
    if cancelled {
        world.resource_mut::<PlayState>().status = "Cancelled pending spawn".into();
    }
    cancelled
}
pub(super) fn clear(world: &mut World) {
    if let Some(mut queue) = world.get_resource_mut::<PendingSpawns>() {
        queue.requests.clear();
        // Keep at most one worker running until it yields. Its result is discarded
        // if no matching request remains, rather than accumulating cancelled jobs.
    }
}
pub(super) fn complete(world: &mut World) {
    if crate::source_frontend::active(world) {
        return;
    }
    world.resource_scope(|world, mut queue: Mut<PendingSpawns>| {
        if let Some((_, task)) = queue.active.as_mut() {
            let Some(result) = block_on(poll_once(task)) else {
                return;
            };
            let (path, _) = queue.active.take().unwrap();
            if queue.requests.front().is_some_and(|r| r.path == path) {
                match result {
                    Ok(prepared) => {
                        install_model(world, format!("{path}#0"), prepared);
                    }
                    Err(e) => {
                        queue.requests.pop_front();
                        world.resource_mut::<PlayState>().status =
                            format!("Cannot spawn {path}: {e}");
                        return;
                    }
                }
            }
        }
        let Some(request) = queue.requests.front() else {
            return;
        };
        if !world
            .resource::<PlayState>()
            .cache
            .contains_key(&format!("{}#0", request.path))
        {
            let source = world.resource::<MountedSource>().clone();
            let path = request.path.clone();
            let task_path = path.clone();
            queue.active = Some((
                path,
                AsyncComputeTaskPool::get().spawn(async move {
                    let start = Instant::now();
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        prepare_model(&source, &task_path, 0)
                    }))
                    .unwrap_or_else(|_| Err("unsupported or malformed model asset".into()));
                    println!(
                        "SPAWN_PREPARE path={task_path} cpu_ms={:.2} ok={}",
                        start.elapsed().as_secs_f64() * 1000.,
                        result.is_ok()
                    );
                    result
                }),
            ));
            return;
        }
        // Commit at most one request per frame, preserving click order and position.
        let request = queue.requests.pop_front().unwrap();
        let start = Instant::now();
        let count = world.query::<&SpawnedProp>().iter(world).count();
        if count >= world.resource::<PlayState>().config.max_props {
            world.resource_mut::<PlayState>().status = "Prop limit reached".into();
            return;
        }
        // Place the lowest support point outside the hit plane, not the model origin.
        // Source models frequently have off-centre origins or extend below local zero.
        let model = &world.resource::<PlayState>().cache[&format!("{}#0", request.path)];
        let support = model
            .geometry
            .iter()
            .flat_map(|g| &g.positions)
            .map(|p| (request.rotation * Vec3::from_array(*p)).dot(request.normal))
            .fold(f32::INFINITY, f32::min);
        let position = request.position
            + request.normal
                * (crate::compiled_frontend_config().spawn_clearance
                    - if request.npc.is_some() { 0. } else { support });
        if let Some(id) = &request.npc {
            if !npcs::clear_at(world, id, position) {
                world.resource_mut::<PlayState>().status = "NPC spawn is blocked".into();
                return;
            }
            if let Err(error) = npcs::prepare(world, id) {
                world.resource_mut::<PlayState>().status = format!("NPC assets failed: {error}");
                return;
            }
        }
        remember(world);
        match spawn_model(world, &request.path, position, request.rotation, false) {
            Ok(entity) => {
                if let Some(id) = &request.vehicle {
                    if let Err(error) = vehicles::attach(world, entity, id) {
                        world.despawn(entity);
                        world.resource_mut::<PlayState>().undo.pop();
                        world.resource_mut::<PlayState>().status =
                            format!("Vehicle failed: {error}");
                        return;
                    }
                }
                if let Some(id) = &request.npc {
                    if let Err(error) = npcs::attach(world, entity, id) {
                        world.despawn(entity);
                        world.resource_mut::<PlayState>().undo.pop();
                        world.resource_mut::<PlayState>().status = format!("NPC failed: {error}");
                        return;
                    }
                }
                world.resource_mut::<PlayState>().status = format!("Spawned {}", request.path);
                println!(
                    "SPAWN_COMMIT path={} entity_ms={:.2} request_ms={:.2}",
                    request.path,
                    start.elapsed().as_secs_f64() * 1000.,
                    request.submitted.elapsed().as_secs_f64() * 1000.
                );
            }
            Err(e) => {
                world.resource_mut::<PlayState>().undo.pop();
                world.resource_mut::<PlayState>().status =
                    format!("Cannot spawn {}: {e}", request.path);
            }
        }
    });
}
