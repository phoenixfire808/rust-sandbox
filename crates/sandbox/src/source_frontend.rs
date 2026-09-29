//! Local startup/pause navigation and a private, persistent player feedback inbox.
use crate::{
    project_root,
    source_play::{PlayState, SpawnedProp},
    source_player::PlayerState,
};
use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use sandbox_catalog::Result;
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
};

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Hidden,
    Home,
    Maps,
    Options,
    Saves,
    Feedback,
    Confirm,
    Loading,
}
#[derive(Resource)]
pub struct Frontend {
    startup: bool,
    map: String,
    selected: String,
    page: Page,
    dirty: bool,
    fields: [String; 4],
    field: usize,
    category: usize,
    message: String,
    pending: String,
    child: Option<(Child, PathBuf)>,
    blocked_frame: bool,
}
impl Frontend {
    pub fn new(startup: bool, map: String) -> Self {
        let fields = std::fs::read(project_root().join("local/feedback-draft.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        Self {
            startup,
            selected: crate::compiled_source_maps()[0].id.clone(),
            map,
            page: if startup { Page::Home } else { Page::Hidden },
            dirty: true,
            fields,
            field: 0,
            category: 0,
            message: String::new(),
            pending: String::new(),
            child: None,
            blocked_frame: false,
        }
    }
}
#[derive(Component)]
struct Root;
#[derive(Component)]
struct Badge;
#[derive(Component, Clone)]
enum Action {
    Page(Page),
    Map(String),
    Field(usize),
    Category,
    Submit,
    Start,
    Leave(String),
    Confirm(bool),
    Save,
    Load,
    Setting(&'static str, f32),
}
pub struct FrontendPlugin;
impl Plugin for FrontendPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (typing, update).chain().before(crate::source_player::input),
        )
        .add_systems(PostUpdate, ready)
        .add_systems(Update, scroll.after(update));
    }
}
pub fn active(world: &World) -> bool {
    world
        .get_resource::<Frontend>()
        .is_some_and(|f| f.page != Page::Hidden || f.blocked_frame)
}
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Workshop | Main Menu".into(),
                resolution: (1280f32, 800f32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Frontend::new(true, String::new()))
        .insert_resource(ClearColor(Color::srgb(0.06, 0.12, 0.18)))
        .add_systems(Startup, |mut c: Commands| {
            c.spawn((Camera2d, IsDefaultUiCamera));
        })
        .add_plugins(FrontendPlugin)
        .run();
}
fn draft(f: &Frontend) -> Result<()> {
    std::fs::create_dir_all(project_root().join("local"))?;
    std::fs::write(
        project_root().join("local/feedback-draft.json"),
        serde_json::to_vec_pretty(&f.fields)?,
    )?;
    Ok(())
}
fn typing(mut events: EventReader<KeyboardInput>, mut f: ResMut<Frontend>) {
    if f.page != Page::Feedback {
        events.clear();
        return;
    }
    let limit = crate::compiled_frontend_config().feedback_limit;
    for e in events.read().filter(|e| e.state == ButtonState::Pressed) {
        let i = f.field;
        match &e.logical_key {
            Key::Character(s) if f.fields[i].len() + s.len() <= limit => f.fields[i].push_str(s),
            Key::Backspace => {
                f.fields[i].pop();
            }
            Key::Enter if f.fields[i].len() < limit => f.fields[i].push('\n'),
            Key::Tab => {
                f.field = (i + 1) % 4;
            }
            _ => continue,
        }
        f.dirty = true;
    }
    if f.dirty {
        if let Err(e) = draft(&f) {
            f.message = format!("Draft save failed: {e}");
        }
    }
}
fn submit(world: &mut World, f: &mut Frontend) -> Result<()> {
    if f.fields[0].trim().is_empty() || f.fields[1..].iter().all(|s| s.trim().is_empty()) {
        return Err("Add a title and details before saving feedback".into());
    }
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let position = world
        .get_resource::<PlayerState>()
        .map(|p| p.eye.translation.to_array());
    let tool = world.get_resource::<PlayState>().map(|p| p.tool.clone());
    let status = world.get_resource::<PlayState>().map(|p| p.status.clone());
    let props = world.query::<&SpawnedProp>().iter(world).count();
    let note = serde_json::json!({"version":1,"created_unix_nanos":time.to_string(),"category":(["Bug","Feature request","Visual mismatch","Performance"][f.category]),"title":f.fields[0],"observed":f.fields[1],"expected":f.fields[2],"steps_and_notes":f.fields[3],"map":f.map,"eye_position":position,"selected_tool":tool,"prop_count":props,"game_status":status,"review_status":"new"});
    let dir = project_root().join("local/feedback");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("feedback-{time}.json"));
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    file.write_all(&serde_json::to_vec_pretty(&note)?)?;
    file.sync_all()?;
    f.message = format!("Saved locally for Jcode review: {}", path.display());
    f.fields = Default::default();
    draft(f)?;
    Ok(())
}
fn launch(f: &mut Frontend, map: Option<&str>) -> Result<()> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let local = project_root().join("local");
    std::fs::create_dir_all(&local)?;
    let ready = local.join(format!("session-{stamp}.ready"));
    let log = std::fs::File::create(local.join(format!("session-{stamp}.log")))?;
    let err = std::fs::File::create(local.join(format!("session-{stamp}.err.log")))?;
    let mut cmd = Command::new(std::env::current_exe()?);
    cmd.current_dir(project_root())
        .env("RUST_SANDBOX_READY", &ready)
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(err));
    if let Some(map) = map {
        cmd.args(["--play", map]);
    }
    f.child = Some((cmd.spawn()?, ready));
    f.page = Page::Loading;
    f.message="Loading original installed content. The current window stays open until the new window is ready.".into();
    Ok(())
}
fn leave(world: &mut World, f: &mut Frontend) -> Result<()> {
    match f.pending.as_str() {
        "quit" => {
            world.send_event(bevy::app::AppExit::Success);
        }
        "disconnect" => launch(f, None)?,
        _ => {
            let map = f.selected.clone();
            launch(f, Some(&map))?;
        }
    }
    Ok(())
}
fn update(world: &mut World) {
    world.resource_scope(|world,mut f:Mut<Frontend>| {
        let previously_blocked = f.blocked_frame;
        f.blocked_frame=false;
        if let Some((child,ready))=f.child.as_mut() {
            match child.try_wait() {
                Ok(Some(code)) => {f.message=format!("New session exited ({code}). See local/session-*.err.log. Your current scene is retained.");f.child=None; f.page=Page::Home;f.dirty=true;},
                Err(e)=>{f.message=e.to_string();f.dirty=true;},
                Ok(None) if ready.exists()=>{ world.send_event(bevy::app::AppExit::Success); },
                _=>{}
            }
        }
        let previous_page=f.page;
        let keys=world.resource::<ButtonInput<KeyCode>>();
        if f.page!=Page::Loading {
            if keys.just_pressed(KeyCode::F10) {f.pending="quit".into();f.page=Page::Confirm;f.dirty=true;}
            else if keys.just_pressed(KeyCode::F8) {f.page=Page::Feedback;f.dirty=true;}
            else if keys.just_pressed(KeyCode::Escape) {
                f.page=match f.page {Page::Hidden=>Page::Home,Page::Home if !f.startup=>Page::Hidden,_=>Page::Home};
                f.blocked_frame=true; f.dirty=true;
            }
        }
        let actions:Vec<_>=if f.page != Page::Loading && world.resource::<ButtonInput<MouseButton>>().just_pressed(MouseButton::Left) {
            world.query::<(&Interaction,&Action)>().iter(world).filter(|(i,_)|**i==Interaction::Pressed).map(|(_,a)|a.clone()).collect()
        } else {Vec::new()};
        for action in actions {
            f.dirty=true;f.blocked_frame=true;
            let result:Result<()>=match action {
                Action::Page(p)=>{f.page=p;Ok(())}, Action::Map(map)=>{f.selected=map;Ok(())},
                Action::Field(i)=>{f.field=i;Ok(())}, Action::Category=>{f.category=(f.category+1)%4;Ok(())},
                Action::Submit=>submit(world,&mut f),
                Action::Start=>{if f.startup {let map=f.selected.clone();launch(&mut f,Some(&map))} else {f.pending="map".into();f.page=Page::Confirm;Ok(())}},
                Action::Leave(s)=>{f.pending=s;f.page=Page::Confirm;Ok(())},
                Action::Confirm(save)=>{if save && !crate::source_play::menu_save(world) {Err("Save failed. Scene retained. See game status.".into())} else {leave(world,&mut f)}},
                Action::Save=>{crate::source_play::menu_save(world);f.message=world.resource::<PlayState>().status.clone();Ok(())},
                Action::Load=>{crate::source_play::menu_load(world);f.message=world.resource::<PlayState>().status.clone();Ok(())},
                Action::Setting(key,delta)=>{crate::source_play::menu_setting(world,key,delta);Ok(())},
            };
            if let Err(e)=result {f.message=e.to_string();}
        }
        if f.page!=previous_page {for mut s in world.query_filtered::<&mut ScrollPosition,With<FrontScroll>>().iter_mut(world) {s.offset_y=0.;}}
        let open=f.page!=Page::Hidden || f.blocked_frame;
        if let Some(mut play)=world.get_resource_mut::<PlayState>() {
            if open {play.menu_open=true;play.search_focus=false;play.held=None;play.beam_active=false;}
            else if f.dirty || previously_blocked {play.menu_open=false;play.dirty=true;}
        }
        if !f.startup {
            let mut time=world.resource_mut::<Time<Virtual>>();
            if open {time.pause();} else {time.unpause();}
        }
        if open {for mut w in world.query_filtered::<&mut Window,With<PrimaryWindow>>().iter_mut(world) {w.cursor_options.grab_mode=CursorGrabMode::None;w.cursor_options.visible=true;}}
        if !world.query_filtered::<Entity,With<Badge>>().iter(world).any(|_|true) && !f.startup {
            let e=button(world,None,"Feedback (F8)",Action::Page(Page::Feedback));
            world.entity_mut(e).insert((Badge,GlobalZIndex(90),Node {position_type:PositionType::Absolute,right:Val::Px(16.),top:Val::Px(12.),padding:UiRect::all(Val::Px(8.)),..default()}));
        }
        if f.dirty {draw(world,&f);f.dirty=false;}
    });
}
fn label(world: &mut World, parent: Entity, value: impl Into<String>, size: f32) {
    world.spawn((
        Text::new(value),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(Color::WHITE),
        ChildOf(parent),
    ));
}
fn button(
    world: &mut World,
    parent: Option<Entity>,
    value: impl Into<String>,
    action: Action,
) -> Entity {
    let c = crate::compiled_frontend_config();
    let e = world
        .spawn((
            Button,
            action,
            Node {
                min_height: Val::Px(c.row_height),
                padding: UiRect::all(Val::Px(8.)),
                margin: UiRect::bottom(Val::Px(4.)),
                flex_shrink: 0.,
                ..default()
            },
            BackgroundColor(Color::srgb(0.16, 0.23, 0.3)),
        ))
        .id();
    if let Some(p) = parent {
        world.entity_mut(e).insert(ChildOf(p));
    }
    label(world, e, value, c.font_size);
    e
}
#[derive(Component)]
struct FrontScroll;
fn ready(mut frame: Local<u8>) {
    if *frame >= 3 {
        return;
    }
    *frame += 1;
    if *frame == 3 {
        if let Some(path) = std::env::var_os("RUST_SANDBOX_READY").map(PathBuf::from) {
            if path.parent() == Some(project_root().join("local").as_path()) {
                if let Err(e) = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)
                {
                    eprintln!("READY_FILE_ERROR {e}");
                }
            }
        }
    }
}
fn scroll(
    mut wheel: EventReader<bevy::input::mouse::MouseWheel>,
    f: Res<Frontend>,
    mut panels: Query<(&mut ScrollPosition, &ComputedNode), With<FrontScroll>>,
) {
    let delta: f32 = wheel
        .read()
        .map(|e| {
            e.y * if e.unit == bevy::input::mouse::MouseScrollUnit::Line {
                32.
            } else {
                1.
            }
        })
        .sum();
    if f.page != Page::Hidden {
        for (mut position, node) in &mut panels {
            let max = ((node.content_size.y - node.size.y) * node.inverse_scale_factor).max(0.);
            position.offset_y = (position.offset_y - delta).clamp(0., max);
        }
    }
}
fn draw(world: &mut World, f: &Frontend) {
    let offset = world
        .query_filtered::<&ScrollPosition, With<FrontScroll>>()
        .iter(world)
        .next()
        .map(|s| s.offset_y)
        .unwrap_or(0.);
    let roots: Vec<_> = world
        .query_filtered::<Entity, With<Root>>()
        .iter(world)
        .collect();
    for e in roots {
        world.despawn(e);
    }
    if f.page == Page::Hidden {
        return;
    }
    let c = crate::compiled_frontend_config();
    let root = world
        .spawn((
            Root,
            GlobalZIndex(100),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                padding: UiRect::all(Val::Px(c.margin)),
                column_gap: Val::Px(c.margin),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.06, 0.1, 0.97)),
        ))
        .id();
    let nav = world
        .spawn((
            Node {
                width: Val::Px(c.sidebar_width),
                flex_shrink: 0.,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(root),
        ))
        .id();
    label(world, nav, "Rust Workshop", 28.);
    if !f.startup {
        button(world, Some(nav), "Resume Game", Action::Page(Page::Hidden));
    }
    button(world, Some(nav), "Start New Game", Action::Page(Page::Maps));
    button(world, Some(nav), "Options", Action::Page(Page::Options));
    if !f.startup {
        button(world, Some(nav), "Saves", Action::Page(Page::Saves));
    }
    button(
        world,
        Some(nav),
        "Detailed Feedback",
        Action::Page(Page::Feedback),
    );
    if !f.startup {
        button(
            world,
            Some(nav),
            "Disconnect",
            Action::Leave("disconnect".into()),
        );
    }
    button(world, Some(nav), "Quit", Action::Leave("quit".into()));
    label(
        world,
        nav,
        "Multiplayer / Addons / Demos / Dupes / mounted games: pending",
        13.,
    );
    let panel = world
        .spawn((
            Node {
                flex_grow: 1.,
                min_width: Val::Px(0.),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            FrontScroll,
            ScrollPosition {
                offset_y: offset,
                ..default()
            },
            ChildOf(root),
        ))
        .id();
    match f.page {
        Page::Home => {
            label(
                world,
                panel,
                if f.startup {
                    "Start New Game"
                } else {
                    "Game paused"
                },
                32.,
            );
            label(world,panel,"Independent local sandbox. Stock menu appearance and all options are still being implemented.",c.font_size);
        }
        Page::Maps => {
            label(world, panel, "Sandbox", 32.);
            let content = world
                .spawn((
                    Node {
                        flex_grow: 1.,
                        column_gap: Val::Px(c.margin),
                        ..default()
                    },
                    ChildOf(panel),
                ))
                .id();
            let row = world
                .spawn((
                    Node {
                        flex_grow: 1.,
                        min_width: Val::Px(0.),
                        align_content: AlignContent::Start,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(8.),
                        row_gap: Val::Px(8.),
                        ..default()
                    },
                    ChildOf(content),
                ))
                .id();
            for map in crate::compiled_source_maps() {
                let e = button(
                    world,
                    Some(row),
                    format!(
                        "{}{}",
                        if f.selected == map.id {
                            "Selected: "
                        } else {
                            ""
                        },
                        map.id
                    ),
                    Action::Map(map.id),
                );
                world.entity_mut(e).insert(Node {
                    width: Val::Px(c.map_icon + 12.),
                    height: Val::Px(c.map_icon),
                    padding: UiRect::all(Val::Px(6.)),
                    ..default()
                });
            }
            let settings = world
                .spawn((
                    Node {
                        width: Val::Px(c.settings_width),
                        flex_shrink: 0.,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.),
                        ..default()
                    },
                    ChildOf(content),
                ))
                .id();
            label(world,settings,"Single Player | Sandbox\nOnly authored supported maps are enabled. Multiplayer and other game modes are not implemented.",c.font_size);
            button(
                world,
                Some(settings),
                format!("Start Game: {}", f.selected),
                Action::Start,
            );
        }
        Page::Feedback => {
            label(world, panel, "Detailed feedback for Jcode", 28.);
            label(world,panel,"Saved on this computer only. Nothing is uploaded. Jcode can review local/feedback/*.json on your next request. Tab changes fields; Enter adds a line.",14.);
            button(
                world,
                Some(panel),
                format!(
                    "Category: {}",
                    ["Bug", "Feature request", "Visual mismatch", "Performance"][f.category]
                ),
                Action::Category,
            );
            for (i, title) in [
                "Title",
                "What happened / what is missing",
                "What you expected",
                "Steps to reproduce / additional notes",
            ]
            .iter()
            .enumerate()
            {
                button(
                    world,
                    Some(panel),
                    format!(
                        "{}{}\n{}",
                        if f.field == i { "> " } else { "" },
                        title,
                        if f.fields[i].is_empty() {
                            "Click here to type"
                        } else {
                            &f.fields[i]
                        }
                    ),
                    Action::Field(i),
                );
            }
            button(world, Some(panel), "Save feedback locally", Action::Submit);
        }
        Page::Options => {
            label(world, panel, "Options", 32.);
            if let Some(p) = world.get_resource::<PlayState>() {
                let values = [
                    ("Field of view", "fov", p.fov, 5.),
                    ("Noclip speed", "speed", p.speed, 1.),
                    ("Mouse sensitivity", "sensitivity", p.sensitivity, 0.0005),
                ];
                for (name, key, value, step) in values {
                    label(world, panel, format!("{name}: {value:.4}"), c.font_size);
                    button(world, Some(panel), "-", Action::Setting(key, -step));
                    button(world, Some(panel), "+", Action::Setting(key, step));
                }
            } else {
                label(world,panel,"Start a map to change current player options. Full startup options and key binding pages are pending.",c.font_size);
            }
        }
        Page::Saves => {
            label(world, panel, "Local scenes", 32.);
            button(world, Some(panel), "Save current scene", Action::Save);
            button(
                world,
                Some(panel),
                "Load latest local scene (Z can undo)",
                Action::Load,
            );
            label(
                world,
                panel,
                "These are local Rust scenes, not GMod save files or Workshop saves.",
                14.,
            );
        }
        Page::Confirm => {
            label(world, panel, "Leave current session?", 32.);
            label(world,panel,"Unsaved scene changes will be lost if you continue without saving. Your feedback draft is retained.",c.font_size);
            if !f.startup {
                button(
                    world,
                    Some(panel),
                    "Save scene and continue",
                    Action::Confirm(true),
                );
            }
            button(
                world,
                Some(panel),
                "Continue without saving",
                Action::Confirm(false),
            );
            button(world, Some(panel), "Cancel", Action::Page(Page::Home));
        }
        Page::Loading => {
            label(world, panel, "Loading...", 32.);
        }
        Page::Hidden => {}
    }
    if !f.message.is_empty() {
        label(world, panel, &f.message, 14.);
    }
}
