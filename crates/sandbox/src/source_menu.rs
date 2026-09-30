//! Stock-inspired panel geometry from authored reference sheets. Not a Derma emulator.
use super::*;
use bevy::ui::{RelativeCursorPosition, ScrollPosition};

#[derive(Component)]
pub(super) struct ScrollPane(u8);
#[derive(Component)]
pub(super) struct ModelHint(String);
#[derive(Component)]
pub(super) struct HintLine;
#[derive(Component)]
pub(super) struct BaseColor(Color);

fn row(world: &mut World, parent: Entity, height: f32) -> Entity {
    container(
        world,
        parent,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            min_height: Val::Px(height),
            flex_shrink: 0.,
            ..default()
        },
    )
}
fn column(world: &mut World, parent: Entity, width: Val) -> Entity {
    container(
        world,
        parent,
        Node {
            width,
            min_width: Val::Px(0.),
            height: Val::Percent(100.),
            min_height: Val::Px(0.),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )
}
fn action(
    world: &mut World,
    parent: Entity,
    label: impl Into<String>,
    action: UiAction,
    selected: bool,
) -> Entity {
    let c = world.resource::<PlayState>().layout.clone();
    let color = if selected {
        Color::srgb(0.65, 0.77, 0.9)
    } else {
        Color::srgb(0.87, 0.88, 0.89)
    };
    let e = button(world, parent, label, action);
    world.entity_mut(e).insert((
        Node {
            min_height: Val::Px(c.row_height),
            padding: UiRect::axes(Val::Px(c.panel_padding), Val::Px(2.)),
            flex_shrink: 0.,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(color),
        BaseColor(color),
    ));
    e
}
fn scroll_panel(world: &mut World, parent: Entity, id: u8) -> Entity {
    let offset = world.resource::<PlayState>().menu_scroll[id as usize];
    world
        .spawn((
            Node {
                flex_grow: 1.,
                flex_basis: Val::Px(0.),
                min_height: Val::Px(0.),
                width: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPane(id),
            ScrollPosition {
                offset_y: offset,
                ..default()
            },
            RelativeCursorPosition::default(),
            BackgroundColor(Color::srgb(0.94, 0.94, 0.94)),
            ChildOf(parent),
        ))
        .id()
}
fn divider(world: &mut World, parent: Entity, width: f32) {
    world.spawn((
        Node {
            width: Val::Px(width),
            flex_shrink: 0.,
            height: Val::Percent(100.),
            ..default()
        },
        BackgroundColor(Color::srgb(0.65, 0.66, 0.67)),
        ChildOf(parent),
    ));
}
fn catalog_browser(world: &mut World, body: Entity, tree_width: f32) {
    let s = world.resource::<PlayState>();
    let catalog = s.spawn_catalog.clone();
    let Some(tab) = catalog.tabs.iter().find(|t| t.id == s.creation_tab) else {
        return;
    };
    let c = s.layout.clone();
    let category = s.category.clone();
    let search = s.search.clone();
    let focused = s.search_focus;
    let selected = s.catalog_selected.clone();
    let page_size = s.config.page_size;
    let page = s.page;
    let entries: Vec<_> = catalog
        .entries
        .iter()
        .filter(|e| e.kind == tab.kind && e.visibility == "spawnmenu")
        .collect();
    let browser = column(world, body, Val::Px(tree_width));
    action(
        world,
        browser,
        if search.is_empty() {
            "Search definitions...".into()
        } else {
            format!("{}{}", search, if focused { "|" } else { "" })
        },
        UiAction::Search,
        focused,
    );
    if tab.kind == "npc" {
        let settings = world.resource::<npcs::NpcSettings>();
        let disabled = settings.disabled;
        let ignore = settings.ignore_players;
        action(
            world,
            browser,
            format!("[{}] Disable thinking", if disabled { "x" } else { " " }),
            UiAction::NpcSetting("disabled"),
            disabled,
        );
        action(
            world,
            browser,
            format!("[{}] Ignore players", if ignore { "x" } else { " " }),
            UiAction::NpcSetting("ignore_players"),
            ignore,
        );
        text(
            world,
            browser,
            "Equipment / corpses / squads: pending",
            c.font_size,
        );
    }
    let tree = scroll_panel(world, browser, 0);
    action(
        world,
        tree,
        "All categories",
        UiAction::Category(String::new()),
        category.is_empty(),
    );
    let categories: std::collections::BTreeSet<_> =
        entries.iter().map(|e| e.category.clone()).collect();
    for cat in categories {
        action(
            world,
            tree,
            &cat,
            UiAction::Category(cat.clone()),
            cat == category,
        );
    }
    divider(world, body, c.divider);
    let content = column(world, body, Val::Auto);
    world.entity_mut(content).insert(Node {
        flex_grow: 1.,
        flex_basis: Val::Px(0.),
        min_width: Val::Px(0.),
        min_height: Val::Px(0.),
        height: Val::Percent(100.),
        flex_direction: FlexDirection::Column,
        ..default()
    });
    text(
        world,
        content,
        format!(
            "{} | Click to use supported entries; Details shows remaining work",
            tab.label
        ),
        c.font_size,
    );
    let needle = search.to_lowercase();
    let results: Vec<_> = entries
        .into_iter()
        .filter(|e| {
            (category.is_empty() || e.category == category)
                && format!(
                    "{} {} {} {}",
                    e.label, e.spawn_name, e.class_name, e.category
                )
                .to_lowercase()
                .contains(&needle)
        })
        .collect();
    let pages = results.len().div_ceil(page_size).max(1);
    let page = page.min(pages - 1);
    world.resource_mut::<PlayState>().page = page;
    let viewport = scroll_panel(world, content, 1);
    if let Some(e) = results.iter().find(|e| e.id == selected) {
        text(world, viewport, format!("{} | {}\nClass: {}\nAvailability: {} (mount not checked) | Admin: {}\nReference: {}:{}\nModel: {}", e.label, e.spawn_name, e.class_name, e.condition, e.admin_only, e.source, e.line,
            if e.model.is_empty() { "native/inherited default; not inferred" } else { &e.model }), c.font_size);
        if let Some(w) = catalog.weapons.iter().find(|w| w.id == e.id) {
            text(
                world,
                viewport,
                format!("Scope: {}\nRemaining: {}", w.scope, w.remaining),
                c.font_size,
            );
        }
        if let Some(n) = catalog.npcs.iter().find(|n| n.id == e.id) {
            text(
                world,
                viewport,
                format!("Scope: {}\nRemaining: {}", n.scope, n.remaining),
                c.font_size,
            );
        }
        if let Some(v) = catalog.vehicles.iter().find(|v| v.id == e.id) {
            text(
                world,
                viewport,
                format!("Scope: {}\nRemaining: {}", v.scope, v.remaining),
                c.font_size,
            );
        }
        action(
            world,
            viewport,
            "Request improvement for this definition (F8)",
            UiAction::Feedback(format!("{} [{}]", e.label, e.id)),
            false,
        );
        text(
            world,
            viewport,
            "Relevant system work, not a per-class implementation claim:",
            c.font_size,
        );
        for cap in catalog.capabilities.iter().filter(|cap| {
            cap.kind == tab.kind
                || (cap.kind == "shared"
                    && ["npc", "entity", "weapon", "vehicle"].contains(&tab.kind.as_str()))
        }) {
            text(
                world,
                viewport,
                format!(
                    "{} [{} / {}] {}",
                    cap.id, cap.status, cap.priority, cap.feature
                ),
                c.font_size,
            );
        }
    }
    for e in results.iter().skip(page * page_size).take(page_size) {
        let weapon = catalog.weapons.iter().find(|w| w.id == e.id);
        let vehicle = catalog.vehicles.iter().find(|v| v.id == e.id);
        let npc = catalog
            .npcs
            .iter()
            .find(|n| n.id == e.id && n.kind != "disabled");
        let usable =
            weapon.is_some_and(|w| w.kind != "disabled") || vehicle.is_some() || npc.is_some();
        let label = if weapon.is_some_and(|w| w.kind != "disabled") {
            "Equip"
        } else if vehicle.is_some() || npc.is_some() {
            "Spawn"
        } else {
            "Pending"
        };
        let buttons = row(world, viewport, c.row_height);
        action(
            world,
            buttons,
            format!(
                "{}{} | {}",
                e.label,
                if e.admin_only { " [Admin]" } else { "" },
                label
            ),
            if usable {
                UiAction::UseEntry(e.id.clone())
            } else {
                UiAction::InspectEntry(e.id.clone())
            },
            e.id == selected,
        );
        action(
            world,
            buttons,
            "Details / F8",
            UiAction::InspectEntry(e.id.clone()),
            false,
        );
    }
    if results.is_empty() {
        text(
            world,
            viewport,
            "No matching spawn definitions. No placeholder objects will be spawned.",
            c.font_size,
        );
        for cap in catalog
            .capabilities
            .iter()
            .filter(|cap| cap.kind == tab.kind)
        {
            text(
                world,
                viewport,
                format!(
                    "{} [{}] {}\nAcceptance: {}",
                    cap.id, cap.status, cap.feature, cap.acceptance
                ),
                c.font_size,
            );
        }
    }
    let footer = row(world, content, c.row_height);
    if pages > 1 {
        action(world, footer, "<", UiAction::Page(-1), false);
    }
    text(
        world,
        footer,
        format!("{} definitions  {}/{}", results.len(), page + 1, pages),
        c.font_size,
    );
    if pages > 1 {
        action(world, footer, ">", UiAction::Page(1), false);
    }
}

pub(super) fn reset_creation_scroll(world: &mut World, tree: bool) {
    for (pane, mut position) in world
        .query::<(&ScrollPane, &mut ScrollPosition)>()
        .iter_mut(world)
    {
        if pane.0 == 1 || (tree && pane.0 == 0) {
            position.offset_y = 0.;
        }
    }
    let mut state = world.resource_mut::<PlayState>();
    state.menu_scroll[1] = 0.;
    if tree {
        state.menu_scroll[0] = 0.;
    }
}

pub(super) fn rebuild(world: &mut World) {
    let offsets: Vec<_> = world
        .query::<(&ScrollPane, &ScrollPosition)>()
        .iter(world)
        .map(|(id, p)| (id.0 as usize, p.offset_y))
        .collect();
    for (id, offset) in offsets {
        world.resource_mut::<PlayState>().menu_scroll[id] = offset;
    }
    let old: Vec<_> = world
        .query_filtered::<Entity, With<MenuRoot>>()
        .iter(world)
        .collect();
    for entity in old {
        world.despawn(entity);
    }
    world.resource_mut::<PlayState>().dirty = false;
    if !world.resource::<PlayState>().menu_open {
        return;
    }
    let state = world.resource::<PlayState>();
    let c = state.layout.clone();
    let size = state.menu_size;
    let small = size.x < c.reference_width || size.y < c.reference_height;
    let margin = if small {
        Vec2::ZERO
    } else {
        ((size - Vec2::new(c.reference_width, c.reference_height)) * c.border_factor)
            .clamp(Vec2::splat(c.min_margin), Vec2::splat(c.max_margin))
    };
    let width = (size.x - margin.x * 2.).max(1.);
    let tools_width = (if size.x < c.reference_width {
        c.small_tool_width
    } else {
        c.tool_width
    })
    .min(width * 0.48);
    let creation_width = (width - tools_width - c.divider).max(1.);
    let tree_width = (if small {
        c.small_tree_width
    } else {
        c.tree_width
    })
    .min(creation_width * 0.4);
    let (models, category, search, page, page_size, tab, tool, search_focus) = (
        state.models.clone(),
        state.category.clone(),
        state.search.clone(),
        state.page,
        state.config.page_size,
        state.tab,
        state.tool.clone(),
        state.search_focus,
    );
    let creation_tab = state.creation_tab.clone();
    let creation_tabs = state.spawn_catalog.tabs.clone();
    let root = world
        .spawn((
            MenuRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(margin.x),
                right: Val::Px(margin.x),
                top: Val::Px(margin.y),
                bottom: Val::Px(margin.y),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            GlobalZIndex(20),
        ))
        .id();
    let creation = column(world, root, Val::Px(creation_width));
    crate::source_frontend::feedback::tag(world, creation, "Spawn browser", 10);
    world
        .entity_mut(creation)
        .insert(BackgroundColor(Color::srgb(0.82, 0.83, 0.84)));
    let tabs = row(world, creation, c.tab_height);
    world.entity_mut(tabs).insert(Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        min_height: Val::Px(c.tab_height),
        flex_shrink: 0.,
        ..default()
    });
    for t in creation_tabs {
        action(
            world,
            tabs,
            t.label,
            UiAction::CreationTab(t.id.clone()),
            t.id == creation_tab,
        );
    }
    action(
        world,
        creation,
        "Feedback on this browser (F8)",
        UiAction::Feedback("Spawn browser".into()),
        false,
    );
    let body = container(
        world,
        creation,
        Node {
            flex_grow: 1.,
            flex_basis: Val::Px(0.),
            min_height: Val::Px(0.),
            flex_direction: FlexDirection::Row,
            ..default()
        },
    );
    if creation_tab != "spawnlists" {
        catalog_browser(world, body, tree_width);
    } else {
        let browser = column(world, body, Val::Px(tree_width));
        let label = if search.is_empty() {
            "Search models...".into()
        } else {
            format!("{}{}", search, if search_focus { "|" } else { "" })
        };
        action(world, browser, label, UiAction::Search, search_focus);
        let tree = scroll_panel(world, browser, 0);
        text(world, tree, "Browse installed models", c.font_size);
        action(
            world,
            tree,
            "All models",
            UiAction::Category(String::new()),
            category.is_empty(),
        );
        let mut categories = std::collections::BTreeSet::new();
        for label in models.iter().flat_map(|m| &m.categories) {
            let mut path = String::new();
            for part in label.split(" / ") {
                if !path.is_empty() {
                    path.push_str(" / ");
                }
                path.push_str(part);
                categories.insert(path.clone());
            }
        }
        for cat in &categories {
            let (parent, label) = cat.rsplit_once(" / ").unwrap_or(("", cat));
            // Keep root groups and the active ancestor branch visible instead of thousands of flat paths.
            if !parent.is_empty()
                && category != parent
                && !category.starts_with(&format!("{parent} / "))
            {
                continue;
            }
            let prefix = format!("{cat} / ");
            let branch = categories.iter().any(|c| c.starts_with(&prefix));
            let depth = cat.matches(" / ").count();
            action(
                world,
                tree,
                format!(
                    "{}{} {label}",
                    "  ".repeat(depth),
                    if branch { ">" } else { " " }
                ),
                UiAction::Category(cat.clone()),
                cat == &category,
            );
        }
        divider(world, body, c.divider);
        let content = column(world, body, Val::Auto);
        world.entity_mut(content).insert(Node {
            flex_grow: 1.,
            flex_basis: Val::Px(0.),
            min_width: Val::Px(0.),
            min_height: Val::Px(0.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            ..default()
        });
        text(
            world,
            content,
            if category.is_empty() {
                "All models"
            } else {
                &category
            },
            c.font_size,
        );
        let results: Vec<_> = models
            .into_iter()
            .filter(|m| {
                (category.is_empty()
                    || m.categories
                        .iter()
                        .any(|c| c == &category || c.starts_with(&format!("{category} / "))))
                    && (m.model.to_lowercase().contains(&search.to_lowercase())
                        || m.categories
                            .iter()
                            .any(|c| c.to_lowercase().contains(&search.to_lowercase())))
            })
            .collect();
        let pages = results.len().div_ceil(page_size).max(1);
        let page = page.min(pages - 1);
        world.resource_mut::<PlayState>().page = page;
        let viewport = scroll_panel(world, content, 1);
        let grid = container(
            world,
            viewport,
            Node {
                width: Val::Percent(100.),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                flex_shrink: 0.,
                align_content: AlignContent::FlexStart,
                column_gap: Val::Px(c.icon_gap),
                row_gap: Val::Px(c.icon_gap),
                padding: UiRect::all(Val::Px(c.panel_padding)),
                ..default()
            },
        );
        for model in results.iter().skip(page * page_size).take(page_size) {
            let icon = thumbnail(world, &model.model);
            let e = action(world, grid, "", UiAction::Spawn(model.model.clone()), false);
            world.entity_mut(e).insert((
                Node {
                    width: Val::Px(c.icon_size),
                    height: Val::Px(c.icon_size),
                    flex_shrink: 0.,
                    overflow: Overflow::clip(),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ModelHint(model.model.clone()),
            ));
            if let Some(image) = icon {
                world.spawn((
                    ImageNode::new(image),
                    bevy::ui::FocusPolicy::Pass,
                    Node {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    ChildOf(e),
                ));
            } else {
                let name = model
                    .model
                    .rsplit('/')
                    .next()
                    .unwrap_or("model")
                    .trim_end_matches(".mdl");
                text(world, e, name, c.font_size);
            }
        }
        let footer = row(world, content, c.row_height);
        if pages > 1 {
            action(world, footer, "<", UiAction::Page(-1), false);
        }
        text(
            world,
            footer,
            format!("{} models  {}/{}", results.len(), page + 1, pages),
            c.font_size,
        );
        if pages > 1 {
            action(world, footer, ">", UiAction::Page(1), false);
        }
        world.spawn((
            Text::new("Hover an icon for its model name"),
            HintLine,
            TextFont {
                font_size: c.font_size,
                ..default()
            },
            TextColor(Color::srgb(0.15, 0.15, 0.15)),
            Node {
                height: Val::Px(c.row_height),
                overflow: Overflow::clip(),
                flex_shrink: 0.,
                ..default()
            },
            ChildOf(content),
        ));
    }
    divider(world, root, c.divider);
    let tools = column(world, root, Val::Px(tools_width));
    world
        .entity_mut(tools)
        .insert(BackgroundColor(Color::srgb(0.82, 0.83, 0.84)));
    let tabs = row(world, tools, c.tab_height);
    for (id, label) in [(0, "Tools"), (1, "Options"), (2, "Utilities")] {
        action(world, tabs, label, UiAction::Tab(id), tab == id);
    }
    let tool_body = container(
        world,
        tools,
        Node {
            flex_grow: 1.,
            flex_basis: Val::Px(0.),
            min_height: Val::Px(0.),
            ..default()
        },
    );
    let list_col = column(
        world,
        tool_body,
        Val::Px(c.tool_list_width.min(tools_width * 0.42)),
    );
    let list = scroll_panel(world, list_col, 2);
    divider(world, tool_body, c.divider);
    let controls = container(
        world,
        tool_body,
        Node {
            flex_grow: 1.,
            flex_basis: Val::Px(0.),
            min_width: Val::Px(0.),
            min_height: Val::Px(0.),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(c.panel_padding)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
    );
    world.entity_mut(controls).insert((
        RelativeCursorPosition::default(),
        ScrollPosition::default(),
        ScrollPane(3),
        BackgroundColor(Color::srgb(0.94, 0.94, 0.94)),
    ));
    crate::source_frontend::feedback::tag(
        world,
        controls,
        format!("Tool panel / tab {tab} / {tool}"),
        20,
    );
    action(
        world,
        controls,
        "Feedback on this panel (F8)",
        UiAction::Feedback(format!("Tool panel / tab {tab} / {tool}")),
        false,
    );
    match tab {
        0 => {
            let catalog = world.resource::<PlayState>().tools.catalog.clone();
            for category in ["Constraints", "Construction", "Render", "Poser", "Internal"] {
                text(world, list, category, c.font_size);
                for def in catalog.tools.iter().filter(|t| t.category == category) {
                    let label = if def.status == "partial" {
                        def.label.clone()
                    } else {
                        format!("{} (pending)", def.label)
                    };
                    action(
                        world,
                        list,
                        label,
                        UiAction::Tool(def.id.clone()),
                        tool == def.id,
                    );
                }
            }
            if let Some(def) = catalog.tools.iter().find(|t| t.id == tool) {
                text(world, controls, &def.label, c.font_size + 2.);
                if def.status != "partial" {
                    text(
                        world,
                        controls,
                        "NOT IMPLEMENTED: reference entry only",
                        c.font_size,
                    );
                }
                text(
                    world,
                    controls,
                    format!(
                        "Left: {}\nRight: {}\nR: {}",
                        def.left, def.right, def.reload
                    ),
                    c.font_size,
                );
                for option in catalog.options.iter().filter(|o| o.tool == tool) {
                    let value = world
                        .resource::<PlayState>()
                        .tools
                        .value(&tool, &option.key);
                    world.spawn((
                        Text::new(format!(
                            "{}: {}{}",
                            option.label,
                            value,
                            if option.enabled { "" } else { " (pending)" }
                        )),
                        TextFont {
                            font_size: c.font_size,
                            ..default()
                        },
                        TextColor(Color::srgb(0.12, 0.15, 0.2)),
                        ToolValue {
                            tool: tool.clone(),
                            key: option.key.clone(),
                            label: option.label.clone(),
                            enabled: option.enabled,
                        },
                        ChildOf(controls),
                    ));
                    if !option.enabled {
                        continue;
                    }
                    if option.kind == "number" {
                        let slider = container(
                            world,
                            controls,
                            Node {
                                width: Val::Percent(100.),
                                height: Val::Px(c.row_height),
                                flex_shrink: 0.,
                                ..default()
                            },
                        );
                        world.entity_mut(slider).insert((
                            Button,
                            RelativeCursorPosition::default(),
                            ToolSlider {
                                tool: tool.clone(),
                                key: option.key.clone(),
                            },
                            BackgroundColor(Color::srgb(0.65, 0.67, 0.7)),
                        ));
                        crate::source_frontend::feedback::tag(
                            world,
                            slider,
                            format!("Tool setting / {tool} / {}", option.key),
                            100,
                        );
                        let fraction = ((value.parse::<f32>().unwrap_or(option.min) - option.min)
                            / (option.max - option.min))
                            .clamp(0., 1.);
                        world.spawn((
                            Node {
                                width: Val::Percent(fraction * 100.),
                                height: Val::Percent(100.),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.5, 0.75)),
                            bevy::ui::FocusPolicy::Pass,
                            SliderFill {
                                tool: tool.clone(),
                                key: option.key.clone(),
                            },
                            ChildOf(slider),
                        ));
                    }
                    let r = row(world, controls, c.row_height);
                    if option.kind != "bool" {
                        action(
                            world,
                            r,
                            "-",
                            UiAction::ToolSetting(option.key.clone(), -1),
                            false,
                        );
                    }
                    action(
                        world,
                        r,
                        if option.kind == "bool" {
                            if value == "1" {
                                "[x] Enabled"
                            } else {
                                "[ ] Disabled"
                            }
                        } else {
                            "+"
                        },
                        UiAction::ToolSetting(option.key.clone(), 1),
                        false,
                    );
                }
                action(
                    world,
                    controls,
                    "Restore tool defaults",
                    UiAction::ToolDefaults,
                    false,
                );
                text(
                    world,
                    controls,
                    format!("Remaining parity: {}", def.remaining),
                    c.font_size,
                );
            }
        }
        1 => {
            text(world, list, "Player", c.font_size);
            text(world, list, "View / controls", c.font_size);
            let s = world.resource::<PlayState>();
            let values = [
                ("Field of view", "fov", s.fov, 5.),
                ("Noclip speed", "speed", s.speed, 1.),
                ("Sensitivity", "sensitivity", s.sensitivity, 0.0005),
            ];
            let visible = s.weapon_visible;
            for (label, key, value, delta) in values {
                text(world, controls, format!("{label}: {value:.4}"), c.font_size);
                let r = row(world, controls, c.row_height);
                action(world, r, "-", UiAction::Setting(key, -delta), false);
                action(world, r, "+", UiAction::Setting(key, delta), false);
            }
            action(
                world,
                controls,
                format!("Viewmodel: {}", if visible { "ON" } else { "OFF" }),
                UiAction::Setting("weapon", 0.),
                visible,
            );
            text(
                world,
                controls,
                "Settings are local. Other stock option pages are not implemented.",
                c.font_size,
            );
        }
        _ => {
            text(world, list, "Local scenes", c.font_size);
            text(
                world,
                controls,
                "Independent scene format, not GMod saves.",
                c.font_size,
            );
            action(world, controls, "Save scene", UiAction::Save, false);
            action(world, controls, "Load scene", UiAction::Load, false);
            action(
                world,
                controls,
                "Remove all constraints (Z undoes)",
                UiAction::CleanupConstraints,
                false,
            );
        }
    }
    let bottom = row(world, tools, c.row_height);
    action(world, bottom, "Close", UiAction::Close, false);
    text(world, bottom, "Hold Q to build", c.font_size);
}
#[derive(Component)]
struct ToolSlider {
    tool: String,
    key: String,
}
#[derive(Component)]
struct SliderFill {
    tool: String,
    key: String,
}
#[derive(Resource)]
struct SliderEditing;
#[derive(Component)]
struct ToolValue {
    tool: String,
    key: String,
    label: String,
    enabled: bool,
}
pub(super) fn tool_sliders(world: &mut World) {
    let pressed = world
        .resource::<ButtonInput<MouseButton>>()
        .pressed(MouseButton::Left);
    let edits: Vec<_> = world
        .query::<(&ToolSlider, &RelativeCursorPosition, &Interaction)>()
        .iter(world)
        .filter(|(_, _, i)| pressed && **i == Interaction::Pressed)
        .filter_map(|(s, c, _)| c.normalized.map(|v| (s.tool.clone(), s.key.clone(), v.x)))
        .collect();
    for (tool, key, value) in &edits {
        tools::set_fraction(world, tool, key, *value);
    }
    if !edits.is_empty() {
        world.insert_resource(SliderEditing);
        let labels: Vec<_> = world
            .query::<(Entity, &ToolValue)>()
            .iter(world)
            .map(|(e, label)| {
                (
                    e,
                    format!(
                        "{}: {}{}",
                        label.label,
                        world
                            .resource::<PlayState>()
                            .tools
                            .value(&label.tool, &label.key),
                        if label.enabled { "" } else { " (pending)" }
                    ),
                )
            })
            .collect();
        for (e, value) in labels {
            if let Some(mut text) = world.get_mut::<Text>(e) {
                if text.0 != value {
                    text.0 = value;
                }
            }
        }
    }
    let values: Vec<_> = world
        .query::<(Entity, &SliderFill)>()
        .iter(world)
        .filter_map(|(e, s)| {
            let p = world.resource::<PlayState>();
            let o = p
                .tools
                .catalog
                .options
                .iter()
                .find(|o| o.tool == s.tool && o.key == s.key)?;
            Some((
                e,
                (p.tools.number(&s.tool, &s.key) - o.min) / (o.max - o.min),
            ))
        })
        .collect();
    for (e, f) in values {
        if let Some(mut n) = world.get_mut::<Node>(e) {
            let width = Val::Percent(f.clamp(0., 1.) * 100.);
            if n.width != width {
                n.width = width;
            }
        }
    }
    if !pressed && world.remove_resource::<SliderEditing>().is_some() {
        tools::persist_settings(world);
        world.resource_mut::<PlayState>().dirty = true;
    }
}
pub(super) fn scroll(
    mut wheel: EventReader<MouseWheel>,
    state: Res<PlayState>,
    mut panels: Query<
        (&RelativeCursorPosition, &ComputedNode, &mut ScrollPosition),
        With<ScrollPane>,
    >,
) {
    for event in wheel.read() {
        if !state.menu_open {
            continue;
        }
        let delta = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y * state.layout.scroll_step,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
        };
        for (cursor, node, mut offset) in &mut panels {
            if cursor.mouse_over() {
                let max = ((node.content_size.y - node.size.y) * node.inverse_scale_factor).max(0.);
                offset.offset_y = (offset.offset_y - delta).clamp(0., max);
            }
        }
    }
}
pub(super) fn hover(
    mut buttons: Query<(
        &Interaction,
        &BaseColor,
        &mut BackgroundColor,
        Option<&ModelHint>,
    )>,
    mut hints: Query<&mut Text, With<HintLine>>,
) {
    let mut hint = "Hover an icon for its model name";
    for (interaction, base, mut background, model) in &mut buttons {
        background.0 = match interaction {
            Interaction::Pressed => Color::srgb(0.48, 0.65, 0.82),
            Interaction::Hovered => Color::srgb(0.74, 0.85, 0.95),
            Interaction::None => base.0,
        };
        if *interaction != Interaction::None {
            if let Some(model) = model {
                hint = &model.0;
            }
        }
    }
    for mut text in &mut hints {
        if text.0 != hint {
            text.0 = hint.into();
        }
    }
}
