//! Tool dispatch, authored settings, and reversible per-prop modifiers.
use super::*;
use sandbox_catalog::toolgun::ToolCatalog;
use serde::{Deserialize, Serialize};
#[path = "source_constraints.rs"]
pub(crate) mod constraints;
#[path = "source_devices.rs"]
pub(crate) mod devices;
#[path = "source_posers.rs"]
pub(crate) mod posers;
#[path = "source_pulley.rs"]
pub(crate) mod pulley;
#[path = "source_render_tools.rs"]
pub(crate) mod render_tools;

#[derive(Clone, Copy)]
pub struct Shot {
    pub time: f32,
    pub point: Vec3,
}
#[derive(Clone, Copy)]
pub(crate) struct Target {
    pub entity: Entity,
    pub point: Vec3,
    pub normal: Vec3,
}
pub struct ToolState {
    pub(super) input_blocked: bool,
    pub catalog: ToolCatalog,
    pub values: BTreeMap<String, String>,
    pub shot: Option<Shot>,
    pub(crate) stage: Option<(String, Target, u8)>,
    clipboard: Option<(SavedScene, Vec3)>,
    material_cache: BTreeMap<String, StandardMaterial>,
}
impl Default for ToolState {
    fn default() -> Self {
        let catalog = crate::compiled_tool_catalog();
        let mut values: BTreeMap<_, _> = catalog
            .options
            .iter()
            .map(|o| (format!("{}.{}", o.tool, o.key), o.default.clone()))
            .collect();
        if let Ok(bytes) = std::fs::read(project_root().join("local/tool-settings.json")) {
            if let Ok(saved) = serde_json::from_slice::<BTreeMap<String, String>>(&bytes) {
                for o in &catalog.options {
                    let key = format!("{}.{}", o.tool, o.key);
                    if let Some(v) = saved.get(&key).filter(|v| o.enabled && o.accepts(v)) {
                        values.insert(key, v.clone());
                    }
                }
            }
        }
        Self {
            input_blocked: false,
            catalog,
            values,
            shot: None,
            stage: None,
            clipboard: None,
            material_cache: BTreeMap::new(),
        }
    }
}
impl ToolState {
    pub fn label(&self, id: &str) -> &str {
        self.catalog
            .tools
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.label.as_str())
            .unwrap_or("Tool Gun")
    }
    pub fn value(&self, tool: &str, key: &str) -> String {
        self.values
            .get(&format!("{tool}.{key}"))
            .cloned()
            .unwrap_or_default()
    }
    pub fn number(&self, tool: &str, key: &str) -> f32 {
        self.value(tool, key).parse().unwrap_or(0.)
    }
}

/// Read-only help contract for the tool HUD. Device targeting is cached by the
/// device input system so drawing never performs a mutable physics query.
pub(crate) fn help_hint(world: &World) -> Option<String> {
    let play = world.get_resource::<PlayState>()?;
    let button = devices::button_hint(world);
    if play.active_weapon != "weapon_gmod_tool" {
        return button;
    }
    let definition = play.tools.catalog.tools.iter().find(|t| t.id == play.tool);
    let mut hint = if let Some((tool, _, action)) = &play.tools.stage {
        let stage_action = if *action == 1 { "LMB" } else { "RMB" };
        format!("{stage_action}: select the second target for {tool}")
    } else if let Some(definition) = definition {
        let mut controls = format!("LMB: {} · RMB: {}", definition.left, definition.right);
        if definition.reload != "No reload action" {
            controls.push_str(&format!(" · R: {}", definition.reload));
        }
        controls
    } else {
        "Choose a tool to see its controls".to_owned()
    };
    if play.tool == "balloon" {
        hint.push_str(&format!(
            "\nLift {:.0} N · rope {:.2} m · LMB tethers, RMB floats free",
            play.tools.number("balloon", "force"),
            play.tools.number("balloon", "ropelength")
        ));
    }
    if let Some(button) = button {
        hint.push('\n');
        hint.push_str(&button);
    }
    Some(hint)
}

pub(super) fn setting(world: &mut World, key: String, direction: i32) {
    let mut play = world.resource_mut::<PlayState>();
    let tool = play.tool.clone();
    let Some(option) = play
        .tools
        .catalog
        .options
        .iter()
        .find(|o| o.tool == tool && o.key == key && o.enabled)
        .cloned()
    else {
        return;
    };
    let current = play.tools.value(&tool, &key);
    let value = match option.kind.as_str() {
        "bool" => if current == "1" { "0" } else { "1" }.to_string(),
        "choice" => {
            let choices: Vec<_> = option.choices.split('|').collect();
            let index = choices.iter().position(|v| *v == current).unwrap_or(0) as i32;
            choices[(index + direction).rem_euclid(choices.len() as i32) as usize].to_string()
        }
        _ => format!(
            "{}",
            (current.parse::<f32>().unwrap_or(option.min) + option.step * direction as f32)
                .clamp(option.min, option.max)
        ),
    };
    play.tools.values.insert(format!("{tool}.{key}"), value);
    play.dirty = true;
    drop(play);
    persist_settings(world);
}
pub(super) fn set_fraction(world: &mut World, tool: &str, key: &str, fraction: f32) {
    let mut p = world.resource_mut::<PlayState>();
    let Some(o) = p
        .tools
        .catalog
        .options
        .iter()
        .find(|o| o.tool == tool && o.key == key && o.enabled && o.kind == "number")
        .cloned()
    else {
        return;
    };
    let v = ((o.min + (o.max - o.min) * fraction.clamp(0., 1.)) / o.step).round() * o.step;
    p.tools.values.insert(
        format!("{tool}.{key}"),
        format!("{}", v.clamp(o.min, o.max)),
    );
}
pub(super) fn reset_settings(world: &mut World) {
    let mut p = world.resource_mut::<PlayState>();
    let tool = p.tool.clone();
    let values: Vec<_> = p
        .tools
        .catalog
        .options
        .iter()
        .filter(|o| o.tool == tool)
        .map(|o| (format!("{}.{}", o.tool, o.key), o.default.clone()))
        .collect();
    p.tools.values.extend(values);
    p.dirty = true;
    drop(p);
    persist_settings(world);
}
pub(super) fn persist_settings(world: &mut World) {
    let p = world.resource::<PlayState>();
    let result = (|| -> Result<()> {
        std::fs::create_dir_all(&p.storage_root)?;
        std::fs::write(
            p.storage_root.join("tool-settings.json"),
            serde_json::to_vec_pretty(&p.tools.values)?,
        )?;
        Ok(())
    })();
    if let Err(e) = result {
        world.resource_mut::<PlayState>().status = format!("Tool settings could not be saved: {e}");
    }
}
#[derive(Component, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Properties {
    pub color: [f32; 4],
    pub material: String,
    pub gravity: bool,
    pub physics_material: String,
    pub world_only: bool,
    pub trail: Option<render_tools::Trail>,
}
impl Default for Properties {
    fn default() -> Self {
        Self {
            color: [1.; 4],
            material: String::new(),
            gravity: true,
            physics_material: String::new(),
            world_only: false,
            trail: None,
        }
    }
}
pub(crate) fn validate_properties(world: &World, p: &Properties) -> Result<()> {
    if let Some(trail) = &p.trail {
        render_tools::validate_trail(world, trail)?;
    }
    if p.color
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
    {
        return Err("invalid prop colour".into());
    }
    if !p.material.is_empty() {
        crate::source_assets::virtual_path(&p.material)?;
    }
    if !p.physics_material.is_empty()
        && !world
            .resource::<PlayState>()
            .tools
            .catalog
            .materials
            .iter()
            .any(|m| m.id == p.physics_material)
    {
        return Err("unknown physical material".into());
    }
    Ok(())
}
fn material(world: &mut World, path: &str) -> Result<StandardMaterial> {
    if let Some(m) = world.resource::<PlayState>().tools.material_cache.get(path) {
        return Ok(m.clone());
    }
    let source = world.resource::<MountedSource>();
    let mat = source.mounts.material(&source.bsp, path)?;
    let image = mat
        .base_texture()
        .map(|p| source.mounts.texture(&source.bsp, p, true))
        .transpose()?;
    let m = StandardMaterial {
        base_color_texture: image.map(|i| world.resource_mut::<Assets<Image>>().add(i)),
        alpha_mode: mat
            .alpha_test()
            .map(AlphaMode::Mask)
            .unwrap_or(if mat.translucent() {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            }),
        unlit: true,
        ..default()
    };
    world
        .resource_mut::<PlayState>()
        .tools
        .material_cache
        .insert(path.into(), m.clone());
    Ok(m)
}
pub(crate) fn prepare_properties(world: &mut World, p: &Properties) -> Result<()> {
    validate_properties(world, p)?;
    if let Some(trail) = &p.trail {
        render_tools::prepare_trail(world, trail)?;
    }
    if !p.material.is_empty() {
        material(world, &p.material)?;
    }
    Ok(())
}
pub(crate) fn apply_visual_properties(
    world: &mut World,
    entity: Entity,
    p: &Properties,
) -> Result<()> {
    prepare_properties(world, p)?;
    let path = world
        .get::<SpawnedProp>(entity)
        .ok_or("target is no longer a prop")?
        .model
        .clone();
    let model = gpu_model(world, &path)?;
    let children: Vec<_> = world
        .get::<Children>(entity)
        .map(|c| {
            c.iter()
                .filter(|e| {
                    world.get::<Mesh3d>(*e).is_some()
                        && world.get::<super::impacts::ImpactMark>(*e).is_none()
                })
                .collect()
        })
        .unwrap_or_default();
    let override_material = if p.material.is_empty() {
        None
    } else {
        Some(material(world, &p.material)?)
    };
    for (child, (_, original)) in children.into_iter().zip(&model.parts) {
        let original = world
            .get::<posers::EyeMaterial>(child)
            .map(|m| &m.0)
            .unwrap_or(original);
        let mut mat = override_material
            .clone()
            .or_else(|| {
                world
                    .resource::<Assets<StandardMaterial>>()
                    .get(original)
                    .cloned()
            })
            .ok_or("missing prop material")?;
        mat.base_color = Color::srgba(p.color[0], p.color[1], p.color[2], p.color[3]);
        if p.color[3] < 1. {
            mat.alpha_mode = AlphaMode::Blend;
        }
        let handle = world.resource_mut::<Assets<StandardMaterial>>().add(mat);
        world.entity_mut(child).insert(MeshMaterial3d(handle));
    }
    Ok(())
}
pub(crate) fn apply_properties(world: &mut World, entity: Entity, p: Properties) -> Result<()> {
    apply_visual_properties(world, entity, &p)?;
    if world
        .get::<Properties>(entity)
        .is_none_or(|old| old.trail != p.trail)
    {
        render_tools::apply_trail(world, entity, p.trail.clone())?;
    }
    let physical = world
        .resource::<PlayState>()
        .tools
        .catalog
        .materials
        .iter()
        .find(|m| m.id == p.physics_material)
        .cloned();
    let (friction, restitution) = physical
        .map(|m| (m.friction, m.restitution))
        .unwrap_or((0.5, 0.));
    world.entity_mut(entity).insert((
        GravityScale(if p.gravity { 1. } else { 0. }),
        Friction::coefficient(friction),
        Restitution::coefficient(restitution),
        CollisionGroups::new(
            Group::GROUP_2,
            if p.world_only {
                Group::GROUP_1
            } else {
                Group::ALL
            },
        ),
        Sleeping::default(),
        p,
    ));
    Ok(())
}
pub(crate) fn trace(world: &mut World, eye: Transform, exclude: Option<Entity>) -> Option<Target> {
    let player = world
        .get_resource::<crate::source_player::PlayerState>()
        .map(|p| p.entity);
    let range = world.resource::<PlayState>().tools.catalog.gun.trace_range;
    let mut q = world.query::<(
        &RapierContextSimulation,
        &RapierContextColliders,
        &RapierRigidBodySet,
        &RapierContextJoints,
        &RapierQueryPipeline,
    )>();
    let (simulation, colliders, rigidbody_set, joints, query_pipeline) = q.single(world).ok()?;
    let context = bevy_rapier3d::plugin::RapierContext {
        simulation,
        colliders,
        rigidbody_set,
        joints,
        query_pipeline,
    };
    let predicate = |e: Entity| Some(e) != exclude && Some(e) != player;
    let filter = QueryFilter::default().predicate(&predicate);
    let (entity, hit) =
        context.cast_ray_and_get_normal(eye.translation, *eye.forward(), range, true, filter)?;
    Some(Target {
        entity,
        point: hit.point,
        normal: hit.normal,
    })
}
pub(crate) fn input(world: &mut World) {
    let p = world.resource::<PlayState>();
    if crate::source_frontend::active(world)
        || npcs::dead(world)
        || devices::viewing(world)
        || p.active_weapon != "weapon_gmod_tool"
        || p.menu_open
        || p.tools.input_blocked
        || world.resource::<vehicles::Occupancy>().vehicle.is_some()
        || world.resource::<weapons::WeaponState>().equip_blocked
    {
        return;
    }
    if !world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .iter(world)
        .any(|w| w.focused)
    {
        return;
    }
    let mouse = world.resource::<ButtonInput<MouseButton>>();
    let action = if mouse.just_pressed(MouseButton::Left) {
        1
    } else if mouse.just_pressed(MouseButton::Right) {
        2
    } else if world
        .resource::<ButtonInput<KeyCode>>()
        .just_pressed(KeyCode::KeyR)
    {
        3
    } else {
        return;
    };
    let Some(eye) = camera(world) else { return };
    let Some(hit) = trace(world, eye, None) else {
        if action == 3 {
            world.resource_mut::<posers::Selection>().0 = None;
            world.resource_mut::<pulley::Stage>().0.clear();
            let mut p = world.resource_mut::<PlayState>();
            p.tools.stage = None;
            if p.tool == "duplicator" {
                p.tools.clipboard = None;
            }
            p.status = "Tool selection cleared".into();
        }
        return;
    };
    let tool = world.resource::<PlayState>().tool.clone();
    match operate(world, &tool, action, hit) {
        Ok(true) => {
            audio::emit(world, "tool.accept", None, 1.);
            let time = world.resource::<Time>().elapsed_secs();
            world.resource_mut::<PlayState>().tools.shot = Some(Shot {
                time,
                point: hit.point,
            });
        }
        Ok(false) => {}
        Err(e) => world.resource_mut::<PlayState>().status = format!("{tool}: {e}"),
    }
}
fn operate(world: &mut World, tool: &str, action: u8, hit: Target) -> Result<bool> {
    if posers::supported(tool) {
        return posers::operate(world, tool, action, hit);
    }
    if matches!(tool, "paint" | "trails") {
        return render_tools::operate(world, tool, action, hit);
    }
    if devices::supported(tool) || tool == "editentity" {
        return devices::operate(world, tool, action, hit);
    }
    if world.get::<npcs::NpcBody>(hit.entity).is_some() && tool != "remover" {
        return Err(
            "This tool does not yet support live NPCs; use Remover or scene save/load".into(),
        );
    }
    if constraints::supported(tool) {
        if tool == "pulley" {
            return pulley::operate(world, action, hit);
        }
        return constraints::operate(world, tool, action, hit);
    }
    if tool == "duplicator" {
        return duplicate(world, action, hit);
    }
    if world.get::<SpawnedProp>(hit.entity).is_none() {
        return Ok(false);
    }
    if tool == "remover" {
        if action == 3 {
            let n = constraints::remove(world, hit.entity, None, true);
            return Ok(n > 0);
        }
        let ids = if action == 2 {
            constraints::connected(world, hit.entity)
        } else {
            vec![hit.entity]
        };
        remember(world);
        for entity in ids {
            constraints::remove(world, entity, None, false);
            world.despawn(entity);
        }
        world.resource_mut::<PlayState>().status = "Removed prop selection".into();
        return Ok(true);
    }
    if tool == "freeze" {
        remember(world);
        world.entity_mut(hit.entity).insert((
            if action == 1 {
                RigidBody::Fixed
            } else {
                RigidBody::Dynamic
            },
            Velocity::zero(),
            Sleeping::default(),
        ));
        return Ok(true);
    }
    let mut properties = world
        .get::<Properties>(hit.entity)
        .cloned()
        .unwrap_or_default();
    match (tool, action) {
        ("colour", 2) => {
            let mut p = world.resource_mut::<PlayState>();
            for (key, v) in ["r", "g", "b", "a"].iter().zip(properties.color) {
                p.tools
                    .values
                    .insert(format!("colour.{key}"), format!("{}", (v * 255.).round()));
            }
            p.dirty = true;
            return Ok(true);
        }
        ("colour", 1) => {
            let p = world.resource::<PlayState>();
            properties.color = ["r", "g", "b", "a"].map(|key| p.tools.number(tool, key) / 255.);
        }
        ("colour", 3) => properties.color = [1.; 4],
        ("material", 2) => {
            world
                .resource_mut::<PlayState>()
                .tools
                .values
                .insert("material.override".into(), properties.material.clone());
            world.resource_mut::<PlayState>().dirty = true;
            return Ok(true);
        }
        ("material", 1) => {
            properties.material = world.resource::<PlayState>().tools.value(tool, "override")
        }
        ("material", 3) => properties.material.clear(),
        ("physprop", 1) => {
            let p = world.resource::<PlayState>();
            properties.gravity = p.tools.number(tool, "gravity_toggle") == 1.;
            properties.physics_material = p.tools.value(tool, "material");
        }
        _ => return Ok(false),
    }
    prepare_properties(world, &properties)?;
    remember(world);
    apply_properties(world, hit.entity, properties)?;
    world.resource_mut::<PlayState>().status = format!(
        "Applied {}",
        world.resource::<PlayState>().tools.label(tool)
    );
    Ok(true)
}
fn duplicate(world: &mut World, action: u8, hit: Target) -> Result<bool> {
    if action == 3 {
        world.resource_mut::<PlayState>().tools.clipboard = None;
        return Ok(true);
    }
    if action == 2 {
        if world.get::<SpawnedProp>(hit.entity).is_none() {
            return Ok(false);
        }
        let ids = constraints::connected(world, hit.entity);
        let scene = snapshot_selection(world, &ids);
        world.resource_mut::<PlayState>().tools.clipboard = Some((scene, hit.point));
        world.resource_mut::<PlayState>().status = "Assembly copied. Left click to paste.".into();
        return Ok(true);
    }
    let Some((scene, origin)) = world.resource::<PlayState>().tools.clipboard.clone() else {
        return Err("Right click a prop to copy first".into());
    };
    let count = world.query::<&SpawnedProp>().iter(world).count();
    if count + scene.props.len() > world.resource::<PlayState>().config.max_props {
        return Err("prop limit reached".into());
    }
    let links = world.query::<&constraints::Link>().iter(world).count();
    if links + scene.links.len()
        > world
            .resource::<PlayState>()
            .tools
            .catalog
            .gun
            .max_constraints
    {
        return Err("constraint limit reached".into());
    }
    validate_scene(world, &scene)?;
    remember(world);
    spawn_scene(world, scene, hit.point - origin)?;
    world.resource_mut::<PlayState>().status = "Pasted copied assembly".into();
    Ok(true)
}
