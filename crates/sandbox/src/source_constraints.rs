//! Rapier-backed staged constraints. Source solver and cable rendering are not equivalent.
use super::*;
use std::collections::BTreeSet;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedLink {
    pub kind: String,
    pub a: Option<usize>,
    pub b: Option<usize>,
    pub anchor1: [f32; 3],
    pub anchor2: [f32; 3],
    pub basis1: [f32; 4],
    pub basis2: [f32; 4],
    pub length: f32,
    pub nocollide: bool,
    pub rigid: bool,
    pub stiffness: f32,
    pub damping: f32,
    pub stretch_only: bool,
    pub friction: f32,
    pub width: f32,
    pub color: [f32; 3],
}
#[derive(Component, Clone)]
pub(crate) struct Link {
    pub a: Entity,
    pub b: Entity,
    pub data: SavedLink,
    visual: Option<Entity>,
}
#[derive(Resource)]
struct WorldAnchor(Entity);
#[derive(Resource)]
struct CableMesh(Handle<Mesh>);
pub(crate) fn supported(tool: &str) -> bool {
    matches!(
        tool,
        "weld" | "axis" | "ballsocket" | "rope" | "slider" | "elastic" | "nocollide"
    )
}
pub(crate) fn validate(d: &SavedLink, n: usize) -> Result<()> {
    if !supported(&d.kind)
        || d.a.is_some_and(|i| i >= n)
        || d.b.is_some_and(|i| i >= n)
        || d.a == d.b
        || d.anchor1
            .iter()
            .chain(&d.anchor2)
            .any(|v| !v.is_finite() || v.abs() > 1e6)
        || [d.basis1, d.basis2].iter().any(|q| {
            q.iter().any(|v| !v.is_finite()) || (Quat::from_array(*q).length() - 1.).abs() > 0.01
        })
        || [d.length, d.stiffness, d.damping, d.friction, d.width]
            .iter()
            .any(|v| !v.is_finite() || *v < 0. || *v > 1e6)
        || d.color
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
    {
        return Err("invalid saved constraint".into());
    }
    Ok(())
}
pub(crate) fn world_anchor(world: &mut World) -> Entity {
    if let Some(a) = world.get_resource::<WorldAnchor>() {
        return a.0;
    }
    let e = world
        .spawn((
            RigidBody::Fixed,
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    world.insert_resource(WorldAnchor(e));
    e
}
pub(crate) fn create(world: &mut World, a: Entity, b: Entity, d: SavedLink) -> Result<()> {
    if a == b || world.get::<RigidBody>(a).is_none() || world.get::<RigidBody>(b).is_none() {
        return Err("invalid constraint bodies".into());
    }
    let axes = match d.kind.as_str() {
        "weld" => JointAxesMask::LOCKED_FIXED_AXES,
        "axis" => JointAxesMask::LOCKED_REVOLUTE_AXES,
        "ballsocket" => JointAxesMask::LOCKED_SPHERICAL_AXES,
        "slider" => JointAxesMask::LOCKED_PRISMATIC_AXES,
        _ => JointAxesMask::empty(),
    };
    let mut joint = GenericJointBuilder::new(axes)
        .local_anchor1(Vec3::from_array(d.anchor1))
        .local_anchor2(Vec3::from_array(d.anchor2))
        .local_basis1(Quat::from_array(d.basis1))
        .local_basis2(Quat::from_array(d.basis2))
        .build();
    joint.set_contacts_enabled(!d.nocollide && d.kind != "nocollide");
    if d.kind == "rope" {
        joint.set_coupled_axes(JointAxesMask::LIN_AXES);
        joint.set_limits(
            JointAxis::LinX,
            [if d.rigid { d.length } else { 0. }, d.length],
        );
    }
    if d.kind == "elastic" {
        joint.set_coupled_axes(JointAxesMask::LIN_AXES);
        joint.set_motor_model(JointAxis::LinX, MotorModel::ForceBased);
        joint.set_motor_position(JointAxis::LinX, d.length, d.stiffness, d.damping);
    }
    if d.kind == "axis" && d.friction > 0. {
        joint.set_motor_velocity(JointAxis::AngX, 0., 1.);
        joint.set_motor_max_force(JointAxis::AngX, d.friction);
    }
    let entity = world
        .spawn((
            ImpulseJoint::new(a, TypedJoint::GenericJoint(joint)),
            ChildOf(b),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    let visual = if matches!(d.kind.as_str(), "rope" | "slider" | "elastic") {
        if !world.contains_resource::<CableMesh>() {
            let mesh = world
                .resource_mut::<Assets<Mesh>>()
                .add(Cylinder::new(1., 1.));
            world.insert_resource(CableMesh(mesh));
        }
        let mesh = world.resource::<CableMesh>().0.clone();
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgb(d.color[0], d.color[1], d.color[2]),
                unlit: true,
                ..default()
            });
        Some(
            world
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::default(),
                    ChildOf(entity),
                ))
                .id(),
        )
    } else {
        None
    };
    world.entity_mut(entity).insert(Link {
        a,
        b,
        data: d,
        visual,
    });
    for e in [a, b] {
        world.entity_mut(e).insert(Sleeping::default());
    }
    Ok(())
}
pub(crate) fn snapshot(world: &mut World, ids: &[Entity]) -> Vec<SavedLink> {
    world
        .query::<&Link>()
        .iter(world)
        .filter_map(|l| {
            let a = ids.iter().position(|e| *e == l.a);
            let b = ids.iter().position(|e| *e == l.b);
            if (a.is_none() && world.get::<SpawnedProp>(l.a).is_some())
                || (b.is_none() && world.get::<SpawnedProp>(l.b).is_some())
            {
                return None;
            }
            let mut d = l.data.clone();
            d.a = a;
            d.b = b;
            Some(d)
        })
        .collect()
}
pub(crate) fn connected(world: &mut World, entity: Entity) -> Vec<Entity> {
    let links: Vec<_> = world
        .query::<&Link>()
        .iter(world)
        .map(|l| (l.a, l.b))
        .collect();
    let mut ids = BTreeSet::from([entity]);
    loop {
        let n = ids.len();
        for &(a, b) in &links {
            if ids.contains(&a) && world.get::<SpawnedProp>(b).is_some() {
                ids.insert(b);
            }
            if ids.contains(&b) && world.get::<SpawnedProp>(a).is_some() {
                ids.insert(a);
            }
        }
        if ids.len() == n {
            break;
        }
    }
    ids.into_iter().collect()
}
pub(crate) fn remove(world: &mut World, target: Entity, kind: Option<&str>, undo: bool) -> usize {
    let ids: Vec<_> = world
        .query::<(Entity, &Link)>()
        .iter(world)
        .filter(|(_, l)| (l.a == target || l.b == target) && kind.is_none_or(|k| k == l.data.kind))
        .map(|(e, _)| e)
        .collect();
    if !ids.is_empty() && undo {
        remember(world)
    }
    let count = ids.len();
    for e in ids {
        world.despawn(e);
    }
    count
}
pub(crate) fn clear(world: &mut World) {
    let ids: Vec<_> = world
        .query_filtered::<Entity, With<Link>>()
        .iter(world)
        .collect();
    for e in ids {
        world.despawn(e);
    }
    world.resource_mut::<PlayState>().tools.stage = None;
}
pub(crate) fn operate(world: &mut World, tool: &str, action: u8, hit: Target) -> Result<bool> {
    if action == 3 {
        world.resource_mut::<PlayState>().tools.stage = None;
        return Ok(remove(world, hit.entity, Some(tool), true) > 0);
    }
    if tool == "nocollide" && action == 2 {
        if world.get::<SpawnedProp>(hit.entity).is_none() {
            return Ok(false);
        }
        let mut p = world
            .get::<Properties>(hit.entity)
            .cloned()
            .unwrap_or_default();
        p.world_only = !p.world_only;
        remember(world);
        apply_properties(world, hit.entity, p)?;
        return Ok(true);
    }
    if action == 2 && matches!(tool, "elastic" | "ballsocket") {
        return Ok(false);
    }
    let stage = world.resource::<PlayState>().tools.stage.clone();
    let mut first = if let Some((kind, local, mode)) = stage {
        if kind != tool || world.get::<Transform>(local.entity).is_none() {
            world.resource_mut::<PlayState>().tools.stage = None;
            return Ok(false);
        }
        if mode != action && tool != "rope" {
            return Ok(false);
        }
        let t = *world.get::<Transform>(local.entity).unwrap();
        Target {
            entity: local.entity,
            point: t.transform_point(local.point),
            normal: t.rotation * local.normal,
        }
    } else {
        if world.get::<SpawnedProp>(hit.entity).is_none()
            && !matches!(tool, "rope" | "elastic" | "slider")
        {
            return Ok(false);
        }
        let t = *world
            .get::<Transform>(hit.entity)
            .ok_or("target transform missing")?;
        let local = Target {
            entity: hit.entity,
            point: t.compute_matrix().inverse().transform_point3(hit.point),
            normal: t.rotation.inverse() * hit.normal,
        };
        world.resource_mut::<PlayState>().tools.stage = Some((tool.into(), local, action));
        world.resource_mut::<PlayState>().status = format!(
            "{}: select second target",
            world.resource::<PlayState>().tools.label(tool)
        );
        if tool == "slider" && action == 2 {
            let eye = Transform::from_translation(hit.point + hit.normal * 0.001)
                .looking_to(hit.normal, Vec3::Y);
            if let Some(second) = trace(world, eye, Some(hit.entity)) {
                return operate(world, tool, action, second);
            }
            world.resource_mut::<PlayState>().tools.stage = None;
            return Ok(false);
        }
        return Ok(true);
    };
    if first.entity == hit.entity {
        return Err("select a different body".into());
    }
    if world.get::<SpawnedProp>(first.entity).is_none()
        && world.get::<SpawnedProp>(hit.entity).is_none()
    {
        return Err("cannot constrain world to world".into());
    }
    if world.query::<&Link>().iter(world).count()
        >= world
            .resource::<PlayState>()
            .tools
            .catalog
            .gun
            .max_constraints
    {
        return Err("constraint limit reached".into());
    }
    remember(world);
    if (tool == "axis" && action == 1) || (tool == "weld" && action == 2) {
        let mut t = world
            .get_mut::<Transform>(first.entity)
            .ok_or("first target removed")?;
        let local = t.rotation.inverse() * (first.point - t.translation);
        t.rotation =
            Quat::from_rotation_arc(first.normal.normalize(), -hit.normal.normalize()) * t.rotation;
        t.translation = hit.point - t.rotation * local;
        first.point = hit.point;
    }
    let t1 = *world.get::<Transform>(first.entity).unwrap();
    let t2 = *world.get::<Transform>(hit.entity).unwrap();
    let point1 = if matches!(tool, "weld" | "ballsocket" | "axis") {
        hit.point
    } else {
        first.point
    };
    let axis = if tool == "slider" {
        (hit.point - first.point)
            .try_normalize()
            .unwrap_or(hit.normal)
    } else {
        hit.normal
    };
    let basis = if matches!(tool, "axis" | "slider") {
        Quat::from_rotation_arc(Vec3::X, axis.normalize())
    } else {
        Quat::IDENTITY
    };
    let p = world.resource::<PlayState>();
    let number = |k| p.tools.number(tool, k);
    let d = SavedLink {
        kind: tool.into(),
        a: None,
        b: None,
        anchor1: t1
            .compute_matrix()
            .inverse()
            .transform_point3(point1)
            .to_array(),
        anchor2: t2
            .compute_matrix()
            .inverse()
            .transform_point3(hit.point)
            .to_array(),
        basis1: (t1.rotation.inverse() * basis).to_array(),
        basis2: (t2.rotation.inverse() * basis).to_array(),
        length: (first.point.distance(hit.point) + number("addlength") * 0.01905).max(0.001),
        nocollide: number("nocollide") == 1.,
        rigid: number("rigid") == 1.,
        stiffness: number("constant"),
        damping: number("damping"),
        stretch_only: number("stretch_only") == 1.,
        friction: number("hingefriction"),
        width: number("width") * 0.01905,
        color: [
            number("color_r") / 255.,
            number("color_g") / 255.,
            number("color_b") / 255.,
        ],
    };
    create(world, first.entity, hit.entity, d)?;
    world.resource_mut::<PlayState>().tools.stage = if tool == "rope" && action == 2 {
        Some((
            tool.into(),
            Target {
                entity: hit.entity,
                point: t2.compute_matrix().inverse().transform_point3(hit.point),
                normal: t2.rotation.inverse() * hit.normal,
            },
            action,
        ))
    } else {
        None
    };
    world.resource_mut::<PlayState>().status = format!("Created {tool} constraint");
    Ok(true)
}
pub(crate) fn update(world: &mut World) {
    let links: Vec<_> = world
        .query::<(Entity, &Link)>()
        .iter(world)
        .map(|(e, l)| (e, l.clone()))
        .collect();
    for (entity, l) in links {
        let (Some(a), Some(b)) = (
            world.get::<Transform>(l.a).copied(),
            world.get::<Transform>(l.b).copied(),
        ) else {
            world.despawn(entity);
            continue;
        };
        let start = a.transform_point(Vec3::from_array(l.data.anchor1));
        let end = b.transform_point(Vec3::from_array(l.data.anchor2));
        let delta = end - start;
        if let Some(v) = l.visual {
            if delta.length_squared() > 1e-10 {
                let t = Transform {
                    translation: b
                        .compute_matrix()
                        .inverse()
                        .transform_point3((start + end) * 0.5),
                    rotation: b.rotation.inverse()
                        * Quat::from_rotation_arc(Vec3::Y, delta.normalize()),
                    scale: Vec3::new(l.data.width * 0.5, delta.length(), l.data.width * 0.5),
                };
                if let Some(mut transform) = world.get_mut::<Transform>(v) {
                    *transform = t;
                }
            }
        }
        if l.data.kind == "elastic" && l.data.stretch_only {
            let active = delta.length() >= l.data.length;
            if let Some(mut j) = world.get_mut::<ImpulseJoint>(entity) {
                let mut data = j.data;
                data.as_mut().set_motor_position(
                    JointAxis::LinX,
                    l.data.length,
                    if active { l.data.stiffness } else { 0. },
                    if active { l.data.damping } else { 0. },
                );
                if j.data != data {
                    j.data = data;
                }
            }
        }
    }
}
