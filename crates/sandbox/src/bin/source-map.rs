use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::MouseMotion,
    pbr::Lightmap,
    prelude::*,
    render::{
        render_resource::Face,
        view::screenshot::{save_to_disk, Screenshot},
    },
    window::{CursorGrabMode, PrimaryWindow},
};
use rust_sandbox::{
    compiled_source_maps, project_root,
    source_assets::{self, Geometry, LoadedMap},
};
use sandbox_catalog::{source_maps::SourceMapDef, Result};
use std::path::PathBuf;

#[derive(Resource)]
struct PendingMap(Option<LoadedMap>);
#[derive(Resource)]
struct Settings {
    def: SourceMapDef,
    smoke: bool,
    frame: u32,
    screenshot_done: bool,
}
#[derive(Component)]
struct PlayerCamera;
#[derive(Component)]
struct SkyFace;

fn main() {
    if let Err(e) = run() {
        eprintln!("SOURCE MAP LOAD FAILED: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let name = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("gm_construct");
    let def = compiled_source_maps()
        .into_iter()
        .find(|m| m.id == name)
        .ok_or("map must be listed in sheets/source_maps.csv")?;
    let install = std::env::var_os("GMOD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"D:\SteamLibrary\steamapps\common\GarrysMod"));
    let loaded = source_assets::load(&install, &def)?;
    let local = project_root().join("local");
    std::fs::create_dir_all(&local)?;
    std::fs::write(
        local.join(format!("{}-import-report.json", def.id)),
        serde_json::to_vec_pretty(&loaded.report)?,
    )?;
    if args.iter().any(|a| a == "--validate") {
        if loaded.surfaces.is_empty()
            || loaded.report["textured_materials"].as_u64().unwrap_or(0) == 0
        {
            return Err("no textured world loaded".into());
        }
        println!("SOURCE_MAP_IMPORT_OK");
        return Ok(());
    }
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("{} | Bevy local Source map", def.id),
                resolution: (1280.0_f32, 800.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.45, 0.65, 0.85)))
        .insert_resource(AmbientLight {
            brightness: 0.,
            ..default()
        })
        .insert_resource(PendingMap(Some(loaded)))
        .insert_resource(Settings {
            def,
            smoke: args.iter().any(|a| a == "--smoke"),
            frame: 0,
            screenshot_done: false,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (movement, smoke).chain())
        .run();
    Ok(())
}
fn setup(
    mut commands: Commands,
    mut pending: ResMut<PendingMap>,
    settings: Res<Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let map = pending.0.take().expect("map loaded before window creation");
    let lightmap = images.add(map.lightmap);
    let texture_handles: std::collections::BTreeMap<_, _> = map
        .textures
        .into_iter()
        .map(|(name, image)| (name, images.add(image)))
        .collect();
    for surface in map.surfaces {
        let material = materials.add(StandardMaterial {
            base_color: surface.tint,
            base_color_texture: surface.texture.map(|name| texture_handles[&name].clone()),
            alpha_mode: surface.alpha,
            unlit: surface.unlit,
            cull_mode: if surface.no_cull {
                None
            } else {
                Some(Face::Back)
            },
            perceptual_roughness: 1.,
            reflectance: 0.,
            ..default()
        });
        commands.spawn((
            Name::new(surface.name),
            Mesh3d(meshes.add(surface.geometry.mesh())),
            MeshMaterial3d(material),
            Lightmap {
                image: lightmap.clone(),
                uv_rect: Rect::new(0., 0., 1., 1.),
                bicubic_sampling: false,
            },
        ));
    }
    // Six original sky textures, rendered on a large camera-centered cube.
    for (side, image) in map.sky {
        let (normal, right, up) = match side.as_str() {
            "ft" => (Vec3::NEG_Z, Vec3::X, Vec3::Y),
            "bk" => (Vec3::Z, Vec3::NEG_X, Vec3::Y),
            "lf" => (Vec3::NEG_X, Vec3::NEG_Z, Vec3::Y),
            "rt" => (Vec3::X, Vec3::Z, Vec3::Y),
            "up" => (Vec3::Y, Vec3::X, Vec3::Z),
            _ => (Vec3::NEG_Y, Vec3::X, Vec3::NEG_Z),
        };
        let corners = [
            normal - right + up,
            normal + right + up,
            normal + right - up,
            normal - right - up,
        ];
        let uv = [[0., 0.], [1., 0.], [1., 1.], [0., 1.]];
        let mut geo = Geometry::default();
        for i in [0, 1, 2, 0, 2, 3] {
            geo.positions.push((corners[i] * 2000.).to_array());
            geo.normals.push((-normal).to_array());
            geo.uv.push(uv[i]);
            geo.light_uv.push([0.; 2]);
        }
        commands.spawn((
            SkyFace,
            Name::new(format!("sky/{side}")),
            Mesh3d(meshes.add(geo.mesh())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(images.add(image)),
                unlit: true,
                cull_mode: None,
                ..default()
            })),
            Transform::from_translation(map.spawn),
        ));
    }
    commands.spawn((
        PlayerCamera,
        Camera3d::default(),
        Tonemapping::None,
        bevy::render::camera::Exposure {
            ev100: -1.2_f32.log2(),
        },
        Projection::Perspective(PerspectiveProjection {
            fov: settings.def.fov_degrees.to_radians(),
            near: 0.05,
            far: 10000.,
            ..default()
        }),
        Transform::from_translation(map.spawn).looking_to(map.forward, Vec3::Y),
    ));
    commands.spawn((Text::new(format!("{}  |  Original local BSP + VMT/VTF\nClick: capture mouse  Esc: release  F10: quit\nWASD: noclip  Space/Ctrl: up/down  Shift: fast\nAsset preview, not gameplay parity",settings.def.id)),TextFont {font_size:16.,..default()},Node {position_type:PositionType::Absolute,left:Val::Px(16.),bottom:Val::Px(16.),padding:UiRect::all(Val::Px(8.)),..default()},BackgroundColor(Color::srgba(0.,0.,0.,0.6))));
    println!("SOURCE_MAP_WINDOW_READY");
}
// Each independent Bevy input resource is injected by the ECS scheduler.
#[allow(clippy::too_many_arguments)]
fn movement(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut events: EventReader<MouseMotion>,
    time: Res<Time>,
    settings: Res<Settings>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<SkyFace>)>,
    mut sky: Query<&mut Transform, (With<SkyFace>, Without<PlayerCamera>)>,
    mut exit: EventWriter<AppExit>,
) {
    let Ok(mut window) = window.single_mut() else {
        return;
    };
    if keys.just_pressed(KeyCode::F10) {
        exit.write(AppExit::Success);
    }
    if mouse.just_pressed(MouseButton::Left) && window.focused {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
    if keys.just_pressed(KeyCode::Escape) || !window.focused {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
    let delta = events.read().fold(Vec2::ZERO, |sum, e| sum + e.delta);
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    if window.cursor_options.grab_mode != CursorGrabMode::None {
        let (yaw, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
        camera.rotation = Quat::from_euler(
            EulerRot::YXZ,
            yaw - delta.x * 0.0025,
            (pitch - delta.y * 0.0025).clamp(-1.55, 1.55),
            0.,
        );
    }
    let mut direction = Vec3::ZERO;
    if window.focused {
        if keys.pressed(KeyCode::KeyW) {
            direction += *camera.forward();
        }
        if keys.pressed(KeyCode::KeyS) {
            direction -= *camera.forward();
        }
        if keys.pressed(KeyCode::KeyD) {
            direction += *camera.right();
        }
        if keys.pressed(KeyCode::KeyA) {
            direction -= *camera.right();
        }
        if keys.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }
        if keys.pressed(KeyCode::ControlLeft) {
            direction -= Vec3::Y;
        }
        let speed = settings.def.fly_speed
            * if keys.pressed(KeyCode::ShiftLeft) {
                4.
            } else {
                1.
            };
        camera.translation += direction.normalize_or_zero() * speed * time.delta_secs();
    }
    for mut t in &mut sky {
        t.translation = camera.translation;
    }
}
fn smoke(mut commands: Commands, mut settings: ResMut<Settings>, mut exit: EventWriter<AppExit>) {
    if !settings.smoke {
        return;
    }
    settings.frame += 1;
    if settings.frame == 120 {
        let path = project_root()
            .join("evidence")
            .join(format!("{}-source.png", settings.def.id));
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path))
            .observe(
                |_: Trigger<bevy::render::view::screenshot::ScreenshotCaptured>,
                 mut settings: ResMut<Settings>| {
                    settings.screenshot_done = true;
                },
            );
    }
    if settings.frame > 180 && settings.screenshot_done {
        println!("SOURCE_MAP_RENDER_OK: {}", settings.def.id);
        exit.write(AppExit::Success);
    }
    if settings.frame > 1800 {
        eprintln!("screenshot timed out");
        exit.write(AppExit::error());
    }
}
