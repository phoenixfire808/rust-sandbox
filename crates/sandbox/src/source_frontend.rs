//! Local startup/pause navigation and a private, persistent player feedback inbox.
pub(crate) mod feedback;
mod view;
use crate::{
    project_root,
    source_play::{PlayState, SpawnedProp},
    source_player::PlayerState,
};
use bevy::ui::FocusPolicy;
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum Page {
    Hidden,
    Home,
    Maps,
    Options,
    Saves,
    Feedback,
    Confirm,
    Loading,
    Info,
}
#[derive(Debug, Clone, Copy, PartialEq)]
enum Popup {
    Games,
    Language,
    Gamemodes,
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
    map_search: String,
    search_focus: bool,
    favorites_only: bool,
    favorites: std::collections::BTreeSet<String>,
    last_map_click: Option<(String, std::time::Instant)>,
    window_size: Vec2,
    info: String,
    popup: Option<Popup>,
    feedback_context: Option<serde_json::Value>,
    feedback_candidate: Option<serde_json::Value>,
    feedback_return: Page,
    feedback_return_menu: bool,
    resume_menu: Option<bool>,
}
impl Frontend {
    pub fn new(startup: bool, map: String) -> Self {
        let saved_draft = feedback::load();
        let prefs: (String, std::collections::BTreeSet<String>) =
            std::fs::read(project_root().join("local/menu-preferences.json"))
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or_else(|| ("gm_flatgrass".into(), Default::default()));
        let selected = crate::compiled_source_maps()
            .into_iter()
            .find(|m| m.id == prefs.0)
            .unwrap_or_else(|| crate::compiled_source_maps()[0].clone())
            .id;
        Self {
            startup,
            selected,
            map,
            page: if startup { Page::Home } else { Page::Hidden },
            dirty: true,
            fields: saved_draft.fields,
            field: 0,
            category: saved_draft.category,
            message: String::new(),
            pending: String::new(),
            child: None,
            blocked_frame: false,
            map_search: String::new(),
            search_focus: false,
            favorites_only: false,
            favorites: prefs.1,
            last_map_click: None,
            window_size: Vec2::ZERO,
            info: String::new(),
            popup: None,
            feedback_context: saved_draft.context,
            feedback_candidate: None,
            feedback_return: if startup { Page::Home } else { Page::Hidden },
            feedback_return_menu: false,
            resume_menu: None,
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
    Info(String),
    Favorite(String),
    MapCategory(bool),
    Search,
    Popup(Option<Popup>),
    BackFeedback,
    RetargetFeedback,
}
pub struct FrontendPlugin;
impl Plugin for FrontendPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Actions>()
            .init_resource::<feedback::Request>()
            .add_systems(Startup, view::load_assets)
            .add_systems(
                Update,
                (
                    typing,
                    collect_actions,
                    update,
                    view::hover,
                    view::favorite_hover,
                )
                    .chain()
                    .before(crate::source_player::input),
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
    feedback::persist(f)
}
pub(crate) fn request_feedback(world: &mut World, subject: String) {
    if let Some(mut request) = world.get_resource_mut::<feedback::Request>() {
        request.0 = Some(subject);
    }
}
fn typing(mut events: EventReader<KeyboardInput>, mut f: ResMut<Frontend>) {
    if f.page == Page::Maps && f.search_focus {
        for e in events.read().filter(|e| e.state == ButtonState::Pressed) {
            match &e.logical_key {
                Key::Character(s) if f.map_search.len() + s.len() <= 256 => {
                    f.map_search.push_str(s)
                }
                Key::Backspace => {
                    f.map_search.pop();
                }
                Key::Enter | Key::Tab => f.search_focus = false,
                _ => continue,
            }
            f.dirty = true;
        }
        return;
    }
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
    let note = serde_json::json!({"version":2,"created_unix_nanos":time.to_string(),"category":(["Bug","Feature request","Visual mismatch","Performance"][f.category]),"title":f.fields[0],"observed":f.fields[1],"expected":f.fields[2],"steps_and_notes":f.fields[3],"map":f.map,"eye_position":position,"selected_tool":tool,"prop_count":props,"game_status":status,"context_at_open":f.feedback_context,"review_status":"new"});
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
#[derive(Resource, Default)]
struct Actions(Vec<Action>);
fn collect_actions(
    buttons: Query<(&Interaction, &Action), Changed<Interaction>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut queue: ResMut<Actions>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    for (interaction, action) in &buttons {
        if *interaction == Interaction::Pressed {
            queue.0.push(action.clone());
        }
    }
}
fn save_preferences(f: &Frontend) -> Result<()> {
    std::fs::create_dir_all(project_root().join("local"))?;
    std::fs::write(
        project_root().join("local/menu-preferences.json"),
        serde_json::to_vec_pretty(&(&f.selected, &f.favorites))?,
    )?;
    Ok(())
}
fn start_map(f: &mut Frontend) -> Result<()> {
    save_preferences(f)?;
    if f.startup {
        let map = f.selected.clone();
        launch(f, Some(&map))
    } else {
        f.pending = "map".into();
        f.page = Page::Confirm;
        Ok(())
    }
}
fn update(world: &mut World) {
    world.resource_scope(|world,mut f:Mut<Frontend>| {
        let size=world.query_filtered::<&Window,With<PrimaryWindow>>().iter(world).next().map(|w|Vec2::new(w.width(),w.height())).unwrap_or(Vec2::new(1280.,800.));
        if size!=f.window_size {f.window_size=size;f.dirty=true;}
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
        let keys=world.resource::<ButtonInput<KeyCode>>().clone();
        let request=world.resource_mut::<feedback::Request>().0.take();
        if f.page!=Page::Loading {
            if keys.just_pressed(KeyCode::F10) {f.pending="quit".into();f.page=Page::Confirm;f.dirty=true;}
            else if request.is_some() {feedback::open(world,&mut f,request);}
            else if keys.just_pressed(KeyCode::F8) {if f.page==Page::Feedback {feedback::close(&mut f);} else {feedback::open(world,&mut f,None);}}
            else if keys.just_pressed(KeyCode::Escape) {
                if f.page==Page::Feedback {feedback::close(&mut f);}
                else if f.popup.take().is_none() {
                    f.page=match f.page {Page::Hidden=>Page::Home,Page::Home if !f.startup=>Page::Hidden,_=>Page::Home};
                }
                f.blocked_frame=true; f.dirty=true;
            }
        }
        let mut actions=std::mem::take(&mut world.resource_mut::<Actions>().0);
        if f.page==Page::Loading || f.page!=previous_page {actions.clear();}
        for action in actions {
            f.dirty=true;f.blocked_frame=true;
            if !matches!(&action, Action::Map(_)) { f.last_map_click=None; }
            let result:Result<()>=match action {
                Action::Page(Page::Feedback)=>{feedback::open(world,&mut f,None);Ok(())},
                Action::BackFeedback=>{feedback::close(&mut f);Ok(())},
                Action::RetargetFeedback=>{f.feedback_context=f.feedback_candidate.clone();draft(&f)},
                Action::Page(p)=>{f.page=p;f.search_focus=p==Page::Maps;f.message.clear();Ok(())},
                Action::Map(map)=>{let double=f.last_map_click.as_ref().is_some_and(|(previous,time)|previous==&map && time.elapsed().as_secs_f32()<0.35);f.selected=map.clone();f.search_focus=false;f.last_map_click=Some((map,std::time::Instant::now()));if double {start_map(&mut f)} else {save_preferences(&f)}},
                Action::Favorite(map)=>{if !f.favorites.remove(&map) {f.favorites.insert(map);}save_preferences(&f)},
                Action::MapCategory(only)=>{f.favorites_only=only;f.search_focus=false;Ok(())},
                Action::Search=>{f.search_focus=true;Ok(())},
                Action::Popup(p)=>{f.popup=if f.popup==p {None} else {p};f.search_focus=false;Ok(())},
                Action::Info(text)=>{f.info=text;f.page=Page::Info;f.search_focus=false;Ok(())},
                Action::Field(i)=>{f.field=i;Ok(())}, Action::Category=>{f.category=(f.category+1)%4;draft(&f)},
                Action::Submit=>submit(world,&mut f),
                Action::Start=>start_map(&mut f),
                Action::Leave(s)=>{f.pending=s;f.page=Page::Confirm;Ok(())},
                Action::Confirm(save)=>{if save && !crate::source_play::menu_save(world) {Err("Save failed. Scene retained. See game status.".into())} else {leave(world,&mut f)}},
                Action::Save=>{crate::source_play::menu_save(world);f.message=world.resource::<PlayState>().status.clone();Ok(())},
                Action::Load=>{crate::source_play::menu_load(world);f.message=world.resource::<PlayState>().status.clone();Ok(())},
                Action::Setting(key,delta)=>{crate::source_play::menu_setting(world,key,delta);Ok(())},
            };
            if let Err(e)=result {f.message=e.to_string();}
        }
        if f.page!=previous_page {f.popup=None;f.last_map_click=None;for mut s in world.query_filtered::<&mut ScrollPosition,With<FrontScroll>>().iter_mut(world) {s.offset_y=0.;}}
        let open=f.page!=Page::Hidden || f.blocked_frame;
        if let Some(mut play)=world.get_resource_mut::<PlayState>() {
            if open {play.menu_open=true;play.search_focus=false;play.held=None;play.beam_active=false;}
            else if f.dirty || previously_blocked {play.menu_open=f.resume_menu.take().unwrap_or(false);play.dirty=true;}
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
        FocusPolicy::Pass,
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
    mut panels: Query<
        (
            &mut ScrollPosition,
            &ComputedNode,
            &bevy::ui::RelativeCursorPosition,
        ),
        With<FrontScroll>,
    >,
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
        for (mut position, node, cursor) in &mut panels {
            if !cursor.mouse_over() {
                continue;
            }
            let max = ((node.content_size.y - node.size.y) * node.inverse_scale_factor).max(0.);
            position.offset_y = (position.offset_y - delta).clamp(0., max);
        }
    }
}
fn draw(world: &mut World, f: &Frontend) {
    view::draw(world, f);
}
