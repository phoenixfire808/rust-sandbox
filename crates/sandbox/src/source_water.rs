//! Original water normals with Bevy's transmissive PBR path. Not the Source planar-water shader.
use crate::source_assets::Surface;
use bevy::prelude::*;

#[derive(Component)]
pub struct WaterMotion(pub Vec2);

pub fn spawn_surface(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    textures: &std::collections::BTreeMap<String, Handle<Image>>,
    mut surface: Surface,
    config: &sandbox_catalog::spawn::Water,
) {
    let Some(water) = surface.water.take() else {
        return;
    };
    // Source swaps above/below shaders at the same boundary. Render one two-sided boundary,
    // not both coplanar material groups. Native underwater fog remains a separate feature.
    if !water.above {
        return;
    }
    surface.geometry.uv = surface
        .geometry
        .positions
        .iter()
        .map(|p| [p[0] * config.uv_scale, p[2] * config.uv_scale])
        .collect();
    let mut mesh = surface.geometry.mesh();
    let normal = water.normal.and_then(|key| textures.get(&key).cloned());
    let normal = match mesh.generate_tangents() {
        Ok(()) => normal,
        Err(error) => {
            eprintln!("WATER_TANGENTS {}: {error}", surface.name);
            None
        }
    };
    let fog = surface.tint.to_srgba();
    let mix = config.tint_mix;
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(
            1. - mix + fog.red * mix,
            1. - mix + fog.green * mix,
            1. - mix + fog.blue * mix,
        ),
        normal_map_texture: normal,
        flip_normal_map_y: true,
        perceptual_roughness: config.roughness,
        reflectance: config.reflectance,
        specular_transmission: config.transmission,
        ior: config.ior,
        thickness: config.thickness,
        attenuation_color: surface.tint,
        attenuation_distance: water.fog_distance,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    commands.spawn((
        Name::new(surface.name),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        WaterMotion(Vec2::new(config.scroll_x, config.scroll_y)),
    ));
}

pub fn animate(
    time: Res<Time>,
    surfaces: Query<(&MeshMaterial3d<StandardMaterial>, &WaterMotion)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (handle, motion) in &surfaces {
        if let Some(material) = materials.get_mut(&handle.0) {
            let offset = motion.0 * time.elapsed_secs();
            material.uv_transform = bevy::math::Affine2::from_translation(Vec2::new(
                offset.x.rem_euclid(1.),
                offset.y.rem_euclid(1.),
            ));
        }
    }
}
