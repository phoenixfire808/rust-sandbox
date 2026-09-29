use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    pbr::Lightmap,
    prelude::*,
    render::{
        render_resource::Face,
        view::screenshot::{save_to_disk, Screenshot},
    },
};
use bevy_rapier3d::prelude::{Collider, RigidBody};
use rust_sandbox::source_play::{PlayState, SourceCamera, SourceHud, SourcePlayPlugin};
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
struct SkyFace;

fn main() {
    if let Err(e) = run() {
        eprintln!("SOURCE MAP LOAD FAILED: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() {
        rust_sandbox::source_frontend::run();
        return Ok(());
    }
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
    if args.iter().any(|a| a == "--export-catalog") {
        let output = project_root().join("local/playable-catalog");
        println!("{}", loaded.source.export_catalog(&output)?);
        return Ok(());
    }
    if args.iter().any(|a| a == "--inspect-weapons") {
        for path in [
            "scripts/weapons/weapon_physgun.txt",
            "lua/weapons/gmod_tool/shared.lua",
        ] {
            match loaded.source.mounts.read(&loaded.source.bsp, path) {
                Ok(bytes) => println!("{}\n{}", path, String::from_utf8_lossy(&bytes)),
                Err(e) => eprintln!("{path}: {e}"),
            }
        }
        return Ok(());
    }
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
    let mut play = PlayState::new(loaded.source.models()?, def.fov_degrees, def.fly_speed);
    if args.iter().any(|a| a == "--smoke") {
        play.storage_root = project_root().join("local/playable-smoke");
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
        .insert_resource(play)
        .insert_resource(rust_sandbox::source_frontend::Frontend::new(
            false,
            def.id.clone(),
        ))
        .add_plugins(rust_sandbox::source_frontend::FrontendPlugin)
        .add_plugins((
            SourcePlayPlugin,
            rust_sandbox::source_player::SourcePlayerPlugin,
        ))
        .insert_resource(Settings {
            def,
            smoke: args.iter().any(|a| a == "--smoke"),
            frame: 0,
            screenshot_done: false,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (movement, smoke)
                .chain()
                .after(rust_sandbox::source_player::input)
                .before(rust_sandbox::source_play::update_play),
        )
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
    commands.insert_resource(map.source);
    let lightmap = images.add(map.lightmap);
    let texture_handles: std::collections::BTreeMap<_, _> = map
        .textures
        .into_iter()
        .map(|(name, image)| (name, images.add(image)))
        .collect();
    let mut water = rust_sandbox::source_assets::WaterSurfaces::default();
    for surface in map.surfaces {
        if surface.name.contains("water") {
            water.0.extend(surface.geometry.positions.chunks_exact(3).map(|p| [Vec3::from_array(p[0]), Vec3::from_array(p[1]), Vec3::from_array(p[2])]));
        }
        if !surface.name.contains("water") {
            let vertices = surface
                .geometry
                .positions
                .iter()
                .map(|p| Vec3::from_array(*p))
                .collect();
            let indices = (0..surface.geometry.positions.len() as u32)
                .collect::<Vec<_>>()
                .chunks_exact(3)
                .map(|t| [t[0], t[1], t[2]])
                .collect();
            if let Ok(collider) = Collider::trimesh(vertices, indices) {
                commands.spawn((
                    RigidBody::Fixed,
                    collider,
                    Transform::default(),
                    Visibility::default(),
                    bevy_rapier3d::prelude::CollisionGroups::new(
                        bevy_rapier3d::prelude::Group::GROUP_1,
                        bevy_rapier3d::prelude::Group::ALL,
                    ),
                ));
            }
        }
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
    commands.insert_resource(water);
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
    rust_sandbox::source_player::spawn(&mut commands, map.spawn, map.forward);
    commands.spawn((
        SourceCamera,
        IsDefaultUiCamera,
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
    commands.spawn((
        Camera2d,
        Camera {
            order: 2,
            clear_color: bevy::render::camera::ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
    ));
    commands.spawn((
        SourceHud,
        Text::new("Loading original weapon and build menu..."),
        TextFont {
            font_size: 16.,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.),
            bottom: Val::Px(16.),
            padding: UiRect::all(Val::Px(8.)),
            ..default()
        },
        BackgroundColor(Color::srgba(0., 0., 0., 0.6)),
    ));
    println!("SOURCE_MAP_WINDOW_READY");
}
fn movement(
    camera: Query<&Transform, (With<SourceCamera>, Without<SkyFace>)>,
    mut sky: Query<&mut Transform, (With<SkyFace>, Without<SourceCamera>)>,
) {
    if let Ok(camera) = camera.single() {
        for mut t in &mut sky {
            t.translation = camera.translation;
        }
    }
}
fn smoke(world: &mut World) {
    if !world.resource::<Settings>().smoke {
        return;
    }
    world.resource_mut::<Settings>().frame += 1;
    let frame = world.resource::<Settings>().frame;
    if frame == 140 {
        let eye = world
            .resource::<rust_sandbox::source_player::PlayerState>()
            .eye;
        let model = world.resource::<PlayState>().config.default_prop.clone();
        let entity = rust_sandbox::source_play::spawn_model(
            world,
            &model,
            eye.translation + *eye.forward() * 4.,
            Quat::IDENTITY,
            true,
        )
        .expect("effects smoke original prop");
        // Aim at the model bounds center, which may differ from its authored origin.
        let center = world
            .get::<rust_sandbox::source_play::SpawnedProp>(entity)
            .unwrap()
            .center;
        world.get_mut::<Transform>(entity).unwrap().translation -= center;
        assert_eq!(
            rust_sandbox::source_play::aimed_prop(world).map(|p| p.0),
            Some(entity),
            "effects fixture must be under the eye trace"
        );
        // Inject pickup immediately, without a frame-rate-dependent simulation gap.
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
    }
    if frame == 141 {
        assert!(
            world.resource::<PlayState>().held.is_some(),
            "pickup input must acquire fixture"
        );
    }
    if frame == 350 {
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
    }
    if matches!(frame, 120 | 165 | 240 | 360 | 460 | 540) {
        let (glows, beam) = rust_sandbox::source_effects::visible_counts(world)
            .expect("effects must load from installed textures");
        if frame < 400 {
            assert!(glows >= 3, "missing attached physgun glow");
            assert_eq!(beam, matches!(frame, 165 | 240), "held/released beam state");
        } else {
            assert_eq!(
                (glows, beam),
                (0, false),
                "stale physgun effects after tool switch"
            );
        }
        println!("PHYSGUN_EFFECT_STATE frame={frame} glows={glows} beam={beam}");
    }
    if frame == 120 && !rust_sandbox::source_player::scene_ready(world) {
        eprintln!("PLAYER_SMOKE_FAILED: player assets did not load");
        world.send_event(AppExit::error());
        return;
    }
    if let Some(mut player) = world.get_resource_mut::<rust_sandbox::source_player::PlayerState>() {
        if frame == 180 {
            player.third_person = true;
        }
        if (260..340).contains(&frame) {
            player.direction = Vec3::NEG_Z;
        }
        if frame == 340 {
            player.direction = Vec3::ZERO;
        }
        if frame == 480 {
            player.third_person = false;
        }
        if matches!(frame, 120 | 240 | 320 | 460 | 540) {
            println!(
                "PLAYER_SMOKE_STATE frame={frame} eye={:?} grounded={} third={} moving={}",
                player.eye.translation, player.grounded, player.third_person, player.moving
            );
        }
    }
    if frame == 400 {
        world.resource_mut::<PlayState>().physgun = false;
    }
    if frame == 620 && world.resource::<Settings>().def.id == "gm_construct" {
        let (entity, center_offset) = {
            let mut p = world.resource_mut::<rust_sandbox::source_player::PlayerState>();
            p.yaw = 0.;
            p.pitch = 0.;
            p.vertical = 0.;
            p.third_person = false;
            (p.entity, p.config.height * 0.5 - p.config.eye_height)
        };
        // Acceptance fixture inside the white room omitted by the old model-0 importer.
        world.get_mut::<Transform>(entity).unwrap().translation =
            source_assets::source_position([-2048., -3600., -192.], 0.01905)
                + Vec3::Y * center_offset;
    }
    let capture = match frame {
        120 => Some("player-first"),
        165 => Some("physgun-held-first"),
        240 => Some("player-third"),
        320 => Some("player-walk"),
        460 => Some("tool-third"),
        540 => Some("tool-first"),
        680 => Some("interior-walls"),
        _ => None,
    };
    if let Some(label) = capture {
        world.resource_mut::<Settings>().screenshot_done = false;
        let path = project_root().join("evidence").join(format!(
            "{}-{label}.png",
            world.resource::<Settings>().def.id
        ));
        world
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path))
            .observe(
                |_: Trigger<bevy::render::view::screenshot::ScreenshotCaptured>,
                 mut settings: ResMut<Settings>| {
                    settings.screenshot_done = true;
                },
            );
    }
    if frame > 740 && world.resource::<Settings>().screenshot_done {
        println!("SOURCE_PLAYER_RENDER_OK");
        world.send_event(AppExit::Success);
    }
    if frame > 1800 {
        eprintln!("player screenshot timed out");
        world.send_event(AppExit::error());
    }
}
