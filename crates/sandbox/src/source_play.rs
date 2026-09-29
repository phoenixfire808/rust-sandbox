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
    Category(String),
    Page(i32),
    Spawn(String),
    Tool(String),
    Setting(&'static str, f32),
    Save,
    Load,
    Close,
}
#[derive(Component)]
struct MenuRoot;
#[derive(Resource)]
pub struct PlayState {
    pub config: PlayConfig,
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
    undo: Vec<Vec<SavedModel>>,
    cache: BTreeMap<String, GpuModel>,
    icons: BTreeMap<String, Option<Handle<Image>>>,
    last_save: Option<std::path::PathBuf>,
}
#[derive(Clone)]
pub(crate) struct GpuModel {
    pub(crate) parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>)>,
    pub(crate) geometry: Vec<crate::source_assets::Geometry>,
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
            config,
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
        .add_systems(
            Update,
            (search_input, update_play, adjust_hold_distance).chain(),
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
    let scale = 0.01905;
    let decoded = world.resource_scope(|_world, source: Mut<MountedSource>| -> Result<_> {
        let geometry = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            source_models::model_parts(&source.bsp, &source.mounts, path, skin)
        }))
        .map_err(|_| "unsupported model format")??;
        let mut parts = Vec::new();
        for (name, mut geo) in geometry {
            for p in &mut geo.positions {
                for axis in p {
                    *axis *= scale;
                }
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
            parts.push((geo, texture, alpha, mat.no_cull()));
        }
        Ok(parts)
    })?;
    let mut points = Vec::new();
    let mut parts = Vec::new();
    let mut geometry = Vec::new();
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for (geo, texture, alpha, nocull) in decoded {
        for p in &geo.positions {
            let v = Vec3::from_array(*p);
            min = min.min(v);
            max = max.max(v);
            points.push(v);
        }
        let tex = texture.map(|image| world.resource_mut::<Assets<Image>>().add(image));
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
    if points.is_empty() || !min.is_finite() || !max.is_finite() {
        return Err("model contains no finite render geometry".into());
    }
    let half = ((max - min) * 0.5).max(Vec3::splat(0.02));
    let center = (min + max) * 0.5;
    let collider =
        Collider::convex_hull(&points).ok_or("cannot build model convex collision hull")?;
    let model = GpuModel {
        parts,
        geometry,
        collider,
        center,
        half,
    };
    world
        .resource_mut::<PlayState>()
        .cache
        .insert(cache_key, model.clone());
    Ok(model)
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
pub fn aimed_prop(world: &mut World) -> Option<(Entity, f32)> {
    let camera = camera(world)?;
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
            .map(|d| (e, d))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
}
fn snapshot(world: &mut World) -> Vec<SavedModel> {
    world
        .query::<(&SpawnedProp, &Transform, &RigidBody)>()
        .iter(world)
        .map(|(p, t, b)| SavedModel {
            model: p.model.clone(),
            position: t.translation.to_array(),
            rotation: t.rotation.to_array(),
            frozen: *b == RigidBody::Fixed,
        })
        .collect()
}
fn remember(world: &mut World) {
    let save = snapshot(world);
    let mut s = world.resource_mut::<PlayState>();
    if s.undo.len() == 32 {
        s.undo.remove(0);
    }
    s.undo.push(save);
}
fn restore(world: &mut World, save: Vec<SavedModel>) -> Result<()> {
    if save.len() > world.resource::<PlayState>().config.max_props {
        return Err("save exceeds prop limit".into());
    }
    for p in &save {
        crate::source_assets::virtual_path(&p.model)?;
        if !p
            .position
            .iter()
            .chain(p.rotation.iter())
            .all(|v| v.is_finite())
            || (Quat::from_array(p.rotation).length() - 1.).abs() > 0.01
        {
            return Err("invalid save transform".into());
        }
        gpu_model(world, &p.model)?;
    }
    let ids: Vec<_> = world
        .query_filtered::<Entity, With<SpawnedProp>>()
        .iter(world)
        .collect();
    for e in ids {
        world.despawn(e);
    }
    world.resource_mut::<PlayState>().held = None;
    for p in save {
        spawn_model(
            world,
            &p.model,
            Vec3::from_array(p.position),
            Quat::from_array(p.rotation),
            p.frozen,
        )?;
    }
    Ok(())
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
    if !state.menu_open || state.tab != 0 {
        events.clear();
        return;
    }
    for e in events.read() {
        if e.state != ButtonState::Pressed {
            continue;
        }
        match &e.logical_key {
            Key::Character(c) => {
                if c.as_str() != "q" && c.as_str() != "Q" {
                    state.search.push_str(c);
                    state.page = 0;
                    state.dirty = true;
                }
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
        .filter(|(i, _)| **i == Interaction::Pressed)
        .map(|(_, a)| a.clone())
        .collect();
    if keys.just_pressed(KeyCode::KeyQ) {
        let mut s = world.resource_mut::<PlayState>();
        s.menu_open = !s.menu_open;
        s.dirty = true;
        s.held = None;
    }
    if keys.just_pressed(KeyCode::Escape) {
        let mut s = world.resource_mut::<PlayState>();
        s.menu_open = true;
        s.tab = 2;
        s.dirty = true;
        s.held = None;
    }
    for (key, physgun) in [(KeyCode::Digit1, true), (KeyCode::Digit2, false)] {
        if keys.just_pressed(key) && !world.resource::<PlayState>().menu_open {
            world.resource_mut::<PlayState>().physgun = physgun;
            set_weapon(world);
        }
    }
    if world.resource::<PlayState>().menu_open {
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
            let save = world.resource_mut::<PlayState>().undo.pop();
            if let Some(save) = save {
                if let Err(e) = restore(world, save) {
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
        let focus = world
            .resource::<PlayState>()
            .held
            .or_else(|| aimed_prop(world).map(|p| p.0));
        if mouse.just_pressed(MouseButton::Left) {
            if let Some(e) = focus {
                if world.resource::<PlayState>().physgun {
                    remember(world);
                    let distance = aimed_prop(world).map(|p| p.1).unwrap_or(5.).clamp(1., 50.);
                    let mut s = world.resource_mut::<PlayState>();
                    s.held = Some(e);
                    s.distance = distance;
                    world.entity_mut(e).insert(RigidBody::Dynamic);
                } else {
                    let tool = world.resource::<PlayState>().tool.clone();
                    remember(world);
                    match tool.as_str() {
                        "remover" => {
                            world.despawn(e);
                        }
                        "duplicator" => {
                            let p = world.get::<SpawnedProp>(e).unwrap().model.clone();
                            let t = *world.get::<Transform>(e).unwrap();
                            if let Err(err) =
                                spawn_model(world, &p, t.translation + Vec3::Y, t.rotation, false)
                            {
                                world.resource_mut::<PlayState>().status = err.to_string();
                            }
                        }
                        "freeze" => {
                            world
                                .entity_mut(e)
                                .insert((RigidBody::Fixed, Velocity::zero()));
                        }
                        _ => {
                            world.resource_mut::<PlayState>().status =
                                format!("Tool {tool} is cataloged but not implemented");
                        }
                    }
                }
            }
        }
        if mouse.just_released(MouseButton::Left) {
            world.resource_mut::<PlayState>().held = None;
        }
        if mouse.just_pressed(MouseButton::Right) {
            if let Some(e) = focus {
                world
                    .entity_mut(e)
                    .insert((RigidBody::Fixed, Velocity::zero()));
                world.resource_mut::<PlayState>().held = None;
            }
        }
        if keys.just_pressed(KeyCode::KeyR) {
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
                let pos = world.get::<Transform>(e).map(|t| t.translation);
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
        if let Projection::Perspective(p) = p.as_mut() {
            p.fov = fov.to_radians();
        }
    }
    for mut c in world.query::<&mut RapierConfiguration>().iter_mut(world) {
        c.gravity = Vec3::Y * gravity;
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
        hud.0 = text.clone();
    }
    if world.resource::<PlayState>().dirty {
        rebuild_menu(world);
    }
}
fn perform(world: &mut World, action: UiAction) {
    match action {
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
            s.dirty = true;
        }
        UiAction::Spawn(path) => {
            if let Some(cam) = camera(world) {
                let d = world.resource::<PlayState>().config.spawn_distance;
                remember(world);
                match spawn_model(
                    world,
                    &path,
                    cam.translation + *cam.forward() * d,
                    Quat::IDENTITY,
                    false,
                ) {
                    Ok(_) => {
                        let mut s = world.resource_mut::<PlayState>();
                        s.selected = path.clone();
                        s.status = format!("Spawned {path}");
                    }
                    Err(e) => {
                        world.resource_mut::<PlayState>().status =
                            format!("Cannot spawn {path}: {e}")
                    }
                }
            }
        }
        UiAction::Tool(tool) => {
            let mut s = world.resource_mut::<PlayState>();
            s.tool = tool;
            s.physgun = false;
            s.status = "Left click: tool action. Right click: freeze.".into();
            s.dirty = true;
            set_weapon(world);
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
                let save = serde_json::from_slice(&std::fs::read(path)?)?;
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
        if state.held.is_some() {
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
    text(world, id, label, 15.);
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
    let old: Vec<_> = world
        .query_filtered::<Entity, With<MenuRoot>>()
        .iter(world)
        .collect();
    for e in old {
        world.despawn(e);
    }
    world.resource_mut::<PlayState>().dirty = false;
    if !world.resource::<PlayState>().menu_open {
        return;
    }
    let root = world
        .spawn((
            MenuRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(3.),
                top: Val::Percent(5.),
                width: Val::Percent(94.),
                height: Val::Percent(82.),
                padding: UiRect::all(Val::Px(12.)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.),
                ..default()
            },
            BackgroundColor(Color::srgba(0.9, 0.92, 0.94, 0.98)),
            GlobalZIndex(20),
        ))
        .id();
    let bar = container(
        world,
        root,
        Node {
            flex_direction: FlexDirection::Row,
            ..default()
        },
    );
    for (i, name) in [(0, "Props"), (1, "Tools"), (2, "Settings")] {
        button(world, bar, name, UiAction::Tab(i));
    }
    button(world, bar, "Save scene", UiAction::Save);
    button(world, bar, "Load scene", UiAction::Load);
    button(world, bar, "Resume [Q]", UiAction::Close);
    let tab = world.resource::<PlayState>().tab;
    if tab == 0 {
        let (search, category, page, page_size, models) = {
            let s = world.resource::<PlayState>();
            (
                s.search.clone(),
                s.category.clone(),
                s.page,
                s.config.page_size,
                s.models.clone(),
            )
        };
        text(world,root,format!("Search: {search}_   Type to filter, Backspace to erase. {} indexed original models.",models.len()),17.);
        let categories: Vec<_> = models
            .iter()
            .map(|m| m.category.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let cats = container(
            world,
            root,
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                max_height: Val::Px(105.),
                overflow: Overflow::clip(),
                ..default()
            },
        );
        button(world, cats, "All models", UiAction::Category(String::new()));
        for c in categories {
            button(world, cats, &c, UiAction::Category(c.clone()));
        }
        let matches: Vec<_> = models
            .into_iter()
            .filter(|m| {
                (category.is_empty() || m.category == category)
                    && m.model.contains(&search.to_lowercase())
            })
            .collect();
        let pages = matches.len().div_ceil(page_size).max(1);
        let page = page.min(pages - 1);
        world.resource_mut::<PlayState>().page = page;
        let pager = container(
            world,
            root,
            Node {
                flex_direction: FlexDirection::Row,
                ..default()
            },
        );
        button(world, pager, "<", UiAction::Page(-1));
        text(
            world,
            pager,
            format!(
                "{} matches | Page {} / {} | {}",
                matches.len(),
                page + 1,
                pages,
                if category.is_empty() {
                    "all"
                } else {
                    &category
                }
            ),
            16.,
        );
        button(world, pager, ">", UiAction::Page(1));
        let grid = container(
            world,
            root,
            Node {
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(6, 1.),
                column_gap: Val::Px(6.),
                row_gap: Val::Px(6.),
                ..default()
            },
        );
        for model in matches.into_iter().skip(page * page_size).take(page_size) {
            let icon = thumbnail(world, &model.model);
            let id = button(world, grid, "", UiAction::Spawn(model.model.clone()));
            world.entity_mut(id).insert(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.),
                height: Val::Px(145.),
                padding: UiRect::all(Val::Px(5.)),
                overflow: Overflow::clip(),
                ..default()
            });
            if let Some(icon) = icon {
                world.spawn((
                    ImageNode::new(icon),
                    Node {
                        width: Val::Px(90.),
                        height: Val::Px(90.),
                        ..default()
                    },
                    ChildOf(id),
                ));
            }
            text(world, id, model.model.trim_start_matches("models/"), 13.);
        }
    } else if tab == 1 {
        text(world,root,"Toolgun: select an implemented tool. Catalog entries below are NOT advertised as working.",18.);
        let row = container(
            world,
            root,
            Node {
                flex_direction: FlexDirection::Row,
                ..default()
            },
        );
        for (tool, label) in [
            ("remover", "Remover"),
            ("duplicator", "Duplicate one prop"),
            ("freeze", "Freeze"),
        ] {
            button(world, row, label, UiAction::Tool(tool.into()));
        }
        let specs = crate::compiled_behaviors();
        let labels = specs
            .iter()
            .filter(|s| s.id.starts_with("tool_"))
            .map(|s| s.id.trim_start_matches("tool_"))
            .collect::<Vec<_>>()
            .join("   |   ");
        text(world,root,format!("Installed tool research catalog:\n{labels}\n\nRemaining tools require implementation. Selecting a label is not tool functionality."),16.);
    } else {
        let (fov, speed, sens, visible) = {
            let s = world.resource::<PlayState>();
            (s.fov, s.speed, s.sensitivity, s.weapon_visible)
        };
        for (label, key, value, delta) in [
            ("Field of view", "fov", fov, 5.),
            ("Noclip speed", "speed", speed, 1.),
            ("Mouse sensitivity", "sensitivity", sens, 0.0005),
        ] {
            let row = container(
                world,
                root,
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                },
            );
            text(world, row, format!("{label}: {value:.4}"), 18.);
            button(world, row, "-", UiAction::Setting(key, -delta));
            button(world, row, "+", UiAction::Setting(key, delta));
        }
        button(
            world,
            root,
            format!("Viewmodel: {}", if visible { "ON" } else { "OFF" }),
            UiAction::Setting("weapon", 0.),
        );
        text(world,root,"Preferences persist locally. Engine audio/video/game settings beyond these controls are not implemented.\nOriginal assets are local only. Physics uses generated convex hulls, not Source PHY collision.\nGrounded walking and F4 third person. V toggles noclip. No NPC AI, ragdolls, Lua addons or multiplayer yet.",16.);
    }
}
