//! Per-instance posing. Original flex deltas and skeletal transforms, not rigid placeholders.
use super::*;
use crate::source_animation::{bevy_matrix, pose_data::Data};
use std::sync::Arc;
#[derive(Component, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct State {
    pub rotations: BTreeMap<usize, [f32; 3]>,
    pub scales: BTreeMap<usize, [f32; 3]>,
    pub flexes: BTreeMap<usize, f32>,
    pub eye: Option<[f32; 3]>,
}
#[derive(Resource, Default)]
struct Cache(BTreeMap<String, Arc<Data>>);
#[derive(Resource, Default)]
struct IrisCache(BTreeMap<String, BTreeMap<usize, Handle<Image>>>);
#[derive(Component)]
pub(crate) struct EyeMaterial(pub Handle<StandardMaterial>);
#[derive(Component)]
struct Posed {
    parts: Vec<(Entity, Handle<Mesh>, Handle<StandardMaterial>)>,
}
#[derive(Resource, Default)]
pub(crate) struct Selection(pub Option<Entity>);
pub(crate) fn supported(tool: &str) -> bool {
    matches!(tool, "finger" | "inflator" | "faceposer" | "eyeposer")
}
fn data(world: &mut World, path: &str) -> Result<Arc<Data>> {
    if let Some(value) = world.get_resource::<Cache>().and_then(|c| c.0.get(path)) {
        return Ok(value.clone());
    }
    let source = world.resource::<MountedSource>();
    let parsed = Arc::new(Data::read(&source.mounts.read(&source.bsp, path)?)?);
    if !world.contains_resource::<Cache>() {
        world.init_resource::<Cache>();
    }
    world
        .resource_mut::<Cache>()
        .0
        .insert(path.into(), parsed.clone());
    Ok(parsed)
}
pub(crate) fn validate(world: &mut World, path: &str, state: &State) -> Result<()> {
    let d = data(world, path)?;
    if state.rotations.len() > 256 || state.scales.len() > 256 || state.flexes.len() > 1024 {
        return Err("pose limit exceeded".into());
    }
    for (id, values) in &state.rotations {
        if *id >= d.skeleton.bones.len() || values.iter().any(|v| !v.is_finite() || v.abs() > 180.)
        {
            return Err("invalid saved bone rotation".into());
        }
    }
    for (id, values) in &state.scales {
        if *id >= d.skeleton.bones.len()
            || values
                .iter()
                .any(|v| !v.is_finite() || !(0.05..=5.).contains(v))
        {
            return Err("invalid saved bone scale".into());
        }
    }
    for (id, value) in &state.flexes {
        if *id >= d.names.len()
            || !value.is_finite()
            || !(-2. ..=2.).contains(value)
            || !d.parts.iter().any(|p| {
                p.flexes
                    .iter()
                    .any(|f| !f.vertices.is_empty() && (f.descriptor == *id || f.pair == Some(*id)))
            })
        {
            return Err("invalid saved flex weight".into());
        }
    }
    if state.eye.is_some_and(|p| {
        p.iter().any(|v| !v.is_finite() || v.abs() > 100000.)
            || !d.parts.iter().any(|p| p.eye.is_some())
    }) {
        return Err("invalid saved eye target".into());
    }
    Ok(())
}
fn globals(d: &Data, state: &State) -> Vec<Mat4> {
    let mut pose = d.skeleton.bind_pose();
    for (i, a) in &state.rotations {
        pose[*i].rotation *= Quat::from_euler(
            EulerRot::ZYX,
            a[2].to_radians(),
            a[1].to_radians(),
            a[0].to_radians(),
        );
    }
    for (i, s) in &state.scales {
        pose[*i].scale = Vec3::from_array(*s);
    }
    d.skeleton.globals(&pose)
}
fn geometry(
    d: &Data,
    base: &GpuModel,
    state: &State,
) -> Result<Vec<crate::source_assets::Geometry>> {
    if d.parts.len() != base.geometry.len() {
        return Err("pose mesh layout does not match model".into());
    }
    let global = globals(d, state);
    let skin: Vec<_> = global
        .iter()
        .zip(&d.skeleton.bones)
        .map(|(g, b)| bevy_matrix(*g * b.inverse, 0.01905))
        .collect();
    let normals: Vec<_> = skin.iter().map(|m| m.inverse().transpose()).collect();
    let mut out = Vec::new();
    for (base, part) in base.geometry.iter().zip(&d.parts) {
        let mut geo = base.clone();
        if geo.weights.len() != geo.positions.len()
            || geo.source_indices.len() != geo.positions.len()
        {
            return Err("model lacks pose vertex mapping".into());
        }
        let mut deltas: BTreeMap<usize, (Vec3, Vec3)> = BTreeMap::new();
        for f in &part.flexes {
            let left = f.weight(*state.flexes.get(&f.descriptor).unwrap_or(&0.));
            let right = f
                .pair
                .map_or(left, |id| f.weight(*state.flexes.get(&id).unwrap_or(&0.)));
            if left == 0. && right == 0. {
                continue;
            }
            for (id, delta) in &f.vertices {
                let weight = left + (right - left) * delta.side;
                let entry = deltas.entry(*id).or_insert((Vec3::ZERO, Vec3::ZERO));
                entry.0 += delta.position * weight;
                entry.1 += delta.normal * weight;
            }
        }
        for i in 0..geo.positions.len() {
            let mut p = Vec3::from_array(base.positions[i]);
            let mut n = Vec3::from_array(base.normals[i]);
            if let Some((position, normal)) = deltas.get(&geo.source_indices[i]) {
                p += *position;
                n += *normal;
            }
            let mut target = Vec3::ZERO;
            let mut normal = Vec3::ZERO;
            let mut total = 0.;
            for (bone, w) in geo.weights[i] {
                if w <= 0. {
                    continue;
                }
                let id = bone as usize;
                let m = skin.get(id).ok_or("vertex bone outside pose skeleton")?;
                target += m.transform_point3(p) * w;
                normal += normals[id].transform_vector3(n) * w;
                total += w;
            }
            if total > 0. {
                p = target / total;
                n = normal.normalize_or_zero();
            }
            if !p.is_finite() || !n.is_finite() {
                return Err("nonfinite posed geometry".into());
            }
            geo.positions[i] = p.to_array();
            geo.normals[i] = n.to_array();
        }
        if let (Some(eye), Some(target)) = (&part.eye, state.eye) {
            let frame = bevy_matrix(global[eye.bone], 0.01905);
            let origin = frame.transform_point3(crate::source_assets::source_position(
                eye.origin.to_array(),
                0.01905,
            ));
            let rest = frame
                .transform_vector3(crate::source_assets::source_position(
                    eye.forward.to_array(),
                    1.,
                ))
                .normalize_or_zero();
            let wanted = (Vec3::from_array(target) - origin).normalize_or_zero();
            let direction = if wanted.length_squared() > 0.5 {
                wanted
            } else {
                rest
            };
            let up = frame
                .transform_vector3(crate::source_assets::source_position(eye.up.to_array(), 1.))
                .normalize_or_zero();
            let right = direction.cross(up).normalize_or_zero();
            let up = right.cross(direction).normalize_or_zero();
            if right.length_squared() < 0.5 {
                return Err("eye target parallel to eye up axis".into());
            }
            for (p, uv) in geo.positions.iter().zip(&mut geo.uv) {
                let relative = (Vec3::from_array(*p) - origin) / 0.01905;
                *uv = [
                    0.5 + relative.dot(right) * eye.iris_scale,
                    0.5 - relative.dot(up) * eye.iris_scale,
                ];
            }
        }
        out.push(geo);
    }
    Ok(out)
}
// Only explicit iris parameters are accepted, never substitute a random eye texture.
fn iris(world: &mut World, path: &str, d: &Data) -> Result<BTreeMap<usize, Handle<Image>>> {
    if let Some(cached) = world
        .get_resource::<IrisCache>()
        .and_then(|c| c.0.get(path))
    {
        return Ok(cached.clone());
    }
    let source = world.resource::<MountedSource>();
    let parts = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        source_models::model_parts(&source.bsp, &source.mounts, path, 0)
    }))
    .map_err(|_| "model parser rejected eye material layout")??;
    let mut images = Vec::new();
    for (i, ((name, _), part)) in parts.iter().zip(&d.parts).enumerate() {
        if part.eye.is_none() {
            continue;
        }
        let raw = String::from_utf8(
            source
                .mounts
                .read(&source.bsp, &format!("materials/{name}.vmt"))?,
        )?;
        let raw = raw
            .lines()
            .map(|line| line.split("//").next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n");
        let tokens: Vec<_> = raw
            .split(|c: char| c.is_whitespace() || c == '\"' || c == '{' || c == '}')
            .filter(|s| !s.is_empty())
            .collect();
        let texture = tokens
            .windows(2)
            .find(|v| v[0].eq_ignore_ascii_case("$iris"))
            .map(|v| v[1])
            .ok_or("eye material has no supported explicit iris texture")?;
        images.push((i, source.mounts.texture(&source.bsp, texture, false)?));
    }
    let images: BTreeMap<_, _> = images
        .into_iter()
        .map(|(i, image)| (i, world.resource_mut::<Assets<Image>>().add(image)))
        .collect();
    world.init_resource::<IrisCache>();
    world
        .resource_mut::<IrisCache>()
        .0
        .insert(path.into(), images.clone());
    Ok(images)
}
fn collision(geos: &[crate::source_assets::Geometry]) -> Result<(Collider, Vec3, Vec3)> {
    let mut points: Vec<_> = geos
        .iter()
        .flat_map(|g| g.positions.iter().map(|p| Vec3::from_array(*p)))
        .collect();
    points.sort_unstable_by(|a, b| {
        a.x.total_cmp(&b.x)
            .then(a.y.total_cmp(&b.y))
            .then(a.z.total_cmp(&b.z))
    });
    points.dedup();
    if points.iter().any(|p| p.abs().max_element() > 10000.) {
        return Err("posed geometry exceeds safe extent".into());
    }
    let collider =
        Collider::convex_hull(&points).ok_or("posed model cannot form a collision hull")?;
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for p in points {
        min = min.min(p);
        max = max.max(p);
    }
    Ok((collider, min, max))
}
pub(crate) fn prepare(world: &mut World, path: &str, state: &State) -> Result<()> {
    validate(world, path, state)?;
    let d = data(world, path)?;
    let base = gpu_model(world, path)?;
    collision(&geometry(&d, &base, state)?)?;
    if state.eye.is_some() {
        iris(world, path, &d)?;
    }
    Ok(())
}
pub(crate) fn apply(world: &mut World, entity: Entity, state: State) -> Result<()> {
    let path = world
        .get::<SpawnedProp>(entity)
        .ok_or("select a spawned model")?
        .model
        .clone();
    validate(world, &path, &state)?;
    let d = data(world, &path)?;
    let base = gpu_model(world, &path)?;
    let geos = geometry(&d, &base, &state)?;
    let eye_materials = if state.eye.is_some() {
        iris(world, &path, &d)?
    } else {
        BTreeMap::new()
    };
    let (collider, min, max) = collision(&geos)?;
    if world.get::<Posed>(entity).is_none() {
        let children: Vec<_> = world
            .get::<Children>(entity)
            .map(|c| {
                c.iter()
                    .filter(|e| {
                        world.get::<Mesh3d>(*e).is_some()
                            && world.get::<impacts::ImpactMark>(*e).is_none()
                    })
                    .collect()
            })
            .unwrap_or_default();
        if children.len() != geos.len() {
            return Err("unsupported model child layout for posing".into());
        }
        let mut parts = Vec::new();
        for (e, geo) in children.into_iter().zip(&geos) {
            let mesh = world.resource_mut::<Assets<Mesh>>().add(geo.clone().mesh());
            let material = world
                .get::<MeshMaterial3d<StandardMaterial>>(e)
                .ok_or("pose material missing")?
                .0
                .clone();
            world
                .entity_mut(e)
                .insert((Mesh3d(mesh.clone()), bevy::render::view::NoFrustumCulling));
            parts.push((e, mesh, material));
        }
        world.entity_mut(entity).insert(Posed { parts });
    }
    let parts = world.get::<Posed>(entity).unwrap().parts.clone();
    for (i, ((e, mesh, original), geo)) in parts.into_iter().zip(geos).enumerate() {
        if let Some(m) = world.resource_mut::<Assets<Mesh>>().get_mut(&mesh) {
            *m = geo.mesh();
        }
        if let Some(texture) = eye_materials.get(&i) {
            let material = world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color_texture: Some(texture.clone()),
                    unlit: true,
                    ..default()
                });
            world.entity_mut(e).insert(EyeMaterial(material));
        } else {
            world.entity_mut(e).remove::<EyeMaterial>();
        }
        world.entity_mut(e).insert(MeshMaterial3d(original));
    }
    let mut prop = world.get_mut::<SpawnedProp>(entity).unwrap();
    prop.center = (min + max) * 0.5;
    prop.half = ((max - min) * 0.5).max(Vec3::splat(0.02));
    world
        .entity_mut(entity)
        .insert((collider, Sleeping::default(), state));
    let properties = world.get::<Properties>(entity).cloned().unwrap_or_default();
    apply_visual_properties(world, entity, &properties)?;
    Ok(())
}
// Keep serialized state on the root while meshes and materials remain instance-local.
fn nearest(d: &Data, base: &GpuModel, state: &State, point: Vec3) -> Result<usize> {
    let geos = geometry(d, base, state)?;
    let mut best = (f32::INFINITY, None);
    for geo in geos {
        for (p, weights) in geo.positions.iter().zip(&geo.weights) {
            let distance = Vec3::from_array(*p).distance_squared(point);
            if distance < best.0 {
                if let Some((bone, _)) = weights
                    .iter()
                    .filter(|(_, w)| *w > 0.)
                    .max_by(|a, b| a.1.total_cmp(&b.1))
                {
                    best = (distance, Some(*bone as usize));
                }
            }
        }
    }
    best.1
        .ok_or_else(|| "model has no weighted pose bones".into())
}
pub(crate) fn operate(world: &mut World, tool: &str, action: u8, hit: Target) -> Result<bool> {
    if tool == "eyeposer" && action == 1 {
        if let Some(selected) = world.resource::<Selection>().0 {
            world.resource_mut::<Selection>().0 = None;
            let t = *world
                .get::<Transform>(selected)
                .ok_or("selected eye target was removed")?;
            let mut state = world.get::<State>(selected).cloned().unwrap_or_default();
            state.eye = Some(
                t.compute_matrix()
                    .inverse()
                    .transform_point3(hit.point)
                    .to_array(),
            );
            let path = world
                .get::<SpawnedProp>(selected)
                .ok_or("selected eye target missing")?
                .model
                .clone();
            prepare(world, &path, &state)?;
            remember(world);
            apply(world, selected, state)?;
            return Ok(true);
        }
    }
    if world.get::<npcs::NpcBody>(hit.entity).is_some()
        || world.get::<vehicles::VehicleBody>(hit.entity).is_some()
        || world.get::<devices::Device>(hit.entity).is_some()
    {
        return Err(
            "Posers currently support spawned model props, not animated NPCs or vehicles/devices"
                .into(),
        );
    }
    let path = world
        .get::<SpawnedProp>(hit.entity)
        .ok_or("select a spawned model prop")?
        .model
        .clone();
    let d = data(world, &path)?;
    let base = gpu_model(world, &path)?;
    let mut state = world.get::<State>(hit.entity).cloned().unwrap_or_default();
    let t = *world
        .get::<Transform>(hit.entity)
        .ok_or("pose transform missing")?;
    let local = t.compute_matrix().inverse().transform_point3(hit.point);
    let settings = &world.resource::<PlayState>().tools;
    let mut label = String::new();
    match tool {
        "inflator" => {
            if action == 3 {
                state.scales.clear();
            } else {
                let bone = nearest(&d, &base, &state, local)?;
                let group = d.physics_bones[bone];
                let amount = settings.number(tool, "step") * if action == 2 { -1. } else { 1. };
                let preserve = settings.number(tool, "preserve_length") == 1.;
                for (i, b) in d.skeleton.bones.iter().enumerate() {
                    if i != bone && (group < 0 || d.physics_bones[i] != group) {
                        continue;
                    }
                    let scale = state.scales.entry(i).or_insert([1.; 3]);
                    let name = b.name.to_ascii_lowercase();
                    for (axis, value) in scale.iter_mut().enumerate() {
                        if axis == 0 && preserve && (name.contains("arm") || name.contains("leg")) {
                            continue;
                        }
                        *value = (*value + amount).clamp(0.05, 5.);
                    }
                }
                label = d.skeleton.bones[bone].name.clone();
            }
        }
        "finger" => {
            let hand = settings.value(tool, "hand");
            let finger = settings.number(tool, "finger") as usize;
            let joint = settings.number(tool, "joint") as usize;
            let name = format!(
                "ValveBiped.Bip01_{hand}_Finger{finger}{}",
                if joint == 0 {
                    String::new()
                } else {
                    joint.to_string()
                }
            );
            let id = d.skeleton.bones.iter().position(|b| b.name == name).ok_or(
                "selected finger bone is absent; only ValveBiped finger naming is supported",
            )?;
            if action == 2 {
                let a = state.rotations.get(&id).copied().unwrap_or([0.; 3]);
                let mut p = world.resource_mut::<PlayState>();
                p.tools
                    .values
                    .insert("finger.curl".into(), a[2].to_string());
                p.tools
                    .values
                    .insert("finger.splay".into(), a[1].to_string());
                p.status = format!("Sampled {name}");
                p.dirty = true;
                return Ok(true);
            }
            if action == 3 {
                state.rotations.remove(&id);
            } else {
                state.rotations.insert(
                    id,
                    [
                        0.,
                        settings.number(tool, "splay"),
                        settings.number(tool, "curl"),
                    ],
                );
            }
            label = name;
        }
        "faceposer" if action == 3 => {
            state.flexes.clear();
        }
        "faceposer" => {
            let id = settings.number(tool, "descriptor") as usize;
            label = d
                .names
                .get(id)
                .ok_or("flex descriptor outside this model; choose a lower index")?
                .clone();
            if !d.parts.iter().any(|p| {
                p.flexes
                    .iter()
                    .any(|f| f.descriptor == id || f.pair == Some(id))
            }) {
                return Err("selected descriptor has no decoded vertex deltas".into());
            }
            if action == 2 {
                let mut p = world.resource_mut::<PlayState>();
                p.tools.values.insert(
                    "faceposer.weight".into(),
                    state.flexes.get(&id).unwrap_or(&0.).to_string(),
                );
                p.status = format!("Sampled flex {id}: {label}");
                p.dirty = true;
                return Ok(true);
            }
            state.flexes.insert(id, settings.number(tool, "weight"));
        }
        "eyeposer" => {
            if !d.parts.iter().any(|p| p.eye.is_some()) {
                return Err("model has no original eyeball mesh metadata".into());
            }
            if action == 3 {
                state.eye = None;
                world.resource_mut::<Selection>().0 = None;
            } else if action == 2
                || world
                    .resource::<ButtonInput<KeyCode>>()
                    .pressed(KeyCode::KeyE)
            {
                let eye = crate::source_player::aim_eye(world).ok_or("player eye missing")?;
                state.eye = Some(
                    t.compute_matrix()
                        .inverse()
                        .transform_point3(eye.translation)
                        .to_array(),
                );
            } else {
                iris(world, &path, &d)?;
                world.resource_mut::<Selection>().0 = Some(hit.entity);
                world.resource_mut::<PlayState>().status =
                    "Eyes selected. Left-click a target point; RMB looks at you; R resets.".into();
                return Ok(true);
            }
        }
        _ => return Ok(false),
    }
    prepare(world, &path, &state)?;
    remember(world);
    apply(world, hit.entity, state)?;
    world.resource_mut::<PlayState>().status = format!(
        "Applied {tool}: {label}. Original convex prop physics, not articulated ragdoll simulation."
    );
    Ok(true)
}
pub(crate) fn cancel(world: &mut World) {
    if world.resource::<PlayState>().tool != "eyeposer"
        || world.resource::<PlayState>().active_weapon != "weapon_gmod_tool"
        || crate::source_frontend::active(world)
    {
        world.resource_mut::<Selection>().0 = None;
    }
}
