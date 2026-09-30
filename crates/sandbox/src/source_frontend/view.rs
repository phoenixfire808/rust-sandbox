//! Source-measured menu geometry. Mounted artwork stays in the user's installation.
use super::*;
use bevy::{
    asset::RenderAssetUsages,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    ui::RelativeCursorPosition,
};
use std::collections::BTreeMap;

#[derive(Resource, Default)]
pub(super) struct Artwork {
    images: BTreeMap<String, Handle<Image>>,
    regular: Handle<Font>,
    bold: Handle<Font>,
}
#[derive(Component)]
pub(super) struct Palette {
    normal: Color,
    hovered: Color,
    text: Color,
    hover_text: Color,
}
#[derive(Component)]
pub(super) struct ButtonLabel;
#[derive(Component)]
pub(super) struct FavoriteVisual {
    card: Entity,
    button: Entity,
    selected: bool,
}
pub(super) fn favorite_hover(
    art: Res<Artwork>,
    cards: Query<&RelativeCursorPosition>,
    buttons: Query<&Interaction>,
    mut icons: Query<(&FavoriteVisual, &mut ImageNode)>,
) {
    for (state, mut image) in &mut icons {
        let hovering = buttons
            .get(state.button)
            .is_ok_and(|v| *v != Interaction::None);
        let visible = state.selected || cards.get(state.card).is_ok_and(|v| v.mouse_over());
        image.color = Color::srgba(1., 1., 1., if visible { 1. } else { 0. });
        let name = if hovering {
            if state.selected {
                "html/img/favourite_remove.png"
            } else {
                "html/img/favourite_add.png"
            }
        } else {
            "html/img/favourite.png"
        };
        if let Some(handle) = art.images.get(name) {
            image.image = handle.clone();
        }
    }
}
fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::srgb_u8(r, g, b)
}
fn gray(v: u8) -> Color {
    rgb(v, v, v)
}

pub(super) fn load_assets(world: &mut World) {
    let root = std::env::var_os("GMOD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"D:\SteamLibrary\steamapps\common\GarrysMod"));
    let mounts = if world.contains_resource::<crate::source_assets::MountedSource>() {
        None
    } else {
        crate::source_assets::Mounts::open(&root).ok()
    };
    let c = crate::compiled_frontend_config();
    let mut names = vec![
        c.background,
        "gamemodes/sandbox/logo.png".into(),
        "gamemodes/sandbox/icon24.png".into(),
        "resource/localization/en.png".into(),
    ];
    for name in [
        "back_to_main_menu",
        "back_to_game",
        "games",
        "error",
        "favourite",
        "favourite_add",
        "favourite_remove",
    ] {
        names.push(format!("html/img/{name}.png"));
    }
    for map in crate::compiled_source_maps() {
        names.push(format!("maps/thumb/{}.png", map.id));
    }
    let mut art = Artwork::default();
    for name in names {
        let bytes =
            if let Some(source) = world.get_resource::<crate::source_assets::MountedSource>() {
                source.mounts.read_file(&name).ok()
            } else {
                mounts.as_ref().and_then(|m| m.read_file(&name).ok())
            };
        if let Some(decoded) = bytes.and_then(|b| image::load_from_memory(&b).ok()) {
            let decoded = decoded.to_rgba8();
            let image = Image::new(
                Extent3d {
                    width: decoded.width(),
                    height: decoded.height(),
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                decoded.into_raw(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            );
            art.images
                .insert(name, world.resource_mut::<Assets<Image>>().add(image));
        } else {
            eprintln!("MENU_ASSET_MISSING {name}");
        }
    }
    let windows = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    for (name, bold) in [("arial.ttf", false), ("arialbd.ttf", true)] {
        if let Some(font) = std::fs::read(windows.join("Fonts").join(name))
            .ok()
            .and_then(|b| Font::try_from_bytes(b).ok())
        {
            let handle = world.resource_mut::<Assets<Font>>().add(font);
            if bold {
                art.bold = handle;
            } else {
                art.regular = handle;
            }
        }
    }
    println!(
        "MENU_ASSETS images={} original files read without modification",
        art.images.len()
    );
    world.insert_resource(art);
}
fn node(world: &mut World, parent: Entity, n: Node, color: Color) -> Entity {
    world
        .spawn((n, BackgroundColor(color), ChildOf(parent)))
        .id()
}
fn text(
    world: &mut World,
    parent: Entity,
    value: impl Into<String>,
    size: f32,
    color: Color,
    bold: bool,
) -> Entity {
    let art = world.resource::<Artwork>();
    let font = if bold {
        art.bold.clone()
    } else {
        art.regular.clone()
    };
    world
        .spawn((
            Text::new(value),
            TextFont {
                font,
                font_size: size,
                ..default()
            },
            TextColor(color),
            FocusPolicy::Pass,
            ChildOf(parent),
        ))
        .id()
}
fn picture(
    world: &mut World,
    parent: Entity,
    name: &str,
    width: f32,
    height: f32,
) -> Option<Entity> {
    let handle = world.resource::<Artwork>().images.get(name)?.clone();
    Some(
        world
            .spawn((
                ImageNode::new(handle),
                Node {
                    width: Val::Px(width),
                    height: Val::Px(height),
                    flex_shrink: 0.,
                    ..default()
                },
                FocusPolicy::Pass,
                ChildOf(parent),
            ))
            .id(),
    )
}
fn intrinsic_size(world: &World, name: &str) -> Vec2 {
    world
        .resource::<Artwork>()
        .images
        .get(name)
        .and_then(|h| world.resource::<Assets<Image>>().get(h))
        .map(|i| i.size().as_vec2())
        .unwrap_or(Vec2::splat(16.))
}
fn btn(
    world: &mut World,
    parent: Entity,
    value: impl Into<String>,
    action: Action,
    size: f32,
    background: Color,
    foreground: Color,
) -> Entity {
    let e = world
        .spawn((
            Button,
            action,
            FocusPolicy::Block,
            Node {
                min_height: Val::Px(28.),
                padding: UiRect::axes(Val::Px(8.), Val::Px(4.)),
                align_items: AlignItems::Center,
                flex_shrink: 0.,
                ..default()
            },
            BackgroundColor(background),
            BorderRadius::all(Val::Px(3.)),
            Palette {
                normal: background,
                hovered: rgb(204, 221, 238),
                text: foreground,
                hover_text: foreground,
            },
            ChildOf(parent),
        ))
        .id();
    let value = value.into();
    if !value.is_empty() {
        feedback::tag(world, e, value.clone(), 100);
    }
    let label = text(world, e, value, size, foreground, false);
    world.entity_mut(label).insert(ButtonLabel);
    e
}
fn standard(world: &mut World, parent: Entity, value: impl Into<String>, action: Action) -> Entity {
    btn(world, parent, value, action, 14., gray(225), gray(45))
}
pub(super) fn hover(
    mut buttons: Query<
        (&Interaction, &Palette, &Children, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut labels: Query<&mut TextColor, With<ButtonLabel>>,
) {
    for (interaction, palette, children, mut bg) in &mut buttons {
        let over = *interaction != Interaction::None;
        bg.0 = if over {
            palette.hovered
        } else {
            palette.normal
        };
        for child in children.iter() {
            if let Ok(mut color) = labels.get_mut(child) {
                color.0 = if over {
                    palette.hover_text
                } else {
                    palette.text
                };
            }
        }
    }
}
fn route(action: &str) -> Action {
    match action {
        "resume"=>Action::Page(Page::Hidden),"maps"=>Action::Page(Page::Maps),"options"=>Action::Page(Page::Options),"saves"=>Action::Page(Page::Saves),
        "disconnect"|"quit"=>Action::Leave(action.into()),
        "servers"=>Action::Info("Find Multiplayer Game\n\nThe multiplayer browser and networking runtime are not implemented. No server connection is attempted. Use Start New Game for local Sandbox.".into()),
        "addons"=>Action::Info("Addons\n\nWorkshop subscriptions and addon execution are not implemented. Installed stock content is mounted read-only. No Steam files or subscriptions are changed.".into()),
        "dupes"=>Action::Info("Dupes\n\nThe toolgun supports local connected-prop duplication. The native dupe file and Workshop browser are not implemented.".into()),
        "demos"=>Action::Info("Demos\n\nRecording and native demo playback are not implemented. This entry is retained at its reference location rather than silently omitted.".into()),
        _=>Action::Page(Page::Home)
    }
}
fn home(world: &mut World, page: Entity, f: &Frontend) {
    let c = crate::compiled_frontend_config();
    let short = f.window_size.y <= c.short_height;
    let tiny = f.window_size.y <= c.tiny_height;
    let narrow = f.window_size.x <= c.narrow_width;
    let font = if short {
        c.short_font
    } else if f.window_size.x <= c.compact_width {
        c.compact_font
    } else {
        c.main_font
    };
    let line = if short { c.short_line } else { font * 1.15 };
    let x = if narrow { 0. } else { c.main_padding }
        + if tiny { 0. } else { c.main_list_margin }
        + c.main_item_padding;
    let y = if short || narrow { 0. } else { c.main_padding }
        + if tiny { 0. } else { font }
        + if short { 0. } else { 2. };
    let column = node(
        world,
        page,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(x),
            top: Val::Px(y),
            bottom: Val::Px(0.),
            width: Val::Px(c.logo_width + 32.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            overflow: Overflow::clip(),
            ..default()
        },
        Color::NONE,
    );
    for entry in crate::compiled_main_menu() {
        if entry.visibility == "game" && f.startup {
            continue;
        }
        match entry.kind.as_str() {
            "logo" if !tiny => {
                picture(
                    world,
                    column,
                    "gamemodes/sandbox/logo.png",
                    c.logo_width,
                    c.logo_height,
                );
                node(
                    world,
                    column,
                    Node {
                        height: Val::Px(font * 0.2 + 1.),
                        flex_shrink: 0.,
                        ..default()
                    },
                    Color::NONE,
                );
            }
            "gap" => {
                node(
                    world,
                    column,
                    Node {
                        height: Val::Px(line + if short { 0. } else { 1. }),
                        flex_shrink: 0.,
                        ..default()
                    },
                    Color::NONE,
                );
            }
            "link" => {
                let e = btn(
                    world,
                    column,
                    &entry.label,
                    route(&entry.action),
                    font,
                    Color::NONE,
                    Color::WHITE,
                );
                world.entity_mut(e).insert((
                    Node {
                        height: Val::Px(line),
                        min_height: Val::Px(line),
                        margin: UiRect::bottom(Val::Px(if short { 0. } else { 1. })),
                        align_items: AlignItems::Center,
                        flex_shrink: 0.,
                        ..default()
                    },
                    Palette {
                        normal: Color::NONE,
                        hovered: Color::NONE,
                        text: Color::WHITE,
                        hover_text: rgb(255, 255, 170),
                    },
                ));
                let children = world.get::<Children>(e).unwrap().to_vec();
                let bold = world.resource::<Artwork>().bold.clone();
                for child in children {
                    world.entity_mut(child).insert((
                        TextFont {
                            font: bold.clone(),
                            font_size: font,
                            ..default()
                        },
                        TextShadow {
                            offset: Vec2::splat(2.),
                            color: Color::BLACK,
                        },
                    ));
                }
            }
            _ => {}
        }
    }
}
fn footer_button(
    world: &mut World,
    parent: Entity,
    label: &str,
    icon: &str,
    action: Action,
) -> Entity {
    let e = btn(world, parent, "", action, 11., Color::WHITE, gray(51));
    feedback::tag(world, e, if label.is_empty() { icon } else { label }, 100);
    let children = world.get::<Children>(e).unwrap().to_vec();
    for child in children {
        world.despawn(child);
    }
    let back = icon == "html/img/back_to_main_menu.png";
    let small = icon == "html/img/games.png"
        || icon == "html/img/error.png"
        || icon.starts_with("resource/localization/");
    world.entity_mut(e).insert(Node {
        height: Val::Px(40.),
        min_height: Val::Px(40.),
        min_width: Val::Px(40.),
        padding: if label.is_empty() {
            UiRect::ZERO
        } else {
            UiRect {
                left: Val::Px(if back { 47. } else { 45. }),
                right: Val::Px(if back { 15. } else { 10. }),
                ..default()
            }
        },
        align_items: AlignItems::Center,
        flex_shrink: 0.,
        ..default()
    });
    let size = intrinsic_size(world, icon);
    if let Some(image) = picture(world, e, icon, size.x, size.y) {
        world.entity_mut(image).insert(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(if small {
                12.
            } else if back {
                5.
            } else {
                4.
            }),
            top: Val::Px(if icon.starts_with("resource/localization/") {
                14.
            } else if small {
                12.
            } else {
                4.
            }),
            width: Val::Px(size.x),
            height: Val::Px(size.y),
            ..default()
        });
    }
    if !label.is_empty() {
        let t = text(world, e, label, 11., gray(51), true);
        world.entity_mut(t).insert(ButtonLabel);
    }
    e
}
fn footer(world: &mut World, root: Entity, f: &Frontend) {
    let c = crate::compiled_frontend_config();
    let bar = node(
        world,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.),
            right: Val::Px(0.),
            bottom: Val::Px(0.),
            height: Val::Px(c.footer_height),
            padding: UiRect {
                left: Val::Px(5.),
                right: Val::Px(3.),
                top: Val::Px(5.),
                bottom: Val::Px(5.),
            },
            align_items: AlignItems::Center,
            ..default()
        },
        Color::srgba(0., 0., 0., 0.5),
    );
    if f.page != Page::Home {
        footer_button(
            world,
            bar,
            if matches!(f.page, Page::Feedback | Page::News) {
                "Back to previous window"
            } else {
                "Back to Main Menu"
            },
            "html/img/back_to_main_menu.png",
            if f.page == Page::Feedback {
                Action::BackFeedback
            } else if f.page == Page::News {
                Action::BackNews
            } else {
                Action::Page(Page::Home)
            },
        );
    }
    let center = node(
        world,
        bar,
        Node {
            flex_grow: 1.,
            justify_content: JustifyContent::End,
            margin: UiRect::horizontal(if f.window_size.x <= c.footer_label_breakpoint {
                Val::Px(c.footer_compact_margin)
            } else {
                Val::Percent(10.)
            }),
            ..default()
        },
        Color::NONE,
    );
    if !f.startup {
        footer_button(
            world,
            center,
            "Back to Game",
            "html/img/back_to_game.png",
            Action::Page(Page::Hidden),
        );
    }
    let right = node(
        world,
        bar,
        Node {
            column_gap: Val::Px(3.),
            ..default()
        },
        Color::NONE,
    );
    let labels = f.window_size.x > c.footer_label_breakpoint;
    footer_button(world,right,if labels {"Problems"} else {""},"html/img/error.png",Action::Info("Problems\n\nUse Detailed Feedback (F8) to record problems for Jcode. Missing reference features are tracked in sheets/menu_reference.csv. Startup diagnostics are in local/session-*.err.log.".into()));
    footer_button(
        world,
        right,
        if labels { "Games" } else { "" },
        "html/img/games.png",
        Action::Popup(Some(Popup::Games)),
    );
    footer_button(
        world,
        right,
        "",
        "resource/localization/en.png",
        Action::Popup(Some(Popup::Language)),
    );
    footer_button(
        world,
        right,
        "Sandbox",
        "gamemodes/sandbox/icon24.png",
        Action::Popup(Some(Popup::Gamemodes)),
    );
}
fn popup(world: &mut World, root: Entity, f: &Frontend) {
    let Some(kind) = f.popup else { return };
    let c = crate::compiled_frontend_config();
    // A dismissing click must never also select a map or enter the world.
    world.spawn((
        Button,
        Action::Popup(None),
        ZIndex(20),
        ChildOf(root),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.),
            right: Val::Px(0.),
            top: Val::Px(0.),
            bottom: Val::Px(c.footer_height),
            ..default()
        },
    ));
    let (right, width) = match kind {
        Popup::Games => (c.popup_games_right, c.popup_games_width),
        Popup::Language => (c.popup_language_right, c.popup_width),
        Popup::Gamemodes => (c.popup_right, c.popup_width),
    };
    let panel = node(
        world,
        root,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(right),
            bottom: Val::Px(c.footer_height),
            width: Val::Px(width),
            max_height: Val::Px(c.popup_max_height),
            margin: UiRect::all(Val::Px(3.)),
            border: UiRect::all(Val::Px(3.)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.),
            overflow: Overflow::clip(),
            ..default()
        },
        gray(238),
    );
    world.entity_mut(panel).insert((
        ZIndex(21),
        BorderColor(gray(51)),
        BorderRadius::all(Val::Px(3.)),
    ));
    match kind {
        Popup::Gamemodes => {
            footer_button(
                world,
                panel,
                "Sandbox",
                "gamemodes/sandbox/icon24.png",
                Action::Popup(None),
            );
            text(world,panel,"Only the implemented Sandbox runtime is listed. Other modes are not yet supported.",11.,gray(85),false);
        }
        Popup::Language => {
            footer_button(
                world,
                panel,
                "English",
                "resource/localization/en.png",
                Action::Popup(None),
            );
            text(
                world,
                panel,
                "Additional language translation is not implemented.",
                11.,
                gray(85),
                false,
            );
        }
        Popup::Games => {
            text(
                world,
                panel,
                "Garry's Mod stock content",
                12.,
                gray(51),
                true,
            );
            text(world,panel,"Read-only source mount. Changing game mounts is not implemented. No Steam settings are changed.",11.,gray(85),false);
        }
    }
}
fn maps(world: &mut World, page: Entity, f: &Frontend, offset: f32) {
    let c = crate::compiled_frontend_config();
    let browser = node(
        world,
        page,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(c.margin),
            right: Val::Px(c.map_right),
            top: Val::Px(c.margin),
            bottom: Val::Px(c.margin),
            ..default()
        },
        Color::WHITE,
    );
    world
        .entity_mut(browser)
        .insert(BorderRadius::all(Val::Px(5.)));
    let controls = node(
        world,
        browser,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.),
            top: Val::Px(10.),
            bottom: Val::Px(10.),
            width: Val::Px(c.sidebar_width),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(c.category_gap),
            ..default()
        },
        Color::NONE,
    );
    let mut all = crate::compiled_source_maps();
    all.sort_by(|a, b| a.id.cmp(&b.id));
    for (name, only) in [("Favorites", true), ("Sandbox", false)] {
        let count = all
            .iter()
            .filter(|m| {
                (!only || f.favorites.contains(&m.id))
                    && m.id.to_lowercase().contains(&f.map_search.to_lowercase())
            })
            .count();
        if count == 0 {
            continue;
        }
        let selected = f.favorites_only == only;
        let e = btn(
            world,
            controls,
            format!("{name}    {count}"),
            Action::MapCategory(only),
            11.,
            if selected {
                rgb(153, 204, 255)
            } else {
                gray(221)
            },
            gray(85),
        );
        world.entity_mut(e).insert(Node {
            height: Val::Px(c.category_height),
            min_height: Val::Px(c.category_height),
            padding: UiRect::axes(Val::Px(6.), Val::Px(0.)),
            align_items: AlignItems::Center,
            flex_shrink: 0.,
            ..default()
        });
    }
    let search = btn(
        world,
        controls,
        if f.map_search.is_empty() {
            "Search maps...".into()
        } else {
            format!("{}{}", f.map_search, if f.search_focus { "|" } else { "" })
        },
        Action::Search,
        12.,
        if f.search_focus {
            rgb(232, 245, 255)
        } else {
            Color::WHITE
        },
        gray(80),
    );
    world.entity_mut(search).insert((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.),
            right: Val::Px(0.),
            bottom: Val::Px(0.),
            height: Val::Px(28.),
            padding: UiRect::all(Val::Px(4.)),
            border: UiRect::all(Val::Px(1.)),
            overflow: Overflow::clip(),
            ..default()
        },
        BorderColor(gray(170)),
    ));
    let grid = node(
        world,
        browser,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(c.map_grid_left),
            top: Val::Px(0.),
            right: Val::Px(0.),
            bottom: Val::Px(0.),
            padding: UiRect::all(Val::Px(10.)),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            ..default()
        },
        Color::NONE,
    );
    world.entity_mut(grid).insert((
        FrontScroll,
        ScrollPosition {
            offset_y: offset,
            ..default()
        },
        RelativeCursorPosition::default(),
    ));
    let visible: Vec<_> = all
        .into_iter()
        .filter(|m| {
            (!f.favorites_only || f.favorites.contains(&m.id))
                && m.id.to_lowercase().contains(&f.map_search.to_lowercase())
        })
        .collect();
    let heading = node(
        world,
        grid,
        Node {
            padding: UiRect::all(Val::Px(5.)),
            align_items: AlignItems::Baseline,
            column_gap: Val::Px(8.),
            flex_shrink: 0.,
            ..default()
        },
        Color::NONE,
    );
    text(
        world,
        heading,
        if f.favorites_only {
            "Favorites"
        } else {
            "Sandbox"
        },
        c.header_font,
        gray(68),
        false,
    );
    text(
        world,
        heading,
        format!("{} Maps", visible.len()),
        14.,
        gray(153),
        false,
    );
    let cards = node(
        world,
        grid,
        Node {
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::Start,
            flex_shrink: 0.,
            ..default()
        },
        Color::NONE,
    );
    for map in visible {
        let bg = if f.selected == map.id {
            rgb(151, 197, 255)
        } else {
            Color::NONE
        };
        let card = btn(
            world,
            cards,
            "",
            Action::Map(map.id.clone()),
            c.caption_font,
            bg,
            gray(40),
        );
        let children = world.get::<Children>(card).unwrap().to_vec();
        for child in children {
            world.despawn(child);
        }
        feedback::tag(world, card, format!("Map / {}", map.id), 100);
        world.entity_mut(card).insert(Node {
            width: Val::Px(c.map_icon + 2. * c.card_padding),
            padding: UiRect::all(Val::Px(c.card_padding)),
            margin: UiRect::all(Val::Px(c.card_margin)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            flex_shrink: 0.,
            ..default()
        });
        if picture(
            world,
            card,
            &format!("maps/thumb/{}.png", map.id),
            c.map_icon,
            c.map_icon,
        )
        .is_none()
        {
            let fallback = node(
                world,
                card,
                Node {
                    width: Val::Px(c.map_icon),
                    height: Val::Px(c.map_icon),
                    ..default()
                },
                gray(220),
            );
            world.entity_mut(fallback).insert(FocusPolicy::Pass);
            text(world, fallback, "No map image", 12., gray(80), false);
        }
        text(world, card, &map.id, c.caption_font, gray(40), false);
        let fav = btn(
            world,
            card,
            "",
            Action::Favorite(map.id.clone()),
            10.,
            Color::NONE,
            gray(30),
        );
        let favorite_size = intrinsic_size(world, "html/img/favourite.png");
        world.entity_mut(fav).insert((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.),
                right: Val::Px(8.),
                width: Val::Px(favorite_size.x),
                height: Val::Px(favorite_size.y),
                min_height: Val::Px(favorite_size.y),
                ..default()
            },
            ZIndex(2),
        ));
        if let Some(icon) = picture(
            world,
            fav,
            "html/img/favourite.png",
            favorite_size.x,
            favorite_size.y,
        ) {
            world.entity_mut(icon).insert(FavoriteVisual {
                card,
                button: fav,
                selected: f.favorites.contains(&map.id),
            });
        }
    }
    let settings = node(
        world,
        page,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(c.margin),
            top: Val::Px(c.margin),
            bottom: Val::Px(c.margin),
            width: Val::Px(c.settings_width),
            ..default()
        },
        Color::WHITE,
    );
    world
        .entity_mut(settings)
        .insert(BorderRadius::all(Val::Px(5.)));
    let count=btn(world,settings,"Single Player   v",Action::Info("Player count\n\nSingle Player is supported. The reference offers 2 / 4 / 8 / 16 / 32 / 64 / 128 players, but networking must be implemented before these can launch real servers.".into()),12.,rgb(40,190,40),Color::WHITE);
    world.entity_mut(count).insert(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(10.),
        right: Val::Px(10.),
        top: Val::Px(10.),
        height: Val::Px(28.),
        padding: UiRect::left(Val::Px(10.)),
        align_items: AlignItems::Center,
        ..default()
    });
    let details = node(
        world,
        settings,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.),
            left: Val::Px(10.),
            right: Val::Px(10.),
            bottom: Val::Px(80.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.),
            overflow: Overflow::clip(),
            ..default()
        },
        Color::NONE,
    );
    text(world, details, "Sandbox", 14., gray(68), true);
    text(
        world,
        details,
        format!("Map: {}", f.selected),
        11.,
        gray(102),
        false,
    );
    text(world,details,"Local sandbox\n\nFull Sandbox settings, multiplayer and native option panels are still being implemented. No inactive setting is presented as functional.",12.,gray(102),false);
    let start = btn(
        world,
        settings,
        "Start Game",
        Action::Start,
        c.start_font,
        rgb(0, 102, 204),
        Color::WHITE,
    );
    world.entity_mut(start).insert(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(c.start_inset),
        right: Val::Px(c.start_inset),
        bottom: Val::Px(c.start_inset),
        padding: UiRect::all(Val::Px(10.)),
        justify_content: JustifyContent::Center,
        ..default()
    });
}
fn feedback_form(world: &mut World, page: Entity, f: &Frontend) {
    let panel = node(
        world,
        page,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(3.),
            right: Val::Percent(3.),
            top: Val::Px(32.),
            bottom: Val::Px(12.),
            padding: UiRect::all(Val::Px(12.)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.),
            overflow: Overflow::clip(),
            ..default()
        },
        Color::WHITE,
    );
    text(
        world,
        panel,
        "Feedback: type or dictate one note",
        24.,
        gray(45),
        true,
    );
    text(world, panel, "Click the box, then use your Windows speech hotkey (Win+H). No in-game microphone or speech service. Windows controls its own listening indicator and speech privacy settings.", 13., gray(65), false);
    let context: String = feedback::summary(f).chars().take(240).collect();
    text(world, panel, context, 12., gray(75), false);
    if !f.message.is_empty() {
        text(world, panel, &f.message, 13., rgb(130, 65, 40), false);
    }
    let editor = standard(
        world,
        panel,
        if f.feedback_native {
            ""
        } else if f.feedback_text.is_empty() {
            "Click here and describe anything you want improved. No separate title or tabs."
        } else {
            &f.feedback_text
        },
        Action::FocusFeedback,
    );
    world.entity_mut(editor).insert((
        feedback::EditorSlot,
        Node {
            width: Val::Percent(100.),
            flex_grow: 1.,
            flex_basis: Val::Px(0.),
            min_height: Val::Px(80.),
            overflow: Overflow::clip(),
            padding: UiRect::all(Val::Px(8.)),
            ..default()
        },
    ));
    let controls = node(
        world,
        panel,
        Node {
            column_gap: Val::Px(8.),
            row_gap: Val::Px(4.),
            flex_wrap: FlexWrap::Wrap,
            flex_shrink: 0.,
            ..default()
        },
        Color::NONE,
    );
    standard(world, controls, "Save note locally", Action::Submit);
    standard(world, controls, "Return (F8 / Esc)", Action::BackFeedback);
    standard(
        world,
        controls,
        "Attach current context",
        Action::RetargetFeedback,
    );
    standard(
        world,
        controls,
        format!(
            "Category: {}",
            ["Bug", "Feature request", "Visual mismatch", "Performance"][f.category]
        ),
        Action::Category,
    );
}
fn form(world: &mut World, page: Entity, f: &Frontend, offset: f32) {
    let panel = node(
        world,
        page,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(if f.window_size.x < 940. { 3. } else { 20. }),
            right: Val::Percent(if f.window_size.x < 940. { 3. } else { 20. }),
            top: Val::Px(40.),
            bottom: Val::Px(24.),
            padding: UiRect::all(Val::Px(20.)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        Color::WHITE,
    );
    world.entity_mut(panel).insert((
        BorderRadius::all(Val::Px(5.)),
        FrontScroll,
        ScrollPosition {
            offset_y: offset,
            ..default()
        },
        RelativeCursorPosition::default(),
    ));
    match f.page {
        Page::News => {
            let notes = crate::compiled_release_notes();
            let checked = notes
                .iter()
                .filter(|n| f.news_checked.contains(&news::key(n)))
                .count();
            text(
                world,
                panel,
                "What's New / What to Test",
                28.,
                gray(50),
                true,
            );
            text(
                world,
                panel,
                format!(
                    "{checked}/{} personally checked | F7 or Esc returns to your previous window",
                    notes.len()
                ),
                14.,
                gray(60),
                true,
            );
            text(world, panel, "These are changes to inspect, not claims that everything works. Checkmarks are your private reminders, not passing test results. Scroll for steps, expected behavior and known limits. Report an issue to attach that item's context.", 14., gray(65), false);
            if !f.news_error.is_empty() {
                text(world, panel, &f.news_error, 14., rgb(160, 40, 40), false);
            }
            if !f.message.is_empty() {
                text(world, panel, &f.message, 14., rgb(80, 80, 120), false);
            }
            standard(
                world,
                panel,
                "Return to previous window (F7 / Esc)",
                Action::BackNews,
            );
            for note in notes {
                let card = node(
                    world,
                    panel,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.),
                        padding: UiRect::all(Val::Px(12.)),
                        flex_shrink: 0.,
                        ..default()
                    },
                    gray(240),
                );
                let status = match note.status.as_str() {
                    "partial" => "PARTIAL",
                    "missing" => "NOT IMPLEMENTED",
                    _ => "NEEDS YOUR CHECK",
                };
                text(
                    world,
                    card,
                    format!("{} | {status}", note.title),
                    18.,
                    gray(40),
                    true,
                );
                text(
                    world,
                    card,
                    format!(
                        "Revision: {}\n{}\n\nTry: {}\n\nExpected: {}\n\nKnown limits: {}",
                        note.revision, note.change, note.steps, note.expected, note.limits
                    ),
                    14.,
                    gray(60),
                    false,
                );
                let checked = f.news_checked.contains(&news::key(&note));
                standard(
                    world,
                    card,
                    if checked {
                        "[x] I checked this (click to undo)"
                    } else {
                        "[ ] Mark personally checked"
                    },
                    Action::CheckNews(note.id.clone()),
                );
                standard(
                    world,
                    card,
                    "Report an issue with this item",
                    Action::ReportNews(note.id),
                );
            }
        }
        Page::Options => {
            text(world, panel, "Options", 28., gray(50), true);
            text(
                world,
                panel,
                "Supported local controls. The full native Options dialog is not yet reproduced.",
                13.,
                gray(80),
                false,
            );
            if let Some(p) = world.get_resource::<PlayState>() {
                let values = [
                    ("Field of view", "fov", p.fov, 5.),
                    ("Noclip speed", "speed", p.speed, 1.),
                    ("Mouse sensitivity", "sensitivity", p.sensitivity, 0.0005),
                ];
                for (name, key, value, step) in values {
                    text(
                        world,
                        panel,
                        format!("{name}: {value:.4}"),
                        14.,
                        gray(60),
                        false,
                    );
                    let row = node(
                        world,
                        panel,
                        Node {
                            column_gap: Val::Px(8.),
                            ..default()
                        },
                        Color::NONE,
                    );
                    standard(world, row, "-", Action::Setting(key, -step));
                    standard(world, row, "+", Action::Setting(key, step));
                }
            } else {
                text(world,panel,"Start a map to adjust current player settings. Startup settings persistence remains unfinished.",14.,gray(80),false);
            }
        }
        Page::Saves => {
            text(world, panel, "Saves", 28., gray(50), true);
            text(
                world,
                panel,
                "Local Rust scenes, not native Garry's Mod or Workshop saves.",
                14.,
                gray(80),
                false,
            );
            if !f.startup {
                standard(world, panel, "Save current scene", Action::Save);
                standard(
                    world,
                    panel,
                    "Load latest local scene (Z can undo)",
                    Action::Load,
                );
            } else {
                text(world,panel,"Start a map before loading a local scene. A map-aware save browser remains unfinished.",14.,gray(80),false);
            }
        }
        Page::Confirm => {
            text(world, panel, "Leave current session?", 28., gray(50), true);
            text(world,panel,"Unsaved changes will be lost if you continue without saving. Your feedback draft is retained.",14.,gray(80),false);
            if !f.startup {
                standard(
                    world,
                    panel,
                    "Save scene and continue",
                    Action::Confirm(true),
                );
            }
            standard(
                world,
                panel,
                "Continue without saving",
                Action::Confirm(false),
            );
            standard(world, panel, "Cancel", Action::Page(Page::Home));
        }
        Page::Loading => {
            text(world, panel, "Loading...", 32., gray(80), true);
        }
        Page::Info => {
            text(world, panel, &f.info, 16., gray(60), false);
            standard(world, panel, "Back", Action::Page(Page::Home));
        }
        _ => {}
    }
    if !f.message.is_empty() && f.page != Page::News {
        text(world, panel, &f.message, 13., rgb(80, 80, 120), false);
    }
}
pub(super) fn draw(world: &mut World, f: &Frontend) {
    let offset = world
        .query_filtered::<&ScrollPosition, With<FrontScroll>>()
        .iter(world)
        .next()
        .map(|s| s.offset_y)
        .unwrap_or(0.);
    let old: Vec<_> = world
        .query_filtered::<Entity, With<Root>>()
        .iter(world)
        .collect();
    for e in old {
        world.despawn(e);
    }
    if f.page == Page::Hidden {
        return;
    }
    let root = world
        .spawn((
            Root,
            GlobalZIndex(100),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(if f.startup {
                rgb(20, 40, 60)
            } else {
                Color::srgba(0., 0., 0., 0.35)
            }),
        ))
        .id();
    let c = crate::compiled_frontend_config();
    if f.startup {
        if let Some(handle) = world
            .resource::<Artwork>()
            .images
            .get(&c.background)
            .cloned()
        {
            let size = world
                .resource::<Assets<Image>>()
                .get(&handle)
                .map(|i| i.size().as_vec2())
                .unwrap_or(Vec2::new(1920., 1080.));
            let ratio = (f.window_size.x / size.x).max(f.window_size.y / size.y);
            let visible = f.window_size / ratio;
            let rect = Rect::from_center_size(size * 0.5, visible);
            world.spawn((
                ImageNode {
                    rect: Some(rect),
                    ..ImageNode::new(handle)
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                FocusPolicy::Pass,
                ChildOf(root),
            ));
        }
    }
    let page = node(
        world,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.),
            right: Val::Px(0.),
            top: Val::Px(0.),
            bottom: Val::Px(c.footer_height),
            ..default()
        },
        Color::NONE,
    );
    match f.page {
        Page::Home => home(world, page, f),
        Page::Maps => maps(world, page, f, offset),
        Page::Feedback => feedback_form(world, page, f),
        _ => form(world, page, f, offset),
    }
    footer(world, root, f);
    popup(world, root, f);
    let badge = node(
        world,
        root,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(3.),
            top: Val::Px(3.),
            padding: UiRect::all(Val::Px(5.)),
            ..default()
        },
        Color::BLACK,
    );
    text(
        world,
        badge,
        "Rust Workshop | independent local sandbox",
        11.,
        gray(170),
        true,
    );
    if f.page == Page::Home {
        let news = standard(
            world,
            root,
            "What's New / Test Checklist (F7)",
            Action::Page(Page::News),
        );
        world.entity_mut(news).insert(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.),
            top: Val::Px(68.),
            padding: UiRect::all(Val::Px(10.)),
            ..default()
        });
        let feedback = standard(
            world,
            root,
            "Detailed Feedback (F8)",
            Action::Page(Page::Feedback),
        );
        world.entity_mut(feedback).insert(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.),
            top: Val::Px(30.),
            padding: UiRect::all(Val::Px(6.)),
            ..default()
        });
    }
    if f.page == Page::Home && !f.message.is_empty() {
        let note = node(
            world,
            page,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(40.),
                right: Val::Px(40.),
                top: Val::Px(90.),
                padding: UiRect::all(Val::Px(12.)),
                ..default()
            },
            Color::srgba(0., 0., 0., 0.8),
        );
        text(world, note, &f.message, 14., Color::WHITE, false);
    }
}
