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
    world
        .entity_mut(creation)
        .insert(BackgroundColor(Color::srgb(0.82, 0.83, 0.84)));
    let tabs = row(world, creation, c.tab_height);
    action(
        world,
        tabs,
        "Spawnlists",
        UiAction::Category(category.clone()),
        true,
    );
    for label in ["Weapons", "NPCs", "Entities", "Vehicles"] {
        let e = action(
            world,
            tabs,
            label,
            UiAction::Unavailable(label.into()),
            false,
        );
        world
            .entity_mut(e)
            .insert(BackgroundColor(Color::srgb(0.72, 0.73, 0.74)));
    }
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
    let categories: std::collections::BTreeSet<_> =
        models.iter().map(|m| m.category.clone()).collect();
    for cat in categories {
        action(
            world,
            tree,
            format!("  {cat}"),
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
            (category.is_empty() || m.category == category)
                && m.model.to_lowercase().contains(&search.to_lowercase())
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
    match tab {
        0 => {
            text(world, list, "Construction", c.font_size);
            for (id, label) in [
                ("remover", "Remover"),
                ("duplicator", "Duplicator"),
                ("freeze", "Freeze"),
            ] {
                action(world, list, label, UiAction::Tool(id.into()), tool == id);
            }
            text(world, list, "Not implemented", c.font_size);
            for spec in crate::compiled_behaviors()
                .iter()
                .filter(|s| s.id.starts_with("tool_"))
            {
                let id = spec.id.trim_start_matches("tool_");
                if ["remover", "duplicator"].contains(&id) {
                    continue;
                }
                action(
                    world,
                    list,
                    format!("{id} (pending)"),
                    UiAction::Unavailable(id.into()),
                    false,
                );
            }
            text(world, controls, &tool, c.font_size + 2.);
            let description = match tool.as_str() {
                "remover" => "Left click: remove the targeted prop.",
                "duplicator" => {
                    "Left click: duplicate one prop. Constrained assemblies are not supported yet."
                }
                _ => "Left click: freeze the targeted prop.",
            };
            text(world, controls, description, c.font_size);
            text(
                world,
                controls,
                "Right click: freeze.\n\nFull stock tool options are still pending.",
                c.font_size,
            );
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
        }
    }
    let bottom = row(world, tools, c.row_height);
    action(world, bottom, "Close", UiAction::Close, false);
    text(world, bottom, "Hold Q to build", c.font_size);
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
