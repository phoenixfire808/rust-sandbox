//! Persistent paint and bounded camera-facing ribbon trails; not Source projected decals.
use super::*;
use bevy::{
    asset::RenderAssetUsages,
    render::{mesh::Indices, render_resource::PrimitiveTopology, view::NoFrustumCulling},
};
use std::collections::VecDeque;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedPaint {
    pub target: Option<usize>,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub material: String,
    pub size: f32,
}
#[derive(Component, Clone)]
struct Paint {
    owner: Option<Entity>,
    data: SavedPaint,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Trail {
    material: String,
    seconds: f32,
    start: f32,
    end: f32,
    color: [f32; 4],
}
#[derive(Component, Clone)]
struct Ribbon {
    owner: Entity,
    settings: Trail,
    points: VecDeque<(f32, Vec3)>,
    mesh: Handle<Mesh>,
}
fn allowed(world: &World, tool: &str, key: &str, value: &str) -> bool {
    world
        .resource::<PlayState>()
        .tools
        .catalog
        .options
        .iter()
        .any(|o| o.tool == tool && o.key == key && o.enabled && o.accepts(value))
}
pub(crate) fn validate_trail(world: &World, t: &Trail) -> Result<()> {
    if !allowed(world, "trails", "material", &t.material)
        || !allowed(world, "trails", "length", &t.seconds.to_string())
        || !allowed(world, "trails", "startsize", &t.start.to_string())
        || !allowed(world, "trails", "endsize", &t.end.to_string())
        || t.color
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
    {
        return Err("invalid trail configuration".into());
    }
    Ok(())
}
pub(crate) fn validate_paint(world: &mut World, p: &SavedPaint, count: usize) -> Result<()> {
    if p.target.is_some_and(|i| i >= count)
        || p.point.iter().any(|v| !v.is_finite() || v.abs() > 1e6)
        || p.normal.iter().any(|v| !v.is_finite())
        || (Vec3::from_array(p.normal).length() - 1.).abs() > 0.01
        || !allowed(world, "paint", "size", &p.size.to_string())
        || !allowed(world, "paint", "material", &p.material)
    {
        return Err("invalid painted mark".into());
    }
    material(world, &p.material)?;
    Ok(())
}
pub(crate) fn prepare_trail(world: &mut World, t: &Trail) -> Result<()> {
    validate_trail(world, t)?;
    material(world, &t.material)?;
    Ok(())
}
pub(crate) fn count(world: &mut World) -> usize {
    world.query::<&Paint>().iter(world).count()
}
pub(crate) fn mark(world: &mut World, owner: Option<Entity>, p: SavedPaint) -> Result<()> {
    let mut mat = material(world, &p.material)?;
    mat.alpha_mode = AlphaMode::Blend;
    mat.cull_mode = None;
    let mat = world.resource_mut::<Assets<StandardMaterial>>().add(mat);
    let mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(Rectangle::new(p.size, p.size));
    let transform = Transform::from_translation(Vec3::from_array(p.point))
        .with_rotation(Quat::from_rotation_arc(Vec3::Z, Vec3::from_array(p.normal)));
    let e = world
        .spawn((
            Paint { owner, data: p },
            impacts::ImpactMark,
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            transform,
            NoFrustumCulling,
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ))
        .id();
    if let Some(parent) = owner {
        world.entity_mut(e).insert(ChildOf(parent));
    }
    Ok(())
}
pub(crate) fn snapshot(world: &mut World, ids: &[Entity], include_world: bool) -> Vec<SavedPaint> {
    world
        .query::<&Paint>()
        .iter(world)
        .filter_map(|p| {
            let target = p.owner.and_then(|e| ids.iter().position(|i| *i == e));
            if (p.owner.is_some() && target.is_none()) || (p.owner.is_none() && !include_world) {
                return None;
            }
            let mut d = p.data.clone();
            d.target = target;
            Some(d)
        })
        .collect()
}
pub(crate) fn clear(world: &mut World) {
    let ids: Vec<_> = world
        .query_filtered::<Entity, Or<(With<Paint>, With<Ribbon>)>>()
        .iter(world)
        .collect();
    for e in ids {
        world.despawn(e);
    }
}
pub(crate) fn apply_trail(world: &mut World, owner: Entity, settings: Option<Trail>) -> Result<()> {
    let old: Vec<_> = world
        .query::<(Entity, &Ribbon)>()
        .iter(world)
        .filter(|(_, r)| r.owner == owner)
        .map(|(e, _)| e)
        .collect();
    for e in old {
        world.despawn(e);
    }
    if let Some(t) = settings {
        validate_trail(world, &t)?;
        let mut mat = material(world, &t.material)?;
        mat.alpha_mode = AlphaMode::Blend;
        mat.cull_mode = None;
        mat.base_color = Color::srgba(t.color[0], t.color[1], t.color[2], t.color[3]);
        let mat = world.resource_mut::<Assets<StandardMaterial>>().add(mat);
        let mesh = world.resource_mut::<Assets<Mesh>>().add(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        ));
        world.spawn((
            Ribbon {
                owner,
                settings: t,
                points: VecDeque::new(),
                mesh: mesh.clone(),
            },
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::default(),
            NoFrustumCulling,
            bevy::pbr::NotShadowCaster,
        ));
    }
    Ok(())
}
pub(crate) fn operate(world: &mut World, tool: &str, action: u8, hit: Target) -> Result<bool> {
    if world.get::<npcs::NpcBody>(hit.entity).is_some() {
        return Err("Live NPC render modifiers remain unsupported".into());
    }
    if tool == "paint" {
        let owner = world.get::<SpawnedProp>(hit.entity).map(|_| hit.entity);
        if action == 3 {
            let ids: Vec<_> = world
                .query::<(Entity, &Paint)>()
                .iter(world)
                .filter(|(_, p)| p.owner == owner)
                .map(|(e, _)| e)
                .collect();
            if ids.is_empty() {
                return Ok(false);
            }
            remember(world);
            for e in ids {
                world.despawn(e);
            }
            return Ok(true);
        }
        if world.query::<&Paint>().iter(world).count()
            >= world.resource::<PlayState>().tools.catalog.gun.max_marks
        {
            return Err("paint limit reached; reload removes marks on the target".into());
        }
        let play = world.resource::<PlayState>();
        let mut point = hit.point + hit.normal * play.spawn_catalog.runtime.impact_offset;
        let mut normal = hit.normal;
        if let Some(e) = owner {
            let t = world.get::<Transform>(e).ok_or("paint target removed")?;
            point = t.compute_matrix().inverse().transform_point3(point);
            normal = t.rotation.inverse() * normal;
        }
        let p = SavedPaint {
            target: None,
            point: point.to_array(),
            normal: normal.to_array(),
            material: play.tools.value("paint", "material"),
            size: play.tools.number("paint", "size"),
        };
        validate_paint(world, &p, 0)?;
        remember(world);
        mark(world, owner, p)?;
    } else {
        if world.get::<SpawnedProp>(hit.entity).is_none() {
            return Err("Trails require a prop or construction device".into());
        }
        let mut props = world
            .get::<Properties>(hit.entity)
            .cloned()
            .unwrap_or_default();
        props.trail = if action == 1 {
            let p = world.resource::<PlayState>();
            Some(Trail {
                material: p.tools.value("trails", "material"),
                seconds: p.tools.number("trails", "length"),
                start: p.tools.number("trails", "startsize"),
                end: p.tools.number("trails", "endsize"),
                color: ["r", "g", "b", "a"].map(|k| p.tools.number("trails", k) / 255.),
            })
        } else {
            None
        };
        prepare_properties(world, &props)?;
        remember(world);
        apply_properties(world, hit.entity, props)?;
    }
    Ok(true)
}
pub(crate) fn update(world: &mut World) {
    let now = world.resource::<Time>().elapsed_secs();
    let max = world.resource::<PlayState>().tools.catalog.gun.trail_points;
    let eye = world
        .query_filtered::<&Transform, With<SourceCamera>>()
        .iter(world)
        .next()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);
    let rows: Vec<_> = world
        .query::<(Entity, &Ribbon)>()
        .iter(world)
        .map(|(e, r)| (e, r.clone()))
        .collect();
    for (e, mut r) in rows {
        let Some(t) = world.get::<Transform>(r.owner) else {
            world.despawn(e);
            continue;
        };
        if r.points
            .back()
            .is_none_or(|(time, _)| now - time >= r.settings.seconds / (max - 1) as f32)
        {
            r.points.push_back((now, t.translation));
        }
        while r.points.len() > max
            || r.points
                .front()
                .is_some_and(|(time, _)| now - time > r.settings.seconds)
        {
            r.points.pop_front();
        }
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uv = Vec::new();
        let mut indices = Vec::new();
        let points: Vec<_> = r.points.iter().copied().collect();
        for segment in points.windows(2) {
            let (ta, a) = segment[0];
            let (tb, b) = segment[1];
            let tangent = (b - a).normalize_or_zero();
            if tangent == Vec3::ZERO {
                continue;
            }
            let side = tangent.cross(eye - (a + b) * 0.5).normalize_or_zero();
            let wa = (now - ta) / r.settings.seconds;
            let wb = (now - tb) / r.settings.seconds;
            let sa = (r.settings.start + (r.settings.end - r.settings.start) * wa) * 0.5;
            let sb = (r.settings.start + (r.settings.end - r.settings.start) * wb) * 0.5;
            let base = positions.len() as u32;
            positions.extend(
                [a - side * sa, a + side * sa, b - side * sb, b + side * sb].map(|p| p.to_array()),
            );
            normals.extend([[0., 1., 0.]; 4]);
            uv.extend([[0., wa], [1., wa], [0., wb], [1., wb]]);
            indices.extend([base, base + 1, base + 2, base + 2, base + 1, base + 3]);
        }
        if let Some(mesh) = world.resource_mut::<Assets<Mesh>>().get_mut(&r.mesh) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
            mesh.insert_indices(Indices::U32(indices));
        }
        if let Some(mut live) = world.get_mut::<Ribbon>(e) {
            live.points = r.points;
        }
    }
}
