//! Render the offscreen toolgun display and contextual gameplay HUD help.
use super::*;
use bevy::render::{camera::RenderTarget, render_resource::TextureUsages, view::RenderLayers};

#[derive(Resource)]
struct Screen {
    image: Handle<Image>,
    camera: Entity,
    label: String,
}
#[derive(Resource)]
struct HudHelp {
    panel: Entity,
    title: Entity,
    description: Entity,
    action: Entity,
}
#[derive(Component)]
struct Marquee;
#[derive(Component)]
struct Caption(bool);

pub(super) fn ensure(world: &mut World) -> Result<Handle<Image>> {
    if let Some(s) = world.get_resource::<Screen>() {
        return Ok(s.image.clone());
    }
    let c = world.resource::<PlayState>().tools.catalog.gun.clone();
    let source = world.resource::<MountedSource>();
    let background = source
        .mounts
        .texture(&source.bsp, &c.screen_background, false)?;
    let bg = world.resource_mut::<Assets<Image>>().add(background);
    let mut image = Image::new_fill(
        Extent3d {
            width: c.screen_size,
            height: c.screen_size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let image = world.resource_mut::<Assets<Image>>().add(image);
    let camera = world
        .spawn((
            Camera2d,
            Camera {
                order: -1,
                target: RenderTarget::Image(image.clone().into()),
                ..default()
            },
            RenderLayers::layer(2),
        ))
        .id();
    let root = world
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                overflow: Overflow::clip(),
                ..default()
            },
            ImageNode::new(bg),
            UiTargetCamera(camera),
        ))
        .id();
    // One measured cycle plus duplicates covers scrolling without a hard-coded character width.
    let row = world
        .spawn((
            Marquee,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(c.text_center_y - c.font_size * 0.5),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ChildOf(root),
        ))
        .id();
    for i in 0..3 {
        world.spawn((
            Text::new("TOOL"),
            Caption(i == 0),
            TextFont {
                font_size: c.font_size,
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new_with_no_wrap(),
            Node {
                flex_shrink: 0.,
                margin: UiRect::right(Val::Px(64.)),
                ..default()
            },
            ChildOf(row),
        ));
    }
    world.insert_resource(Screen {
        image: image.clone(),
        camera,
        label: String::new(),
    });
    Ok(image)
}
fn ensure_hud(world: &mut World) {
    if world.contains_resource::<HudHelp>() {
        return;
    }
    let Some(camera) = world
        .query_filtered::<Entity, With<SourceCamera>>()
        .iter(world)
        .next()
    else {
        return;
    };
    let panel = world
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.),
                top: Val::Px(40.),
                width: Val::Percent(60.),
                padding: UiRect::new(
                    Val::Px(50.),
                    Val::Px(16.),
                    Val::Px(12.),
                    Val::Px(10.),
                ),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.13, 0.15, 0.68)),
            UiTargetCamera(camera),
        ))
        .id();
    let title = world
        .spawn((
            Text::new(""),
            TextFont {
                font_size: 70.,
                ..default()
            },
            TextColor(Color::WHITE),
            ChildOf(panel),
        ))
        .id();
    let description = world
        .spawn((
            Text::new(""),
            TextFont {
                font_size: 20.,
                ..default()
            },
            TextColor(Color::WHITE),
            ChildOf(panel),
        ))
        .id();
    let action = world
        .spawn((
            Text::new(""),
            TextFont {
                font_size: 16.,
                ..default()
            },
            TextColor(Color::srgb(0.35, 0.86, 1.)),
            Node {
                width: Val::Percent(100.),
                padding: UiRect::axes(Val::Px(10.), Val::Px(4.)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.06, 0.78)),
            ChildOf(panel),
        ))
        .id();
    world.insert_resource(HudHelp {
        panel,
        title,
        description,
        action,
    });
}

pub(super) fn update(world: &mut World) {
    ensure_hud(world);
    update_help_hud(world);
    if !world.contains_resource::<Screen>() {
        return;
    }
    let play = world.resource::<PlayState>();
    let label = play.tools.label(&play.tool).to_owned();
    let active = play.active_weapon == "weapon_gmod_tool"
        && (play.weapon_visible
            || world
                .get_resource::<crate::source_player::PlayerState>()
                .is_some_and(|p| p.third_person));
    let speed = play.tools.catalog.gun.scroll_speed;
    let time = world.resource::<Time>().elapsed_secs();
    let camera = world.resource::<Screen>().camera;
    if let Some(mut cam) = world.get_mut::<Camera>(camera) {
        if cam.is_active != active {
            cam.is_active = active;
        }
    }
    if world.resource::<Screen>().label != label {
        for mut text in world
            .query_filtered::<&mut Text, With<Caption>>()
            .iter_mut(world)
        {
            text.0 = label.clone();
        }
        world.resource_mut::<Screen>().label = label;
    }
    if !active {
        return;
    }
    let width = world
        .query::<(&Caption, &ComputedNode)>()
        .iter(world)
        .find(|(c, _)| c.0)
        .map(|(_, n)| n.size.x * n.inverse_scale_factor + 64.)
        .unwrap_or(256.);
    if width > 0. {
        for mut node in world
            .query_filtered::<&mut Node, With<Marquee>>()
            .iter_mut(world)
        {
            node.left = Val::Px(-(time * speed).rem_euclid(width));
        }
    }
}

fn update_help_hud(world: &mut World) {
    if !world.contains_resource::<HudHelp>() {
        return;
    }
    let Some(play) = world.get_resource::<PlayState>() else {
        return;
    };
    let menu_open = play.menu_open;
    let active_toolgun = play.active_weapon == "weapon_gmod_tool";
    let reference_height = play.layout.reference_height;
    let selected = play.tool.clone();
    let definition = play
        .tools
        .catalog
        .tools
        .iter()
        .find(|tool| tool.id == selected)
        .map(|tool| (tool.label.clone(), tool.left.clone()));
    let context_blocked = menu_open
        || crate::source_frontend::active(world)
        || super::npcs::dead(world)
        || super::tools::devices::viewing(world)
        || world
            .get_resource::<crate::source_player::PlayerState>()
            .is_some_and(|player| player.vehicle.is_some())
        || !world.query::<&Window>().iter(world).any(|window| window.focused);
    if context_blocked {
        let panel = world.resource::<HudHelp>().panel;
        if let Some(mut node) = world.get_mut::<Node>(panel) {
            node.display = Display::None;
        }
        return;
    }
    let button_action = if active_toolgun {
        None
    } else {
        super::tools::devices::button_hint(world)
    };
    let title = if button_action.is_some() {
        "Button".to_owned()
    } else {
        definition
            .as_ref()
            .map(|(label, _)| label.clone())
            .unwrap_or_default()
    };
    let description = if button_action.is_some() {
        "Press E to use this button.".to_owned()
    } else {
        definition
            .map(|(_, description)| description)
            .unwrap_or_default()
    };
    let action = if active_toolgun {
        super::tools::help_hint(world)
    } else {
        button_action
    };
    let visible = active_toolgun || action.is_some();
    let hud = world.resource::<HudHelp>();
    let (panel, title_entity, description_entity, action_entity) = (
        hud.panel,
        hud.title,
        hud.description,
        hud.action,
    );
    let window_height = world
        .query::<&Window>()
        .iter(world)
        .next()
        .map(Window::height)
        .unwrap_or(reference_height);
    let scale = ((window_height / reference_height) * (reference_height / 720.))
        .clamp(0.5, 1.5);
    if let Some(mut node) = world.get_mut::<Node>(panel) {
        node.left = Val::Px(0.);
        node.top = Val::Px(40. * scale);
        node.width = Val::Percent(60.);
        node.padding = UiRect::new(
            Val::Px(50. * scale),
            Val::Px(16. * scale),
            Val::Px(12. * scale),
            Val::Px(10. * scale),
        );
        node.display = if visible { Display::Flex } else { Display::None };
    }
    if let Some(mut font) = world.get_mut::<TextFont>(title_entity) {
        font.font_size = 70. * scale;
    }
    if let Some(mut font) = world.get_mut::<TextFont>(description_entity) {
        font.font_size = 20. * scale;
    }
    if let Some(mut node) = world.get_mut::<Node>(action_entity) {
        node.width = Val::Percent(100.);
        node.padding = UiRect::axes(Val::Px(10. * scale), Val::Px(4. * scale));
    }
    if let Some(mut font) = world.get_mut::<TextFont>(action_entity) {
        font.font_size = 18. * scale;
    }
    for (entity, value) in [
        (title_entity, title),
        (description_entity, description),
        (action_entity, action.map(|text| format!("ℹ  {text}")).unwrap_or_default()),
    ] {
        if let Some(mut text) = world.get_mut::<Text>(entity) {
            if text.0 != value {
                text.0 = value;
            }
        }
    }
}
