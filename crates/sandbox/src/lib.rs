use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use sandbox_catalog::{
    content::Content,
    scene::{SavedProp, Scene as SavedScene},
    Result,
};
use std::path::PathBuf;

include!(concat!(env!("OUT_DIR"), "/content_generated.rs"));

#[derive(Resource, Clone)]
pub struct Catalog(pub Content);
#[derive(Component, Debug)]
pub struct Prop {
    pub definition: usize,
}
#[derive(Resource, Default)]
pub struct Session {
    pub undo: Vec<SavedScene>,
    pub held: Option<Entity>,
    pub selected: usize,
    pub status: String,
    pub last_save: Option<PathBuf>,
}
#[derive(Resource)]
pub struct Aim {
    pub origin: Vec3,
    pub direction: Vec3,
    pub distance: f32,
}
#[derive(Resource, Default)]
pub struct Palette(pub Vec<(Handle<Mesh>, Handle<StandardMaterial>)>);

pub struct SandboxPlugin;
impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        let content = compiled_content();
        let hz = content.world.fixed_hz as f64;
        let distance = content.world.grab_distance;
        app.insert_resource(Catalog(content))
            .insert_resource(Time::<Fixed>::from_hz(hz))
            .insert_resource(TimestepMode::Fixed {
                dt: (1. / hz) as f32,
                substeps: 1,
            })
            .insert_resource(Aim {
                origin: Vec3::new(8., 6., 12.),
                direction: Vec3::NEG_Z,
                distance,
            })
            .init_resource::<Session>()
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule())
            .add_systems(Startup, setup_scene)
            .add_systems(FixedUpdate, drive_grab.before(PhysicsSet::SyncBackend));
    }
}

pub fn setup_scene(world: &mut World) {
    let content = world.resource::<Catalog>().0.clone();
    for mut config in world.query::<&mut RapierConfiguration>().iter_mut(world) {
        config.gravity = Vec3::Y * content.world.gravity;
    }
    let h = content.world.ground_half_size;
    world.spawn((
        Name::new("Ground"),
        Transform::from_xyz(0., -0.25, 0.),
        RigidBody::Fixed,
        Collider::cuboid(h, 0.25, h),
        Friction::coefficient(0.8),
    ));
    restore(world, &SavedScene::from_content(&content)).expect("validated compiled scene");
    world.resource_mut::<Session>().status =
        "Ready. Original procedural assets. Not GMod compatible.".into();
}

pub fn spawn_prop(world: &mut World, index: usize, state: &SavedProp) -> Entity {
    let def = world.resource::<Catalog>().0.props[index].clone();
    let visual = world
        .get_resource::<Palette>()
        .and_then(|p| p.0.get(index))
        .cloned();
    let mut entity = world.spawn((
        Name::new(def.label.clone()),
        Prop { definition: index },
        Transform::from_translation(Vec3::from_array(state.position))
            .with_rotation(Quat::from_array(state.rotation)),
        if state.frozen {
            RigidBody::Fixed
        } else {
            RigidBody::Dynamic
        },
        Collider::cuboid(def.size_x / 2., def.size_y / 2., def.size_z / 2.),
        ColliderMassProperties::Mass(def.mass),
        Friction::coefficient(def.friction),
        Restitution::coefficient(def.restitution),
        Velocity {
            linvel: Vec3::from_array(state.linear_velocity),
            angvel: Vec3::from_array(state.angular_velocity),
        },
        Damping {
            linear_damping: 0.05,
            angular_damping: 0.1,
        },
        Ccd::enabled(),
        Sleeping::default(),
    ));
    if let Some((mesh, material)) = visual {
        entity.insert((Mesh3d(mesh), MeshMaterial3d(material)));
    }
    entity.id()
}

pub fn snapshot(world: &mut World) -> SavedScene {
    let defs = world.resource::<Catalog>().0.props.clone();
    let props = world
        .query::<(&Prop, &Transform, &Velocity, &RigidBody)>()
        .iter(world)
        .map(|(p, t, v, b)| SavedProp {
            prop_id: defs[p.definition].id.clone(),
            position: t.translation.to_array(),
            rotation: t.rotation.to_array(),
            linear_velocity: v.linvel.to_array(),
            angular_velocity: v.angvel.to_array(),
            frozen: *b == RigidBody::Fixed,
        })
        .collect();
    SavedScene { version: 1, props }
}

pub fn restore(world: &mut World, scene: &SavedScene) -> Result<()> {
    let catalog = world.resource::<Catalog>().0.clone();
    scene.validate(&catalog)?; // Validate everything before mutating the live world.
    let entities: Vec<_> = world
        .query_filtered::<Entity, With<Prop>>()
        .iter(world)
        .collect();
    for e in entities {
        world.despawn(e);
    }
    world.resource_mut::<Session>().held = None;
    for p in &scene.props {
        let index = catalog
            .props
            .iter()
            .position(|d| d.id == p.prop_id)
            .unwrap();
        spawn_prop(world, index, p);
    }
    Ok(())
}
fn remember(world: &mut World) {
    let scene = snapshot(world);
    let mut session = world.resource_mut::<Session>();
    if session.undo.len() == 32 {
        session.undo.remove(0);
    }
    session.undo.push(scene);
}

#[derive(Debug, Clone)]
pub enum Action {
    Spawn(usize, Vec3),
    Remove(Entity),
    Freeze(Entity, bool),
    Grab(Entity),
    Release,
    Duplicate(Entity),
    Undo,
    Reset,
    Save,
    Load(PathBuf),
}
pub fn apply_action(world: &mut World, action: Action) -> Result<()> {
    match action {
        Action::Spawn(index, position) => {
            let c = &world.resource::<Catalog>().0;
            let def = c.props.get(index).ok_or("unknown prop definition")?;
            if !position.is_finite() || position.abs().max_element() > 10000. {
                return Err("spawn position out of bounds".into());
            }
            let id = def.id.clone();
            let max = c.world.max_props;
            if world.query::<&Prop>().iter(world).count() >= max {
                return Err("prop limit reached".into());
            }
            remember(world);
            spawn_prop(
                world,
                index,
                &SavedProp {
                    prop_id: id,
                    position: position.to_array(),
                    rotation: [0., 0., 0., 1.],
                    linear_velocity: [0.; 3],
                    angular_velocity: [0.; 3],
                    frozen: false,
                },
            );
        }
        Action::Remove(e) => {
            if world.get::<Prop>(e).is_none() {
                return Err("not a prop".into());
            }
            remember(world);
            world.despawn(e);
            if world.resource::<Session>().held == Some(e) {
                world.resource_mut::<Session>().held = None;
            }
        }
        Action::Freeze(e, frozen) => {
            if world.get::<Prop>(e).is_none() {
                return Err("not a prop".into());
            }
            remember(world);
            world.entity_mut(e).insert((
                if frozen {
                    RigidBody::Fixed
                } else {
                    RigidBody::Dynamic
                },
                Velocity::zero(),
                Sleeping::default(),
            ));
            if world.resource::<Session>().held == Some(e) {
                world.resource_mut::<Session>().held = None;
            }
        }
        Action::Grab(e) => {
            if world.get::<Prop>(e).is_none() {
                return Err("not a prop".into());
            }
            remember(world);
            world
                .entity_mut(e)
                .insert((RigidBody::Dynamic, Sleeping::disabled()));
            world.resource_mut::<Session>().held = Some(e);
        }
        Action::Release => {
            let e = world.resource_mut::<Session>().held.take();
            if let Some(e) = e {
                if let Ok(mut e) = world.get_entity_mut(e) {
                    e.insert(Sleeping::default());
                }
            }
        }
        Action::Duplicate(e) => {
            let p = world.get::<Prop>(e).ok_or("not a prop")?.definition;
            let t = *world.get::<Transform>(e).ok_or("missing transform")?;
            let max = world.resource::<Catalog>().0.world.max_props;
            if world.query::<&Prop>().iter(world).count() >= max {
                return Err("prop limit reached".into());
            }
            let id = world.resource::<Catalog>().0.props[p].id.clone();
            remember(world);
            spawn_prop(
                world,
                p,
                &SavedProp {
                    prop_id: id,
                    position: (t.translation + Vec3::Y * 2.).to_array(),
                    rotation: t.rotation.to_array(),
                    linear_velocity: [0.; 3],
                    angular_velocity: [0.; 3],
                    frozen: false,
                },
            );
        }
        Action::Undo => {
            let previous = world.resource_mut::<Session>().undo.pop();
            if let Some(s) = previous {
                restore(world, &s)?;
            } else {
                return Err("nothing to undo".into());
            }
        }
        Action::Reset => {
            remember(world);
            let s = SavedScene::from_content(&world.resource::<Catalog>().0);
            restore(world, &s)?;
        }
        Action::Save => {
            let s = snapshot(world);
            let path = s.save_new(
                &world.resource::<Catalog>().0,
                &project_root().join("local/saves"),
            )?;
            world.resource_mut::<Session>().last_save = Some(path.clone());
            world.resource_mut::<Session>().status = format!("Saved {}", path.display());
            return Ok(());
        }
        Action::Load(path) => {
            let s = SavedScene::load(&path, &world.resource::<Catalog>().0)?;
            remember(world);
            restore(world, &s)?;
        }
    }
    world.resource_mut::<Session>().status = "Action applied. Z undoes the last edit.".into();
    Ok(())
}

pub fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn drive_grab(
    session: Res<Session>,
    aim: Res<Aim>,
    mut props: Query<(&Transform, &mut Velocity), With<Prop>>,
) {
    if let Some(e) = session.held {
        if let Ok((t, mut v)) = props.get_mut(e) {
            v.linvel = ((aim.origin + aim.direction * aim.distance - t.translation) * 12.)
                .clamp_length_max(30.);
            v.angvel = Vec3::ZERO;
        }
    }
}

/// Ray against a rotated authored box. Used for UI targeting, not physics collision response.
pub fn box_hit(origin: Vec3, direction: Vec3, t: &Transform, half: Vec3) -> Option<f32> {
    let o = t.rotation.inverse() * (origin - t.translation);
    let d = t.rotation.inverse() * direction;
    let mut near = 0f32;
    let mut far = 200f32;
    for axis in 0..3 {
        if d[axis].abs() < 1e-6 {
            if o[axis].abs() > half[axis] {
                return None;
            }
        } else {
            let a = (-half[axis] - o[axis]) / d[axis];
            let b = (half[axis] - o[axis]) / d[axis];
            near = near.max(a.min(b));
            far = far.min(a.max(b));
            if near > far {
                return None;
            }
        }
    }
    Some(near)
}
pub fn target(world: &mut World) -> Option<Entity> {
    let aim = world.resource::<Aim>();
    let (origin, direction) = (aim.origin, aim.direction);
    let defs = world.resource::<Catalog>().0.props.clone();
    let ground_distance = if direction.y < -1e-6 && origin.y > 0. {
        -origin.y / direction.y
    } else {
        200.
    };
    world
        .query::<(Entity, &Prop, &Transform)>()
        .iter(world)
        .filter_map(|(e, p, t)| {
            let d = &defs[p.definition];
            box_hit(
                origin,
                direction,
                t,
                Vec3::new(d.size_x, d.size_y, d.size_z) / 2.,
            )
            .filter(|v| *v < ground_distance)
            .map(|v| (e, v))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(e, _)| e)
}

pub fn headless_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, SandboxPlugin));
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1. / 60.),
    ));
    app.finish();
    app.cleanup();
    app.update();
    app
}
pub fn smoke() -> Result<()> {
    let mut app = headless_app();
    for _ in 0..240 {
        app.update();
    }
    let s = snapshot(app.world_mut());
    s.validate(&app.world().resource::<Catalog>().0)?;
    if s.props.len() != 5
        || s.props
            .iter()
            .any(|p| p.position[1] < 0. || p.position[1] > 3.)
    {
        return Err(format!("physics acceptance failed: {s:?}").into());
    }
    apply_action(app.world_mut(), Action::Spawn(0, Vec3::new(5., 4., 0.)))?;
    if snapshot(app.world_mut()).props.len() != 6 {
        return Err("spawn failed".into());
    }
    apply_action(app.world_mut(), Action::Undo)?;
    if snapshot(app.world_mut()).props.len() != 5 {
        return Err("undo failed".into());
    }
    println!("PASS: compiled spreadsheet content -> Bevy ECS -> Rapier floor collisions -> spawn -> undo");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_content_matches_sheets() {
        let c = sandbox_catalog::content::load(&project_root().join("sheets")).unwrap();
        assert_eq!(compiled_content(), c);
    }
    #[test]
    fn physics_spawn_undo_acceptance() {
        smoke().unwrap();
    }
    #[test]
    fn freeze_remove_duplicate_undo() {
        let mut a = headless_app();
        let w = a.world_mut();
        let e = w
            .query_filtered::<Entity, With<Prop>>()
            .iter(w)
            .next()
            .unwrap();
        apply_action(w, Action::Freeze(e, true)).unwrap();
        assert_eq!(*w.get::<RigidBody>(e).unwrap(), RigidBody::Fixed);
        apply_action(w, Action::Duplicate(e)).unwrap();
        assert_eq!(snapshot(w).props.len(), 6);
        apply_action(w, Action::Remove(e)).unwrap();
        assert_eq!(snapshot(w).props.len(), 5);
        apply_action(w, Action::Undo).unwrap();
        assert_eq!(snapshot(w).props.len(), 6);
    }
    #[test]
    fn invalid_restore_preserves_world() {
        let mut a = headless_app();
        let w = a.world_mut();
        let before = snapshot(w);
        let mut bad = before.clone();
        bad.props[0].prop_id = "missing".into();
        assert!(restore(w, &bad).is_err());
        assert_eq!(snapshot(w), before);
    }
    #[test]
    fn oriented_ray_hit() {
        let t = Transform::from_xyz(0., 0., -5.).with_rotation(Quat::from_rotation_y(0.4));
        assert!(box_hit(Vec3::ZERO, Vec3::NEG_Z, &t, Vec3::ONE).is_some());
        assert!(box_hit(Vec3::ZERO, Vec3::X, &t, Vec3::ONE).is_none());
    }
}
