//! Authored capability states are distinct from extracted reference definitions.
use crate::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Entry {
    pub id: String,
    pub kind: String,
    pub spawn_name: String,
    pub class_name: String,
    pub label: String,
    pub category: String,
    pub model: String,
    pub icon: String,
    pub condition: String,
    pub visibility: String,
    pub admin_only: bool,
    pub defaults_json: String,
    pub source: String,
    pub line: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tab {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub order: usize,
    pub reference_order: usize,
    pub gap_id: String,
    pub source: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Capability {
    pub id: String,
    pub kind: String,
    pub gap_id: String,
    pub feature: String,
    pub status: String,
    pub priority: String,
    pub depends_on: String,
    pub acceptance: String,
    pub source: String,
    pub result: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpawnCatalog {
    pub entries: Vec<Entry>,
    pub tabs: Vec<Tab>,
    pub capabilities: Vec<Capability>,
    pub weapons: Vec<Weapon>,
    pub vehicles: Vec<Vehicle>,
    pub runtime: Runtime,
    pub model_categories: Vec<ModelCategory>,
    pub water: Water,
    pub npcs: Vec<Npc>,
    pub npc_rules: NpcRules,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Npc {
    pub id: String,
    pub kind: String,
    pub model: String,
    pub faction: String,
    pub health: f32,
    pub speed: f32,
    pub range: f32,
    pub damage: f32,
    pub interval: f32,
    pub sight: f32,
    pub height: f32,
    pub radius: f32,
    pub forward_yaw: f32,
    pub idle: String,
    pub walk: String,
    pub attack: String,
    pub scope: String,
    pub remaining: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NpcRules {
    pub limit: usize,
    pub gravity: f32,
    pub player_health: f32,
    pub respawn_seconds: f32,
    pub step_height: f32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Water {
    pub roughness: f32,
    pub reflectance: f32,
    pub transmission: f32,
    pub ior: f32,
    pub thickness: f32,
    pub uv_scale: f32,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub tint_mix: f32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModelCategory {
    pub model: String,
    pub category: String,
    pub source: String,
    pub line: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Runtime {
    pub projectile_limit: usize,
    pub grenade_fuse: f32,
    pub projectile_lifetime: f32,
    pub trace_seconds: f32,
    pub prop_health: f32,
    pub damage_impulse: f32,
    pub equip_delay: f32,
    pub enter_range: f32,
    pub vehicle_spawn_distance: f32,
    pub seat_eye_height: f32,
    pub vehicle_camera_distance: f32,
    pub wheel_span_x: f32,
    pub wheel_span_z: f32,
    pub spring_force_limit: f32,
    pub min_up: f32,
    pub steer_speed: f32,
    pub vehicle_linear_damping: f32,
    pub vehicle_angular_damping: f32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Weapon {
    pub draw: String,
    pub id: String,
    pub kind: String,
    pub view_model: String,
    pub world_model: String,
    pub idle: String,
    pub fire: String,
    pub reload: String,
    pub clip: u32,
    pub reserve: u32,
    pub interval: f32,
    pub reload_seconds: f32,
    pub damage: f32,
    pub pellets: u32,
    pub spread: f32,
    pub range: f32,
    pub speed: f32,
    pub blast: f32,
    pub gravity: f32,
    pub automatic: bool,
    pub scope: String,
    pub remaining: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Vehicle {
    pub id: String,
    pub kind: String,
    pub mass: f32,
    pub engine_accel: f32,
    pub brake_accel: f32,
    pub max_speed: f32,
    pub steer_rate: f32,
    pub suspension: f32,
    pub spring: f32,
    pub damping: f32,
    pub grip: f32,
    pub seat_x: f32,
    pub seat_y: f32,
    pub seat_z: f32,
    pub seat_yaw: f32,
    pub forward_yaw: f32,
    pub eye_height: f32,
    pub exit_distance: f32,
    pub entry_seconds: f32,
    pub exit_seconds: f32,
    pub pose: String,
    pub scope: String,
    pub remaining: String,
}
fn rows<T: serde::de::DeserializeOwned>(dir: &Path, name: &str) -> Result<Vec<T>> {
    csv::Reader::from_path(dir.join(name))?
        .deserialize()
        .map(|r| Ok(r?))
        .collect()
}
fn unique<'a>(ids: impl Iterator<Item = &'a str>) -> Result<()> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if id.is_empty() || !seen.insert(id) {
            return Err(format!("empty or duplicate catalog id: {id}").into());
        }
    }
    Ok(())
}
pub fn load(dir: &Path) -> Result<SpawnCatalog> {
    let mut runtime = rows::<Runtime>(dir, "source_gameplay.csv")?;
    if runtime.len() != 1 {
        return Err("source_gameplay must have one row".into());
    }
    let runtime = runtime.remove(0);
    if runtime.projectile_limit == 0
        || runtime.projectile_limit > 1024
        || ![
            runtime.grenade_fuse,
            runtime.projectile_lifetime,
            runtime.trace_seconds,
            runtime.prop_health,
            runtime.damage_impulse,
            runtime.equip_delay,
            runtime.enter_range,
            runtime.vehicle_spawn_distance,
            runtime.seat_eye_height,
            runtime.vehicle_camera_distance,
            runtime.wheel_span_x,
            runtime.wheel_span_z,
            runtime.spring_force_limit,
            runtime.min_up,
            runtime.steer_speed,
            runtime.vehicle_linear_damping,
            runtime.vehicle_angular_damping,
        ]
        .iter()
        .all(|v| v.is_finite() && *v > 0. && *v <= 1000.)
        || runtime.wheel_span_x > 1.
        || runtime.wheel_span_z > 1.
        || runtime.min_up >= 1.
    {
        return Err("invalid shared gameplay tuning".into());
    }
    let mut water = rows::<Water>(dir, "source_water.csv")?;
    if water.len() != 1 {
        return Err("source_water requires one row".into());
    }
    let water = water.remove(0);
    if ![
        water.roughness,
        water.reflectance,
        water.transmission,
        water.tint_mix,
    ]
    .iter()
    .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        || !water.ior.is_finite()
        || !(1.0..=2.0).contains(&water.ior)
        || ![water.thickness, water.uv_scale]
            .iter()
            .all(|v| v.is_finite() && *v > 0. && *v <= 10.)
        || ![water.scroll_x, water.scroll_y]
            .iter()
            .all(|v| v.is_finite() && v.abs() <= 1.)
    {
        return Err("invalid water tuning".into());
    }
    let mut rules = rows::<NpcRules>(dir, "source_npc_rules.csv")?;
    if rules.len() != 1 {
        return Err("source_npc_rules requires one row".into());
    }
    let npc_rules = rules.remove(0);
    if npc_rules.limit == 0
        || npc_rules.limit > 128
        || ![
            npc_rules.gravity,
            npc_rules.player_health,
            npc_rules.respawn_seconds,
            npc_rules.step_height,
        ]
        .iter()
        .all(|v| v.is_finite() && *v > 0. && *v <= 1000.)
    {
        return Err("invalid NPC runtime rules".into());
    }
    let mut c = SpawnCatalog {
        npcs: rows(dir, "source_npcs.csv")?,
        npc_rules,
        water,
        runtime,
        model_categories: rows(dir, "source_model_categories.csv")?,
        entries: rows(dir, "spawn_reference.csv")?,
        tabs: rows(dir, "source_creation_tabs.csv")?,
        capabilities: rows(dir, "spawn_capabilities.csv")?,
        weapons: rows(dir, "source_weapons.csv")?,
        vehicles: rows(dir, "source_vehicles.csv")?,
    };
    #[derive(Deserialize)]
    struct Gap {
        id: String,
    }
    let gaps: BTreeSet<_> = rows::<Gap>(dir, "parity_gaps.csv")?
        .into_iter()
        .map(|g| g.id)
        .collect();
    let mut memberships = BTreeSet::new();
    for row in &c.model_categories {
        if !row.model.starts_with("models/")
            || !row.model.ends_with(".mdl")
            || row.model.contains("..")
            || row.model.contains('\\')
            || row.category.is_empty()
            || row.source.is_empty()
            || row.line == 0
            || !memberships.insert((&row.model, &row.category))
        {
            return Err("invalid or duplicate model category membership".into());
        }
    }
    unique(c.npcs.iter().map(|e| e.id.as_str()))?;
    for entry in c.entries.iter().filter(|e| e.kind == "npc") {
        if !c.npcs.iter().any(|n| n.id == entry.id) {
            return Err(format!("missing NPC coverage: {}", entry.id).into());
        }
    }
    for n in &c.npcs {
        if !c.entries.iter().any(|e| e.id == n.id && e.kind == "npc")
            || !["disabled", "melee", "ranged", "passive"].contains(&n.kind.as_str())
            || !["combine", "resistance", "zombie", "neutral"].contains(&n.faction.as_str())
            || ![
                n.health,
                n.speed,
                n.range,
                n.damage,
                n.interval,
                n.sight,
                n.height,
                n.radius,
                n.forward_yaw,
            ]
            .iter()
            .all(|v| v.is_finite())
            || n.scope.is_empty()
            || n.remaining.is_empty()
        {
            return Err(format!("invalid NPC definition: {}", n.id).into());
        }
        if n.kind != "disabled"
            && (!n.model.starts_with("models/")
                || !n.model.ends_with(".mdl")
                || n.model.contains("..")
                || n.model.contains('\\')
                || n.health <= 0.
                || n.health > 10000.
                || n.speed < 0.
                || n.speed > 30.
                || n.range <= 0.
                || n.range > n.sight
                || n.sight > 200.
                || n.damage < 0.
                || n.damage > 1000.
                || n.interval < 0.1
                || n.interval > 60.
                || n.radius <= 0.
                || n.height < 2. * n.radius
                || n.height > 10.
                || n.idle.is_empty()
                || n.walk.is_empty()
                || n.attack.is_empty())
        {
            return Err(format!("invalid enabled NPC tuning: {}", n.id).into());
        }
    }
    unique(c.entries.iter().map(|e| e.id.as_str()))?;
    unique(c.tabs.iter().map(|e| e.id.as_str()))?;
    unique(c.capabilities.iter().map(|e| e.id.as_str()))?;
    unique(c.weapons.iter().map(|e| e.id.as_str()))?;
    unique(c.vehicles.iter().map(|e| e.id.as_str()))?;
    for e in c
        .entries
        .iter()
        .filter(|e| e.kind == "weapon" || e.kind == "vehicle")
    {
        if (e.kind == "weapon" && !c.weapons.iter().any(|w| w.id == e.id))
            || (e.kind == "vehicle" && !c.vehicles.iter().any(|v| v.id == e.id))
        {
            return Err(format!("missing explicit runtime coverage for {}", e.id).into());
        }
    }
    for w in &c.weapons {
        if !c.entries.iter().any(|e| e.id == w.id && e.kind == "weapon")
            || ![
                "disabled",
                "physgun",
                "toolgun",
                "hitscan",
                "projectile",
                "grenade",
                "melee",
            ]
            .contains(&w.kind.as_str())
            || w.clip > 1000
            || w.reserve > 10000
            || !(1..=32).contains(&w.pellets)
            || ![w.interval, w.reload_seconds, w.range]
                .iter()
                .all(|n| n.is_finite() && *n > 0. && *n <= 1000.)
            || ![w.damage, w.spread, w.speed, w.blast, w.gravity]
                .iter()
                .all(|n| n.is_finite() && *n >= 0. && *n <= 1000.)
            || w.remaining.is_empty()
            || w.scope.is_empty()
        {
            return Err(format!("invalid weapon runtime row {}", w.id).into());
        }
        if matches!(
            w.kind.as_str(),
            "hitscan" | "projectile" | "grenade" | "melee"
        ) && [&w.idle, &w.fire, &w.draw]
            .iter()
            .any(|clip| clip.trim().is_empty())
        {
            return Err(
                format!("enabled combat weapon needs idle/fire/draw clips: {}", w.id).into(),
            );
        }
        if w.kind != "disabled" {
            for path in [&w.view_model, &w.world_model] {
                if !path.starts_with("models/")
                    || !path.ends_with(".mdl")
                    || path.contains("..")
                    || path.contains('\\')
                {
                    return Err(format!("unsafe weapon model in {}", w.id).into());
                }
            }
        }
        if (w.kind == "physgun" && w.id != "weapon_weapon_physgun")
            || (w.kind == "toolgun" && w.id != "weapon_gmod_tool")
        {
            return Err("special weapon routes must retain their native identity".into());
        }
    }
    for v in &c.vehicles {
        if !c
            .entries
            .iter()
            .any(|e| e.id == v.id && e.kind == "vehicle" && !e.model.is_empty())
            || !["seat", "wheels", "airboat"].contains(&v.kind.as_str())
            || ![
                v.mass,
                v.suspension,
                v.spring,
                v.damping,
                v.exit_distance,
                v.entry_seconds,
                v.exit_seconds,
            ]
            .iter()
            .all(|n| n.is_finite() && *n > 0.)
            || ![
                v.engine_accel,
                v.brake_accel,
                v.max_speed,
                v.steer_rate,
                v.grip,
            ]
            .iter()
            .all(|n| n.is_finite() && *n >= 0. && *n <= 100.)
            || ![v.seat_x, v.seat_y, v.seat_z, v.seat_yaw]
                .iter()
                .all(|n| n.is_finite() && n.abs() < 100.)
            || v.mass > 10000.
            || v.spring > 1000000.
            || v.damping > 100000.
            || v.suspension > 2.
            || v.entry_seconds > 5.
            || v.exit_seconds > 5.
            || v.exit_distance > 10.
            || !v.forward_yaw.is_finite()
            || v.forward_yaw.abs() > std::f32::consts::TAU
            || !v.eye_height.is_finite()
            || !(0.1..=2.).contains(&v.eye_height)
            || v.pose.is_empty()
            || v.remaining.is_empty()
            || v.scope.is_empty()
            || (v.kind == "seat" && (v.engine_accel != 0. || v.max_speed != 0.))
        {
            return Err(format!("invalid vehicle runtime row {}", v.id).into());
        }
    }
    let kinds = [
        "model",
        "weapon",
        "npc",
        "entity",
        "vehicle",
        "postprocess",
        "dupe",
        "save",
    ];
    let mut orders = BTreeSet::new();
    for t in &c.tabs {
        if !kinds.contains(&t.kind.as_str()) || !gaps.contains(&t.gap_id) || !orders.insert(t.order)
        {
            return Err(format!("invalid creation tab {}", t.id).into());
        }
    }
    if c.tabs.iter().filter(|t| t.kind == "model").count() != 1
        || !c
            .tabs
            .iter()
            .any(|t| t.id == "spawnlists" && t.kind == "model")
    {
        return Err("creation catalog requires one spawnlists model tab".into());
    }
    for e in &c.entries {
        if !["weapon", "npc", "entity", "vehicle"].contains(&e.kind.as_str())
            || !c.tabs.iter().any(|t| t.kind == e.kind)
            || !["spawnmenu", "internal", "inherited_or_internal"].contains(&e.visibility.as_str())
            || e.line == 0
            || e.class_name.is_empty()
            || e.source.is_empty()
            || e.model.contains("..")
            || e.icon.contains("..")
        {
            return Err(format!("invalid spawn reference {}", e.id).into());
        }
        serde_json::from_str::<std::collections::BTreeMap<String, String>>(&e.defaults_json)?;
    }
    // Dependency rows must precede dependents: rejects cycles and unknown references.
    let mut preceding = BTreeSet::new();
    for cap in &c.capabilities {
        if !gaps.contains(&cap.gap_id)
            || !(kinds.contains(&cap.kind.as_str())
                || ["shared", "menu", "world"].contains(&cap.kind.as_str()))
            || !["missing", "partial", "implemented", "verified"].contains(&cap.status.as_str())
            || !["P0", "P1", "P2", "P3"].contains(&cap.priority.as_str())
            || cap.acceptance.is_empty()
            || cap.source.is_empty()
            || !["not_run", "passed", "failed"].contains(&cap.result.as_str())
            || (cap.status == "verified" && cap.result != "passed")
        {
            return Err(format!("invalid capability {}", cap.id).into());
        }
        for dep in cap.depends_on.split('|').filter(|s| !s.is_empty()) {
            if !preceding.contains(dep) {
                return Err(format!("{}: dependency {dep} must precede it", cap.id).into());
            }
        }
        preceding.insert(cap.id.as_str());
    }
    c.tabs.sort_by_key(|t| t.order);
    c.entries.sort_by_key(|e| {
        (
            e.kind.clone(),
            e.category.clone(),
            e.label.clone(),
            e.id.clone(),
        )
    });
    Ok(c)
}
pub fn generate(c: &SpawnCatalog) -> String {
    format!("pub fn compiled_spawn_catalog() -> sandbox_catalog::spawn::SpawnCatalog {{ serde_json::from_str({:?}).expect(\"build-validated spawn catalog\") }}\n", serde_json::to_string(c).expect("serializable catalog"))
}
