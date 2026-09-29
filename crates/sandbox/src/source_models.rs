//! Static Source MDL/VVD/VTX geometry. Animation and PHY hulls are separate work.
use crate::source_assets::{source_position, Geometry, Mounts};
use bevy::prelude::*;
use sandbox_catalog::Result;
use std::collections::BTreeMap;
use vbsp::AsPropPlacement;

pub fn model_parts(
    bsp: &vbsp::Bsp,
    mounts: &Mounts,
    path: &str,
    skin: i32,
) -> Result<Vec<(String, Geometry)>> {
    let stem = path
        .strip_suffix(".mdl")
        .ok_or("model must have .mdl extension")?;
    let mut bytes = mounts.read(bsp, path)?;
    // Geometry decoding must not eagerly decode unrelated external animation blocks.
    crate::source_animation::Skeleton::read(&bytes)?;
    bytes[180..184].copy_from_slice(&0i32.to_le_bytes());
    bytes[188..192].copy_from_slice(&0i32.to_le_bytes());
    let mut mdl = vmdl::Mdl::read(&bytes)?;
    let vvd = vmdl::Vvd::read(&mounts.read(bsp, &format!("{stem}.vvd"))?)?;
    let mut vtx = vmdl::Vtx::read(&mounts.read(bsp, &format!("{stem}.dx90.vtx"))?)?;
    // Bodygroup zero is a selection, not a union of all alternate meshes.
    for p in &mut mdl.body_parts {
        p.models.truncate(1);
    }
    for p in &mut vtx.body_parts {
        p.models.truncate(1);
    }
    let model = vmdl::Model::from_parts(mdl, vtx, vvd);
    let skins = model
        .skin_tables()
        .nth(skin.max(0) as usize)
        .or_else(|| model.skin_tables().next())
        .ok_or("model has no skin table")?;
    let mut parts = Vec::new();
    for mesh in model.meshes() {
        let texture = skins
            .texture(mesh.material_index())
            .ok_or("invalid model material index")?;
        let mut resolved = None;
        for directory in model.texture_directories() {
            let path = format!(
                "{}{texture}",
                if directory.is_empty() {
                    String::new()
                } else {
                    format!("{}/", directory.trim_end_matches('/'))
                }
            );
            if mounts.read(bsp, &format!("materials/{path}.vmt")).is_ok() {
                resolved = Some(path);
                break;
            }
        }
        let material = resolved
            .ok_or_else(|| format!("{path}: material {texture} not in model search paths"))?;
        let mut geo = Geometry::default();
        for strip in mesh.vertex_strip_indices() {
            let indices: Vec<_> = strip.collect();
            if indices.len() % 3 != 0 {
                return Err("non-triangle model strip is not yet supported".into());
            }
            for triangle in indices.chunks_exact(3) {
                let vertices = triangle
                    .iter()
                    .map(|i| {
                        model
                            .vertices()
                            .get(*i)
                            .ok_or("model vertex index outside VVD")
                    })
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                let mut positions = Vec::new();
                let mut normals = Vec::new();
                let mut uvs = Vec::new();
                let mut weights = Vec::new();
                for v in vertices {
                    let mut row = [(0, 0.); 3];
                    let total: f32 = v.bone_weights.weights().map(|w| w.weight).sum();
                    for (i, w) in v.bone_weights.weights().enumerate() {
                        row[i] = (w.bone_id, if total > 0. { w.weight / total } else { 0. });
                    }
                    weights.push(row);
                    positions.push(source_position(
                        [v.position.x, v.position.y, v.position.z],
                        1.,
                    ));
                    normals.push(source_position([v.normal.x, v.normal.y, v.normal.z], 1.));
                    uvs.push(v.texture_coordinates);
                }
                let order = if (positions[1] - positions[0])
                    .cross(positions[2] - positions[0])
                    .dot(normals[0] + normals[1] + normals[2])
                    < 0.
                {
                    [0, 2, 1]
                } else {
                    [0, 1, 2]
                };
                for i in order {
                    geo.positions.push(positions[i].to_array());
                    geo.normals.push(normals[i].to_array());
                    geo.uv.push(uvs[i]);
                    geo.weights.push(weights[i]);
                    geo.light_uv.push([0.5 / 4096.; 2]);
                }
            }
        }
        parts.push((material, geo));
    }
    Ok(parts)
}

pub fn append_static_props(
    bsp: &vbsp::Bsp,
    mounts: &Mounts,
    scale: f32,
    groups: &mut BTreeMap<String, Geometry>,
    warnings: &mut Vec<String>,
) -> usize {
    let mut cache = BTreeMap::new();
    let mut count = 0;
    for prop in bsp.static_props() {
        let p = prop.as_prop_placement();
        let key = (p.model.to_string(), p.skin);
        let parts = cache.entry(key).or_insert_with(|| {
            println!("Model: {} skin {}", p.model, p.skin);
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                model_parts(bsp, mounts, p.model, p.skin)
            }))
            .map_err(|_| format!("{}: malformed or unsupported model", p.model))
            .and_then(|r| r.map_err(|e| format!("{}: {e}", p.model)))
        });
        let parts = match parts {
            Ok(parts) => parts,
            Err(e) => {
                if !warnings.contains(e) {
                    warnings.push(e.clone());
                }
                continue;
            }
        };
        let q = p.rotation;
        let rotation = Quat::from_xyzw(-q.v.y, q.v.z, -q.v.x, q.s).normalize();
        let origin = source_position([p.origin.x, p.origin.y, p.origin.z], scale);
        for (material, geometry) in parts {
            let target = groups.entry(material.clone()).or_default();
            target.positions.extend(
                geometry.positions.iter().map(|v| {
                    (origin + rotation * Vec3::from_array(*v) * scale * p.scale).to_array()
                }),
            );
            target.normals.extend(
                geometry
                    .normals
                    .iter()
                    .map(|v| (rotation * Vec3::from_array(*v)).to_array()),
            );
            target.uv.extend_from_slice(&geometry.uv);
            target.light_uv.extend_from_slice(&geometry.light_uv);
        }
        count += 1;
    }
    count
}
