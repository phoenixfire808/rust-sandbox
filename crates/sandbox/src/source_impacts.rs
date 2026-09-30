//! Bounded surface marks and collision feedback. Not Source decal/particle parity.
use super::*;
use std::collections::VecDeque;

#[derive(Component)]
pub(crate) struct ImpactMark;

#[derive(Resource, Default)]
pub struct Impacts {
    marks: VecDeque<(Entity, f32)>,
    bursts: VecDeque<(Vec3, Vec3, f32)>,
    assets: Option<Option<(Handle<Mesh>, Handle<StandardMaterial>)>>,
}
impl Impacts {
    pub(super) fn push_burst(&mut self, point: Vec3, normal: Vec3, now: f32, limit: usize) {
        while self.bursts.len() >= limit {
            self.bursts.pop_front();
        }
        self.bursts
            .push_back((point, normal.normalize_or(Vec3::Y), now));
    }
}
pub fn burst(world: &mut World, point: Vec3, normal: Vec3) {
    let now = world.resource::<Time>().elapsed_secs();
    let limit = world
        .resource::<PlayState>()
        .spawn_catalog
        .runtime
        .impact_limit;
    let mut s = world.resource_mut::<Impacts>();
    s.push_burst(point, normal, now, limit);
}
pub fn mark(world: &mut World, target: Entity, point: Vec3, normal: Vec3) {
    if normal.length_squared() < 0.5 {
        return;
    }
    burst(world, point, normal);
    if world.get::<npcs::NpcBody>(target).is_some() {
        return;
    }
    let c = world.resource::<PlayState>().spawn_catalog.runtime.clone();
    if world.resource::<Impacts>().assets.is_none() {
        let image = {
            let source = world.resource::<MountedSource>();
            source.mounts.texture(&source.bsp, &c.impact_texture, false)
        };
        let assets = match image {
            Ok(image) => {
                let image = world.resource_mut::<Assets<Image>>().add(image);
                let material =
                    world
                        .resource_mut::<Assets<StandardMaterial>>()
                        .add(StandardMaterial {
                            base_color_texture: Some(image),
                            alpha_mode: AlphaMode::Multiply,
                            unlit: true,
                            cull_mode: None,
                            ..default()
                        });
                let mesh = world
                    .resource_mut::<Assets<Mesh>>()
                    .add(Rectangle::new(c.impact_size, c.impact_size));
                Some((mesh, material))
            }
            Err(e) => {
                eprintln!("IMPACT_MATERIAL_MISSING: {e}");
                None
            }
        };
        world.resource_mut::<Impacts>().assets = Some(assets);
    }
    let Some((mesh, material)) = world.resource::<Impacts>().assets.clone().flatten() else {
        return;
    };
    let Some(parent) = world.get::<GlobalTransform>(target) else {
        return;
    };
    let transform = Transform::from_translation(point + normal * c.impact_offset)
        .with_rotation(Quat::from_rotation_arc(Vec3::Z, normal.normalize()));
    let local =
        Transform::from_matrix(parent.compute_matrix().inverse() * transform.compute_matrix());
    let now = world.resource::<Time>().elapsed_secs();
    while world.resource::<Impacts>().marks.len() >= c.impact_limit {
        if let Some((old, _)) = world.resource_mut::<Impacts>().marks.pop_front() {
            if world.get_entity(old).is_ok() {
                world.despawn(old);
            }
        }
    }
    let e = world
        .spawn((
            ImpactMark,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            local,
            ChildOf(target),
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ))
        .id();
    world
        .resource_mut::<Impacts>()
        .marks
        .push_back((e, now + c.impact_seconds));
}
pub fn expire(world: &mut World) {
    let now = world.resource::<Time>().elapsed_secs();
    let seconds = world
        .resource::<PlayState>()
        .spawn_catalog
        .runtime
        .spark_seconds;
    world
        .resource_mut::<Impacts>()
        .bursts
        .retain(|(_, _, born)| now - *born < seconds);
    while world
        .resource::<Impacts>()
        .marks
        .front()
        .is_some_and(|(_, until)| *until <= now)
    {
        let (entity, _) = world.resource_mut::<Impacts>().marks.pop_front().unwrap();
        if world.get_entity(entity).is_ok() {
            world.despawn(entity);
        }
    }
}
pub fn draw(mut gizmos: Gizmos, time: Res<Time>, play: Res<PlayState>, state: Res<Impacts>) {
    let c = &play.spawn_catalog.runtime;
    for (point, normal, born) in &state.bursts {
        let age = (time.elapsed_secs() - *born).max(0.);
        let tangent = normal.any_orthonormal_vector();
        let across = normal.cross(tangent);
        for i in 0..6 {
            let angle = i as f32 * std::f32::consts::TAU / 6.;
            let velocity = (*normal + tangent * angle.cos() + across * angle.sin()) * c.spark_speed;
            let end = *point + velocity * age + Vec3::Y * play.config.gravity * age * age * 0.5;
            let tail = end - velocity * c.trace_seconds.min(age);
            gizmos.line(
                tail,
                end,
                Color::srgba(1., 0.7, 0.25, (1. - age / c.spark_seconds).max(0.)),
            );
        }
    }
}
pub fn clear(world: &mut World) {
    let marks = std::mem::take(&mut world.resource_mut::<Impacts>().marks);
    world.resource_mut::<Impacts>().bursts.clear();
    for (e, _) in marks {
        if world.get_entity(e).is_ok() {
            world.despawn(e);
        }
    }
}
