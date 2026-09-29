//! Read-only local Source content mounting and native Bevy world geometry import.
//! This intentionally does not execute Lua, entity I/O, or material proxies.
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
};
use sandbox_catalog::{source_maps::SourceMapDef, Result};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub fn source_position(v: [f32; 3], scale: f32) -> Vec3 {
    // Source Z-up, +X forward, +Y left -> Bevy Y-up, -Z forward, -X left.
    Vec3::new(-v[1], v[2], -v[0]) * scale
}

pub fn virtual_path(name: &str) -> Result<String> {
    let path = name.replace('\\', "/").to_ascii_lowercase();
    if path.is_empty()
        || path.starts_with('/')
        || path.contains(':')
        || path
            .split('/')
            .any(|p| p == ".." || p == "." || p.is_empty())
    {
        return Err(format!("unsafe virtual asset path: {name}").into());
    }
    Ok(path)
}

pub struct Mounts {
    pub(crate) roots: Vec<(PathBuf, Vec<vpk::VPK>)>,
}
impl Mounts {
    pub fn open(install: &Path) -> Result<Self> {
        if !install.join("garrysmod/maps/gm_construct.bsp").is_file() {
            return Err(format!("Garry's Mod installation not found at {}. Set GMOD_DIR to the Steam GarrysMod directory.", install.display()).into());
        }
        let mut roots = Vec::new();
        for sub in ["garrysmod", "sourceengine", "hl2", "platform"] {
            let root = install.join(sub);
            if !root.is_dir() {
                continue;
            }
            let mut paths = std::fs::read_dir(&root)?
                .map(|e| e.map(|e| e.path()))
                .collect::<std::io::Result<Vec<_>>>()?;
            paths.sort();
            let mut archives = Vec::new();
            for path in paths {
                if path.to_string_lossy().ends_with("_dir.vpk") {
                    println!("Mount read-only: {}", path.display());
                    archives.push(vpk::from_path(path)?);
                }
            }
            roots.push((root, archives));
        }
        Ok(Self { roots })
    }
    pub fn read(&self, bsp: &vbsp::Bsp, name: &str) -> Result<Vec<u8>> {
        let name = virtual_path(name)?;
        if let Some(data) = bsp.pack.get(&name)? {
            return Ok(data);
        }
        for (root, archives) in &self.roots {
            let file = root.join(&name);
            if file.is_file() {
                if file.metadata()?.len() > 128 * 1024 * 1024 {
                    return Err("asset exceeds 128 MiB limit".into());
                }
                // Keep symlinks from escaping the mounted content root.
                if !file.canonicalize()?.starts_with(root.canonicalize()?) {
                    return Err("asset escapes mount".into());
                }
                return Ok(std::fs::read(file)?);
            }
            for archive in archives {
                if let Some(entry) = archive.tree.get(&name) {
                    if u64::from(entry.dir_entry.file_length) + entry.preload_data.len() as u64
                        > 128 * 1024 * 1024
                    {
                        return Err("archive asset exceeds size limit".into());
                    }
                    return Ok(entry.get()?.into_owned());
                }
            }
        }
        Err(format!("missing mounted asset: {name}").into())
    }
    pub fn material(&self, bsp: &vbsp::Bsp, name: &str) -> Result<vmt_parser::material::Material> {
        let name = if name.starts_with("materials/") {
            name.to_string()
        } else {
            format!("materials/{name}")
        };
        let name = if name.ends_with(".vmt") {
            name
        } else {
            format!("{name}.vmt")
        };
        let mut text = String::from_utf8(self.read(bsp, &name)?)?;
        // The parser lacks Source's Teeth/Eyes shaders. Preserve their parameters and
        // base texture via VertexLitGeneric; eye/teeth lighting remains approximate.
        if let Some(brace) = text.find('{') {
            let shader = text[..brace].trim().trim_matches('"').to_ascii_lowercase();
            if matches!(shader.as_str(), "teeth" | "eyes") {
                text = format!("VertexLitGeneric {}", &text[brace..]);
            }
        }
        let mut mat = vmt_parser::from_str(&text)?;
        for _ in 0..8 {
            if !matches!(mat, vmt_parser::material::Material::Patch(_)) {
                return Ok(mat);
            }
            mat = mat.resolve::<Box<dyn std::error::Error + Send + Sync>, _>(|path| {
                let path = if path.starts_with("materials/") {
                    path.to_string()
                } else {
                    format!("materials/{path}")
                };
                Ok(String::from_utf8(self.read(bsp, &path)?)?)
            })?;
        }
        Err(format!("material patch depth exceeded: {name}").into())
    }
    pub fn texture(&self, bsp: &vbsp::Bsp, name: &str, repeat: bool) -> Result<Image> {
        let name = name.trim_end_matches(".vtf");
        let name = if name.starts_with("materials/") {
            format!("{name}.vtf")
        } else {
            format!("materials/{name}.vtf")
        };
        let bytes = self.read(bsp, &name)?;
        // Third-party VTF parser uses slice indexing. Turn corrupt-input panics into diagnostics.
        let decoded = std::panic::catch_unwind(|| -> Result<_> {
            let vtf = vtf::from_bytes(&bytes)?;
            if vtf.highres_image.width > 8192 || vtf.highres_image.height > 8192 {
                return Err("texture exceeds 8192 pixel limit".into());
            }
            Ok(vtf.highres_image.decode(0)?.to_rgba8())
        })
        .map_err(|_| format!("malformed VTF: {name}"))??;
        let (width, height) = decoded.dimensions();
        let mut mip = decoded.clone();
        let mut mip_bytes = decoded.as_raw().clone();
        let mut mip_count = 1;
        while mip.width() > 1 || mip.height() > 1 {
            mip = image::imageops::resize(
                &mip,
                (mip.width() / 2).max(1),
                (mip.height() / 2).max(1),
                image::imageops::FilterType::Triangle,
            );
            mip_bytes.extend_from_slice(mip.as_raw());
            mip_count += 1;
        }
        let mut image = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            decoded.into_raw(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.texture_descriptor.mip_level_count = mip_count;
        image.data = Some(mip_bytes);
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            anisotropy_clamp: 16,
            address_mode_u: if repeat {
                ImageAddressMode::Repeat
            } else {
                ImageAddressMode::ClampToEdge
            },
            address_mode_v: if repeat {
                ImageAddressMode::Repeat
            } else {
                ImageAddressMode::ClampToEdge
            },
            ..ImageSamplerDescriptor::linear()
        });
        Ok(image)
    }
}

#[derive(Default, Clone)]
pub struct Geometry {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uv: Vec<[f32; 2]>,
    pub light_uv: Vec<[f32; 2]>,
    pub weights: Vec<[(u8, f32); 3]>,
}
impl Geometry {
    pub fn mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uv)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, self.light_uv)
    }
}
pub struct Surface {
    pub name: String,
    pub geometry: Geometry,
    pub texture: Option<String>,
    pub alpha: AlphaMode,
    pub unlit: bool,
    pub no_cull: bool,
    pub tint: Color,
}
pub struct LoadedMap {
    pub source: MountedSource,
    pub surfaces: Vec<Surface>,
    pub textures: BTreeMap<String, Image>,
    pub lightmap: Image,
    pub sky: Vec<(String, Image)>,
    pub spawn: Vec3,
    pub forward: Vec3,
    pub report: serde_json::Value,
}

#[derive(Resource)]
pub struct MountedSource {
    pub mounts: Mounts,
    pub bsp: vbsp::Bsp,
}

fn lump(bytes: &[u8], index: usize) -> Result<&[u8]> {
    let entry = bytes
        .get(8 + index * 16..24 + index * 16)
        .ok_or("truncated BSP directory")?;
    let offset = u32::from_le_bytes(entry[0..4].try_into()?) as usize;
    let size = u32::from_le_bytes(entry[4..8].try_into()?) as usize;
    if entry[12..16] != [0, 0, 0, 0] {
        return Err("compressed lighting lump is not yet supported".into());
    }
    bytes
        .get(offset..offset.checked_add(size).ok_or("BSP range overflow")?)
        .ok_or_else(|| "truncated BSP lump".into())
}

const ATLAS: usize = 4096;
struct LightAtlas {
    data: Vec<u8>,
    x: usize,
    y: usize,
    row_height: usize,
}
impl LightAtlas {
    fn new() -> Self {
        let mut data = vec![255; ATLAS * ATLAS * 4];
        data[0..4].copy_from_slice(&[255; 4]);
        Self {
            data,
            x: 2,
            y: 0,
            row_height: 2,
        }
    }
    fn face(&mut self, face: &vbsp::Face, bytes: &[u8]) -> Result<Option<[usize; 4]>> {
        if face.light_offset < 0 {
            return Ok(None);
        }
        let w = usize::try_from(face.light_map_texture_size[0] + 1)?;
        let h = usize::try_from(face.light_map_texture_size[1] + 1)?;
        if w == 0 || h == 0 || w + 2 > ATLAS || h + 2 > ATLAS {
            return Err("invalid face lightmap dimensions".into());
        }
        if self.x + w + 2 > ATLAS {
            self.x = 0;
            self.y += self.row_height;
            self.row_height = 0;
        }
        if self.y + h + 2 > ATLAS {
            return Err("lightmap atlas capacity exceeded".into());
        }
        let start = face.light_offset as usize;
        let samples = bytes
            .get(start..start + w * h * 4)
            .ok_or("face lightmap outside lighting lump")?;
        let ox = self.x + 1;
        let oy = self.y + 1;
        // Duplicate a one-texel border so linear filtering cannot bleed between faces.
        for y in 0..h + 2 {
            for x in 0..w + 2 {
                let source =
                    (y.saturating_sub(1).min(h - 1) * w + x.saturating_sub(1).min(w - 1)) * 4;
                let target = ((self.y + y) * ATLAS + self.x + x) * 4;
                let multiplier = 2f32.powi(samples[source + 3] as i8 as i32);
                for c in 0..3 {
                    self.data[target + c] =
                        (samples[source + c] as f32 * multiplier).clamp(0., 255.) as u8;
                }
                self.data[target + 3] = 255;
            }
        }
        self.x += w + 2;
        self.row_height = self.row_height.max(h + 2);
        Ok(Some([ox, oy, w, h]))
    }
    fn image(self) -> Image {
        let mut image = Image::new(
            Extent3d {
                width: ATLAS as u32,
                height: ATLAS as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            self.data,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::linear();
        image
    }
}

fn triple(text: &str) -> Option<[f32; 3]> {
    let numbers = text
        .split_whitespace()
        .map(str::parse::<f32>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .ok()?;
    let result: [f32; 3] = numbers.try_into().ok()?;
    result.iter().all(|v| v.is_finite()).then_some(result)
}
fn axis(a: [f32; 4], p: vbsp::Vector) -> f32 {
    a[0] * p.x + a[1] * p.y + a[2] * p.z + a[3]
}

pub fn load(install: &Path, def: &SourceMapDef) -> Result<LoadedMap> {
    let mounts = Mounts::open(install)?;
    let path = install.join("garrysmod").join(virtual_path(&def.bsp)?);
    let bytes = std::fs::read(&path)?;
    println!("Parse {} ({} bytes)", path.display(), bytes.len());
    let bsp = vbsp::Bsp::read(&bytes)?;
    let lighting = lump(&bytes, 8)?;
    let mut atlas = LightAtlas::new();
    let mut groups = BTreeMap::<String, Geometry>::new();
    let mut face_count = 0;
    let mut displacement_faces = 0;
    let mut lightmapped_faces = 0;
    let mut brush_models = 0;
    let mut brush_faces = 0;
    for (model_index, model) in bsp.models().enumerate() {
        let entity = bsp
            .entities
            .iter()
            .find(|e| e.prop("model") == Some(format!("*{model_index}").as_str()));
        let (origin, rotation) = if model_index == 0 {
            (Vec3::ZERO, Quat::IDENTITY)
        } else {
            let Some(entity) = entity else { continue };
            let class = entity.prop("classname").unwrap_or("");
            if !matches!(
                class,
                "func_brush"
                    | "func_wall"
                    | "func_illusionary"
                    | "func_reflective_glass"
                    | "func_door"
                    | "func_door_rotating"
                    | "func_movelinear"
                    | "func_button"
                    | "func_breakable"
            ) || entity.prop("StartDisabled") == Some("1")
                || entity.prop("rendermode") == Some("10")
            {
                continue;
            }
            brush_models += 1;
            let origin = entity.prop("origin").and_then(triple).unwrap_or([0.; 3]);
            let angles = entity.prop("angles").and_then(triple).unwrap_or([0.; 3]);
            // Source angles are pitch(Y), yaw(Z), roll(X), converted to Bevy axes.
            let rotation = Quat::from_rotation_y(angles[1].to_radians())
                * Quat::from_rotation_x(-angles[0].to_radians())
                * Quat::from_rotation_z(-angles[2].to_radians());
            (source_position(origin, def.unit_scale), rotation)
        };
        for face in model.faces() {
            if face.texture_info < 0 || !face.is_visible() {
                continue;
            }
            let texture = face.texture();
            if texture.flags.intersects(
                vbsp::TextureFlags::SKY
                    | vbsp::TextureFlags::SKY2D
                    | vbsp::TextureFlags::NODRAW
                    | vbsp::TextureFlags::SKIP,
            ) {
                continue;
            }
            let name = texture.name().replace('\\', "/").to_ascii_lowercase();
            let geometry = groups.entry(name).or_default();
            let light = if lighting.is_empty() {
                None
            } else {
                atlas.face(&face, lighting)?
            };
            if light.is_some() {
                lightmapped_faces += 1;
            }
            if face.displacement().is_some() {
                displacement_faces += 1;
            }
            let positions: Vec<_> = face.vertex_positions().collect();
            // The indexed BSP plane already carries its signed orientation. Applying
            // face.side again flips negative-axis faces (including the room ceiling).
            let normal = source_position([face.normal().x, face.normal().y, face.normal().z], 1.);
            for triangle in positions.chunks_exact(3) {
                let mut tri = [triangle[0], triangle[1], triangle[2]];
                let a = source_position([tri[0].x, tri[0].y, tri[0].z], def.unit_scale);
                let b = source_position([tri[1].x, tri[1].y, tri[1].z], def.unit_scale);
                let c = source_position([tri[2].x, tri[2].y, tri[2].z], def.unit_scale);
                if (b - a).cross(c - a).dot(normal) < 0. {
                    tri.swap(1, 2);
                }
                for p in tri {
                    geometry.positions.push(
                        (origin + rotation * source_position([p.x, p.y, p.z], def.unit_scale))
                            .to_array(),
                    );
                    geometry.normals.push((rotation * normal).to_array());
                    geometry.uv.push(texture.uv(p));
                    let uv = light
                        .map(|[x, y, _, _]| {
                            [
                                (x as f32 + axis(texture.light_map_scale, p)
                                    - face.light_map_texture_min[0] as f32
                                    + 0.5)
                                    / ATLAS as f32,
                                (y as f32 + axis(texture.light_map_transform, p)
                                    - face.light_map_texture_min[1] as f32
                                    + 0.5)
                                    / ATLAS as f32,
                            ]
                        })
                        .unwrap_or([0.5 / ATLAS as f32; 2]);
                    geometry.light_uv.push(uv);
                }
            }
            face_count += 1;
            if model_index != 0 {
                brush_faces += 1;
            }
        }
    }
    let mut warnings = Vec::new();
    let loaded_props = crate::source_models::append_static_props(
        &bsp,
        &mounts,
        def.unit_scale,
        &mut groups,
        &mut warnings,
    );
    let mut surfaces = Vec::new();
    let mut textured = 0;
    let mut textures = BTreeMap::new();
    let mut triangles = 0;
    for (name, geometry) in groups {
        println!("Material: {name}");
        triangles += geometry.positions.len() / 3;
        let mut surface = Surface {
            name: name.clone(),
            geometry,
            texture: None,
            alpha: AlphaMode::Opaque,
            unlit: false,
            no_cull: false,
            tint: Color::srgb(1., 0., 1.),
        };
        match mounts.material(&bsp, &name) {
            Ok(mat) => {
                surface.alpha =
                    mat.alpha_test()
                        .map(AlphaMode::Mask)
                        .unwrap_or(if mat.translucent() {
                            AlphaMode::Blend
                        } else {
                            AlphaMode::Opaque
                        });
                surface.no_cull = mat.no_cull();
                surface.unlit = matches!(mat, vmt_parser::material::Material::UnlitGeneric(_));
                if let Some(base) = mat.base_texture() {
                    let result = if textures.contains_key(base) {
                        Ok(())
                    } else {
                        mounts.texture(&bsp, base, true).map(|image| {
                            textures.insert(base.to_string(), image);
                        })
                    };
                    match result {
                        Ok(()) => {
                            surface.texture = Some(base.to_string());
                            surface.tint = Color::WHITE;
                            textured += 1;
                        }
                        Err(e) => warnings.push(format!("{name}: {e}")),
                    }
                } else if let vmt_parser::material::Material::Water(water) = &mat {
                    let [r, g, b] = water.fog_color.0;
                    surface.tint = Color::srgba(r / 255., g / 255., b / 255., 0.85);
                    surface.unlit = true;
                    warnings.push(format!("{name}: original water fog color only, reflection/refraction and waves not implemented"));
                } else {
                    surface.tint = Color::srgb(1., 0., 1.);
                    warnings.push(format!("{name}: no supported base texture"));
                }
            }
            Err(e) => warnings.push(format!("{name}: {e}")),
        }
        surfaces.push(surface);
    }
    let player = bsp
        .entities
        .iter()
        .find(|e| e.prop("classname") == Some("info_player_start"));
    let origin = player
        .as_ref()
        .and_then(|e| e.prop("origin"))
        .and_then(triple)
        .unwrap_or([0., 0., 128.]);
    let angles = player
        .as_ref()
        .and_then(|e| e.prop("angles"))
        .and_then(triple)
        .unwrap_or([0.; 3]);
    let yaw = angles[1].to_radians();
    let spawn = source_position(
        [origin[0], origin[1], origin[2] + def.eye_height_units],
        def.unit_scale,
    );
    let forward = source_position([yaw.cos(), yaw.sin(), 0.], 1.);
    let skyname = bsp
        .entities
        .iter()
        .find(|e| e.prop("classname") == Some("worldspawn"))
        .and_then(|e| e.prop("skyname").map(str::to_owned))
        .unwrap_or_default();
    let mut sky = Vec::new();
    for side in ["ft", "bk", "lf", "rt", "up", "dn"] {
        let name = format!("skybox/{skyname}{side}");
        let tex = mounts
            .material(&bsp, &name)
            .ok()
            .and_then(|m| m.base_texture().map(str::to_owned))
            .unwrap_or(name.clone());
        match mounts.texture(&bsp, &tex, false) {
            Ok(image) => sky.push((side.to_string(), image)),
            Err(e) => warnings.push(format!("sky {side}: {e}")),
        }
    }
    let report = serde_json::json!({ "map":def.id, "bsp_bytes":bytes.len(), "world_faces":face_count-brush_faces,"brush_models":brush_models,"brush_faces":brush_faces,"displacement_faces":displacement_faces,"lightmapped_faces":lightmapped_faces,"triangles":triangles,"materials":surfaces.len(),"textured_materials":textured,"sky_faces":sky.len(),"skyname":skyname,"spawn_source":origin,"spawn_bevy":spawn.to_array(),"source_static_props":bsp.static_props().count(),"loaded_static_props":loaded_props,"warnings":warnings,"limitations":["World, visible brush entities and static models imported; brush entity simulation and 3D sky scaling not implemented; static prop lighting is approximate","Source shaders, water, blend textures, overlays and material proxies not equivalent","LDR lightmaps clamped to normalized range, HDR exposure not matched","Grounded Rapier player controller and optional noclip, not exact Source movement"] });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(LoadedMap {
        source: MountedSource { mounts, bsp },
        surfaces,
        textures,
        lightmap: atlas.image(),
        sky,
        spawn,
        forward,
        report,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coordinates_preserve_scale_and_handedness() {
        assert_eq!(
            source_position([0., 0., 64.], 0.01905),
            Vec3::Y * (64. * 0.01905)
        );
        let x = source_position([1., 0., 0.], 1.);
        let y = source_position([0., 1., 0.], 1.);
        let z = source_position([0., 0., 1.], 1.);
        assert_eq!(x.cross(y), z);
    }
    #[test]
    fn virtual_paths_reject_escape() {
        for p in [
            "../secret",
            "materials/../../file",
            "C:/file",
            "/absolute",
            "models/./x",
        ] {
            assert!(virtual_path(p).is_err());
        }
        assert_eq!(
            virtual_path("Materials\\BRICK/a.vmt").unwrap(),
            "materials/brick/a.vmt"
        );
    }
    #[test]
    fn lump_range_checked() {
        assert!(lump(&[0; 4], 8).is_err());
    }
}
