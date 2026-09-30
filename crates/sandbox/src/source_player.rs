//! Original player/weapon skinning and grounded locomotion, independent of sandbox props.
use crate::{
    compiled_player_config,
    source_animation::{self, Clip, Skeleton},
    source_assets::{Geometry, MountedSource},
    source_play::{self, PlayState, SourceCamera},
};
use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::MouseMotion,
    prelude::*,
    render::{
        camera::ClearColorConfig,
        view::{NoFrustumCulling, RenderLayers},
    },
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;
use sandbox_catalog::{player::PlayerConfig, Result};
use std::collections::BTreeMap;

#[derive(Component)]
pub struct PlayerBody;
#[derive(Component)]
pub(crate) struct PlayerViewCamera;
#[derive(Resource)]
pub struct PlayerState {
    pub config: PlayerConfig,
    pub entity: Entity,
    pub eye: Transform,
    pub muzzle: Vec3,
    pub glow_points: Vec<(Vec3, bool)>,
    pub local_velocity: Vec3,
    pub third_person: bool,
    pub noclip: bool,
    pub moving: bool,
    pub grounded: bool,
    pub yaw: f32,
    pub pitch: f32,
    pub direction: Vec3,
    pub jump: bool,
    pub vertical: f32,
    pub running: bool,
    pub precision_flight: bool,
    pub prop_rotation_delta: Vec2,
    pub crouched: bool,
    pub duck_fraction: f32,
    wants_crouch: bool,
    pub horizontal_velocity: Vec3,
    pub vehicle: Option<Entity>,
    jumped: bool,
    movement_submitted: bool,
}
pub struct SourcePlayerPlugin;
impl Plugin for SourcePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::source_effects::SourceEffectsPlugin);
        app.add_systems(Update, input.before(source_play::update_play))
            .add_systems(Update, sync_cursor.after(source_play::update_play))
            .add_systems(FixedUpdate, crouch.before(walk))
            .add_systems(FixedUpdate, walk.before(PhysicsSet::SyncBackend))
            .add_systems(FixedUpdate, finish_walk.after(PhysicsSet::Writeback))
            .add_systems(Update, visuals.after(source_play::tools::input));
    }
}
pub fn spawn(commands: &mut Commands, eye: Vec3, forward: Vec3) {
    let c = compiled_player_config();
    let entity = commands
        .spawn((
            PlayerBody,
            Name::new("Local player hull"),
            RigidBody::KinematicPositionBased,
            Collider::capsule_y(c.height * 0.5 - c.radius, c.radius),
            CollisionGroups::new(Group::GROUP_3, Group::ALL),
            Transform::from_translation(eye + Vec3::Y * (c.height * 0.5 - c.eye_height)),
            KinematicCharacterController {
                // Character sweeps do not inherit the body's collision groups in Rapier.
                filter_groups: Some(CollisionGroups::new(Group::GROUP_3, Group::ALL)),
                offset: CharacterLength::Absolute(0.005),
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(c.step_height),
                    min_width: CharacterLength::Absolute(0.1),
                    include_dynamic_bodies: false,
                }),
                snap_to_ground: Some(CharacterLength::Absolute(c.step_height)),
                max_slope_climb_angle: c.walkable_normal.acos(),
                min_slope_slide_angle: c.walkable_normal.acos(),
                ..default()
            },
        ))
        .id();
    let t = Transform::from_translation(eye).looking_to(forward, Vec3::Y);
    let (yaw, pitch, _) = t.rotation.to_euler(EulerRot::YXZ);
    commands.insert_resource(PlayerState {
        config: c,
        entity,
        eye: t,
        muzzle: eye,
        glow_points: Vec::new(),
        local_velocity: Vec3::ZERO,
        third_person: false,
        noclip: false,
        moving: false,
        grounded: false,
        yaw,
        pitch,
        direction: Vec3::ZERO,
        jump: false,
        vertical: 0.,
        running: false,
        precision_flight: false,
        prop_rotation_delta: Vec2::ZERO,
        crouched: false,
        duck_fraction: 0.,
        wants_crouch: false,
        horizontal_velocity: Vec3::ZERO,
        vehicle: None,
        jumped: false,
        movement_submitted: false,
    });
}
#[allow(clippy::too_many_arguments)]
pub fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut motion: EventReader<MouseMotion>,
    play: Res<PlayState>,
    weapon: Res<source_play::weapons::WeaponState>,
    remote: Res<source_play::tools::devices::Remote>,
    state: Option<ResMut<PlayerState>>,
    window: Query<&Window, With<PrimaryWindow>>,
    frontend: Option<Res<crate::source_frontend::Frontend>>,
    life: Option<Res<source_play::npcs::PlayerLife>>,
    mut exit: EventWriter<AppExit>,
) {
    let delta = motion.read().fold(Vec2::ZERO, |a, e| a + e.delta);
    let Some(mut s) = state else { return };
    let Ok(w) = window.single() else {
        return;
    };
    if keys.just_pressed(KeyCode::F10) && frontend.is_none() {
        exit.write(AppExit::Success);
    }
    s.prop_rotation_delta = Vec2::ZERO;
    s.direction = Vec3::ZERO;
    s.running = false;
    s.precision_flight = false;
    s.wants_crouch = false;
    if !w.focused
        || play.menu_open
        || remote.0.is_some()
        || frontend.as_ref().is_some_and(|f| f.is_open())
        || life.is_some_and(|l| l.health <= 0.)
    {
        s.jump = false;
        return;
    }
    if w.cursor_options.grab_mode != CursorGrabMode::None {
        if play.physgun && play.held.is_some() && keys.pressed(KeyCode::KeyE) {
            s.prop_rotation_delta = delta * play.sensitivity;
        } else {
            let aim_sensitivity = play
                .spawn_catalog
                .weapon_handling
                .iter()
                .find(|h| h.id == play.active_weapon)
                .map(|h| h.aim_sensitivity)
                .unwrap_or(1.);
            let sensitivity =
                play.sensitivity * (1. + (aim_sensitivity - 1.) * weapon.aim_fraction);
            s.yaw -= delta.x * sensitivity;
            s.pitch = (s.pitch - delta.y * sensitivity).clamp(-1.55, 1.55);
        }
    }
    if keys.just_pressed(KeyCode::F4) {
        s.third_person = !s.third_person;
    }
    if s.vehicle.is_some() {
        s.jump = false;
        return;
    }
    if keys.just_pressed(KeyCode::KeyV) {
        s.noclip = !s.noclip;
        s.vertical = 0.;
        s.horizontal_velocity = Vec3::ZERO;
        s.jumped = false;
        s.movement_submitted = false;
        s.grounded = false;
    }
    let axis =
        |positive, negative| (keys.pressed(positive) as i32 - keys.pressed(negative) as i32) as f32;
    s.direction = Vec3::new(
        axis(KeyCode::KeyD, KeyCode::KeyA),
        if s.noclip {
            keys.pressed(KeyCode::Space) as i32 as f32
                - (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)) as i32
                    as f32
        } else {
            0.
        },
        axis(KeyCode::KeyS, KeyCode::KeyW),
    );
    s.wants_crouch =
        !s.noclip && (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight));
    s.running =
        (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)) && !s.wants_crouch;
    s.precision_flight =
        s.noclip && (keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight));
    s.jump |= keys.just_pressed(KeyCode::Space);
}
fn sync_cursor(world: &mut World) {
    let in_ui = crate::source_frontend::active(world) || world.resource::<PlayState>().menu_open;
    for mut window in world
        .query_filtered::<&mut Window, With<PrimaryWindow>>()
        .iter_mut(world)
    {
        let captured = window.focused && !in_ui;
        let grab = if captured {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        if window.cursor_options.grab_mode != grab || window.cursor_options.visible == captured {
            window.cursor_options.grab_mode = grab;
            window.cursor_options.visible = !captured;
        }
    }
}
pub(crate) fn reset_crouch(world: &mut World) {
    let Some(s) = world.get_resource::<PlayerState>() else {
        return;
    };
    let (entity, c) = (s.entity, s.config.clone());
    world
        .entity_mut(entity)
        .insert(Collider::capsule_y(c.height * 0.5 - c.radius, c.radius));
    let mut s = world.resource_mut::<PlayerState>();
    s.crouched = false;
    s.wants_crouch = false;
    s.duck_fraction = 0.;
}
fn crouch(world: &mut World) {
    let Some(s) = world.get_resource::<PlayerState>() else {
        return;
    };
    if s.vehicle.is_some()
        || source_play::npcs::dead(world)
        || crate::source_frontend::active(world)
        || world.resource::<PlayState>().menu_open
    {
        return;
    }
    let (entity, c, was, wanted, grounded) = (
        s.entity,
        s.config.clone(),
        s.crouched,
        s.wants_crouch && !s.noclip,
        s.grounded,
    );
    if !world.query::<&Window>().iter(world).any(|w| w.focused) {
        return;
    }
    if was != wanted {
        let old_height = if was { c.crouch_height } else { c.height };
        let new_height = if wanted { c.crouch_height } else { c.height };
        // Grounded: preserve feet. Airborne: preserve top, lifting feet for jump-crouch.
        let shift = (new_height - old_height) * if grounded { 0.5 } else { -0.5 };
        let center = world.get::<Transform>(entity).unwrap().translation + Vec3::Y * shift;
        if wanted || source_play::vehicles::clear_hull(world, center) {
            world.get_mut::<Transform>(entity).unwrap().translation = center;
            world
                .entity_mut(entity)
                .insert(Collider::capsule_y(new_height * 0.5 - c.radius, c.radius));
            let mut s = world.resource_mut::<PlayerState>();
            s.crouched = wanted;
            if !grounded {
                s.duck_fraction = if wanted { 1. } else { 0. };
            }
        }
    }
    let step = world.resource::<Time<Fixed>>().delta_secs() / c.duck_seconds;
    let mut s = world.resource_mut::<PlayerState>();
    let target = if s.crouched { 1. } else { 0. };
    s.duck_fraction += (target - s.duck_fraction).clamp(-step, step);
}
pub(crate) fn walk(
    time: Res<Time<Fixed>>,
    frontend: Option<Res<crate::source_frontend::Frontend>>,
    window: Query<&Window, With<PrimaryWindow>>,
    life: Option<Res<source_play::npcs::PlayerLife>>,
    play: Res<PlayState>,
    state: Option<ResMut<PlayerState>>,
    mut q: Query<(&mut Transform, &mut KinematicCharacterController), With<PlayerBody>>,
) {
    let Some(mut s) = state else { return };
    let Ok((mut t, mut controller)) = q.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    s.movement_submitted = false;
    s.jumped = false;
    if s.vehicle.is_some() || life.is_some_and(|l| l.health <= 0.) {
        controller.translation = None;
        return;
    }
    if s.noclip
        && (play.menu_open
            || frontend.as_ref().is_some_and(|f| f.is_open())
            || !window.iter().any(|w| w.focused))
    {
        controller.translation = None;
        s.local_velocity = Vec3::ZERO;
        s.moving = false;
        s.jump = false;
        return;
    }
    let rotation = Quat::from_euler(
        EulerRot::YXZ,
        s.yaw,
        if s.noclip { s.pitch } else { 0. },
        0.,
    );
    // Forward follows pitch, but Space/Ctrl remains world vertical, as in FullNoClipMove.
    let direction = (rotation * Vec3::new(s.direction.x, 0., s.direction.z)
        + Vec3::Y * s.direction.y)
        .normalize_or_zero();
    let speed = if s.noclip {
        play.speed
            * if s.precision_flight {
                s.config.noclip_precision
            } else if s.running {
                s.config.noclip_boost
            } else {
                1.
            }
    } else if s.crouched {
        s.config.walk_speed * s.config.crouch_speed
    } else if s.running {
        s.config.run_speed
    } else {
        s.config.walk_speed
    };
    s.moving = s.direction.length_squared() > 0.01;
    if s.noclip {
        controller.translation = None;
        t.translation += direction * speed * dt;
        s.vertical = 0.;
        s.local_velocity = rotation.inverse() * direction * speed;
        s.horizontal_velocity = Vec3::ZERO;
        s.grounded = false;
    } else {
        // Source keeps velocity separately from step/snap/contact position corrections.
        if s.grounded {
            s.vertical = 0.;
        }
        let jumping = s.jump && s.grounded;
        if jumping {
            s.vertical = s.config.jump_speed;
            s.grounded = false;
        }
        s.jumped = jumping;
        let c = &s.config;
        let mut velocity = s.horizontal_velocity;
        if s.grounded {
            let current = velocity.length();
            if current > f32::EPSILON {
                // At crouch wish speed, the standing stop floor could remove
                // more velocity each tick than acceleration can restore.
                // Retain full release braking, but bound the floor while moving.
                let stop = if s.crouched && direction.length_squared() > 0. {
                    c.stop_speed.min(speed)
                } else {
                    c.stop_speed
                };
                let remaining = (current - current.max(stop) * c.ground_friction * dt).max(0.);
                velocity *= remaining / current;
            }
        }
        if direction.length_squared() > 0. {
            let cap = if s.grounded {
                speed
            } else {
                speed.min(c.air_speed_cap)
            };
            let missing = (cap - velocity.dot(direction)).max(0.);
            let acceleration = if s.grounded {
                c.ground_acceleration
            } else {
                c.air_acceleration
            };
            // Source's legacy AirAccelerate uses uncapped wish speed for acceleration.
            velocity += direction * missing.min(acceleration * speed * dt);
        }
        if s.grounded && velocity.length() < c.minimum_move_speed {
            velocity = Vec3::ZERO;
        }
        s.horizontal_velocity = velocity;
        if !s.grounded {
            s.vertical = (s.vertical + play.config.gravity * dt * 0.5).max(-50.);
        }
        controller.autostep = s.grounded.then_some(CharacterAutostep {
            max_height: CharacterLength::Absolute(s.config.step_height),
            min_width: CharacterLength::Absolute(0.1),
            include_dynamic_bodies: false,
        });
        controller.snap_to_ground = if s.grounded {
            Some(CharacterLength::Absolute(s.config.step_height))
        } else {
            None
        };
        // This downward support probe enables Rapier snapping. It is NOT falling velocity.
        let vertical_move = if s.grounded {
            -s.config.ground_probe
        } else {
            s.vertical * dt
        };
        controller.translation = Some(velocity * dt + Vec3::Y * vertical_move);
        s.movement_submitted = true;
    }
    s.jump = false;
}

fn finish_walk(
    time: Res<Time<Fixed>>,
    play: Res<PlayState>,
    state: Option<ResMut<PlayerState>>,
    q: Query<&KinematicCharacterControllerOutput, With<PlayerBody>>,
) {
    let Some(mut s) = state else { return };
    if !s.movement_submitted || s.noclip {
        return;
    }
    s.movement_submitted = false;
    let Ok(output) = q.single() else { return };
    let was_grounded = s.grounded;
    let mut velocity = s.horizontal_velocity + Vec3::Y * s.vertical;
    let mut planes = Vec::new();
    for collision in &output.collisions {
        let Some(hit) = collision.hit.details else {
            continue;
        };
        // KCC query hits expose obstacle normals in world space, as used by handle_slopes.
        let mut normal = hit.normal1.normalize_or_zero();
        if was_grounded && normal.y >= s.config.walkable_normal {
            continue;
        }
        // A stair face successfully stepped over must not cancel forward momentum.
        if output.effective_translation.dot(normal)
            <= output.desired_translation.dot(normal) + 1.0e-5
        {
            continue;
        }
        if was_grounded && normal.y >= 0. {
            normal = Vec3::new(normal.x, 0., normal.z).normalize_or_zero();
        }
        if normal != Vec3::ZERO && !planes.iter().any(|p: &Vec3| p.dot(normal) > 0.99999) {
            planes.push(normal);
        }
    }
    velocity = clip_movement_velocity(velocity, &planes);
    // Successful steps/snaps can report support without a floor collision event.
    // Keep Rapier's support result, not a heuristic over sweep events. Source's
    // hull/quadrant ground trace remains a separate parity boundary.
    s.grounded = output.grounded && s.vertical <= 0.;
    s.vertical = if s.grounded {
        0.
    } else {
        (velocity.y + play.config.gravity * time.delta_secs() * 0.5).max(-50.)
    };
    s.horizontal_velocity = Vec3::new(velocity.x, 0., velocity.z);
    if s.jumped {
        // Sandbox FinishMove observes collision-clipped velocity, not pre-move wish velocity.
        let forward = Quat::from_rotation_y(s.yaw) * Vec3::NEG_Z;
        let speed = if s.running {
            s.config.run_speed
        } else {
            s.config.walk_speed
        };
        let fraction = s.config.jump_boost;
        let mut addition = (-s.direction.z * speed * fraction).abs();
        addition -= (s.horizontal_velocity.length() + addition - speed * (1. + fraction)).max(0.);
        if s.horizontal_velocity.dot(forward) < 0. {
            addition = -addition;
        }
        s.horizontal_velocity += forward * addition;
    }
    s.jumped = false;
    // Actual displacement is useful for animation, but never becomes next-tick momentum.
    s.local_velocity =
        Quat::from_rotation_y(-s.yaw) * output.effective_translation / time.delta_secs();
}

fn clip_movement_velocity(velocity: Vec3, planes: &[Vec3]) -> Vec3 {
    if planes.is_empty() {
        return velocity;
    }
    for normal in planes {
        let candidate = velocity - *normal * velocity.dot(*normal).min(0.);
        if planes.iter().all(|p| candidate.dot(*p) >= -1.0e-5) {
            return candidate;
        }
    }
    // Two blocking planes leave a crease. More conflicting planes stop movement.
    if planes.len() == 2 {
        let crease = planes[0].cross(planes[1]).normalize_or_zero();
        return crease * velocity.dot(crease);
    }
    Vec3::ZERO
}
pub(crate) struct Actor {
    pub(crate) root: Entity,
    pub(crate) skeleton: Skeleton,
    parts: Vec<(Handle<Mesh>, Geometry)>,
    pub(crate) clips: BTreeMap<String, Clip>,
}
pub(crate) fn actor(
    world: &mut World,
    path: &str,
    layer: usize,
    parent: Option<Entity>,
) -> Result<Actor> {
    let config = &world.resource::<PlayerState>().config;
    let skin = if path == config.world_physgun {
        config.world_physgun_skin
    } else {
        0
    };
    let model = source_play::gpu_model_skin(world, path, skin)?;
    let source = world.resource::<MountedSource>();
    let skeleton = Skeleton::read(&source.mounts.read(&source.bsp, path)?)?;
    let root = world
        .spawn((
            Name::new(path.to_owned()),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    if let Some(p) = parent {
        world.entity_mut(root).insert(ChildOf(p));
    }
    let mut parts = Vec::new();
    for ((_, material), geo) in model.parts.into_iter().zip(model.geometry.iter().cloned()) {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(geo.clone().mesh());
        world.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material),
            Transform::default(),
            ChildOf(root),
            RenderLayers::layer(layer),
            NoFrustumCulling,
        ));
        parts.push((mesh, geo));
    }
    Ok(Actor {
        root,
        skeleton,
        parts,
        clips: BTreeMap::new(),
    })
}
pub(crate) fn add_clip(world: &World, actor: &mut Actor, path: &str, name: &str) -> Result<()> {
    let s = world.resource::<MountedSource>();
    let clip = source_animation::load_clip(&s.mounts, &s.bsp, path, name, &actor.skeleton)?;
    println!("PLAYER_CLIP_OK {name}: {} frames", clip.frames.len());
    actor.clips.insert(name.into(), clip);
    Ok(())
}
pub(crate) fn skin(world: &mut World, actor: &Actor, globals: &[Mat4]) {
    let matrices: Vec<_> = actor
        .skeleton
        .bones
        .iter()
        .zip(globals)
        .map(|(b, g)| source_animation::bevy_matrix(*g * b.inverse, 0.01905))
        .collect();
    let mut meshes = world.resource_mut::<Assets<Mesh>>();
    for (handle, geo) in &actor.parts {
        let mut positions = Vec::with_capacity(geo.positions.len());
        let mut normals = Vec::with_capacity(geo.positions.len());
        for ((p, n), weights) in geo.positions.iter().zip(&geo.normals).zip(&geo.weights) {
            let p = Vec3::from_array(*p);
            let n = Vec3::from_array(*n);
            let mut sp = Vec3::ZERO;
            let mut sn = Vec3::ZERO;
            let mut total = 0.;
            for &(bone, weight) in weights {
                if weight > 0. {
                    if let Some(m) = matrices.get(bone as usize) {
                        sp += m.transform_point3(p) * weight;
                        sn += m.transform_vector3(n) * weight;
                        total += weight;
                    }
                }
            }
            if total <= 0. {
                sp = p;
                sn = n;
            }
            positions.push(sp.to_array());
            normals.push(sn.normalize_or_zero().to_array());
        }
        if let Some(mesh) = meshes.get_mut(handle) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        }
    }
}
#[derive(Resource)]
struct Scene {
    body: Actor,
    view: Actor,
    hands: Actor,
    held: Actor,
    view_camera: Entity,
    physgun: bool,
    weapon_id: String,
    draw_started: f32,
    playback: crate::source_pose::PosePlayback,
    animation_states: Vec<sandbox_catalog::presentation::AnimationState>,
    layout: sandbox_catalog::presentation::LayoutConfig,
}
fn create_scene(world: &mut World) -> Result<Scene> {
    let c = world.resource::<PlayerState>().config.clone();
    let physgun = world.resource::<PlayState>().physgun;
    let weapon_id = world.resource::<PlayState>().active_weapon.clone();
    let mut body = actor(world, &c.model, 0, None)?;
    let animation_states = crate::compiled_animation_states();
    for mapping in &animation_states {
        add_clip(world, &mut body, &c.animations, &mapping.idle)?;
        add_clip(world, &mut body, &c.animations, &mapping.jump)?;
        add_clip(world, &mut body, &c.animations, &mapping.crouch_idle)?;
        for direction in crate::source_pose::DIRECTIONS {
            for template in [&mapping.walk, &mapping.run, &mapping.crouch_walk] {
                add_clip(
                    world,
                    &mut body,
                    &c.animations,
                    &template.replace("{direction}", direction),
                )?;
            }
        }
    }
    let poses: std::collections::BTreeSet<_> = world
        .resource::<PlayState>()
        .spawn_catalog
        .vehicles
        .iter()
        .map(|v| v.pose.clone())
        .collect();
    for pose in poses {
        if let Err(error) = add_clip(world, &mut body, &c.animations, &pose) {
            eprintln!("VEHICLE_POSE_MISSING {pose}: {error}");
        }
    }
    let view_camera = world
        .spawn((
            Camera3d::default(),
            PlayerViewCamera,
            Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            Projection::Perspective(PerspectiveProjection {
                fov: c.view_fov.to_radians(),
                near: 0.005,
                far: 20.,
                ..default()
            }),
            Tonemapping::None,
            Transform::default(),
            RenderLayers::layer(1),
        ))
        .id();
    let (view, hands, held) = create_weapons(world, view_camera, &weapon_id)?;
    Ok(Scene {
        body,
        view,
        hands,
        held,
        view_camera,
        physgun,
        weapon_id,
        draw_started: world.resource::<Time>().elapsed_secs(),
        playback: crate::source_pose::PosePlayback::default(),
        animation_states,
        layout: crate::compiled_layout_config(),
    })
}
fn create_weapons(
    world: &mut World,
    camera: Entity,
    weapon_id: &str,
) -> Result<(Actor, Actor, Actor)> {
    let c = world.resource::<PlayerState>().config.clone();
    let physgun = weapon_id == "weapon_weapon_physgun";
    let toolgun = weapon_id == "weapon_gmod_tool";
    let definition = world
        .resource::<PlayState>()
        .spawn_catalog
        .weapons
        .iter()
        .find(|w| w.id == weapon_id)
        .cloned()
        .ok_or("unknown weapon")?;
    let path = if !physgun && !toolgun {
        definition.view_model.clone()
    } else if physgun {
        world.resource::<PlayState>().config.physgun_model.clone()
    } else {
        world.resource::<PlayState>().config.toolgun_model.clone()
    };
    let mut created = Vec::new();
    let result = (|| -> Result<_> {
        let mut view = actor(world, &path, 1, Some(camera))?;
        created.push(view.root);
        if !physgun && !toolgun {
            for (key, candidates) in [
                ("runtime_idle", &definition.idle),
                ("runtime_fire", &definition.fire),
                ("runtime_reload", &definition.reload),
                ("runtime_draw", &definition.draw),
            ] {
                let mut loaded = false;
                for name in candidates.split('|').filter(|n| !n.is_empty()) {
                    if add_clip(world, &mut view, &path, name).is_ok() {
                        let clip = view.clips.remove(name).unwrap();
                        view.clips.insert(key.into(), clip);
                        loaded = true;
                        break;
                    }
                }
                if !loaded && !candidates.is_empty() {
                    return Err(format!(
                        "Required weapon animation missing: {weapon_id} {key} ({candidates})"
                    )
                    .into());
                }
            }
        } else {
            add_clip(
                world,
                &mut view,
                &path,
                if physgun {
                    &c.physgun_idle
                } else {
                    &c.toolgun_idle
                },
            )?;
            if !physgun {
                let fire = world
                    .resource::<PlayState>()
                    .tools
                    .catalog
                    .gun
                    .fire_clip
                    .clone();
                add_clip(world, &mut view, &path, &fire)?;
            }
        }
        let hands = actor(world, &c.hands, 1, Some(camera))?;
        created.push(hands.root);
        let held = actor(
            world,
            if !physgun && !toolgun {
                &definition.world_model
            } else if physgun {
                &c.world_physgun
            } else {
                &c.world_toolgun
            },
            0,
            None,
        )?;
        created.push(held.root);
        Ok((view, hands, held))
    })();
    if result.is_err() {
        for entity in created {
            world.despawn(entity);
        }
    }
    result
}
/// Current gameplay eye, also used before visual skinning when a shot is accepted.
pub(crate) fn aim_eye(world: &World) -> Option<Transform> {
    let s = world.get_resource::<PlayerState>()?;
    if s.vehicle.is_some() {
        return Some(s.eye);
    }
    let center = world.get::<Transform>(s.entity)?.translation;
    let c = &s.config;
    let height = if s.crouched {
        c.crouch_height
    } else {
        c.height
    };
    let eye_height = c.eye_height + (c.crouch_eye - c.eye_height) * s.duck_fraction;
    let recoil = world
        .get_resource::<source_play::weapons::WeaponState>()
        .map(|w| w.recoil)
        .unwrap_or(Vec2::ZERO);
    Some(
        Transform::from_translation(center + Vec3::Y * (eye_height - height * 0.5)).with_rotation(
            Quat::from_euler(
                EulerRot::YXZ,
                s.yaw + recoil.y,
                (s.pitch + recoil.x).clamp(-1.55, 1.55),
                0.,
            ),
        ),
    )
}
pub(crate) fn visuals(world: &mut World) {
    if !world.contains_resource::<PlayerState>() {
        return;
    }
    if !world.contains_resource::<Scene>() {
        // Fail visibly instead of retrying an expensive asset load every frame.
        if world.contains_resource::<FailedScene>() {
            return;
        }
        match create_scene(world) {
            Ok(s) => world.insert_resource(s),
            Err(e) => {
                let message = format!("Player assets failed: {e}");
                eprintln!("{message}");
                world.resource_mut::<PlayState>().status = message;
                world.insert_resource(FailedScene);
                return;
            }
        }
    }
    world.resource_scope(|world, mut scene: Mut<Scene>| {
        let physgun = world.resource::<PlayState>().physgun;
        let weapon_id = world.resource::<PlayState>().active_weapon.clone();
        if scene.weapon_id != weapon_id {
            match create_weapons(world, scene.view_camera, &weapon_id) {
                Ok((view, hands, held)) => {
                    for e in [scene.view.root, scene.hands.root, scene.held.root] {
                        world.despawn(e);
                    }
                    scene.view = view;
                    scene.hands = hands;
                    scene.held = held;
                    scene.physgun = physgun;
                    scene.weapon_id = weapon_id.clone();
                    scene.draw_started = world.resource::<Time>().elapsed_secs();
                    if let Some(clip) = scene.view.clips.get("runtime_draw") {
                        let until = scene.draw_started + clip.duration();
                        let mut state = world.resource_mut::<source_play::weapons::WeaponState>();
                        state.next_fire = state.next_fire.max(until);
                    }
                }
                Err(e) => {
                    world
                        .resource_mut::<source_play::weapons::WeaponState>()
                        .reset_handling();
                    let mut play = world.resource_mut::<PlayState>();
                    play.physgun = scene.physgun;
                    play.active_weapon = scene.weapon_id.clone();
                    play.tools.stage = None;
                    play.tools.shot = None;
                    play.status = format!("Weapon failed: {e}");
                    eprintln!("{}", play.status);
                    return;
                }
            }
        }
        let s = world.resource::<PlayerState>();
        let occupied = s.vehicle.is_some();
        let occupied_vehicle = s.vehicle;
        let c = s.config.clone();
        let center = world.get::<Transform>(s.entity).unwrap().translation;
        let hull_height = if s.crouched {
            c.crouch_height
        } else {
            c.height
        };
        let feet = center - Vec3::Y * hull_height * 0.5;
        let crouched = s.crouched;
        let tuning = world.resource::<PlayState>().spawn_catalog.runtime.clone();
        let eye_height = if occupied {
            let seat = world.resource::<source_play::vehicles::Occupancy>();
            source_play::vehicles::definition(world, &seat.definition)
                .map(|v| v.eye_height)
                .unwrap_or(tuning.seat_eye_height)
        } else {
            c.eye_height + (c.crouch_eye - c.eye_height) * s.duck_fraction
        };
        let seat_eye = Transform::from_translation(feet + Vec3::Y * eye_height)
            .with_rotation(Quat::from_euler(EulerRot::YXZ, s.yaw, s.pitch, 0.));
        let eye = if occupied {
            seat_eye
        } else {
            aim_eye(world).unwrap_or(seat_eye)
        };
        let weapon = world.resource::<source_play::weapons::WeaponState>();
        let aim_fraction = weapon.aim_fraction;
        let kick = weapon.kick;
        let third =
            (s.third_person && aim_fraction == 0.) || source_play::tools::devices::viewing(world);
        let local_velocity = s.local_velocity;
        let airborne = !s.grounded || s.vertical > 0.1;
        let noclip = s.noclip;
        let yaw = s.yaw;
        let player_entity = s.entity;
        let mut camera = eye;
        if third {
            let back = -*eye.forward();
            let mut distance = if occupied {
                tuning.vehicle_camera_distance
            } else {
                c.camera_distance
            };
            let mut query = world.query::<(
                &RapierContextSimulation,
                &RapierContextColliders,
                &RapierRigidBodySet,
                &RapierContextJoints,
                &RapierQueryPipeline,
            )>();
            if let Ok((simulation, colliders, bodies, joints, query_pipeline)) = query.single(world)
            {
                let context = bevy_rapier3d::plugin::RapierContext {
                    simulation,
                    colliders,
                    rigidbody_set: bodies,
                    joints,
                    query_pipeline,
                };
                let mut filter = QueryFilter::default().exclude_collider(player_entity);
                if let Some(vehicle) = occupied_vehicle {
                    filter = filter.exclude_rigid_body(vehicle);
                }
                if let Some((_, hit)) =
                    context.cast_ray(eye.translation, back, distance, true, filter)
                {
                    distance = (hit - 0.1).max(0.05);
                }
            }
            camera.translation += back * distance;
        }
        let shake = source_play::audio::shake(world);
        camera.rotation *= shake;
        world.resource_mut::<PlayerState>().eye = eye;
        if let Some(mut t) = world
            .query_filtered::<&mut Transform, With<SourceCamera>>()
            .iter_mut(world)
            .next()
        {
            *t = camera;
        }
        *world.get_mut::<Transform>(scene.view_camera).unwrap() = eye;
        let offset = world
            .resource::<PlayState>()
            .spawn_catalog
            .weapon_handling
            .iter()
            .find(|h| h.id == scene.weapon_id)
            .map(|h| Vec3::new(h.aim_x, h.aim_y, h.aim_z) * aim_fraction)
            .unwrap_or(Vec3::ZERO)
            + Vec3::Z * kick;
        let view_local = Transform::from_translation(offset);
        *world.get_mut::<Transform>(scene.view.root).unwrap() = view_local;
        *world.get_mut::<Transform>(scene.hands.root).unwrap() = view_local;
        let view_matrix = eye.compute_matrix() * view_local.compute_matrix();
        world
            .get_mut::<Camera>(scene.view_camera)
            .unwrap()
            .is_active = !third && !occupied && world.resource::<PlayState>().weapon_visible;
        let visible = if third {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        *world.get_mut::<Visibility>(scene.body.root).unwrap() = visible;
        *world.get_mut::<Visibility>(scene.held.root).unwrap() = if occupied {
            Visibility::Hidden
        } else {
            visible
        };
        let mut body_transform =
            Transform::from_translation(feet).with_rotation(Quat::from_rotation_y(yaw));
        if occupied {
            let seat = world.resource::<source_play::vehicles::Occupancy>();
            body_transform.rotation = body_transform
                .rotation
                .slerp(seat.body_rotation, seat.weight);
        }
        *world.get_mut::<Transform>(scene.body.root).unwrap() = body_transform;
        let time = world.resource::<Time>().elapsed_secs();
        let hold = if physgun { "physgun" } else { "pistol" };
        let dt = world.resource::<Time>().delta_secs();
        let mut pose = {
            let scene = &mut *scene;
            let mapping = scene
                .animation_states
                .iter()
                .find(|m| m.hold == hold)
                .unwrap();
            scene.playback.sample(
                &scene.body.clips,
                mapping,
                &scene.layout,
                local_velocity,
                airborne,
                noclip,
                crouched,
                c.crouch_speed,
                dt,
            )
        };
        if occupied {
            let seat = world.resource::<source_play::vehicles::Occupancy>();
            if let Some(clip) = scene.body.clips.get(&seat.pose) {
                let seated = clip.sample(time);
                for (p, target) in pose.iter_mut().zip(seated) {
                    p.translation = p.translation.lerp(target.translation, seat.weight);
                    p.rotation = p.rotation.slerp(target.rotation, seat.weight);
                }
            }
        }
        let globals = scene.body.skeleton.globals(&pose);
        if third {
            skin(world, &scene.body, &globals);
        }
        let mut held_globals = scene
            .held
            .skeleton
            .globals(&scene.held.skeleton.bind_pose());
        let mut merged = false;
        for (i, bone) in scene.held.skeleton.bones.iter().enumerate() {
            if let Some(j) = scene
                .body
                .skeleton
                .bones
                .iter()
                .position(|b| b.name == bone.name)
            {
                held_globals[i] = globals[j];
                merged = true;
            } else if bone.parent >= 0 {
                held_globals[i] = held_globals[bone.parent as usize] * bone.bind.compute_matrix();
            }
        }
        let held_matrix = if merged {
            body_transform.compute_matrix()
        } else if let Some((_, hand, local)) = scene
            .body
            .skeleton
            .attachments
            .iter()
            .find(|(name, _, _)| name == "anim_attachment_RH")
        {
            body_transform.compute_matrix()
                * source_animation::bevy_matrix(globals[*hand] * *local, 0.01905)
        } else {
            body_transform.compute_matrix()
        };
        *world.get_mut::<Transform>(scene.held.root).unwrap() = Transform::from_matrix(held_matrix);
        if third {
            skin(world, &scene.held, &held_globals);
            update_effect_attachments(world, &scene.held, &held_globals, held_matrix);
            return;
        }
        let play = world.resource::<PlayState>();
        let idle = if physgun {
            &c.physgun_idle
        } else {
            &c.toolgun_idle
        };
        let generic =
            scene.weapon_id != "weapon_weapon_physgun" && scene.weapon_id != "weapon_gmod_tool";
        let pose = if generic {
            let state = world.resource::<source_play::weapons::WeaponState>();
            let reload = state
                .reload
                .as_ref()
                .filter(|(id, _, end)| id == &scene.weapon_id && time < *end)
                .and_then(|(_, start, end)| {
                    scene.view.clips.get("runtime_reload").map(|clip| {
                        clip.sample_mode(
                            (time - *start) / (*end - *start).max(0.001) * clip.duration(),
                            false,
                        )
                    })
                });
            let fire = state.fired_at.and_then(|start| {
                scene
                    .view
                    .clips
                    .get("runtime_fire")
                    .filter(|clip| time - start < clip.duration())
                    .map(|clip| clip.sample_mode(time - start, false))
            });
            let draw = scene
                .view
                .clips
                .get("runtime_draw")
                .filter(|clip| time - scene.draw_started < clip.duration())
                .map(|clip| clip.sample_mode(time - scene.draw_started, false));
            reload.or(fire).or(draw).unwrap_or_else(|| {
                scene
                    .view
                    .clips
                    .get("runtime_idle")
                    .map(|clip| clip.sample(time))
                    .unwrap_or_else(|| scene.view.skeleton.bind_pose())
            })
        } else if !physgun {
            let fire = &scene.view.clips[&play.tools.catalog.gun.fire_clip];
            if let Some(shot) = play.tools.shot.filter(|s| time - s.time < fire.duration()) {
                fire.sample_mode(time - shot.time, false)
            } else {
                scene.view.clips[idle].sample(time)
            }
        } else {
            scene.view.clips[idle].sample(time)
        };
        let view_globals = scene.view.skeleton.globals(&pose);
        skin(world, &scene.view, &view_globals);
        update_effect_attachments(world, &scene.view, &view_globals, view_matrix);
        if let Some((_, bone, local)) = scene
            .view
            .skeleton
            .attachments
            .iter()
            .find(|(n, _, _)| n == "muzzle")
        {
            world.resource_mut::<PlayerState>().muzzle = (view_matrix
                * source_animation::bevy_matrix(view_globals[*bone] * *local, 0.01905))
            .transform_point3(Vec3::ZERO);
        }
        let mut hands_globals = scene
            .hands
            .skeleton
            .globals(&scene.hands.skeleton.bind_pose());
        for (i, b) in scene.hands.skeleton.bones.iter().enumerate() {
            if let Some(j) = scene
                .view
                .skeleton
                .bones
                .iter()
                .position(|v| v.name == b.name)
            {
                hands_globals[i] = view_globals[j];
            } else if b.parent >= 0 {
                hands_globals[i] = hands_globals[b.parent as usize] * b.bind.compute_matrix();
            }
        }
        skin(world, &scene.hands, &hands_globals);
    });
}
fn update_effect_attachments(world: &mut World, actor: &Actor, globals: &[Mat4], root: Mat4) {
    let position = |bone: usize, local: Mat4| {
        (root * source_animation::bevy_matrix(globals[bone] * local, 0.01905))
            .transform_point3(Vec3::ZERO)
    };
    let mut points = Vec::new();
    if let Some((_, bone, local)) = actor
        .skeleton
        .attachments
        .iter()
        .find(|(name, _, _)| name == "muzzle" || name == "core")
    {
        points.push((position(*bone, *local), true));
    }
    for (name, bone, local) in &actor.skeleton.attachments {
        if name.starts_with("fork") && name.ends_with('t') {
            points.push((position(*bone, *local), false));
        }
    }
    let mut player = world.resource_mut::<PlayerState>();
    player.muzzle = points
        .first()
        .map(|p| p.0)
        .unwrap_or(player.eye.translation);
    player.glow_points = points;
}
pub fn validate_assets(world: &mut World) -> Result<()> {
    let scene = create_scene(world)?;
    let (tool, hands, held) = create_weapons(world, scene.view_camera, "weapon_gmod_tool")?;
    for actor in [
        &scene.body,
        &scene.view,
        &scene.hands,
        &scene.held,
        &tool,
        &hands,
        &held,
    ] {
        let bind = actor.skeleton.globals(&actor.skeleton.bind_pose());
        let error = bind
            .iter()
            .zip(&actor.skeleton.bones)
            .map(|(g, b)| {
                let m = *g * b.inverse;
                m.to_cols_array()
                    .iter()
                    .zip(Mat4::IDENTITY.to_cols_array())
                    .map(|(a, b)| (a - b).abs())
                    .fold(0f32, f32::max)
            })
            .fold(0f32, f32::max);
        if error > 0.02 {
            return Err(format!("bind skinning error {error}").into());
        }
        for clip in actor.clips.values() {
            for frame in &clip.frames {
                let globals = actor.skeleton.globals(frame);
                if globals.iter().any(|m| !m.is_finite()) {
                    return Err("nonfinite animation pose".into());
                }
            }
            skin(world, actor, &actor.skeleton.globals(&clip.sample(0.)));
            println!("SKIN_OK {} bind_error={error}", clip.name);
        }
    }
    Ok(())
}
pub fn scene_ready(world: &World) -> bool {
    world.contains_resource::<Scene>()
}
#[derive(Resource)]
struct FailedScene;

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> App {
        let mut a = App::new();
        a.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule(),
        ))
        .insert_resource(Time::<Fixed>::from_hz(60.))
        .insert_resource(TimestepMode::Fixed {
            dt: 1. / 60.,
            substeps: 1,
        })
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1. / 60.),
        ))
        .insert_resource(PlayState::new(Vec::new(), 75., 10.))
        .add_systems(Startup, |mut commands: Commands| {
            spawn(&mut commands, Vec3::Y * 2., Vec3::NEG_Z);
            commands.spawn((
                RigidBody::Fixed,
                Collider::cuboid(20., 0.5, 20.),
                Transform::from_xyz(0., -0.5, 0.),
            ));
            commands.spawn((
                RigidBody::Fixed,
                Collider::cuboid(5., 2., 0.1),
                Transform::from_xyz(0., 2., -2.),
            ));
        })
        .add_systems(FixedUpdate, walk.before(PhysicsSet::SyncBackend))
        .add_systems(FixedUpdate, finish_walk.after(PhysicsSet::Writeback));
        a.finish();
        a.cleanup();
        for _ in 0..180 {
            a.update();
        }
        a
    }
    fn position(a: &App) -> Vec3 {
        a.world()
            .get::<Transform>(a.world().resource::<PlayerState>().entity)
            .unwrap()
            .translation
    }
    #[test]
    fn grounded_walk_stops_at_wall_independent_of_pitch() {
        let mut a = app();
        let c = a.world().resource::<PlayerState>().config.clone();
        assert!(a.world().resource::<PlayerState>().grounded);
        assert!((position(&a).y - c.height * 0.5).abs() < 0.03);
        {
            let mut s = a.world_mut().resource_mut::<PlayerState>();
            s.direction = Vec3::NEG_Z;
            s.pitch = 1.3;
        }
        for _ in 0..180 {
            a.update();
        }
        let p = position(&a);
        assert!(p.z < -1. && p.z > -1.7, "wall collision: {p:?}");
        assert!(
            (p.y - c.height * 0.5).abs() < 0.03,
            "looking up must not fly: {p:?}"
        );
    }
    #[test]
    fn gmod_ground_stop_does_not_use_sdk_friction() {
        let mut a = app();
        assert_eq!(
            a.world().resource::<PlayerState>().config.ground_friction,
            8.
        );
        {
            let mut s = a.world_mut().resource_mut::<PlayerState>();
            s.direction = Vec3::X;
            s.running = true;
        }
        for _ in 0..60 {
            a.update();
        }
        let start = position(&a);
        a.world_mut().resource_mut::<PlayerState>().direction = Vec3::ZERO;
        for _ in 0..24 {
            a.update();
        }
        let stopped = position(&a);
        assert!(
            (stopped - start).length() < 0.85,
            "excess stopping drift: {:?}",
            stopped - start
        );
        assert_eq!(
            a.world().resource::<PlayerState>().horizontal_velocity,
            Vec3::ZERO
        );
        assert_eq!(a.world().resource::<PlayerState>().vertical, 0.);
        for _ in 0..120 {
            a.update();
        }
        assert!((position(&a) - stopped).length() < 0.001, "idle creep");
    }
    #[test]
    fn contact_clipping_removes_only_inward_momentum() {
        let incoming = Vec3::new(3., 0., -4.);
        assert_eq!(
            clip_movement_velocity(incoming, &[Vec3::Z]),
            Vec3::new(3., 0., 0.)
        );
        assert_eq!(
            clip_movement_velocity(incoming, &[Vec3::NEG_X, Vec3::Z]),
            Vec3::ZERO
        );
        let away = Vec3::new(3., 0., 4.);
        assert_eq!(clip_movement_velocity(away, &[Vec3::Z]), away);
    }
    #[test]
    fn jump_lands_and_cannot_rejump_in_air() {
        let mut a = app();
        let start = position(&a).y;
        a.world_mut().resource_mut::<PlayerState>().jump = true;
        for _ in 0..12 {
            a.update();
        }
        assert!(position(&a).y > start + 0.3);
        let before = a.world().resource::<PlayerState>().vertical;
        a.world_mut().resource_mut::<PlayerState>().jump = true;
        a.update();
        assert!(a.world().resource::<PlayerState>().vertical < before);
        for _ in 0..120 {
            a.update();
        }
        assert!(a.world().resource::<PlayerState>().grounded);
        assert!((position(&a).y - start).abs() < 0.03);
    }
}
