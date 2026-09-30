//! Four-anchor independent pulley. Fixed guides and coupled cable tension, not native solver parity.
use super::*;
#[derive(Resource, Default)]
pub(crate) struct Stage(pub Vec<Target>);
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pulley {
    pub guides: [[f32; 3]; 2],
    pub stiffness: f32,
    pub damping: f32,
    pub force: f32,
    pub rigid: bool,
}
pub(crate) fn operate(world: &mut World, action: u8, hit: Target) -> Result<bool> {
    if action != 1 {
        world.resource_mut::<Stage>().0.clear();
        return Ok(if action == 3 {
            constraints::remove(world, hit.entity, Some("pulley"), true) > 0
        } else {
            true
        });
    }
    let mut points = world.resource::<Stage>().0.clone();
    if points.is_empty() {
        if world.get::<SpawnedProp>(hit.entity).is_none() {
            return Err("First pulley anchor must be a prop".into());
        }
        let t = world
            .get::<Transform>(hit.entity)
            .ok_or("missing anchor transform")?;
        points.push(Target {
            point: t.compute_matrix().inverse().transform_point3(hit.point),
            ..hit
        });
    } else if points.len() < 3 {
        points.push(hit); // Native pulley guides are fixed world positions.
    } else {
        let first = points[0];
        if first.entity == hit.entity || world.get::<SpawnedProp>(hit.entity).is_none() {
            return Err("Fourth pulley anchor must be a different prop".into());
        }
        let Some(t) = world.get::<Transform>(first.entity) else {
            world.resource_mut::<Stage>().0.clear();
            return Err("First pulley body was removed".into());
        };
        let start = t.transform_point(first.point);
        if world.query::<&constraints::Link>().iter(world).count()
            >= world
                .resource::<PlayState>()
                .tools
                .catalog
                .gun
                .max_constraints
        {
            return Err("constraint limit reached".into());
        }
        let mut link = constraints::between(
            world,
            "pulley",
            first.entity,
            hit.entity,
            start,
            hit.point,
            Vec3::Y,
        )?;
        let p = world.resource::<PlayState>();
        let n = |key| p.tools.number("pulley", key);
        link.length = start.distance(points[1].point)
            + points[1].point.distance(points[2].point)
            + points[2].point.distance(hit.point);
        link.width = n("width") * 0.01905;
        link.color = [
            n("color_r") / 255.,
            n("color_g") / 255.,
            n("color_b") / 255.,
        ];
        link.pulley = Some(Pulley {
            guides: [points[1].point.to_array(), points[2].point.to_array()],
            stiffness: n("constant"),
            damping: n("damping"),
            force: n("max_force"),
            rigid: n("rigid") == 1.,
        });
        remember(world);
        constraints::create(world, first.entity, hit.entity, link)?;
        world.resource_mut::<Stage>().0.clear();
        world.resource_mut::<PlayState>().status =
            "Pulley created: two prop anchors and two fixed guides".into();
        return Ok(true);
    }
    let count = points.len();
    world.resource_mut::<Stage>().0 = points;
    world.resource_mut::<PlayState>().status = format!(
        "Pulley {count}/4: choose {}",
        if count < 3 {
            "fixed guide"
        } else {
            "second prop anchor"
        }
    );
    Ok(true)
}
pub(crate) fn physics(world: &mut World) {
    let dt = world.resource::<Time<Fixed>>().delta_secs();
    let links: Vec<_> = world
        .query::<&constraints::Link>()
        .iter(world)
        .filter(|l| l.data.pulley.is_some())
        .cloned()
        .collect();
    for link in links {
        let p = link.data.pulley.as_ref().unwrap();
        let (Some(a), Some(b)) = (
            world.get::<Transform>(link.a),
            world.get::<Transform>(link.b),
        ) else {
            continue;
        };
        let start = a.transform_point(Vec3::from_array(link.data.anchor1));
        let end = b.transform_point(Vec3::from_array(link.data.anchor2));
        let guide1 = Vec3::from_array(p.guides[0]);
        let guide2 = Vec3::from_array(p.guides[1]);
        let dir1 = (guide1 - start).normalize_or_zero();
        let dir2 = (guide2 - end).normalize_or_zero();
        let length = start.distance(guide1) + guide1.distance(guide2) + guide2.distance(end);
        let va = world
            .get::<Velocity>(link.a)
            .map(|v| v.linvel.dot(-dir1))
            .unwrap_or(0.);
        let vb = world
            .get::<Velocity>(link.b)
            .map(|v| v.linvel.dot(-dir2))
            .unwrap_or(0.);
        let error = length - link.data.length;
        let force = if error > 0. || p.rigid {
            (error * p.stiffness + (va + vb) * p.damping)
                .clamp(if p.rigid { -p.force } else { 0. }, p.force)
        } else {
            0.
        };
        devices::impulse(world, link.a, dir1 * force, start, dt);
        devices::impulse(world, link.b, dir2 * force, end, dt);
    }
}
pub(crate) fn clear_stage(world: &mut World) {
    if crate::source_frontend::active(world) || world.resource::<PlayState>().tool != "pulley" {
        world.resource_mut::<Stage>().0.clear();
    }
}
