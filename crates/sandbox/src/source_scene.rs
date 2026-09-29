//! Versioned local scenes retain modifiers and constraints. Legacy prop-array saves still load.
use super::*;
use tools::constraints;
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SavedScene {
    pub version: u32,
    pub props: Vec<SavedModel>,
    pub links: Vec<constraints::SavedLink>,
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
        },
    })
}
pub(super) fn snapshot(world: &mut World) -> SavedScene {
    let ids: Vec<_> = world
        .query_filtered::<Entity, With<SpawnedProp>>()
        .iter(world)
        .collect();
    snapshot_selection(world, &ids)
}
pub(super) fn snapshot_selection(world: &mut World, ids: &[Entity]) -> SavedScene {
    let props = ids
        .iter()
        .filter_map(|e| {
            let p = world.get::<SpawnedProp>(*e)?;
            let t = world.get::<Transform>(*e)?;
            let b = world.get::<RigidBody>(*e)?;
            Some(SavedModel {
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
        version: 2,
        props,
        links: constraints::snapshot(world, ids),
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
    if save.version != 2
        || save.props.len() > p.config.max_props
        || save.links.len() > p.tools.catalog.gun.max_constraints
    {
        return Err("unsupported scene version or scene limit exceeded".into());
    }
    for l in &save.links {
        constraints::validate(l, save.props.len())?;
    }
    for p in &save.props {
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
        tools::apply_properties(world, id, p.properties)?;
        ids.push(id);
    }
    for mut l in scene.links {
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
