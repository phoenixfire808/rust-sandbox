//! Bounded physgun presentation using read-only mounted textures, not Source renderer parity.
use crate::{
    compiled_effects_config,
    source_assets::MountedSource,
    source_play::{PlayState, SourceCamera},
    source_player::PlayerState,
};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        mesh::Indices,
        render_resource::PrimitiveTopology,
        view::{NoFrustumCulling, RenderLayers},
    },
};
use sandbox_catalog::{effects::EffectsConfig, Result};

pub struct SourceEffectsPlugin;
impl Plugin for SourceEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update.after(crate::source_player::visuals));
    }
}
#[derive(Component)]
struct Glow;
#[derive(Component)]
struct Beam;
#[derive(Resource)]
struct Effects {
    config: EffectsConfig,
    glows: Vec<Entity>,
    endpoint: Entity,
    beam: Entity,
    beam_mesh: Handle<Mesh>,
}
#[derive(Resource)]
struct Failed;

fn material(world: &mut World, image: Image, c: &EffectsConfig) -> Handle<StandardMaterial> {
    let texture = world.resource_mut::<Assets<Image>>().add(image);
    world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::srgb(c.red, c.green, c.blue),
            base_color_texture: Some(texture),
            unlit: true,
            alpha_mode: AlphaMode::Add,
            cull_mode: None,
            ..default()
        })
}
fn create(world: &mut World, glow: Image, beam: Image) -> Effects {
    let c = compiled_effects_config();
    let glow_mat = material(world, glow, &c);
    let beam_mat = material(world, beam, &c);
    let quad = world
        .resource_mut::<Assets<Mesh>>()
        .add(Rectangle::new(1., 1.));
    // Fixed pool: one core and up to three tip glows plus one world endpoint.
    let mut glows = Vec::new();
    for _ in 0..5 {
        glows.push(
            world
                .spawn((
                    Glow,
                    Mesh3d(quad.clone()),
                    MeshMaterial3d(glow_mat.clone()),
                    Transform::default(),
                    Visibility::Hidden,
                    NoFrustumCulling,
                    RenderLayers::layer(0),
                ))
                .id(),
        );
    }
    let endpoint = glows.pop().unwrap();
    let beam_mesh = world.resource_mut::<Assets<Mesh>>().add(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.; 3]; 4])
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 4])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.; 2]; 4])
        .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3])),
    );
    let beam = world
        .spawn((
            Beam,
            Mesh3d(beam_mesh.clone()),
            MeshMaterial3d(beam_mat),
            Transform::default(),
            Visibility::Hidden,
            NoFrustumCulling,
            RenderLayers::layer(0),
        ))
        .id();
    Effects {
        config: c,
        glows,
        endpoint,
        beam,
        beam_mesh,
    }
}
fn load(world: &mut World) -> Result<Effects> {
    let c = compiled_effects_config();
    let source = world.resource::<MountedSource>();
    let glow = source.mounts.texture(&source.bsp, &c.glow_texture, false)?;
    let beam = source.mounts.texture(&source.bsp, &c.beam_texture, true)?;
    println!(
        "PHYSGUN_EFFECT_TEXTURES_OK glow={}x{} beam={}x{}",
        glow.width(),
        glow.height(),
        beam.width(),
        beam.height()
    );
    Ok(create(world, glow, beam))
}
/// A camera-facing ribbon. Looking along its axis uses a stable orthogonal fallback.
fn ribbon(start: Vec3, end: Vec3, camera: Vec3, width: f32) -> Option<[[f32; 3]; 4]> {
    if !start.is_finite()
        || !end.is_finite()
        || !camera.is_finite()
        || !width.is_finite()
        || width <= 0.
    {
        return None;
    }
    let delta = end - start;
    if delta.length_squared() < 1e-8 {
        return None;
    }
    let axis = delta.normalize();
    let mut side = axis.cross(camera - (start + end) * 0.5).normalize_or_zero();
    if side.length_squared() < 0.5 {
        side = axis
            .cross(if axis.y.abs() < 0.9 { Vec3::Y } else { Vec3::X })
            .normalize();
    }
    let offset = side * width * 0.5;
    Some([
        (start - offset).to_array(),
        (start + offset).to_array(),
        (end + offset).to_array(),
        (end - offset).to_array(),
    ])
}
/// Match the viewmodel's screen-space muzzle to the world's different FOV.
fn world_muzzle(eye: Transform, muzzle: Vec3, view_fov: f32, world_fov: f32) -> Vec3 {
    let mut local = eye.rotation.inverse() * (muzzle - eye.translation);
    let ratio = (world_fov * 0.5).tan() / (view_fov * 0.5).tan();
    local.x *= ratio;
    local.y *= ratio;
    eye.translation + eye.rotation * local
}
fn show(
    world: &mut World,
    entity: Entity,
    position: Vec3,
    rotation: Quat,
    size: f32,
    layer: usize,
) {
    world.entity_mut(entity).insert((
        Transform::from_translation(position)
            .with_rotation(rotation)
            .with_scale(Vec3::splat(size)),
        Visibility::Visible,
        RenderLayers::layer(layer),
    ));
}
fn update(world: &mut World) {
    let Some(player) = world.get_resource::<PlayerState>() else {
        return;
    };
    if !world.contains_resource::<Effects>() {
        if player.glow_points.is_empty() {
            return;
        }
        if world.contains_resource::<Failed>() || !world.contains_resource::<MountedSource>() {
            return;
        }
        match load(world) {
            Ok(fx) => world.insert_resource(fx),
            Err(e) => {
                eprintln!("PHYSGUN_EFFECTS_FAILED: {e}");
                world.resource_mut::<PlayState>().status = format!("Physgun effects failed: {e}");
                world.insert_resource(Failed);
                return;
            }
        }
    }
    world.resource_scope(|world, fx: Mut<Effects>| {
        for &entity in fx.glows.iter().chain([&fx.endpoint, &fx.beam]) {
            *world.get_mut::<Visibility>(entity).unwrap() = Visibility::Hidden;
        }
        let target = crate::source_play::beam_target(world);
        let play = world.resource::<PlayState>();
        let player = world.resource::<PlayerState>();
        let physgun = play.physgun;
        let beam_width = if physgun {
            fx.config.beam_width
        } else {
            play.tools.catalog.gun.tracer_width
        };
        if (!physgun && target.is_none()) || (!player.third_person && !play.weapon_visible) {
            return;
        }
        let third = player.third_person;
        let eye = player.eye;
        let muzzle = player.muzzle;
        let points = player.glow_points.clone();
        let view_fov = player.config.view_fov.to_radians();
        let Some((camera, fov)) = world
            .query_filtered::<(&Transform, &Projection), With<SourceCamera>>()
            .iter(world)
            .next()
            .map(|(t, p)| {
                (
                    *t,
                    match p {
                        Projection::Perspective(p) => p.fov,
                        _ => view_fov,
                    },
                )
            })
        else {
            return;
        };
        let seconds = world.resource::<Time>().elapsed_secs();
        let pulse = 1.
            + fx.config.pulse_depth * (seconds * fx.config.pulse_hz * std::f32::consts::TAU).sin();
        for (&entity, &(position, core)) in fx.glows.iter().zip(&points) {
            if position.is_finite() && (physgun || core) {
                show(
                    world,
                    entity,
                    position,
                    if third { camera.rotation } else { eye.rotation },
                    pulse
                        * if core {
                            fx.config.core_size
                        } else {
                            fx.config.tip_size
                        },
                    usize::from(!third),
                );
            }
        }
        if let Some(target) = target {
            let start = if third {
                muzzle
            } else {
                world_muzzle(eye, muzzle, view_fov, fov)
            };
            if let Some(positions) = ribbon(start, target, camera.translation, beam_width) {
                let scroll = (seconds * fx.config.beam_scroll).rem_euclid(1.);
                let length = start.distance(target) / fx.config.beam_repeat;
                let mut meshes = world.resource_mut::<Assets<Mesh>>();
                let mesh = meshes.get_mut(&fx.beam_mesh).unwrap();
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.to_vec());
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_UV_0,
                    vec![
                        [0., scroll],
                        [1., scroll],
                        [1., scroll + length],
                        [0., scroll + length],
                    ],
                );
                *world.get_mut::<Visibility>(fx.beam).unwrap() = Visibility::Visible;
                show(
                    world,
                    fx.endpoint,
                    target,
                    camera.rotation,
                    fx.config.endpoint_size * pulse,
                    0,
                );
            }
        }
    });
}
/// Reads actual renderer entities, used by the graphical acceptance fixture.
pub fn visible_counts(world: &mut World) -> Option<(usize, bool)> {
    if !world.contains_resource::<Effects>() {
        return None;
    }
    let glows = world
        .query_filtered::<&Visibility, With<Glow>>()
        .iter(world)
        .filter(|v| **v == Visibility::Visible)
        .count();
    let beam = world
        .query_filtered::<&Visibility, With<Beam>>()
        .iter(world)
        .any(|v| *v == Visibility::Visible);
    Some((glows, beam))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn effects_reuse_resources_and_clean_up_on_switch_hide_release_and_deletion() {
        let mut app = App::new();
        app.insert_resource(Assets::<Mesh>::default())
            .insert_resource(Assets::<Image>::default())
            .insert_resource(Assets::<StandardMaterial>::default())
            .insert_resource(Time::<()>::default())
            .insert_resource(PlayState::new(Vec::new(), 75., 8.))
            .add_systems(Startup, |mut commands: Commands| {
                crate::source_player::spawn(&mut commands, Vec3::ZERO, Vec3::NEG_Z);
            });
        app.update();
        let world = app.world_mut();
        world.spawn((
            SourceCamera,
            Transform::default(),
            Projection::Perspective(PerspectiveProjection::default()),
        ));
        let target = world.spawn(Transform::from_xyz(0., 0., -5.)).id();
        let fx = create(world, Image::default(), Image::default());
        world.insert_resource(fx);
        world.resource_mut::<PlayState>().weapon_visible = true;
        world.resource_mut::<PlayState>().held = Some(target);
        world.resource_mut::<PlayerState>().glow_points = vec![(Vec3::new(0.2, -0.1, -0.7), true)];
        world.resource_mut::<PlayerState>().muzzle = Vec3::new(0.2, -0.1, -0.7);
        let meshes = world.resource::<Assets<Mesh>>().len();
        let materials = world.resource::<Assets<StandardMaterial>>().len();
        for _ in 0..100 {
            update(world);
        }
        assert_eq!(visible_counts(world), Some((2, true)));
        let beam_mesh = &world.resource::<Effects>().beam_mesh;
        let mesh = world.resource::<Assets<Mesh>>().get(beam_mesh).unwrap();
        let bevy::render::mesh::VertexAttributeValues::Float32x2(uvs) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!("beam requires 2D UVs");
        };
        assert_eq!(
            uvs.iter().map(|uv| uv[0]).collect::<Vec<_>>(),
            vec![0., 1., 1., 0.]
        );
        assert_eq!(uvs[0][1], uvs[1][1]);
        assert!(uvs[2][1] > uvs[0][1]);
        assert_eq!(uvs[2][1], uvs[3][1]);
        assert_eq!(world.resource::<Assets<Mesh>>().len(), meshes);
        assert_eq!(
            world.resource::<Assets<StandardMaterial>>().len(),
            materials
        );
        let glow = world.resource::<Effects>().glows[0];
        assert_eq!(
            world.get::<RenderLayers>(glow).unwrap(),
            &RenderLayers::layer(1)
        );
        world.resource_mut::<PlayerState>().third_person = true;
        update(world);
        assert_eq!(
            world.get::<RenderLayers>(glow).unwrap(),
            &RenderLayers::layer(0)
        );
        world.resource_mut::<PlayState>().held = None;
        update(world);
        assert_eq!(visible_counts(world), Some((1, false)));
        world.resource_mut::<PlayState>().held = Some(target);
        world.despawn(target);
        update(world);
        assert_eq!(visible_counts(world), Some((1, false)));
        world.resource_mut::<PlayState>().physgun = false;
        update(world);
        assert_eq!(visible_counts(world), Some((0, false)));
        world.resource_mut::<PlayState>().physgun = true;
        world.resource_mut::<PlayState>().weapon_visible = false;
        world.resource_mut::<PlayerState>().third_person = false;
        update(world);
        assert_eq!(visible_counts(world), Some((0, false)));
    }
    #[test]
    fn ribbon_is_finite_at_axis_alignment_and_rejects_degenerate_input() {
        for camera in [Vec3::ZERO, Vec3::Z * 5., Vec3::X * 4.] {
            let points = ribbon(Vec3::ZERO, Vec3::Z, camera, 0.1).unwrap();
            assert!(points.iter().flatten().all(|x| x.is_finite()));
            assert!(
                (Vec3::from_array(points[0]).distance(Vec3::from_array(points[1])) - 0.1).abs()
                    < 1e-5
            );
        }
        assert!(ribbon(Vec3::ZERO, Vec3::ZERO, Vec3::ONE, 0.1).is_none());
        assert!(ribbon(Vec3::NAN, Vec3::ONE, Vec3::ONE, 0.1).is_none());
    }
    #[test]
    fn muzzle_projection_preserves_screen_position_across_fovs() {
        let eye = Transform::from_xyz(2., 3., 4.).with_rotation(Quat::from_rotation_y(0.7));
        let local = Vec3::new(0.2, -0.1, -1.);
        let point = eye.translation + eye.rotation * local;
        let corrected = world_muzzle(eye, point, 50f32.to_radians(), 75f32.to_radians());
        let back = eye.rotation.inverse() * (corrected - eye.translation);
        assert!(
            (local.x / 25f32.to_radians().tan() - back.x / 37.5f32.to_radians().tan()).abs() < 1e-5
        );
        assert!((back.z - local.z).abs() < 1e-5);
    }
}
