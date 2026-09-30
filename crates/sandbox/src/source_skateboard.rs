//! Independent skateboard riding using existing prop, scene and occupancy lifecycles.
use super::*;
use sandbox_catalog::skateboard::{Profile, SkateCatalog};
#[path = "source_skate_input.rs"]
mod input;
#[path = "source_skate_physics.rs"]
mod physics;
pub(crate) use input::use_pressed;
pub(crate) use physics::step;

pub const RAIL: &str = "models/jcode/skate_rail.mdl";
pub const BANK: &str = "models/jcode/skate_bank.mdl";
#[derive(Resource)]
pub struct Catalog(pub SkateCatalog);
#[derive(Resource, Default)]
pub struct Controls {
    release_elapsed: f32,
    pub steer: f32,
    pub push: bool,
    pub brake: bool,
    pub slide: bool,
    pub manual: f32,
    pub flip: f32,
    pub spin: f32,
    pub catch: bool,
    pub grind: bool,
    pub charge: f32,
    pub pop: Option<f32>,
    pub nollie: bool,
    pub goofy: bool,
    pub status: String,
    previous_charge: bool,
    previous_front: bool,
    armed: bool,
    previous_pad: bool,
    previous_buttons: u16,
    owner: Option<Entity>,
    was_blocked: bool,
}
#[derive(Component, Default)]
pub struct Board {
    pub wheel_spin: f32,
    pub steering: f32,
    pub grounded: bool,
    pub airtime: f32,
    pub next_push: f32,
    pub next_pop: f32,
    pub pop_ignore: f32,
    pub caught: bool,
    pub trick: bool,
    pub was_ridden: bool,
    pub last_velocity: Vec3,
    pub grind: bool,
}
fn profile<'a>(catalog: &'a SkateCatalog, id: &str) -> Option<&'a Profile> {
    catalog
        .profiles
        .iter()
        .find(|p| id == format!("skate_{}", p.id))
}
pub fn is_model(path: &str) -> bool {
    matches!(
        path,
        RAIL | BANK
            | "models/jcode/skate_realism.mdl"
            | "models/jcode/skate_skate.mdl"
            | "models/jcode/skate_session.mdl"
            | "models/jcode/skate_sandbox.mdl"
    )
}
pub fn model_id(id: &str) -> String {
    format!("models/jcode/{id}.mdl")
}
pub fn definition(world: &World, id: &str) -> Option<sandbox_catalog::spawn::Vehicle> {
    let catalog = &world.get_resource::<Catalog>()?.0;
    let p = profile(catalog, id)?;
    let b = &catalog.board;
    Some(sandbox_catalog::spawn::Vehicle {
        id: id.into(),
        kind: "skateboard".into(),
        mass: b.mass,
        engine_accel: 0.,
        brake_accel: b.brake_accel,
        max_speed: b.max_speed,
        steer_rate: b.steering_angle,
        suspension: b.suspension,
        spring: b.spring,
        damping: b.damping,
        grip: p.grip,
        seat_x: 0.,
        seat_y: b.thickness * 0.5,
        seat_z: 0.,
        seat_yaw: std::f32::consts::FRAC_PI_2,
        forward_yaw: 0.,
        eye_height: b.eye_height,
        exit_distance: b.exit_distance,
        entry_seconds: b.enter_seconds,
        exit_seconds: b.exit_seconds,
        pose: "@cidle_pistol".into(),
        scope: p.scope.clone(),
        remaining: p.remaining.clone(),
    })
}
pub fn attach(world: &mut World, entity: Entity, id: &str) -> Result<()> {
    let c = definition(world, id).ok_or("unknown skateboard preset")?;
    world.entity_mut(entity).insert((
        vehicles::VehicleBody { id: id.into() },
        Board::default(),
        ColliderMassProperties::Mass(c.mass),
        GravityScale(0.),
        ExternalForce::default(),
        Damping {
            linear_damping: 0.,
            angular_damping: 0.05,
        },
        Friction::coefficient(0.15),
        Restitution::coefficient(0.),
        Ccd::enabled(),
    ));
    Ok(())
}
/// Procedural original geometry has a stable virtual identity for saves and duplication.
pub(super) fn model(world: &mut World, path: &str) -> Result<GpuModel> {
    let b = world.resource::<Catalog>().0.board.clone();
    let mut pieces: Vec<(Mesh, Color)> = Vec::new();
    let (collider, half) = if path == RAIL {
        let half = Vec3::new(b.rail_width * 0.5, b.rail_height * 0.5, b.rail_length * 0.5);
        pieces.push((
            Mesh::from(Cuboid::from_size(half * 2.)),
            Color::srgb(0.5, 0.55, 0.6),
        ));
        (Collider::cuboid(half.x, half.y, half.z), half)
    } else if path == BANK {
        let half = Vec3::new(b.bank_width * 0.5, b.thickness, b.bank_length * 0.5);
        pieces.push((
            Mesh::from(Cuboid::from_size(half * 2.)),
            Color::srgb(0.38, 0.42, 0.48),
        ));
        (Collider::cuboid(half.x, half.y, half.z), half)
    } else {
        let deck = Vec3::new(b.width, b.thickness, b.length);
        pieces.push((
            Mesh::from(Cuboid::from_size(deck)),
            Color::srgb(0.15, 0.18, 0.2),
        ));
        let grip = Mesh::from(Cuboid::new(
            b.width * 0.94,
            b.thickness * 0.1,
            b.length * 0.92,
        ))
        .transformed_by(Transform::from_xyz(0., b.thickness * 0.55, 0.));
        pieces.push((grip, Color::srgb(0.035, 0.035, 0.04)));
        let mut shapes = vec![(
            Vec3::ZERO,
            Quat::IDENTITY,
            Collider::cuboid(deck.x * 0.5, deck.y * 0.5, deck.z * 0.5),
        )];
        for z in [-b.wheelbase * 0.5, b.wheelbase * 0.5] {
            let truck = Mesh::from(Cuboid::new(b.track, b.thickness, b.wheel_radius))
                .transformed_by(Transform::from_xyz(0., -b.wheel_radius, z));
            pieces.push((truck, Color::srgb(0.6, 0.62, 0.64)));
            for x in [-b.track * 0.5, b.track * 0.5] {
                let pos = Vec3::new(x, -b.wheel_radius, z);
                let rot = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
                pieces.push((
                    Mesh::from(Cylinder::new(b.wheel_radius, b.wheel_radius))
                        .transformed_by(Transform::from_translation(pos).with_rotation(rot)),
                    Color::srgb(0.86, 0.75, 0.45),
                ));
                shapes.push((
                    pos,
                    rot,
                    Collider::cylinder(b.wheel_radius * 0.5, b.wheel_radius),
                ));
            }
        }
        (
            Collider::compound(shapes),
            Vec3::new(
                b.width * 0.5 + b.wheel_radius,
                b.wheel_radius * 2.,
                b.length * 0.5,
            ),
        )
    };
    let mut parts = Vec::new();
    let mut geometry = Vec::new();
    for (mut mesh, color) in pieces {
        mesh.duplicate_vertices();
        use bevy::render::mesh::VertexAttributeValues;
        let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(v)) => v.clone(),
            _ => return Err("procedural positions missing".into()),
        };
        let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
            Some(VertexAttributeValues::Float32x3(v)) => v.clone(),
            _ => return Err("procedural normals missing".into()),
        };
        let uv = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            Some(VertexAttributeValues::Float32x2(v)) => v.clone(),
            _ => vec![[0.; 2]; positions.len()],
        };
        let n = positions.len();
        geometry.push(crate::source_assets::Geometry {
            positions,
            normals,
            uv,
            light_uv: vec![[0.; 2]; n],
            weights: vec![[(0, 0.); 3]; n],
            source_indices: (0..n).collect(),
        });
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: color,
                unlit: true,
                ..default()
            });
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        parts.push((mesh, material));
    }
    Ok(GpuModel {
        parts,
        geometry: geometry.into(),
        collider,
        center: Vec3::ZERO,
        half,
    })
}
pub fn spawn(world: &mut World, choice: &str) {
    if world.resource::<vehicles::Occupancy>().vehicle.is_some() || npcs::dead(world) {
        return;
    }
    let path = match choice {
        "rail" => RAIL.into(),
        "bank" => BANK.into(),
        id if definition(world, id).is_some() => model_id(id),
        _ => return,
    };
    let Some(eye) = camera(world) else { return };
    let b = world.resource::<Catalog>().0.board.clone();
    let hit = tools::trace(world, eye, None);
    let Some(hit) = hit
        .filter(|h| h.normal.y > 0.5 && h.point.distance(eye.translation) <= b.spawn_distance * 4.)
    else {
        world.resource_mut::<PlayState>().status =
            "Aim at clear ground nearby to place a skateboard or obstacle".into();
        return;
    };
    let yaw = world
        .get_resource::<crate::source_player::PlayerState>()
        .map_or(0., |p| p.yaw);
    let rotation = Quat::from_rotation_y(yaw)
        * if choice == "bank" {
            Quat::from_rotation_x(b.bank_angle)
        } else {
            Quat::IDENTITY
        };
    let height = match choice {
        "rail" => b.rail_height * 0.5,
        "bank" => b.bank_length * 0.5 * b.bank_angle.sin() + b.thickness,
        _ => b.wheel_radius * 2. + b.suspension,
    };
    remember(world);
    match spawn_model(
        world,
        &path,
        hit.point + Vec3::Y * (height + 0.02),
        rotation,
        matches!(choice, "rail" | "bank"),
    ) {
        Ok(id) => {
            if definition(world, choice).is_some() {
                if let Err(e) = attach(world, id, choice) {
                    world.despawn(id);
                    world.resource_mut::<PlayState>().status = e.to_string();
                    return;
                }
            }
            let mut p = world.resource_mut::<PlayState>();
            p.menu_open = false;
            p.search_focus = false;
            p.dirty = true;
            p.status = "Skateboard: aim + E to ride | W push | S brake | A/D carve | Space charge/release | F7 guide".into();
        }
        Err(e) => world.resource_mut::<PlayState>().status = format!("Skateboard spawn: {e}"),
    }
}
pub fn update(world: &mut World) {
    input::update(world);
    let b = world.resource::<Catalog>().0.board.clone();
    let dt = world.resource::<Time>().delta_secs().min(0.1);
    let rows: Vec<_> = world
        .query::<(Entity, &Transform, &Velocity, &Board, &Children)>()
        .iter(world)
        .map(|(e, t, v, s, children)| {
            (
                e,
                v.linvel.dot(t.rotation * Vec3::NEG_Z),
                s.steering,
                s.wheel_spin,
                children.iter().collect::<Vec<_>>(),
            )
        })
        .collect();
    for (e, speed, steer, spin, children) in rows {
        let spin = (spin + speed * dt / b.wheel_radius).rem_euclid(std::f32::consts::TAU);
        if let Some(mut state) = world.get_mut::<Board>(e) {
            state.wheel_spin = spin;
        }
        // Procedural mesh order is deck, grip, then truck and its two wheels per axle.
        for (index, child) in children.into_iter().enumerate() {
            if !(2..=7).contains(&index) {
                continue;
            }
            let front = index <= 4;
            let z = if front {
                -b.wheelbase * 0.5
            } else {
                b.wheelbase * 0.5
            };
            let turn =
                Quat::from_rotation_y(steer * b.steering_angle * if front { 1. } else { -1. });
            let wheel = index != 2 && index != 5;
            let x = if wheel {
                if index == 3 || index == 6 {
                    -b.track * 0.5
                } else {
                    b.track * 0.5
                }
            } else {
                0.
            };
            let axle = Vec3::new(0., -b.wheel_radius, z);
            let pivot = Vec3::new(x, -b.wheel_radius, z);
            let matrix = Mat4::from_translation(axle)
                * Mat4::from_quat(turn)
                * Mat4::from_translation(-axle)
                * if wheel {
                    Mat4::from_translation(pivot)
                        * Mat4::from_rotation_x(-spin)
                        * Mat4::from_translation(-pivot)
                } else {
                    Mat4::IDENTITY
                };
            if let Some(mut transform) = world.get_mut::<Transform>(child) {
                *transform = Transform::from_matrix(matrix);
            }
        }
    }
}
pub fn riding(world: &World) -> bool {
    world
        .get_resource::<vehicles::Occupancy>()
        .and_then(|v| v.vehicle)
        .is_some_and(|e| world.get::<Board>(e).is_some())
}
pub fn hud(world: &World) -> String {
    let c = world.resource::<Controls>();
    let id = &world.resource::<vehicles::Occupancy>().definition;
    let label =
        profile(&world.resource::<Catalog>().0, id).map_or("Skateboard", |p| p.label.as_str());
    let speed = world
        .resource::<vehicles::Occupancy>()
        .vehicle
        .and_then(|e| world.get::<Velocity>(e))
        .map_or(0., |v| v.linvel.length());
    let pop = if profile(&world.resource::<Catalog>().0, id).is_some_and(|p| p.controls == "dual") {
        "Space load + Up pop"
    } else {
        "Space load/release"
    };
    format!(
        "{label} | {} | {speed:.1} m/s | {} | W push / S brake / A-D carve / {pop} / J-L flip / I-K spin / C catch / M-N manual / G grind / E exit",
        if c.goofy { "goofy" } else { "regular" },
        c.status
    )
}
