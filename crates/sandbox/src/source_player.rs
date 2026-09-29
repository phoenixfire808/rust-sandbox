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
}
pub struct SourcePlayerPlugin;
impl Plugin for SourcePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::source_effects::SourceEffectsPlugin);
        app.add_systems(Update, input.before(source_play::update_play))
            .add_systems(FixedUpdate, walk.before(PhysicsSet::SyncBackend))
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
                offset: CharacterLength::Absolute(0.005),
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(c.step_height),
                    min_width: CharacterLength::Absolute(0.1),
                    include_dynamic_bodies: false,
                }),
                snap_to_ground: Some(CharacterLength::Absolute(c.step_height)),
                max_slope_climb_angle: 45f32.to_radians(),
                min_slope_slide_angle: 46f32.to_radians(),
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
    });
}
#[allow(clippy::too_many_arguments)]
pub fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    play: Res<PlayState>,
    state: Option<ResMut<PlayerState>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    frontend: Option<Res<crate::source_frontend::Frontend>>,
    mut exit: EventWriter<AppExit>,
) {
    let delta = motion.read().fold(Vec2::ZERO, |a, e| a + e.delta);
    let Some(mut s) = state else { return };
    let Ok(mut w) = window.single_mut() else {
        return;
    };
    if keys.just_pressed(KeyCode::F10) && frontend.is_none() {
        exit.write(AppExit::Success);
    }
    if keys.just_pressed(KeyCode::Escape) || !w.focused || play.menu_open {
        w.cursor_options.grab_mode = CursorGrabMode::None;
        w.cursor_options.visible = true;
    }
    if mouse.just_pressed(MouseButton::Left) && w.focused && !play.menu_open {
        w.cursor_options.grab_mode = CursorGrabMode::Locked;
        w.cursor_options.visible = false;
    }
    s.direction = Vec3::ZERO;
    s.running = false;
    if !w.focused || play.menu_open {
        s.jump = false;
        return;
    }
    if w.cursor_options.grab_mode != CursorGrabMode::None {
        s.yaw -= delta.x * play.sensitivity;
        s.pitch = (s.pitch - delta.y * play.sensitivity).clamp(-1.55, 1.55);
    }
    if keys.just_pressed(KeyCode::F4) {
        s.third_person = !s.third_person;
    }
    if keys.just_pressed(KeyCode::KeyV) {
        s.noclip = !s.noclip;
        s.vertical = 0.;
    }
    let axis =
        |positive, negative| (keys.pressed(positive) as i32 - keys.pressed(negative) as i32) as f32;
    s.direction = Vec3::new(
        axis(KeyCode::KeyD, KeyCode::KeyA),
        if s.noclip {
            axis(KeyCode::Space, KeyCode::ControlLeft)
        } else {
            0.
        },
        axis(KeyCode::KeyS, KeyCode::KeyW),
    )
    .normalize_or_zero();
    s.running = keys.pressed(KeyCode::ShiftLeft);
    s.jump |= keys.just_pressed(KeyCode::Space);
}
fn walk(
    time: Res<Time<Fixed>>,
    play: Res<PlayState>,
    state: Option<ResMut<PlayerState>>,
    mut q: Query<
        (
            &mut Transform,
            &mut KinematicCharacterController,
            Option<&KinematicCharacterControllerOutput>,
        ),
        With<PlayerBody>,
    >,
) {
    let Some(mut s) = state else { return };
    let Ok((mut t, mut controller, output)) = q.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    s.grounded = output.is_some_and(|o| o.grounded);
    s.local_velocity = Quat::from_rotation_y(-s.yaw)
        * output
            .map(|o| o.effective_translation / dt)
            .unwrap_or(Vec3::ZERO);
    let rotation = Quat::from_euler(
        EulerRot::YXZ,
        s.yaw,
        if s.noclip { s.pitch } else { 0. },
        0.,
    );
    let direction = rotation * s.direction;
    let speed = if s.noclip {
        play.speed
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
        s.local_velocity = s.direction * speed;
    } else {
        if s.grounded && s.vertical < 0. {
            s.vertical = 0.;
        }
        if s.jump && s.grounded {
            s.vertical = s.config.jump_speed;
        }
        s.vertical = (s.vertical + play.config.gravity * dt).max(-50.);
        controller.translation = Some((direction * speed + Vec3::Y * s.vertical) * dt);
    }
    s.jump = false;
}
struct Actor {
    root: Entity,
    skeleton: Skeleton,
    parts: Vec<(Handle<Mesh>, Geometry)>,
    clips: BTreeMap<String, Clip>,
}
fn actor(world: &mut World, path: &str, layer: usize, parent: Option<Entity>) -> Result<Actor> {
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
fn add_clip(world: &World, actor: &mut Actor, path: &str, name: &str) -> Result<()> {
    let s = world.resource::<MountedSource>();
    let clip = source_animation::load_clip(&s.mounts, &s.bsp, path, name, &actor.skeleton)?;
    println!("PLAYER_CLIP_OK {name}: {} frames", clip.frames.len());
    actor.clips.insert(name.into(), clip);
    Ok(())
}
fn skin(world: &mut World, actor: &Actor, globals: &[Mat4]) {
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
    playback: crate::source_pose::PosePlayback,
    animation_states: Vec<sandbox_catalog::presentation::AnimationState>,
    layout: sandbox_catalog::presentation::LayoutConfig,
}
fn create_scene(world: &mut World) -> Result<Scene> {
    let c = world.resource::<PlayerState>().config.clone();
    let physgun = world.resource::<PlayState>().physgun;
    let mut body = actor(world, &c.model, 0, None)?;
    let animation_states = crate::compiled_animation_states();
    for mapping in &animation_states {
        add_clip(world, &mut body, &c.animations, &mapping.idle)?;
        add_clip(world, &mut body, &c.animations, &mapping.jump)?;
        for direction in crate::source_pose::DIRECTIONS {
            for template in [&mapping.walk, &mapping.run] {
                add_clip(
                    world,
                    &mut body,
                    &c.animations,
                    &template.replace("{direction}", direction),
                )?;
            }
        }
    }
    let view_camera = world
        .spawn((
            Camera3d::default(),
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
    let (view, hands, held) = create_weapons(world, view_camera, physgun)?;
    Ok(Scene {
        body,
        view,
        hands,
        held,
        view_camera,
        physgun,
        playback: crate::source_pose::PosePlayback::default(),
        animation_states,
        layout: crate::compiled_layout_config(),
    })
}
fn create_weapons(
    world: &mut World,
    camera: Entity,
    physgun: bool,
) -> Result<(Actor, Actor, Actor)> {
    let c = world.resource::<PlayerState>().config.clone();
    let path = if physgun {
        world.resource::<PlayState>().config.physgun_model.clone()
    } else {
        world.resource::<PlayState>().config.toolgun_model.clone()
    };
    let mut created = Vec::new();
    let result = (|| -> Result<_> {
        let mut view = actor(world, &path, 1, Some(camera))?;
        created.push(view.root);
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
        let hands = actor(world, &c.hands, 1, Some(camera))?;
        created.push(hands.root);
        let held = actor(
            world,
            if physgun {
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
        if scene.physgun != physgun {
            match create_weapons(world, scene.view_camera, physgun) {
                Ok((view, hands, held)) => {
                    for e in [scene.view.root, scene.hands.root, scene.held.root] {
                        world.despawn(e);
                    }
                    scene.view = view;
                    scene.hands = hands;
                    scene.held = held;
                    scene.physgun = physgun;
                }
                Err(e) => {
                    let mut play = world.resource_mut::<PlayState>();
                    play.physgun = scene.physgun;
                    play.tools.stage = None;
                    play.tools.shot = None;
                    play.status = format!("Weapon failed: {e}");
                    eprintln!("{}", play.status);
                    return;
                }
            }
        }
        let s = world.resource::<PlayerState>();
        let c = s.config.clone();
        let center = world.get::<Transform>(s.entity).unwrap().translation;
        let feet = center - Vec3::Y * c.height * 0.5;
        let eye = Transform::from_translation(feet + Vec3::Y * c.eye_height)
            .with_rotation(Quat::from_euler(EulerRot::YXZ, s.yaw, s.pitch, 0.));
        let third = s.third_person;
        let local_velocity = s.local_velocity;
        let airborne = !s.grounded || s.vertical > 0.1;
        let noclip = s.noclip;
        let yaw = s.yaw;
        let player_entity = s.entity;
        let mut camera = eye;
        if third {
            let back = -*eye.forward();
            let mut distance = c.camera_distance;
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
                if let Some((_, hit)) = context.cast_ray(
                    eye.translation,
                    back,
                    distance,
                    true,
                    QueryFilter::default().exclude_collider(player_entity),
                ) {
                    distance = (hit - 0.1).max(0.05);
                }
            }
            camera.translation += back * distance;
        }
        world.resource_mut::<PlayerState>().eye = eye;
        if let Some(mut t) = world
            .query_filtered::<&mut Transform, With<SourceCamera>>()
            .iter_mut(world)
            .next()
        {
            *t = camera;
        }
        *world.get_mut::<Transform>(scene.view_camera).unwrap() = eye;
        world
            .get_mut::<Camera>(scene.view_camera)
            .unwrap()
            .is_active = !third && world.resource::<PlayState>().weapon_visible;
        let visible = if third {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        *world.get_mut::<Visibility>(scene.body.root).unwrap() = visible;
        *world.get_mut::<Visibility>(scene.held.root).unwrap() = visible;
        let body_transform =
            Transform::from_translation(feet).with_rotation(Quat::from_rotation_y(yaw));
        *world.get_mut::<Transform>(scene.body.root).unwrap() = body_transform;
        let time = world.resource::<Time>().elapsed_secs();
        let hold = if physgun { "physgun" } else { "pistol" };
        let dt = world.resource::<Time>().delta_secs();
        let pose = {
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
                dt,
            )
        };
        let globals = scene.body.skeleton.globals(&pose);
        if third {
            skin(world, &scene.body, &globals);
        }
        if let Some((_, hand, local)) = scene
            .body
            .skeleton
            .attachments
            .iter()
            .find(|(name, _, _)| name == "anim_attachment_RH")
        {
            // Attach to the authored weapon attachment, including its local orientation.
            let matrix = body_transform.compute_matrix()
                * source_animation::bevy_matrix(globals[*hand] * *local, 0.01905);
            *world.get_mut::<Transform>(scene.held.root).unwrap() = Transform::from_matrix(matrix);
            if let Some((_, bone, local)) = scene
                .held
                .skeleton
                .attachments
                .iter()
                .find(|(n, _, _)| n == "muzzle")
            {
                let held_globals = scene
                    .held
                    .skeleton
                    .globals(&scene.held.skeleton.bind_pose());
                world.resource_mut::<PlayerState>().muzzle = (matrix
                    * source_animation::bevy_matrix(held_globals[*bone] * *local, 0.01905))
                .transform_point3(Vec3::ZERO);
            }
        }
        if third {
            let held_globals = scene
                .held
                .skeleton
                .globals(&scene.held.skeleton.bind_pose());
            let transform = world
                .get::<Transform>(scene.held.root)
                .unwrap()
                .compute_matrix();
            update_effect_attachments(world, &scene.held, &held_globals, transform);
            return;
        }
        let play = world.resource::<PlayState>();
        let idle = if physgun {
            &c.physgun_idle
        } else {
            &c.toolgun_idle
        };
        let pose = if !physgun {
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
        update_effect_attachments(world, &scene.view, &view_globals, eye.compute_matrix());
        if let Some((_, bone, local)) = scene
            .view
            .skeleton
            .attachments
            .iter()
            .find(|(n, _, _)| n == "muzzle")
        {
            world.resource_mut::<PlayerState>().muzzle = (eye.compute_matrix()
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
    let (tool, hands, held) = create_weapons(world, scene.view_camera, false)?;
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
        .add_systems(FixedUpdate, walk.before(PhysicsSet::SyncBackend));
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
