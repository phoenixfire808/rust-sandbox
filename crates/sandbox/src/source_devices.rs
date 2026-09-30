//! Independent construction entities. Configuration is sheet-owned and scene-persistent.
use super::*;
use std::collections::BTreeSet;

#[derive(Component, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Device {
    pub kind: String,
    pub values: BTreeMap<String, String>,
    pub height: f32,
    #[serde(default)]
    pub look_at: Option<[f32; 3]>,
    #[serde(skip)]
    pub enabled: bool,
    #[serde(skip)]
    deadline: Option<f32>,
    #[serde(skip)]
    last_emit: f32,
}
impl Device {
    pub fn number(&self, key: &str) -> f32 {
        self.values
            .get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.)
    }
    pub fn text(&self, key: &str) -> &str {
        self.values.get(key).map(String::as_str).unwrap_or("")
    }
    fn axis(&self) -> Vec3 {
        match self.text("axis") {
            "X" => Vec3::X,
            "Z" => Vec3::Z,
            _ => Vec3::Y,
        }
    }
}
#[derive(Component)]
struct DeviceLight;
#[derive(Resource, Default)]
pub struct Remote(pub Option<Entity>);
pub(crate) fn viewing(world: &World) -> bool {
    world.get_resource::<Remote>().is_some_and(|r| {
        r.0.is_some_and(|e| {
            world
                .get::<Device>(e)
                .is_some_and(|d| d.kind == "camera" && d.enabled)
        })
    })
}
#[derive(Resource, Default)]
pub(crate) struct Controls {
    pub down: BTreeSet<String>,
    pub pressed: BTreeSet<String>,
    pub(crate) aimed_button: Option<Entity>,
    previous: BTreeSet<String>,
    blocked: BTreeSet<String>,
}
pub(crate) const KEYS: [(&str, KeyCode); 10] = [
    ("Numpad0", KeyCode::Numpad0),
    ("Numpad1", KeyCode::Numpad1),
    ("Numpad2", KeyCode::Numpad2),
    ("Numpad3", KeyCode::Numpad3),
    ("Numpad4", KeyCode::Numpad4),
    ("Numpad5", KeyCode::Numpad5),
    ("Numpad6", KeyCode::Numpad6),
    ("Numpad7", KeyCode::Numpad7),
    ("Numpad8", KeyCode::Numpad8),
    ("Numpad9", KeyCode::Numpad9),
];
pub(crate) fn supported(kind: &str) -> bool {
    matches!(
        kind,
        "balloon"
            | "button"
            | "dynamite"
            | "emitter"
            | "hoverball"
            | "lamp"
            | "light"
            | "thruster"
            | "wheel"
            | "camera"
    )
}
pub(crate) fn validate(world: &World, d: &Device, model: &str) -> Result<()> {
    let catalog = &world.resource::<PlayState>().tools.catalog;
    if d.look_at
        .is_some_and(|p| p.iter().any(|v| !v.is_finite() || v.abs() > 1e6))
    {
        return Err("invalid camera look target".into());
    }
    if !supported(&d.kind)
        || !d.height.is_finite()
        || d.height.abs() > 1e6
        || d.text("model") != model
    {
        return Err("invalid saved construction device".into());
    }
    let options: Vec<_> = catalog
        .options
        .iter()
        .filter(|o| o.tool == d.kind && o.enabled)
        .collect();
    if d.values.len() != options.len()
        || options
            .iter()
            .any(|o| d.values.get(&o.key).is_none_or(|v| !o.accepts(v)))
    {
        return Err("invalid or incomplete device settings".into());
    }
    Ok(())
}
pub(crate) fn attach(world: &mut World, entity: Entity, mut d: Device) -> Result<()> {
    let model = &world
        .get::<SpawnedProp>(entity)
        .ok_or("device body missing")?
        .model;
    validate(world, &d, model)?;
    let children: Vec<_> = world
        .get::<Children>(entity)
        .map(|c| {
            c.iter()
                .filter(|e| world.get::<DeviceLight>(*e).is_some())
                .collect()
        })
        .unwrap_or_default();
    for child in children {
        world.despawn(child);
    }
    d.enabled = d.number("starton") == 1.;
    d.deadline = None;
    d.last_emit = 0.;
    if d.kind == "balloon" {
        // The stock gmod_balloon entity disables gravity and supplies its own
        // continuous upward force in PhysicsSimulate.
        if let Some(mut properties) = world.get_mut::<Properties>(entity) {
            properties.gravity = false;
        }
        world.entity_mut(entity).insert(GravityScale(0.));
    }
    let color = Color::srgb(
        d.number("r") / 255.,
        d.number("g") / 255.,
        d.number("b") / 255.,
    );
    if d.kind == "light" {
        world.spawn((
            DeviceLight,
            PointLight {
                color,
                intensity: d.number("brightness"),
                range: d.number("range"),
                shadows_enabled: false,
                ..default()
            },
            Transform::default(),
            ChildOf(entity),
        ));
    } else if d.kind == "lamp" {
        world.spawn((
            DeviceLight,
            SpotLight {
                color,
                intensity: d.number("brightness"),
                range: d.number("range"),
                outer_angle: d.number("fov").to_radians() * 0.5,
                inner_angle: 0.,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ChildOf(entity),
        ));
    }
    world
        .entity_mut(entity)
        .insert((ColliderMassProperties::Mass(d.number("mass")), d));
    Ok(())
}
pub(crate) fn operate(world: &mut World, tool: &str, action: u8, hit: Target) -> Result<bool> {
    if action == 3 {
        return Ok(false);
    }
    if tool == "editentity" {
        let d = world
            .get::<Device>(hit.entity)
            .cloned()
            .ok_or("Select a supported construction device to edit")?;
        let mut p = world.resource_mut::<PlayState>();
        p.tool = d.kind.clone();
        for (key, value) in d.values {
            p.tools.values.insert(format!("{}.{}", d.kind, key), value);
        }
        p.menu_open = true;
        p.dirty = true;
        p.status =
            "Device settings sampled. Change controls then click the same device to apply.".into();
        return Ok(true);
    }
    let p = world.resource::<PlayState>();
    let values = p
        .tools
        .catalog
        .options
        .iter()
        .filter(|o| o.tool == tool && o.enabled)
        .map(|o| (o.key.clone(), p.tools.value(tool, &o.key)))
        .collect();
    let d = Device {
        kind: tool.into(),
        values,
        height: hit.point.y,
        look_at: if tool == "camera" && action == 2 {
            Some(hit.point.to_array())
        } else {
            None
        },
        enabled: false,
        deadline: None,
        last_emit: 0.,
    };
    let existing = world
        .get::<Device>(hit.entity)
        .is_some_and(|old| old.kind == tool);
    let model = d.text("model").to_string();
    validate(world, &d, &model)?;
    let mesh = gpu_model(world, &model)?;
    if existing {
        if world
            .get::<SpawnedProp>(hit.entity)
            .is_none_or(|p| p.model != model)
        {
            return Err("Model changes require a new device; existing device is preserved".into());
        }
        if tool == "wheel"
            && world
                .get::<Device>(hit.entity)
                .is_some_and(|old| old.text("axis") != d.text("axis"))
        {
            return Err("Axle changes require a new wheel; existing joint is preserved".into());
        }
        remember(world);
        attach(world, hit.entity, d)?;
        return Ok(true);
    }
    if tool == "wheel" && action == 2 {
        return Err("Right click an existing wheel to update it".into());
    }
    if world.query::<&Device>().iter(world).count()
        >= world.resource::<PlayState>().tools.catalog.gun.max_devices
    {
        return Err("construction device limit reached".into());
    }
    let attach_kind = if tool == "wheel" {
        "axis"
    } else if matches!(tool, "balloon" | "light") {
        "rope"
    } else {
        "weld"
    };
    let attached =
        tool == "wheel" || (action == 1 && !matches!(tool, "hoverball" | "dynamite" | "camera"));
    if attached
        && world.query::<&constraints::Link>().iter(world).count()
            >= world
                .resource::<PlayState>()
                .tools
                .catalog
                .gun
                .max_constraints
    {
        return Err("constraint limit reached".into());
    }
    if world.get::<npcs::NpcBody>(hit.entity).is_some() {
        return Err("Devices cannot attach to live NPCs".into());
    }
    let rotation = if matches!(tool, "balloon" | "hoverball" | "light" | "dynamite") {
        Quat::IDENTITY
    } else {
        Quat::from_rotation_arc(if tool == "wheel" { d.axis() } else { Vec3::Y }, hit.normal)
    };
    let local_normal = rotation.inverse() * hit.normal;
    let clearance = local_normal.abs().dot(mesh.half) - local_normal.dot(mesh.center);
    let position = hit.point + hit.normal * (clearance + 0.02);
    let (position, rotation) = if tool == "camera" {
        let eye = crate::source_player::aim_eye(world).ok_or("player camera missing")?;
        (eye.translation, eye.rotation)
    } else {
        (position, rotation)
    };
    remember(world);
    let entity = spawn_model(world, &model, position, rotation, tool == "camera")?;
    let mut d = d;
    d.height = position.y;
    let rope_length = d.number("ropelength");
    attach(world, entity, d)?;
    if tool == "camera" {
        let mut properties = world.get::<Properties>(entity).cloned().unwrap_or_default();
        properties.world_only = true;
        apply_properties(world, entity, properties)?;
    }
    if attached {
        let target = if world.get::<RigidBody>(hit.entity).is_some() {
            hit.entity
        } else {
            constraints::world_anchor(world)
        };
        let point = if attach_kind == "rope" {
            position
        } else {
            hit.point
        };
        let mut link = constraints::between(
            world,
            attach_kind,
            entity,
            target,
            point,
            hit.point,
            hit.normal,
        )?;
        if attach_kind == "rope" {
            if tool == "balloon" {
                // The balloon tool's ropelength is the constraint's full length,
                // not additional slack on top of the initial anchor separation.
                link.length = rope_length.max(0.001);
            } else {
                link.length += rope_length;
            }
        }
        constraints::create(world, entity, target, link)?;
    }
    world.resource_mut::<PlayState>().status =
        format!("Created {tool}. Keypad bindings are in its settings; E uses buttons.");
    Ok(true)
}
pub(crate) fn input(world: &mut World) {
    let p = world.resource::<PlayState>();
    let blocked = crate::source_frontend::active(world)
        || p.menu_open
        || p.tools.input_blocked
        || npcs::dead(world)
        || world.resource::<vehicles::Occupancy>().vehicle.is_some();
    let focused = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .iter(world)
        .any(|w| w.focused);
    let keys = world.resource::<ButtonInput<KeyCode>>().clone();
    let physical: BTreeSet<_> = KEYS
        .iter()
        .filter(|(_, k)| keys.pressed(*k))
        .map(|(n, _)| n.to_string())
        .collect();
    if blocked || !focused {
        world.resource_mut::<Remote>().0 = None;
        {
            let mut c = world.resource_mut::<Controls>();
            c.blocked.extend(physical);
            c.down.clear();
            c.pressed.clear();
            c.aimed_button = None;
        }
        // A momentary gmod_button must release when focus/UI blocks the use input.
        // Toggle buttons retain their latched state.
        for mut device in world
            .query::<&mut Device>()
            .iter_mut(world)
            .filter(|device| device.kind == "button" && device.number("toggle") != 1.)
        {
            device.enabled = false;
        }
        return;
    }
    let hit = crate::source_player::aim_eye(world)
        .and_then(|eye| trace(world, eye, None).map(|h| (h, eye.translation.distance(h.point))));
    let usable = hit
        .filter(|(_, distance)| {
            !viewing(world)
                && world.resource::<PlayState>().held.is_none()
                && *distance
                    <= world
                        .resource::<PlayState>()
                        .spawn_catalog
                        .runtime
                        .enter_range
        })
        .map(|(h, _)| h.entity);
    let aimed_button = usable.filter(|entity| {
        world
            .get::<Device>(*entity)
            .is_some_and(|device| device.kind == "button")
    });
    world.resource_mut::<Controls>().aimed_button = aimed_button;
    let buttons: Vec<_> = world
        .query::<(Entity, &Device)>()
        .iter(world)
        .filter(|(_, d)| d.kind == "button")
        .map(|(e, _)| e)
        .collect();
    let mut virtual_down = BTreeSet::new();
    for e in buttons {
        let mut d = world.get_mut::<Device>(e).unwrap();
        if d.number("toggle") == 1. {
            if usable == Some(e) && keys.just_pressed(KeyCode::KeyE) {
                d.enabled = !d.enabled;
            }
        } else {
            d.enabled = usable == Some(e) && keys.pressed(KeyCode::KeyE);
        }
        if d.enabled {
            virtual_down.insert(d.text("key").to_string());
        }
    }
    {
        let mut c = world.resource_mut::<Controls>();
        c.blocked.retain(|k| physical.contains(k));
        let mut down: BTreeSet<_> = physical.difference(&c.blocked).cloned().collect();
        down.extend(virtual_down);
        c.pressed = down.difference(&c.previous).cloned().collect();
        c.previous = down.clone();
        c.down = down;
    }
    let pressed = world.resource::<Controls>().pressed.clone();
    let down = world.resource::<Controls>().down.clone();
    let now = world.resource::<Time>().elapsed_secs();
    let mut selected_camera = None;
    for (entity, mut d) in world.query::<(Entity, &mut Device)>().iter_mut(world) {
        if d.kind == "button" {
            continue;
        }
        if d.kind == "camera" && d.number("toggle") != 1. {
            d.enabled = down.contains(d.text("key"));
        }
        if pressed.contains(d.text("key")) {
            if d.kind == "dynamite" {
                if d.deadline.is_none() {
                    d.deadline = Some(now + d.number("delay"));
                }
            } else if d.number("toggle") == 1. || d.kind == "hoverball" {
                d.enabled = !d.enabled;
            }
            if d.kind == "camera" && d.enabled {
                selected_camera = Some(entity);
            }
        }
    }
    if let Some(selected) = selected_camera {
        for (e, mut d) in world.query::<(Entity, &mut Device)>().iter_mut(world) {
            if d.kind == "camera" && e != selected {
                d.enabled = false;
            }
        }
    }
    world.resource_mut::<Remote>().0 = world
        .query::<(Entity, &Device)>()
        .iter(world)
        .find(|(_, d)| d.kind == "camera" && d.enabled)
        .map(|(e, _)| e);
}
pub(crate) fn button_hint(world: &World) -> Option<String> {
    let entity = world.get_resource::<Controls>()?.aimed_button?;
    let device = world.get::<Device>(entity).filter(|d| d.kind == "button")?;
    let mode = if device.number("toggle") == 1. {
        "toggle"
    } else {
        "hold"
    };
    let state = if device.enabled { "on" } else { "off" };
    Some(format!(
        "E: use button · output {} · {mode} · {state}",
        device.text("key")
    ))
}
pub(crate) fn view(world: &mut World) {
    let active = world.resource::<Remote>().0;
    for (e, d, mut visibility) in world
        .query::<(Entity, &Device, &mut Visibility)>()
        .iter_mut(world)
    {
        if d.kind == "camera" {
            *visibility = if active == Some(e) {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
        }
    }
    let Some(e) = active else { return };
    let (Some(d), Some(transform)) = (world.get::<Device>(e), world.get::<Transform>(e)) else {
        return;
    };
    let mut transform = *transform;
    let fov = d.number("fov").to_radians();
    if let Some(point) = d.look_at {
        let direction = Vec3::from_array(point) - transform.translation;
        if direction.length_squared() > 1e-6 {
            transform = transform.looking_to(direction, Vec3::Y);
        }
    }
    for (mut t, mut p) in world
        .query_filtered::<(&mut Transform, &mut Projection), With<SourceCamera>>()
        .iter_mut(world)
    {
        *t = transform;
        if let Projection::Perspective(p) = p.as_mut() {
            p.fov = fov;
        }
    }
    for mut c in world
        .query_filtered::<&mut Camera, With<crate::source_player::PlayerViewCamera>>()
        .iter_mut(world)
    {
        c.is_active = false;
    }
}
pub(crate) fn impulse(world: &mut World, e: Entity, force: Vec3, point: Vec3, dt: f32) {
    if world.get::<RigidBody>(e) != Some(&RigidBody::Dynamic) {
        return;
    }
    let center = world
        .get::<Transform>(e)
        .map(|t| t.translation)
        .unwrap_or(point);
    let value = force * dt;
    if let Some(mut i) = world.get_mut::<ExternalImpulse>(e) {
        i.impulse += value;
        i.torque_impulse += (point - center).cross(value);
    } else {
        world.entity_mut(e).insert(ExternalImpulse {
            impulse: value,
            torque_impulse: (point - center).cross(value),
        });
    }
    world.entity_mut(e).insert(Sleeping::default());
}
pub(crate) fn physics(world: &mut World) {
    let dt = world.resource::<Time<Fixed>>().delta_secs();
    let down = world.resource::<Controls>().down.clone();
    let gravity = world.resource::<PlayState>().config.gravity;
    let rows: Vec<_> = world
        .query::<(Entity, &Device, &Transform, &Velocity)>()
        .iter(world)
        .map(|(e, d, t, v)| (e, d.clone(), *t, *v))
        .collect();
    for (e, mut d, t, velocity) in rows {
        let hold = down.contains(d.text("key"));
        let reverse = down.contains(d.text("reverse"));
        let on = if d.number("toggle") == 1. {
            d.enabled
        } else {
            hold
        };
        let force = match d.kind.as_str() {
            "balloon" => Vec3::Y * d.number("force"),
            "thruster" => {
                *t.up()
                    * d.number("force")
                    * if reverse {
                        -1.
                    } else if on {
                        1.
                    } else {
                        0.
                    }
            }
            "hoverball" if d.enabled => {
                let axis =
                    down.contains(d.text("up")) as i32 - down.contains(d.text("down")) as i32;
                d.height += axis as f32 * d.number("speed") * dt;
                let accel = (-gravity + (d.height - t.translation.y) * d.number("strength")
                    - velocity.linvel.y * d.number("damping"))
                .clamp(-d.number("max_accel"), d.number("max_accel"));
                if let Some(mut live) = world.get_mut::<Device>(e) {
                    live.height = d.height;
                }
                Vec3::Y * accel * d.number("mass")
            }
            _ => Vec3::ZERO,
        };
        if force.length_squared() > 0. {
            impulse(world, e, force, t.translation, dt);
        }
        if d.kind == "wheel" {
            let sign = if reverse {
                -1.
            } else if on {
                1.
            } else {
                0.
            };
            let axis = t.rotation * d.axis();
            let torque = axis
                * (sign * d.number("torque") - velocity.angvel.dot(axis) * d.number("friction"));
            if world.get::<RigidBody>(e) == Some(&RigidBody::Dynamic) {
                if let Some(mut i) = world.get_mut::<ExternalImpulse>(e) {
                    i.torque_impulse += torque * dt;
                } else {
                    world.entity_mut(e).insert(ExternalImpulse {
                        impulse: Vec3::ZERO,
                        torque_impulse: torque * dt,
                    });
                }
                if torque.length_squared() > 0. {
                    world.entity_mut(e).insert(Sleeping::default());
                }
            }
        }
    }
}
pub(crate) fn effects(world: &mut World) {
    if crate::source_frontend::active(world) {
        return;
    }
    let now = world.resource::<Time>().elapsed_secs();
    let down = world.resource::<Controls>().down.clone();
    let rows: Vec<_> = world
        .query::<(Entity, &Device, &Transform)>()
        .iter(world)
        .map(|(e, d, t)| (e, d.clone(), *t))
        .collect();
    for (e, d, t) in rows {
        let on = if d.number("toggle") == 1. {
            d.enabled
        } else {
            down.contains(d.text("key"))
        };
        if d.kind == "dynamite" && d.deadline.is_some_and(|end| now >= end) {
            if let Some(mut live) = world.get_mut::<Device>(e) {
                live.deadline = None;
            }
            weapons::explode_ignoring(
                world,
                t.translation,
                d.number("radius"),
                d.number("damage"),
                Some(e),
            );
            if d.number("remove") == 1. && world.get_entity(e).is_ok() {
                constraints::remove(world, e, None, false);
                world.despawn(e);
            }
        } else if d.kind == "emitter" && on && now - d.last_emit >= d.number("delay") {
            impacts::burst(world, t.translation, *t.up());
            if let Some(mut live) = world.get_mut::<Device>(e) {
                live.last_emit = now;
            }
        }
        if matches!(d.kind.as_str(), "light" | "lamp") {
            let children: Vec<_> = world
                .get::<Children>(e)
                .map(|c| c.iter().collect())
                .unwrap_or_default();
            for child in children {
                if let Some(mut light) = world.get_mut::<PointLight>(child) {
                    light.intensity = if on { d.number("brightness") } else { 0. };
                }
                if let Some(mut light) = world.get_mut::<SpotLight>(child) {
                    light.intensity = if on { d.number("brightness") } else { 0. };
                }
            }
        }
    }
}
