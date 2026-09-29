//! Playable original-model sandbox. Source physics, shaders and Lua are not emulated.
use crate::{
    compiled_play_config, project_root, source_assets::MountedSource, source_catalog::ModelEntry,
    source_models,
};
use bevy::{
    asset::RenderAssetUsages,
    input::{
        keyboard::{Key, KeyboardInput},
        mouse::MouseWheel,
        ButtonState,
    },
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;
use sandbox_catalog::{play::PlayConfig, Result};
use std::collections::BTreeMap;
#[path = "source_menu.rs"]
mod menu;
#[path = "source_scene.rs"]
mod scene;
#[path = "source_spawn.rs"]
mod spawn;
#[path = "source_tool_screen.rs"]
mod tool_screen;
#[path = "source_tools.rs"]
pub(crate) mod tools;
use scene::{
    remember, restore, snapshot, snapshot_selection, spawn_scene, validate_scene, SavedScene,
};

#[derive(Component)]
pub struct SourceCamera;
#[derive(Component)]
pub struct SourceHud;
#[derive(Component)]
pub struct SpawnedProp {
    pub model: String,
    pub center: Vec3,
    pub half: Vec3,
}
#[derive(Component, Clone)]
enum UiAction {
    Tab(u8),
    Search,
    Unavailable(String),
    Category(String),
    Page(i32),
    Spawn(String),
    Tool(String),
    ToolSetting(String, i32),
    ToolDefaults,
    CleanupConstraints,
    Setting(&'static str, f32),
    Save,
    Load,
    Close,
}
#[derive(Component)]
struct MenuRoot;
#[derive(Resource)]
pub struct PlayState {
    pub tools: tools::ToolState,
    pub config: PlayConfig,
    pub layout: sandbox_catalog::presentation::LayoutConfig,
    pub search_focus: bool,
    pub menu_size: Vec2,
    pub menu_scroll: [f32; 4],
    pub held_anchor: Vec3,
    pub beam_active: bool,
    pub models: Vec<ModelEntry>,
    pub menu_open: bool,
    pub tab: u8,
    pub category: String,
    pub search: String,
    pub page: usize,
    pub selected: String,
    pub tool: String,
    pub physgun: bool,
    pub held: Option<Entity>,
    pub distance: f32,
    pub status: String,
    pub dirty: bool,
    pub fov: f32,
    pub speed: f32,
    pub sensitivity: f32,
    pub weapon_visible: bool,
    pub storage_root: std::path::PathBuf,
    weapon_attempted: bool,
    undo: Vec<SavedScene>,
    cache: BTreeMap<String, GpuModel>,
    icons: BTreeMap<String, Option<Handle<Image>>>,
    last_save: Option<std::path::PathBuf>,
}
#[derive(Clone)]
pub(crate) struct GpuModel {
    pub(crate) parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>)>,
    pub(crate) geometry: std::sync::Arc<Vec<crate::source_assets::Geometry>>,
    collider: Collider,
    center: Vec3,
    half: Vec3,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct SavedModel {
    model: String,
    position: [f32; 3],
    rotation: [f32; 4],
    frozen: bool,
    #[serde(default)]
    properties: tools::Properties,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Preferences {
    fov: f32,
    speed: f32,
    sensitivity: f32,
    weapon_visible: bool,
}
impl PlayState {
    pub fn new(models: Vec<ModelEntry>, fov: f32, speed: f32) -> Self {
        let config = compiled_play_config();
        let selected = config.default_prop.clone();
        let sensitivity = config.sensitivity;
        let distance = config.spawn_distance;
        let mut out = Self {
            tools: tools::ToolState::default(),
            config,
            layout: crate::compiled_layout_config(),
            search_focus: false,
            menu_size: Vec2::ZERO,
            menu_scroll: [0.; 4],
            held_anchor: Vec3::ZERO,
            beam_active: false,
            models,
            menu_open: false,
            tab: 0,
            category: "props_c17".into(),
            search: String::new(),
            page: 0,
            selected,
            tool: "remover".into(),
            physgun: true,
            held: None,
            distance,
            status: "WASD: walk | Space: jump | Shift: run | F4: third person | V: noclip".into(),
            dirty: true,
            fov,
            speed,
            sensitivity,
            weapon_visible: true,
            storage_root: project_root().join("local"),
            weapon_attempted: false,
            undo: Vec::new(),
            cache: BTreeMap::new(),
            icons: BTreeMap::new(),
            last_save: None,
        };
        if let Ok(bytes) = std::fs::read(project_root().join("local/player-settings.json")) {
            if let Ok(p) = serde_json::from_slice::<Preferences>(&bytes) {
                if (40.0..=120.).contains(&p.fov)
                    && (0.1..=100.).contains(&p.speed)
                    && (0.0001..=0.05).contains(&p.sensitivity)
                {
                    out.fov = p.fov;
                    out.speed = p.speed;
                    out.sensitivity = p.sensitivity;
                    out.weapon_visible = p.weapon_visible;
                }
            }
        }
        out
    }
}
pub struct SourcePlayPlugin;
impl Plugin for SourcePlayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimestepMode::Fixed {
            dt: 1. / 60.,
            substeps: 1,
        })
        .insert_resource(Time::<Fixed>::from_hz(60.))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule())
        .init_resource::<spawn::PendingSpawns>()
        .add_systems(
            Update,
            (
                search_input,
                update_play,
                tools::input,
                tool_screen::update,
                tools::constraints::update,
                menu::tool_sliders,
                spawn::complete,
                spawn::report,
                adjust_hold_distance,
                menu::scroll,
                menu::hover,
            )
                .chain(),
        );
    }
}
fn rgba_image(decoded: image::RgbaImage) -> Image {
    let (w, h) = decoded.dimensions();
    Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        decoded.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

pub(crate) fn gpu_model(world: &mut World, path: &str) -> Result<GpuModel> {
    gpu_model_skin(world, path, 0)
}
pub(crate) fn gpu_model_skin(world: &mut World, path: &str, skin: i32) -> Result<GpuModel> {
    let cache_key = format!("{path}#{skin}");
    if let Some(model) = world.resource::<PlayState>().cache.get(&cache_key) {
        return Ok(model.clone());
    }
    let prepared = prepare_model(world.resource::<MountedSource>(), path, skin)?;
    Ok(install_model(world, cache_key, prepared))
}
struct PreparedModel {
    parts: Vec<(
        String,
        crate::source_assets::Geometry,
        Option<Image>,
        AlphaMode,
        bool,
    )>,
    collider: Collider,
    center: Vec3,
    half: Vec3,
}
fn prepare_model(source: &MountedSource, path: &str, skin: i32) -> Result<PreparedModel> {
    let scale = 0.01905;
    let geometry = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        source_models::model_parts(&source.bsp, &source.mounts, path, skin)
    }))
    .map_err(|_| "unsupported model format")??;
    let mut parts = Vec::new();
    let mut points = Vec::new();
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for (name, mut geo) in geometry {
        for p in &mut geo.positions {
            for axis in p.iter_mut() {
                *axis *= scale;
            }
            let v = Vec3::from_array(*p);
            if !v.is_finite() {
                return Err("model contains nonfinite geometry".into());
            }
            min = min.min(v);
            max = max.max(v);
            points.push(v);
        }
        let mat = source.mounts.material(&source.bsp, &name)?;
        let texture = mat
            .base_texture()
            .map(|base| source.mounts.texture(&source.bsp, base, true))
            .transpose()?;
        let alpha = mat
            .alpha_test()
            .map(AlphaMode::Mask)
            .unwrap_or(if mat.translucent() {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            });
        parts.push((name, geo, texture, alpha, mat.no_cull()));
    }
    if points.is_empty() {
        return Err("model contains no render geometry".into());
    }
    // Render triangles repeat vertices. Exact deduplication preserves the hull.
    points.sort_unstable_by(|a, b| {
        a.x.total_cmp(&b.x)
            .then(a.y.total_cmp(&b.y))
            .then(a.z.total_cmp(&b.z))
    });
    points.dedup();
    let collider =
        Collider::convex_hull(&points).ok_or("cannot build model convex collision hull")?;
    Ok(PreparedModel {
        parts,
        collider,
        center: (min + max) * 0.5,
        half: ((max - min) * 0.5).max(Vec3::splat(0.02)),
    })
}
fn install_model(world: &mut World, cache_key: String, prepared: PreparedModel) -> GpuModel {
    let mut parts = Vec::new();
    let mut geometry = Vec::new();
    for (name, geo, texture, alpha, nocull) in prepared.parts {
        let mut tex = texture.map(|image| world.resource_mut::<Assets<Image>>().add(image));
        if name.eq_ignore_ascii_case(
            &world
                .resource::<PlayState>()
                .tools
                .catalog
                .gun
                .screen_material,
        ) {
            match tool_screen::ensure(world) {
                Ok(image) => tex = Some(image),
                Err(e) => eprintln!("TOOL_SCREEN_ERROR {e}"),
            }
        }
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color_texture: tex,
                alpha_mode: alpha,
                cull_mode: if nocull {
                    None
                } else {
                    Some(bevy::render::render_resource::Face::Back)
                },
                unlit: true,
                ..default()
            });
        geometry.push(geo.clone());
        let mesh = world.resource_mut::<Assets<Mesh>>().add(geo.mesh());
        parts.push((mesh, material));
    }
    let model = GpuModel {
        parts,
        geometry: geometry.into(),
        collider: prepared.collider,
        center: prepared.center,
        half: prepared.half,
    };
    world
        .resource_mut::<PlayState>()
        .cache
        .insert(cache_key, model.clone());
    model
}
pub fn spawn_model(
    world: &mut World,
    path: &str,
    position: Vec3,
    rotation: Quat,
    frozen: bool,
) -> Result<Entity> {
    let count = world.query::<&SpawnedProp>().iter(world).count();
    if count >= world.resource::<PlayState>().config.max_props {
        return Err("prop limit reached".into());
    }
    let model = gpu_model(world, path)?;
    let mass = world.resource::<PlayState>().config.prop_mass;
    let id = world
        .spawn((
            Name::new(path.to_string()),
            SpawnedProp {
                model: path.into(),
                center: model.center,
                half: model.half,
            },
            Transform::from_translation(position).with_rotation(rotation),
            Visibility::default(),
            if frozen {
                RigidBody::Fixed
            } else {
                RigidBody::Dynamic
            },
            model.collider,
            ColliderMassProperties::Mass(mass),
            Velocity::zero(),
            Damping {
                linear_damping: 0.05,
                angular_damping: 0.1,
            },
            Ccd::enabled(),
            tools::Properties::default(),
            CollisionGroups::new(Group::GROUP_2, Group::ALL),
        ))
        .id();
    for (mesh, material) in model.parts {
        world.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::default(),
            ChildOf(id),
        ));
    }
    Ok(id)
}
fn camera(world: &mut World) -> Option<Transform> {
    if let Some(player) = world.get_resource::<crate::source_player::PlayerState>() {
        return Some(player.eye);
    }
    world
        .query_filtered::<&Transform, With<SourceCamera>>()
        .iter(world)
        .next()
        .copied()
}
fn ray_box(origin: Vec3, direction: Vec3, center: Vec3, half: Vec3) -> Option<f32> {
    let mut near = 0f32;
    let mut far = 200f32;
    for i in 0..3 {
        if direction[i].abs() < 1e-6 {
            if origin[i] < center[i] - half[i] || origin[i] > center[i] + half[i] {
                return None;
            }
        } else {
            let a = (center[i] - half[i] - origin[i]) / direction[i];
            let b = (center[i] + half[i] - origin[i]) / direction[i];
            near = near.max(a.min(b));
            far = far.min(a.max(b));
        }
    }
    (far >= near).then_some(near)
}
pub(crate) fn world_ray(
    world: &mut World,
    eye: Transform,
    max_distance: f32,
) -> Option<(Entity, f32)> {
    let player = world
        .get_resource::<crate::source_player::PlayerState>()
        .map(|p| p.entity);
    let mut query = world.query::<(
        &RapierContextSimulation,
        &RapierContextColliders,
        &RapierRigidBodySet,
        &RapierContextJoints,
        &RapierQueryPipeline,
    )>();
    let (simulation, colliders, bodies, joints, pipeline) = query.single(world).ok()?;
    let context = bevy_rapier3d::plugin::RapierContext {
        simulation,
        colliders,
        rigidbody_set: bodies,
        joints,
        query_pipeline: pipeline,
    };
    let mut filter = QueryFilter::default();
    if let Some(entity) = player {
        filter = filter.exclude_collider(entity);
    }
    context.cast_ray(eye.translation, *eye.forward(), max_distance, true, filter)
}
pub(crate) fn beam_target(world: &mut World) -> Option<Vec3> {
    let state = world.resource::<PlayState>();
    if !state.physgun {
        return state
            .tools
            .shot
            .filter(|s| {
                world.resource::<Time>().elapsed_secs() - s.time
                    < state.tools.catalog.gun.tracer_seconds
            })
            .map(|s| s.point);
    }
    if let Some(t) = state.held.and_then(|e| world.get::<Transform>(e)) {
        return Some(t.translation + t.rotation * state.held_anchor);
    }
    if !state.beam_active {
        return None;
    }
    let range = state.layout.beam_range;
    let eye = camera(world)?;
    let distance = world_ray(world, eye, range)
        .map(|(_, d)| d)
        .unwrap_or(range);
    Some(eye.translation + *eye.forward() * distance)
}
pub fn aimed_prop(world: &mut World) -> Option<(Entity, f32)> {
    let camera = camera(world)?;
    let range = world.resource::<PlayState>().layout.beam_range;
    let hit = world_ray(world, camera, range);
    if let Some((entity, distance)) = hit {
        if world.get::<SpawnedProp>(entity).is_some() {
            return Some((entity, distance));
        }
    }
    // Newly spawned bodies may not have reached Rapier's next fixed sync yet.
    // Keep the bounds fallback, but never select through a nearer world surface.
    let limit = hit.map(|(_, d)| d).unwrap_or(range);
    world
        .query::<(Entity, &SpawnedProp, &Transform)>()
        .iter(world)
        .filter_map(|(e, p, t)| {
            let inv = t.rotation.inverse();
            ray_box(
                inv * (camera.translation - t.translation),
                inv * *camera.forward(),
                p.center,
                p.half,
            )
            .filter(|d| *d <= limit)
            .map(|d| (e, d))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
}
fn set_weapon(world: &mut World) {
    // The player renderer owns posed view/world weapons and switches them atomically.
    world.resource_mut::<PlayState>().weapon_attempted = true;
}
fn save_preferences(world: &World) -> Result<()> {
    let s = world.resource::<PlayState>();
    let p = Preferences {
        fov: s.fov,
        speed: s.speed,
        sensitivity: s.sensitivity,
        weapon_visible: s.weapon_visible,
    };
    let target = s.storage_root.join("player-settings.json");
    std::fs::create_dir_all(&s.storage_root)?;
    std::fs::write(target, serde_json::to_vec_pretty(&p)?)?;
    Ok(())
}

fn search_input(mut events: EventReader<KeyboardInput>, mut state: ResMut<PlayState>) {
    if !state.menu_open || !state.search_focus {
        events.clear();
        return;
    }
    for e in events.read() {
        if e.state != ButtonState::Pressed {
            continue;
        }
        match &e.logical_key {
            Key::Character(c) => {
                if state.search.len() + c.len() <= 256 {
                    state.search.push_str(c);
                    state.page = 0;
                    state.dirty = true;
                }
            }
            Key::Enter => {
                state.search_focus = false;
                state.dirty = true;
            }
            Key::Backspace => {
                state.search.pop();
                state.page = 0;
                state.dirty = true;
            }
            _ => {}
        }
    }
}

pub fn update_play(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>().clone();
    let mouse = world.resource::<ButtonInput<MouseButton>>().clone();
    if !world.resource::<PlayState>().weapon_attempted
        && world.resource::<PlayState>().weapon_visible
    {
        set_weapon(world);
    }
    let mut actions: Vec<_> = world
        .query_filtered::<(&Interaction, &UiAction), Changed<Interaction>>()
        .iter(world)
        .filter(|(i, _)| mouse.just_pressed(MouseButton::Left) && **i == Interaction::Pressed)
        .map(|(_, a)| a.clone())
        .collect();
    let (focused, size) = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .iter(world)
        .next()
        .map(|w| (w.focused, Vec2::new(w.width(), w.height())))
        .unwrap_or((true, Vec2::new(1280., 800.)));
    {
        let mut s = world.resource_mut::<PlayState>();
        s.tools.input_blocked = s.menu_open || !focused;
        if s.menu_size != size {
            s.menu_size = size;
            s.dirty = true;
        }
        if !focused {
            s.held = None;
            s.beam_active = false;
            s.tools.stage = None;
            s.tools.shot = None;
        }
    }
    if keys.just_pressed(KeyCode::KeyQ) && !world.resource::<PlayState>().search_focus {
        let mut s = world.resource_mut::<PlayState>();
        s.menu_open = s.layout.hold_q || !s.menu_open;
        s.dirty = true;
        s.held = None;
    }
    if keys.just_released(KeyCode::KeyQ) {
        let mut s = world.resource_mut::<PlayState>();
        if s.layout.hold_q && !s.search_focus {
            s.menu_open = false;
            s.dirty = true;
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        let mut s = world.resource_mut::<PlayState>();
        s.menu_open = !s.menu_open;
        s.search_focus = false;
        s.tab = 1;
        s.dirty = true;
        s.held = None;
    }
    for (key, physgun) in [(KeyCode::Digit1, true), (KeyCode::Digit2, false)] {
        if keys.just_pressed(key) && !world.resource::<PlayState>().menu_open {
            world.resource_mut::<PlayState>().physgun = physgun;
            world.resource_mut::<PlayState>().tools.stage = None;
            world.resource_mut::<PlayState>().tools.shot = None;
            world.resource_mut::<PlayState>().held = None;
            set_weapon(world);
        }
    }
    if world.resource::<PlayState>().menu_open || !focused {
        for mut w in world
            .query_filtered::<&mut Window, With<PrimaryWindow>>()
            .iter_mut(world)
        {
            w.cursor_options.grab_mode = CursorGrabMode::None;
            w.cursor_options.visible = true;
        }
    } else {
        if keys.just_pressed(KeyCode::Enter) {
            actions.push(UiAction::Spawn(
                world.resource::<PlayState>().selected.clone(),
            ));
        }
        if keys.just_pressed(KeyCode::KeyZ) {
            let cancelled = spawn::cancel_latest(world);
            let save = if cancelled {
                None
            } else {
                world.resource_mut::<PlayState>().undo.pop()
            };
            if let Some(save) = save {
                if let Err(e) = restore(world, save.clone()) {
                    world.resource_mut::<PlayState>().undo.push(save);
                    world.resource_mut::<PlayState>().status = e.to_string();
                }
            }
        }
        if keys.just_pressed(KeyCode::F5) {
            actions.push(UiAction::Save);
        }
        if keys.just_pressed(KeyCode::F6) {
            actions.push(UiAction::Load);
        }
        let needs_target = world.resource::<PlayState>().physgun
            && (mouse.just_pressed(MouseButton::Left)
                || mouse.just_pressed(MouseButton::Right)
                || keys.just_pressed(KeyCode::KeyR));
        let focus = world.resource::<PlayState>().held.or_else(|| {
            needs_target
                .then(|| aimed_prop(world))
                .flatten()
                .map(|p| p.0)
        });
        if world.resource::<PlayState>().physgun && mouse.just_pressed(MouseButton::Left) {
            if let Some(e) = focus {
                remember(world);
                let distance = aimed_prop(world).map(|p| p.1).unwrap_or(5.).clamp(0.1, 50.);
                let eye = camera(world).unwrap();
                let hit = eye.translation + *eye.forward() * distance;
                let transform = *world.get::<Transform>(e).unwrap();
                let mut s = world.resource_mut::<PlayState>();
                s.held_anchor = transform.rotation.inverse() * (hit - transform.translation);
                s.held = Some(e);
                s.distance = distance;
                world.entity_mut(e).insert(RigidBody::Dynamic);
            }
        }
        if mouse.just_released(MouseButton::Left) {
            world.resource_mut::<PlayState>().held = None;
        }
        if world.resource::<PlayState>().physgun && mouse.just_pressed(MouseButton::Right) {
            if let Some(e) = focus {
                world
                    .entity_mut(e)
                    .insert((RigidBody::Fixed, Velocity::zero()));
                world.resource_mut::<PlayState>().held = None;
            }
        }
        if world.resource::<PlayState>().physgun && keys.just_pressed(KeyCode::KeyR) {
            if let Some(e) = focus {
                remember(world);
                world.entity_mut(e).insert(RigidBody::Dynamic);
            }
        }
        if let Some(e) = world.resource::<PlayState>().held {
            if let Some(cam) = camera(world) {
                let (distance, gain, speed) = {
                    let s = world.resource::<PlayState>();
                    (s.distance, s.config.hold_gain, s.config.hold_max_speed)
                };
                let target = cam.translation + *cam.forward() * distance;
                let anchor = world.resource::<PlayState>().held_anchor;
                let pos = world
                    .get::<Transform>(e)
                    .map(|t| t.translation + t.rotation * anchor);
                if let Some(pos) = pos {
                    if let Some(mut v) = world.get_mut::<Velocity>(e) {
                        v.linvel = ((target - pos) * gain).clamp_length_max(speed);
                        v.angvel = Vec3::ZERO;
                    }
                    if keys.pressed(KeyCode::KeyE) {
                        let dt = world.resource::<Time>().delta_secs();
                        world.get_mut::<Transform>(e).unwrap().rotate_y(dt);
                    }
                } else {
                    world.resource_mut::<PlayState>().held = None;
                }
            }
        }
    }
    {
        let mut s = world.resource_mut::<PlayState>();
        s.beam_active = focused
            && !s.menu_open
            && s.physgun
            && mouse.pressed(MouseButton::Left)
            && !mouse.pressed(MouseButton::Right);
        if !focused || s.menu_open || !s.physgun || !mouse.pressed(MouseButton::Left) {
            s.held = None;
        }
    }
    for action in actions {
        perform(world, action);
    }
    let (fov, gravity) = {
        let s = world.resource::<PlayState>();
        (s.fov, s.config.gravity)
    };
    for mut p in world
        .query_filtered::<&mut Projection, With<SourceCamera>>()
        .iter_mut(world)
    {
        if matches!(&*p, Projection::Perspective(v) if v.fov != fov.to_radians()) {
            if let Projection::Perspective(p) = p.as_mut() {
                p.fov = fov.to_radians();
            }
        }
    }
    for mut c in world.query::<&mut RapierConfiguration>().iter_mut(world) {
        if c.gravity != Vec3::Y * gravity {
            c.gravity = Vec3::Y * gravity;
        }
    }
    let count = world.query::<&SpawnedProp>().iter(world).count();
    let text = {
        let s = world.resource::<PlayState>();
        format!("{} | {} props | Q: build menu | 1: Physgun  2: Toolgun | Z: undo  F5/F6: save/load\n{}",if s.physgun{"PHYSICS GUN"}else{&s.tool},count,s.status)
    };
    for mut hud in world
        .query_filtered::<&mut Text, With<SourceHud>>()
        .iter_mut(world)
    {
        if hud.0 != text {
            hud.0 = text.clone();
        }
    }
    if world.resource::<PlayState>().dirty {
        rebuild_menu(world);
    }
}
fn perform(world: &mut World, action: UiAction) {
    if !matches!(action, UiAction::Search) {
        world.resource_mut::<PlayState>().search_focus = false;
    }
    match action {
        UiAction::Search => {
            let mut s = world.resource_mut::<PlayState>();
            s.search_focus = true;
            s.dirty = true;
        }
        UiAction::Unavailable(label) => {
            world.resource_mut::<PlayState>().status = format!("{label}: not implemented yet");
        }
        UiAction::Tab(tab) => {
            let mut s = world.resource_mut::<PlayState>();
            s.tab = tab;
            s.dirty = true;
        }
        UiAction::Category(c) => {
            let mut s = world.resource_mut::<PlayState>();
            s.category = c;
            s.page = 0;
            s.dirty = true;
        }
        UiAction::Page(delta) => {
            let mut s = world.resource_mut::<PlayState>();
            s.page = s.page.saturating_add_signed(delta as isize);
            s.dirty = true;
        }
        UiAction::Close => {
            let mut s = world.resource_mut::<PlayState>();
            s.menu_open = false;
            s.search_focus = false;
            s.dirty = true;
        }
        UiAction::Spawn(path) => {
            if let Some(cam) = camera(world) {
                let d = world.resource::<PlayState>().config.spawn_distance;
                spawn::enqueue(world, path, cam.translation + *cam.forward() * d);
            }
        }
        UiAction::Tool(tool) => {
            let mut s = world.resource_mut::<PlayState>();
            s.tools.stage = None;
            s.tools.shot = None;
            s.tool = tool;
            s.physgun = false;
            s.status = "Tool selected. Q shows left / right / reload controls and settings.".into();
            s.dirty = true;
            set_weapon(world);
        }
        UiAction::ToolSetting(key, direction) => tools::setting(world, key, direction),
        UiAction::ToolDefaults => tools::reset_settings(world),
        UiAction::CleanupConstraints => {
            remember(world);
            tools::constraints::clear(world);
        }
        UiAction::Setting(key, value) => {
            let mut s = world.resource_mut::<PlayState>();
            match key {
                "fov" => s.fov = (s.fov + value).clamp(40., 120.),
                "speed" => s.speed = (s.speed + value).clamp(1., 40.),
                "sensitivity" => s.sensitivity = (s.sensitivity + value).clamp(0.0001, 0.02),
                "weapon" => s.weapon_visible = !s.weapon_visible,
                _ => {}
            }
            s.dirty = true;
            let _ = s;
            if let Err(e) = save_preferences(world) {
                world.resource_mut::<PlayState>().status = e.to_string();
            }
            if key == "weapon" {
                set_weapon(world);
            }
        }
        UiAction::Save => {
            let save = snapshot(world);
            let dir = world
                .resource::<PlayState>()
                .storage_root
                .join("source-saves");
            let result = (|| -> Result<_> {
                std::fs::create_dir_all(&dir)?;
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_nanos();
                let path = dir.join(format!("scene-{stamp}.json"));
                std::fs::write(&path, serde_json::to_vec_pretty(&save)?)?;
                Ok(path)
            })();
            match result {
                Ok(path) => {
                    let mut s = world.resource_mut::<PlayState>();
                    s.status = format!("Saved {}", path.display());
                    s.last_save = Some(path);
                }
                Err(e) => world.resource_mut::<PlayState>().status = e.to_string(),
            }
        }
        UiAction::Load => {
            let path = world.resource::<PlayState>().last_save.clone().or_else(|| {
                std::fs::read_dir(
                    world
                        .resource::<PlayState>()
                        .storage_root
                        .join("source-saves"),
                )
                .ok()?
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "json"))
                .max()
            });
            let result = (|| -> Result<()> {
                let path = path.ok_or("No saved Source scene")?;
                let save = scene::read(&std::fs::read(path)?)?;
                validate_scene(world, &save)?;
                remember(world);
                restore(world, save)
            })();
            world.resource_mut::<PlayState>().status = match result {
                Ok(()) => "Loaded saved model scene".into(),
                Err(e) => e.to_string(),
            };
        }
    }
}
fn adjust_hold_distance(mut state: ResMut<PlayState>, mut wheel: EventReader<MouseWheel>) {
    for e in wheel.read() {
        if state.held.is_some() && !state.menu_open {
            state.distance = (state.distance + e.y * 0.5).clamp(1., 50.);
        }
    }
}

fn text(world: &mut World, parent: Entity, label: impl Into<String>, size: f32) {
    world.spawn((
        Text::new(label),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(Color::srgb(0.12, 0.15, 0.2)),
        ChildOf(parent),
    ));
}
fn button(world: &mut World, parent: Entity, label: impl Into<String>, action: UiAction) -> Entity {
    let id = world
        .spawn((
            Button,
            action,
            Node {
                padding: UiRect::all(Val::Px(7.)),
                margin: UiRect::all(Val::Px(2.)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.77, 0.81, 0.85)),
            ChildOf(parent),
        ))
        .id();
    let size = world.resource::<PlayState>().layout.font_size;
    text(world, id, label, size);
    id
}
fn container(world: &mut World, parent: Entity, node: Node) -> Entity {
    world.spawn((node, ChildOf(parent))).id()
}
fn thumbnail(world: &mut World, path: &str) -> Option<Handle<Image>> {
    if let Some(icon) = world.resource::<PlayState>().icons.get(path) {
        return icon.clone();
    }
    let name = format!("materials/spawnicons/{}.png", path.trim_end_matches(".mdl"));
    let result = {
        let source = world.resource::<MountedSource>();
        source
            .mounts
            .read(&source.bsp, &name)
            .ok()
            .and_then(|b| image::load_from_memory(&b).ok())
            .map(|i| rgba_image(i.to_rgba8()))
    };
    let handle = result.map(|image| world.resource_mut::<Assets<Image>>().add(image));
    world
        .resource_mut::<PlayState>()
        .icons
        .insert(path.into(), handle.clone());
    handle
}
fn rebuild_menu(world: &mut World) {
    menu::rebuild(world);
}
