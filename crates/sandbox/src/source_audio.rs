//! Mounted one-shot audio and bounded impact feedback. Not Source mixer parity.
use super::*;
use bevy::audio::{AudioSinkPlayback, Decodable, SpatialScale, Volume};
use bevy::tasks::{block_on, poll_once, IoTaskPool, Task};
use std::collections::{BTreeSet, VecDeque};
#[derive(Component)]
struct Voice {
    born: f32,
    gain: f32,
    range: f32,
    point: Option<Vec3>,
    weapon: Option<String>,
    loop_id: Option<String>,
}
#[derive(Component)]
struct Listener;
#[derive(Component, Default)]
pub(crate) struct Motion(Vec3);
struct Request {
    id: String,
    point: Option<Vec3>,
    gain: f32,
}
#[derive(Resource)]
pub(crate) struct Audio {
    pub catalog: sandbox_catalog::audio::Catalog,
    pub volume: f32,
    pub shake_enabled: bool,
    pending: VecDeque<String>,
    task: Option<(String, Task<std::result::Result<AudioSource, String>>)>,
    cache: BTreeMap<String, Option<Handle<AudioSource>>>,
    queue: VecDeque<Request>,
    cooldown: BTreeMap<String, f32>,
    contacts: BTreeMap<Entity, f32>,
    serial: usize,
    shake: f32,
    listener: Vec3,
    cache_bytes: usize,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Preferences {
    volume: f32,
    shake_enabled: bool,
}
impl Default for Audio {
    fn default() -> Self {
        let catalog = crate::compiled_audio_catalog();
        let pending = catalog
            .events
            .iter()
            .flat_map(|e| e.paths.split('|').map(str::to_string))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut volume = catalog.config.master;
        let mut shake_enabled = true;
        if let Ok(raw) = std::fs::read(project_root().join("local/audio-settings.json")) {
            if let Ok(p) = serde_json::from_slice::<Preferences>(&raw) {
                if p.volume.is_finite() && (0. ..=1.).contains(&p.volume) {
                    volume = p.volume;
                    shake_enabled = p.shake_enabled;
                }
            }
        }
        Self {
            catalog,
            volume,
            shake_enabled,
            pending,
            task: None,
            cache: BTreeMap::new(),
            queue: VecDeque::new(),
            cooldown: BTreeMap::new(),
            contacts: BTreeMap::new(),
            serial: 0,
            shake: 0.,
            listener: Vec3::ZERO,
            cache_bytes: 0,
        }
    }
}
pub(crate) fn settings(world: &mut World, delta: f32, shake: bool) {
    let mut s = world.resource_mut::<Audio>();
    if shake {
        s.shake_enabled = !s.shake_enabled;
        s.shake = 0.;
    } else {
        s.volume = (s.volume + delta).clamp(0., 1.);
    }
    let p = Preferences {
        volume: s.volume,
        shake_enabled: s.shake_enabled,
    };
    let root = world.resource::<PlayState>().storage_root.clone();
    let result = (|| -> Result<()> {
        std::fs::create_dir_all(&root)?;
        std::fs::write(
            root.join("audio-settings.json"),
            serde_json::to_vec_pretty(&p)?,
        )?;
        Ok(())
    })();
    if let Err(e) = result {
        world.resource_mut::<PlayState>().status = format!("Audio settings save failed: {e}");
    }
}
pub(crate) fn emit(world: &mut World, id: &str, point: Option<Vec3>, gain: f32) {
    let mut s = world.resource_mut::<Audio>();
    enqueue(&mut s, id, point, gain);
}
fn enqueue(s: &mut Audio, id: &str, point: Option<Vec3>, gain: f32) {
    if !gain.is_finite()
        || point.is_some_and(|p| !p.is_finite())
        || s.queue.len() >= s.catalog.config.max_voices * 2
    {
        return;
    }
    s.queue.push_back(Request {
        id: id.into(),
        point,
        gain: gain.clamp(0., 1.),
    });
}
fn prepare(world: &mut World) {
    let ready = {
        let mut s = world.resource_mut::<Audio>();
        s.task.as_mut().and_then(|(_, t)| block_on(poll_once(t)))
    };
    if let Some(result) = ready {
        let (path, _) = world.resource_mut::<Audio>().task.take().unwrap();
        let handle = match result {
            Ok(source) => {
                let mut audio = world.resource_mut::<Audio>();
                if source.bytes.len()
                    > audio
                        .catalog
                        .config
                        .max_cache_bytes
                        .saturating_sub(audio.cache_bytes)
                {
                    eprintln!("AUDIO_CACHE_LIMIT {path}");
                    None
                } else {
                    audio.cache_bytes += source.bytes.len();
                    Some(world.resource_mut::<Assets<AudioSource>>().add(source))
                }
            }
            Err(e) => {
                eprintln!("AUDIO_ASSET_REJECTED {path}: {e}");
                world.resource_mut::<PlayState>().status =
                    format!("Sound unavailable: {path} ({e})");
                None
            }
        };
        world.resource_mut::<Audio>().cache.insert(path, handle);
    }
    if world.resource::<Audio>().task.is_some() {
        return;
    }
    let Some(path) = world.resource_mut::<Audio>().pending.pop_front() else {
        return;
    };
    let mounted = world.resource::<MountedSource>();
    let mounts = mounted.mounts.clone();
    let bsp = mounted.bsp.clone();
    let config = world.resource::<Audio>().catalog.config.clone();
    let name = path.clone();
    let task = IoTaskPool::get().spawn(async move {
        let bytes = mounts.read(&bsp, &name).map_err(|e| e.to_string())?;
        if bytes.len() > config.max_asset_bytes {
            return Err("sound exceeds byte budget".into());
        }
        let source = AudioSource {
            bytes: bytes.into(),
        };
        let count = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            source.decoder().take(config.max_samples + 1).count()
        }))
        .map_err(|_| "unsupported or malformed sound".to_string())?;
        if count == 0 || count > config.max_samples {
            return Err("decoded sound exceeds sample budget or is empty".into());
        }
        Ok(source)
    });
    world.resource_mut::<Audio>().task = Some((path, task));
}
fn stop(world: &mut World, e: Entity) {
    if let Some(s) = world.get::<AudioSink>(e) {
        s.stop();
    }
    if let Some(s) = world.get::<SpatialAudioSink>(e) {
        s.stop();
    }
    if world.get_entity(e).is_ok() {
        world.despawn(e);
    }
}
pub(crate) fn clear(world: &mut World) {
    let entities: Vec<_> = world
        .query_filtered::<Entity, With<Voice>>()
        .iter(world)
        .collect();
    for e in entities {
        stop(world, e);
    }
    let mut s = world.resource_mut::<Audio>();
    s.queue.clear();
    s.cooldown.clear();
    s.contacts.clear();
    s.shake = 0.;
}
pub(crate) fn update(world: &mut World) {
    prepare(world);
    let focused = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .iter(world)
        .any(|w| w.focused);
    let blocked = crate::source_frontend::active(world)
        || world.resource::<PlayState>().menu_open
        || !focused
        || npcs::dead(world);
    if blocked {
        clear(world);
        return;
    }
    let camera = world
        .query_filtered::<&Transform, With<SourceCamera>>()
        .iter(world)
        .next()
        .copied()
        .unwrap_or_default();
    world.resource_mut::<Audio>().listener = camera.translation;
    if let Some(e) = world
        .query_filtered::<Entity, With<Listener>>()
        .iter(world)
        .next()
    {
        *world.get_mut::<Transform>(e).unwrap() = camera;
    } else {
        world.spawn((Listener, SpatialListener::new(0.2), camera));
    }
    let now = world.resource::<Time>().elapsed_secs();
    let dt = world.resource::<Time>().delta_secs();
    {
        let mut s = world.resource_mut::<Audio>();
        s.shake *= (-s.catalog.config.shake_decay * dt).exp();
    }
    let volume = world.resource::<Audio>().volume;
    let ttl = world.resource::<Audio>().catalog.config.voice_seconds;
    let p = world.resource::<PlayState>();
    let desired_loop = (p.physgun
        && p.beam_active
        && world.resource::<vehicles::Occupancy>().vehicle.is_none()
        && !tools::devices::viewing(world))
    .then(|| (if p.held.is_some() {
        "physgun.hold"
    } else {
        "physgun.scan"
    }.to_owned(), 1.0));
    // Occupied vehicle and physgun loops are mutually exclusive. Engine ownership is
    // the live occupancy, not the currently selected weapon.
    let desired_loop = vehicles::engine_loop(world).or(desired_loop);
    let desired_id = desired_loop.as_ref().map(|(id, _)| id.as_str());
    let loop_speed = desired_loop.as_ref().map_or(1., |(_, speed)| *speed);
    let voices: Vec<_> = world
        .query::<(Entity, &Voice)>()
        .iter(world)
        .map(|(e, v)| {
            (
                e,
                v.born,
                v.gain,
                v.range,
                v.point,
                v.weapon.clone(),
                v.loop_id.clone(),
            )
        })
        .collect();
    for (e, born, gain, range, point, weapon, loop_id) in voices {
        if (loop_id.is_none() && now - born > ttl)
            || loop_id
                .as_deref()
                .is_some_and(|id| Some(id) != desired_id)
            || weapon
                .as_ref()
                .is_some_and(|id| id != &world.resource::<PlayState>().active_weapon)
        {
            stop(world, e);
            continue;
        }
        let attenuation = point.map_or(1., |p| {
            (1. - p.distance(camera.translation) / range)
                .clamp(0., 1.)
                .powi(2)
        });
        let v = Volume::Linear(volume * gain * attenuation);
        if let Some(mut sink) = world.get_mut::<AudioSink>(e) {
            sink.set_volume(v);
            if loop_id.is_some() {
                sink.set_speed(loop_speed);
            }
        }
        if let Some(mut sink) = world.get_mut::<SpatialAudioSink>(e) {
            sink.set_volume(v);
            if loop_id.is_some() {
                sink.set_speed(loop_speed);
            }
        }
    }
    if let Some(id) = desired_id {
        let exists = world
            .query::<&Voice>()
            .iter(world)
            .any(|v| v.loop_id.as_deref() == Some(id));
        if !exists && volume > 0. {
            if let Some(event) = world
                .resource::<Audio>()
                .catalog
                .events
                .iter()
                .find(|e| e.id == id)
                .cloned()
            {
                let handle = event
                    .paths
                    .split('|')
                    .find_map(|p| world.resource::<Audio>().cache.get(p).cloned().flatten());
                if let Some(handle) = handle {
                    if world.query::<&Voice>().iter(world).count()
                        < world.resource::<Audio>().catalog.config.max_voices
                    {
                        let weapon = id.starts_with("physgun.")
                            .then(|| world.resource::<PlayState>().active_weapon.clone());
                        world.spawn((
                            Voice {
                                born: now,
                                gain: event.volume,
                                range: event.range,
                                point: None,
                                weapon,
                                loop_id: Some(id.into()),
                            },
                            AudioPlayer::new(handle),
                            PlaybackSettings {
                                volume: Volume::Linear(volume * event.volume),
                                speed: loop_speed,
                                ..PlaybackSettings::LOOP
                            },
                        ));
                    }
                }
            }
        }
    }
    let queue = std::mem::take(&mut world.resource_mut::<Audio>().queue);
    for req in queue {
        let Some(event) = world
            .resource::<Audio>()
            .catalog
            .events
            .iter()
            .find(|e| e.id == req.id)
            .cloned()
        else {
            continue;
        };
        if world
            .resource::<Audio>()
            .cooldown
            .get(&req.id)
            .is_some_and(|until| now < *until)
        {
            continue;
        }
        let attenuation = req.point.map_or(1., |p| {
            (1. - p.distance(camera.translation) / event.range)
                .clamp(0., 1.)
                .powi(2)
        });
        if attenuation == 0. || volume == 0. {
            continue;
        }
        let choices: Vec<_> = event
            .paths
            .split('|')
            .filter_map(|p| world.resource::<Audio>().cache.get(p).cloned().flatten())
            .collect();
        // Never replay an old shot after asynchronous loading finishes.
        if choices.is_empty() {
            continue;
        }
        let mut s = world.resource_mut::<Audio>();
        let handle = choices[s.serial % choices.len()].clone();
        s.serial = s.serial.wrapping_add(1);
        s.cooldown.insert(req.id, now + event.cooldown);
        let max = s.catalog.config.max_voices;
        if world.query::<&Voice>().iter(world).count() >= max {
            continue;
        }
        let gain = event.volume * req.gain;
        let weapon = req
            .point
            .is_none()
            .then(|| world.resource::<PlayState>().active_weapon.clone());
        world.spawn((
            Voice {
                born: now,
                gain,
                range: event.range,
                point: req.point,
                weapon,
                loop_id: None,
            },
            AudioPlayer::new(handle),
            PlaybackSettings {
                volume: Volume::Linear(volume * gain * attenuation),
                spatial: req.point.is_some(),
                spatial_scale: Some(SpatialScale::new(0.1)),
                ..PlaybackSettings::DESPAWN
            },
            Transform::from_translation(req.point.unwrap_or(camera.translation)),
        ));
    }
}
pub(crate) fn record_motion(mut props: Query<(&Velocity, &mut Motion)>) {
    for (v, mut prior) in &mut props {
        prior.0 = v.linvel;
    }
}
pub(crate) fn contacts(
    mut events: EventReader<ContactForceEvent>,
    time: Res<Time>,
    context: ReadRapierContext,
    props: Query<(&Motion, &tools::Properties, &RigidBody), With<SpawnedProp>>,
    mut audio: ResMut<Audio>,
) {
    let Ok(context) = context.single() else {
        events.clear();
        return;
    };
    let now = time.elapsed_secs();
    let c = audio.catalog.config.clone();
    audio.contacts.retain(|_, until| *until > now);
    for event in events.read() {
        if event.total_force_magnitude < c.impact_force {
            continue;
        }
        let a = props.get(event.collider1).ok();
        let b = props.get(event.collider2).ok();
        let chosen = match (a, b) {
            (Some(a), _) if *a.2 == RigidBody::Dynamic => Some((event.collider1, a)),
            (_, Some(b)) if *b.2 == RigidBody::Dynamic => Some((event.collider2, b)),
            _ => None,
        };
        let Some((entity, (_, p, _))) = chosen else {
            continue;
        };
        if audio.contacts.contains_key(&event.collider1)
            || audio.contacts.contains_key(&event.collider2)
        {
            continue;
        }
        let Some(pair) = context.contact_pair(event.collider1, event.collider2) else {
            continue;
        };
        let Some((point, normal)) = pair
            .manifolds()
            .find_map(|m| m.solver_contact(0).map(|p| (p.point(), m.normal())))
        else {
            continue;
        };
        let relative = a.map_or(Vec3::ZERO, |a| a.0 .0) - b.map_or(Vec3::ZERO, |b| b.0 .0);
        let speed = relative.dot(normal).abs();
        if speed < c.impact_speed {
            continue;
        }
        audio.contacts.insert(entity, now + c.impact_cooldown);
        if a.is_some() {
            audio
                .contacts
                .insert(event.collider1, now + c.impact_cooldown);
        }
        if b.is_some() {
            audio
                .contacts
                .insert(event.collider2, now + c.impact_cooldown);
        }
        let id = format!("impact.{}", p.physics_material);
        let id = if audio.catalog.events.iter().any(|e| e.id == id) {
            id
        } else {
            "impact.default".into()
        };
        let severity = (event.total_force_magnitude / c.impact_force).ln_1p();
        enqueue(
            &mut audio,
            &id,
            Some(point),
            (severity * 0.3).clamp(0.1, 1.),
        );
        let range = audio
            .catalog
            .events
            .iter()
            .find(|e| e.id == id)
            .map_or(1., |e| e.range);
        let attenuation = (1. - point.distance(audio.listener) / range)
            .clamp(0., 1.)
            .powi(2);
        if audio.shake_enabled {
            audio.shake = (audio.shake + severity * c.shake_gain * attenuation).min(c.shake_limit);
        }
    }
}
pub(crate) fn shake(world: &mut World) -> Quat {
    if !world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .iter(world)
        .any(|w| w.focused)
        || npcs::dead(world)
    {
        return Quat::IDENTITY;
    }
    let Some(s) = world.get_resource::<Audio>() else {
        return Quat::IDENTITY;
    };
    if !s.shake_enabled
        || crate::source_frontend::active(world)
        || world.resource::<PlayState>().menu_open
    {
        return Quat::IDENTITY;
    }
    let t = world.resource::<Time>().elapsed_secs();
    Quat::from_euler(
        EulerRot::YXZ,
        (t * 47.).sin() * s.shake,
        (t * 61.).sin() * s.shake * 0.7,
        0.,
    )
}
