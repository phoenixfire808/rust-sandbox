use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use rust_sandbox::*;

#[derive(Component)]
struct FlyCamera;
#[derive(Component)]
struct Hud;
#[derive(Component)]
struct SpawnButton(usize);
#[derive(Resource, Default)]
struct SmokeFrames {
    frame: u32,
    enabled: bool,
}

fn main() {
    if std::env::args().any(|a| a == "--headless-smoke") {
        if let Err(e) = smoke() {
            eprintln!("FAIL: {e}");
            std::process::exit(1);
        }
        return;
    }
    let smoke = std::env::args().any(|a| a == "--smoke");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Workshop | Bevy sandbox foundation".into(),
                resolution: (1280.0_f32, 800.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.055, 0.075, 0.105)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 450.,
            ..default()
        })
        .insert_resource(SmokeFrames {
            enabled: smoke,
            ..default()
        })
        .add_plugins(SandboxPlugin)
        .add_systems(Startup, setup_visuals.before(setup_scene))
        .add_systems(
            Update,
            (camera_input, smoke_frames, controls, draw_overlay).chain(),
        )
        .run();
}

fn setup_visuals(
    mut commands: Commands,
    catalog: Res<Catalog>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut palette = Palette::default();
    for p in &catalog.0.props {
        palette.0.push((
            meshes.add(Cuboid::new(p.size_x, p.size_y, p.size_z)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(p.red, p.green, p.blue),
                perceptual_roughness: 0.7,
                ..default()
            }),
        ));
    }
    commands.insert_resource(palette);
    let h = catalog.0.world.ground_half_size;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(h * 2., 0.5, h * 2.))),
        MeshMaterial3d(materials.add(Color::srgb(0.14, 0.18, 0.22))),
        Transform::from_xyz(0., -0.25, 0.),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.6, 0.)),
    ));
    commands.spawn((
        Camera3d::default(),
        Tonemapping::None,
        FlyCamera,
        Transform::from_xyz(10., 8., 14.).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));
    commands.spawn((
        Text::new("RUST WORKSHOP  /  BEVY + RAPIER\nSpreadsheet-driven sandbox foundation"),
        TextFont {
            font_size: 24.,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.92, 1.)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.),
            top: Val::Px(20.),
            ..default()
        },
    ));
    commands.spawn((Node{position_type:PositionType::Absolute,left:Val::Px(20.),top:Val::Px(100.),width:Val::Px(225.),padding:UiRect::all(Val::Px(12.)),flex_direction:FlexDirection::Column,row_gap:Val::Px(8.),..default()},BackgroundColor(Color::srgba(0.025,0.04,0.06,0.93))))
        .with_children(|parent| {
            parent.spawn((Text::new("SPAWN CATALOG"),TextFont{font_size:18.,..default()},TextColor(Color::srgb(0.5,0.8,0.9))));
            for (index,p) in catalog.0.props.iter().enumerate() {
                parent.spawn((Button,SpawnButton(index),Node{padding:UiRect::all(Val::Px(10.)),..default()},BackgroundColor(Color::srgb(0.10,0.19,0.25))))
                    .with_children(|button|{button.spawn((Text::new(format!("{}  {}\n{} kg",index+1,p.label,p.mass)),TextFont{font_size:16.,..default()}));});
            }
            parent.spawn((Text::new("Click a prop to spawn\nTab: cycle  Enter: spawn\n\nWASD + Space/Shift: fly\nMiddle mouse: look\nLeft mouse: hold prop\nWheel: hold distance\nRight mouse / F: freeze\nR: unfreeze  E: rotate\nX: remove  C: duplicate\nZ: undo  F9: reset\nF5: save  F6: load latest\nEsc: exit"),TextFont{font_size:15.,..default()},TextColor(Color::srgb(0.73,0.79,0.84))));
        });
    commands.spawn((
        Text::new("+"),
        TextFont {
            font_size: 24.,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.),
            top: Val::Percent(50.),
            ..default()
        },
    ));
    commands.spawn((
        Text::new("Initializing..."),
        TextFont {
            font_size: 17.,
            ..default()
        },
        Hud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.),
            bottom: Val::Px(20.),
            ..default()
        },
    ));
}

// Bevy injects independent ECS resources and event readers into this system.
#[allow(clippy::too_many_arguments)]
fn camera_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut wheel: EventReader<MouseWheel>,
    time: Res<Time>,
    catalog: Res<Catalog>,
    mut camera: Query<&mut Transform, With<FlyCamera>>,
    mut aim: ResMut<Aim>,
) {
    let Ok(mut t) = camera.single_mut() else {
        return;
    };
    let delta = motion.read().fold(Vec2::ZERO, |sum, e| sum + e.delta);
    if mouse.pressed(MouseButton::Middle) {
        let (yaw, pitch, _) = t.rotation.to_euler(EulerRot::YXZ);
        t.rotation = Quat::from_euler(
            EulerRot::YXZ,
            yaw - delta.x * 0.003,
            (pitch - delta.y * 0.003).clamp(-1.5, 1.5),
            0.,
        );
    }
    let mut movement = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        movement += *t.forward();
    }
    if keys.pressed(KeyCode::KeyS) {
        movement -= *t.forward();
    }
    if keys.pressed(KeyCode::KeyD) {
        movement += *t.right();
    }
    if keys.pressed(KeyCode::KeyA) {
        movement -= *t.right();
    }
    if keys.pressed(KeyCode::Space) {
        movement += Vec3::Y;
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        movement -= Vec3::Y;
    }
    t.translation += movement.normalize_or_zero() * catalog.0.world.move_speed * time.delta_secs();
    t.translation.y = t.translation.y.max(0.5);
    for event in wheel.read() {
        aim.distance = (aim.distance + event.y * 0.5).clamp(1., 50.);
    }
    aim.origin = t.translation;
    aim.direction = *t.forward();
}

fn controls(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>().clone();
    let mouse = world.resource::<ButtonInput<MouseButton>>().clone();
    let mut actions = Vec::new();
    let clicked: Vec<usize> = world
        .query_filtered::<(&Interaction, &SpawnButton), Changed<Interaction>>()
        .iter(world)
        .filter(|(i, _)| **i == Interaction::Pressed)
        .map(|(_, b)| b.0)
        .collect();
    let pointer_on_ui = world
        .query::<&Interaction>()
        .iter(world)
        .any(|i| *i != Interaction::None);
    let count = world.resource::<Catalog>().0.props.len();
    if keys.just_pressed(KeyCode::Tab) {
        let mut s = world.resource_mut::<Session>();
        s.selected = (s.selected + 1) % count;
    }
    for (i, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ]
    .iter()
    .enumerate()
    {
        if keys.just_pressed(*key) && i < count {
            world.resource_mut::<Session>().selected = i;
        }
    }
    let mut requested = clicked;
    if keys.just_pressed(KeyCode::Enter) {
        requested.push(world.resource::<Session>().selected);
    }
    for index in requested {
        let aim = world.resource::<Aim>();
        let mut position =
            aim.origin + aim.direction * world.resource::<Catalog>().0.world.spawn_distance;
        position.y = position
            .y
            .max(world.resource::<Catalog>().0.props[index].size_y / 2. + 0.1);
        actions.push(Action::Spawn(index, position));
    }
    let selected = target(world);
    if !pointer_on_ui {
        if mouse.just_pressed(MouseButton::Left) {
            if let Some(e) = selected {
                actions.push(Action::Grab(e));
            }
        }
        let focus = world.resource::<Session>().held.or(selected);
        if let Some(e) = focus {
            if mouse.just_pressed(MouseButton::Right) || keys.just_pressed(KeyCode::KeyF) {
                actions.push(Action::Freeze(e, true));
            }
            if keys.just_pressed(KeyCode::KeyR) {
                actions.push(Action::Freeze(e, false));
            }
            if keys.just_pressed(KeyCode::KeyX) {
                actions.push(Action::Remove(e));
            }
            if keys.just_pressed(KeyCode::KeyC) {
                actions.push(Action::Duplicate(e));
            }
        }
    }
    if mouse.just_released(MouseButton::Left) {
        actions.push(Action::Release);
    }
    if keys.just_pressed(KeyCode::KeyZ) {
        actions.push(Action::Undo);
    }
    if keys.just_pressed(KeyCode::F9) {
        actions.push(Action::Reset);
    }
    if keys.just_pressed(KeyCode::F5) {
        actions.push(Action::Save);
    }
    if keys.just_pressed(KeyCode::F6) {
        let last = world.resource::<Session>().last_save.clone().or_else(|| {
            let mut paths: Vec<_> = std::fs::read_dir(project_root().join("local/saves"))
                .ok()?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|e| e == "json"))
                .collect();
            paths.sort();
            paths.pop()
        });
        if let Some(path) = last {
            actions.push(Action::Load(path));
        } else {
            world.resource_mut::<Session>().status =
                "No saved scene found. F5 saves a new file.".into();
        }
    }
    if keys.pressed(KeyCode::KeyE) {
        if let Some(e) = world.resource::<Session>().held {
            let dt = world.resource::<Time>().delta_secs();
            if let Some(mut t) = world.get_mut::<Transform>(e) {
                t.rotate_y(dt);
            }
        }
    }
    for action in actions {
        if let Err(e) = apply_action(world, action) {
            world.resource_mut::<Session>().status = format!("Error: {e}");
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        world.send_event(bevy::app::AppExit::Success);
    }
}

fn draw_overlay(
    catalog: Res<Catalog>,
    session: Res<Session>,
    aim: Res<Aim>,
    props: Query<(&Prop, &Transform)>,
    mut hud: Query<&mut Text, With<Hud>>,
    mut gizmos: Gizmos,
) {
    let h = catalog.0.world.ground_half_size;
    for i in -25..=25 {
        let v = i as f32 * h / 25.;
        let color = Color::srgb(0.22, 0.29, 0.35);
        gizmos.line(Vec3::new(v, 0.01, -h), Vec3::new(v, 0.01, h), color);
        gizmos.line(Vec3::new(-h, 0.01, v), Vec3::new(h, 0.01, v), color);
    }
    if let Some(e) = session.held {
        if let Ok((_, t)) = props.get(e) {
            gizmos.line(
                aim.origin + aim.direction * 0.7 - Vec3::Y * 0.2,
                t.translation,
                Color::srgb(0.2, 0.85, 1.),
            );
        }
    }
    if let Ok(mut text) = hud.single_mut() {
        text.0 = format!(
            "{} props  |  Selected: {}  |  {} Hz  |  {} undo steps\n{}",
            props.iter().count(),
            catalog.0.props[session.selected].label,
            catalog.0.world.fixed_hz,
            session.undo.len(),
            session.status
        );
    }
}

fn smoke_frames(world: &mut World) {
    if !world.resource::<SmokeFrames>().enabled {
        return;
    }
    let frame = {
        let mut s = world.resource_mut::<SmokeFrames>();
        s.frame += 1;
        s.frame
    };
    // Exercise the same keyboard and UI bindings used interactively.
    for (at, key) in [
        (30, KeyCode::Enter),
        (60, KeyCode::KeyZ),
        (100, KeyCode::F5),
        (120, KeyCode::F9),
        (130, KeyCode::F6),
    ] {
        if frame == at {
            world.resource_mut::<ButtonInput<KeyCode>>().press(key);
        }
        if frame == at + 1 {
            world.resource_mut::<ButtonInput<KeyCode>>().release(key);
        }
    }
    if frame == 75 {
        for (button, mut interaction) in world
            .query::<(&SpawnButton, &mut Interaction)>()
            .iter_mut(world)
        {
            if button.0 == 1 {
                *interaction = Interaction::Pressed;
            }
        }
    }
    if frame == 76 {
        for mut interaction in world
            .query_filtered::<&mut Interaction, With<SpawnButton>>()
            .iter_mut(world)
        {
            *interaction = Interaction::None;
        }
    }
    for (at, expected) in [(32, 6), (62, 5), (78, 6), (122, 5), (132, 6)] {
        if frame == at {
            assert_eq!(
                world.query::<&Prop>().iter(world).count(),
                expected,
                "graphical input acceptance at frame {at}"
            );
        }
    }
    if frame == 102 {
        assert!(
            world
                .resource::<Session>()
                .last_save
                .as_ref()
                .is_some_and(|p| p.exists()),
            "F5 save did not create file"
        );
    }
    if frame == 150 {
        std::fs::create_dir_all(project_root().join("evidence")).expect("evidence directory");
        world
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(
                project_root().join("evidence/bevy-smoke.png"),
            ));
    }
    if frame == 240 {
        println!(
            "GRAPHICAL_SMOKE_COMPLETE: keyboard spawn/undo, menu spawn, F5 save, F9 reset, F6 load, 240 rendered frames"
        );
        world.send_event(bevy::app::AppExit::Success);
    }
}
