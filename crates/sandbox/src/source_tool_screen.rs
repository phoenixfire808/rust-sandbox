//! Render the original toolgun screen surface from a dedicated Bevy UI camera.
use super::*;
use bevy::render::{camera::RenderTarget, render_resource::TextureUsages, view::RenderLayers};

#[derive(Resource)]
struct Screen {
    image: Handle<Image>,
    camera: Entity,
    label: String,
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
pub(super) fn update(world: &mut World) {
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
