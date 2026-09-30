//! Versioned local scenes retain modifiers and constraints. Legacy prop-array saves still load.
use super::*;
use tools::constraints;
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SavedScene {
    pub version: u32,
    pub props: Vec<SavedModel>,
    pub links: Vec<constraints::SavedLink>,
    #[serde(default)]
    pub paint: Vec<tools::render_tools::SavedPaint>,
}
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum Input {
    Current(SavedScene),
    Legacy(Vec<SavedModel>),
}
pub(super) fn read(bytes: &[u8]) -> Result<SavedScene> {
    Ok(match serde_json::from_slice::<Input>(bytes)? {
        Input::Current(s) => s,
        Input::Legacy(props) => SavedScene {
            version: 2,
            props,
            links: Vec::new(),
            paint: Vec::new(),
        },
    })
}
pub(super) fn snapshot(world: &mut World) -> SavedScene {
    let ids: Vec<_> = world
        .query_filtered::<Entity, With<SpawnedProp>>()
        .iter(world)
        .collect();
    let mut save = snapshot_selection(world, &ids);
    save.paint = tools::render_tools::snapshot(world, &ids, true);
    save
}
pub(super) fn snapshot_selection(world: &mut World, ids: &[Entity]) -> SavedScene {
    let props = ids
        .iter()
        .filter_map(|e| {
            let p = world.get::<SpawnedProp>(*e)?;
            let t = world.get::<Transform>(*e)?;
            let b = world.get::<RigidBody>(*e)?;
            Some(SavedModel {
                pose: world.get::<tools::posers::State>(*e).cloned(),
                device: world.get::<tools::devices::Device>(*e).cloned(),
                vehicle: world.get::<vehicles::VehicleBody>(*e).map(|v| v.id.clone()),
                npc: world.get::<npcs::NpcBody>(*e).map(|n| n.id.clone()),
                health: world.get::<weapons::Health>(*e).copied(),
                model: p.model.clone(),
                position: t.translation.to_array(),
                rotation: t.rotation.to_array(),
                frozen: *b == RigidBody::Fixed,
                properties: world
                    .get::<tools::Properties>(*e)
                    .cloned()
                    .unwrap_or_default(),
            })
        })
        .collect();
    SavedScene {
        version: 6,
        props,
        links: constraints::snapshot(world, ids),
        paint: tools::render_tools::snapshot(world, ids, false),
    }
}
pub(super) fn remember(world: &mut World) {
    let save = snapshot(world);
    let mut p = world.resource_mut::<PlayState>();
    if p.undo.len() == 32 {
        p.undo.remove(0);
    }
    p.undo.push(save);
}
pub(super) fn validate_scene(world: &mut World, save: &SavedScene) -> Result<()> {
    let p = world.resource::<PlayState>();
    if ![2, 3, 4, 5, 6].contains(&save.version)
        || save.props.iter().filter(|p| p.device.is_some()).count()
            > p.tools.catalog.gun.max_devices
        || save.props.iter().filter(|p| p.npc.is_some()).count() > p.spawn_catalog.npc_rules.limit
        || save.props.len() > p.config.max_props
        || save.links.len() > p.tools.catalog.gun.max_constraints
        || save.paint.len() > p.tools.catalog.gun.max_marks
    {
        return Err("unsupported scene version or scene limit exceeded".into());
    }
    for mark in &save.paint {
        tools::render_tools::validate_paint(world, mark, save.props.len())?;
    }
    for l in &save.links {
        constraints::validate(l, save.props.len())?;
        if [l.a, l.b]
            .into_iter()
            .flatten()
            .any(|i| save.props[i].npc.is_some())
        {
            return Err("constraints on live NPCs are not supported".into());
        }
    }
    for p in &save.props {
        if let Some(pose) = &p.pose {
            if p.npc.is_some() || p.vehicle.is_some() || p.device.is_some() {
                return Err("posed typed actors are not supported".into());
            }
            tools::posers::prepare(world, &p.model, pose)?;
        }
        if let Some(d) = &p.device {
            if p.npc.is_some() || p.vehicle.is_some() {
                return Err("conflicting construction entity state".into());
            }
            tools::devices::validate(world, d, &p.model)?;
        }
        if p.npc.is_some() && (p.vehicle.is_some() || p.frozen) {
            return Err("conflicting NPC scene state".into());
        }
        if let Some(id) = &p.npc {
            let n = npcs::definition(world, id).ok_or("unknown saved NPC")?;
            if n.model != p.model {
                return Err("mismatched saved NPC model".into());
            }
            npcs::prepare(world, id)?;
        }
        if let Some(id) = &p.vehicle {
            let catalog = &world.resource::<PlayState>().spawn_catalog;
            let skateboard = skateboard::definition(world, id).is_some() && p.model == skateboard::model_id(id);
            if !skateboard && (!catalog.vehicles.iter().any(|v| &v.id == id)
                || !catalog
                    .entries
                    .iter()
                    .any(|e| &e.id == id && e.model == p.model))
            {
                return Err("unknown or mismatched saved vehicle".into());
            }
        }
        if p.health.is_some_and(|h| {
            !h.0.is_finite()
                || h.0 <= 0.
                || h.0
                    > p.npc
                        .as_ref()
                        .and_then(|id| npcs::definition(world, id))
                        .map(|n| n.health)
                        .unwrap_or(
                            world
                                .resource::<PlayState>()
                                .spawn_catalog
                                .runtime
                                .prop_health,
                        )
        }) {
            return Err("invalid prop health".into());
        }
        crate::source_assets::virtual_path(&p.model)?;
        if !p.position.iter().chain(&p.rotation).all(|v| v.is_finite())
            || (Quat::from_array(p.rotation).length() - 1.).abs() > 0.01
        {
            return Err("invalid save transform".into());
        }
        gpu_model(world, &p.model)?;
        tools::prepare_properties(world, &p.properties)?;
    }
    Ok(())
}
pub(super) fn spawn_scene(world: &mut World, scene: SavedScene, offset: Vec3) -> Result<()> {
    if tools::render_tools::count(world) + scene.paint.len()
        > world.resource::<PlayState>().tools.catalog.gun.max_marks
    {
        return Err("paint limit reached".into());
    }
    if world.query::<&tools::devices::Device>().iter(world).count()
        + scene.props.iter().filter(|p| p.device.is_some()).count()
        > world.resource::<PlayState>().tools.catalog.gun.max_devices
    {
        return Err("construction device limit reached".into());
    }
    if world.query::<&npcs::NpcBody>().iter(world).count()
        + scene.props.iter().filter(|p| p.npc.is_some()).count()
        > world.resource::<PlayState>().spawn_catalog.npc_rules.limit
    {
        return Err("NPC limit reached".into());
    }
    let mut ids = Vec::new();
    // All fallible asset preparation and limit checks precede changes to the world.
    for p in scene.props {
        let id = spawn_model(
            world,
            &p.model,
            Vec3::from_array(p.position) + offset,
            Quat::from_array(p.rotation),
            p.frozen,
        )?;
        if let Some(pose) = p.pose {
            tools::posers::apply(world, id, pose)?;
        }
        tools::apply_properties(world, id, p.properties)?;
        if let Some(mut device) = p.device {
            device.height += offset.y;
            if let Some(point) = &mut device.look_at {
                *point = (Vec3::from_array(*point) + offset).to_array();
            }
            tools::devices::attach(world, id, device)?;
        }
        if let Some(vehicle) = p.vehicle {
            vehicles::attach(world, id, &vehicle)?;
        }
        if let Some(npc) = p.npc {
            npcs::attach(world, id, &npc)?;
        }
        if let Some(health) = p.health {
            world.entity_mut(id).insert(health);
        }
        ids.push(id);
    }
    for mut mark in scene.paint {
        let owner = mark.target.map(|i| ids[i]);
        if owner.is_none() {
            mark.point = (Vec3::from_array(mark.point) + offset).to_array();
        }
        tools::render_tools::mark(world, owner, mark)?;
    }
    for mut l in scene.links {
        if let Some(p) = &mut l.pulley {
            for guide in &mut p.guides {
                *guide = (Vec3::from_array(*guide) + offset).to_array();
            }
        }
        let a = if let Some(i) = l.a {
            ids[i]
        } else {
            l.anchor1 = (Vec3::from_array(l.anchor1) + offset).to_array();
            constraints::world_anchor(world)
        };
        let b = if let Some(i) = l.b {
            ids[i]
        } else {
            l.anchor2 = (Vec3::from_array(l.anchor2) + offset).to_array();
            constraints::world_anchor(world)
        };
        constraints::create(world, a, b, l)?;
    }
    Ok(())
}
pub(super) fn restore(world: &mut World, save: SavedScene) -> Result<()> {
    validate_scene(world, &save)?;
    audio::clear(world);
    physgun::reset(world);
    tools::render_tools::clear(world);
    world.resource_mut::<tools::devices::Remote>().0 = None;
    world.resource_mut::<tools::posers::Selection>().0 = None;
    vehicles::release(world, None);
    weapons::clear_transients(world);
    spawn::clear(world);
    constraints::clear(world);
    let ids: Vec<_> = world
        .query_filtered::<Entity, With<SpawnedProp>>()
        .iter(world)
        .collect();
    for e in ids {
        world.despawn(e);
    }
    world.resource_mut::<PlayState>().held = None;
    spawn_scene(world, save, Vec3::ZERO)
}
