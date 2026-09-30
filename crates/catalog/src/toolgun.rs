use crate::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolgunConfig {
    pub fire_clip: String,
    pub screen_material: String,
    pub screen_background: String,
    pub screen_size: u32,
    pub font_size: f32,
    pub scroll_speed: f32,
    pub text_center_y: f32,
    pub tracer_seconds: f32,
    pub tracer_width: f32,
    pub trace_range: f32,
    pub max_constraints: usize,
    pub max_devices: usize,
    pub max_marks: usize,
    pub trail_points: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolDef {
    pub id: String,
    pub label: String,
    pub category: String,
    pub status: String,
    pub left: String,
    pub right: String,
    pub reload: String,
    pub remaining: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolOption {
    pub tool: String,
    pub key: String,
    pub label: String,
    pub kind: String,
    pub default: String,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub choices: String,
    pub enabled: bool,
}
impl ToolOption {
    pub fn accepts(&self, value: &str) -> bool {
        match self.kind.as_str() {
            "choice" => self.choices.split('|').any(|v| v == value),
            "bool" => value == "0" || value == "1",
            "number" => value
                .parse::<f32>()
                .is_ok_and(|v| v.is_finite() && v >= self.min && v <= self.max),
            _ => false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicsMaterial {
    pub id: String,
    pub friction: f32,
    pub restitution: f32,
    pub evidence: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCatalog {
    pub gun: ToolgunConfig,
    pub tools: Vec<ToolDef>,
    pub options: Vec<ToolOption>,
    pub materials: Vec<PhysicsMaterial>,
}
fn rows<T: serde::de::DeserializeOwned>(p: &Path) -> Result<Vec<T>> {
    Ok(csv::Reader::from_path(p)?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?)
}
fn material_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains(['\\', ':'])
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}
pub fn load(path: &Path) -> Result<ToolCatalog> {
    let mut gun: Vec<ToolgunConfig> = rows(&path.join("source_toolgun.csv"))?;
    if gun.len() != 1 {
        return Err("source_toolgun requires one row".into());
    }
    let gun = gun.remove(0);
    if !(64..=1024).contains(&gun.screen_size)
        || !gun.screen_size.is_power_of_two()
        || !(1..=10000).contains(&gun.max_constraints)
        || !(1..=256).contains(&gun.max_devices)
        || !(1..=2048).contains(&gun.max_marks)
        || !(2..=256).contains(&gun.trail_points)
        || gun.fire_clip.is_empty()
        || !material_path(&gun.screen_material)
        || !material_path(&gun.screen_background)
        || [
            gun.font_size,
            gun.scroll_speed,
            gun.text_center_y,
            gun.tracer_seconds,
            gun.tracer_width,
            gun.trace_range,
        ]
        .iter()
        .any(|v| !v.is_finite() || *v <= 0. || *v > 16384.)
    {
        return Err("invalid toolgun dimensions or timing".into());
    }
    let tools: Vec<ToolDef> = rows(&path.join("source_tools.csv"))?;
    let mut ids = BTreeSet::new();
    for t in &tools {
        if t.id.is_empty()
            || !t.id.bytes().all(|c| c.is_ascii_lowercase() || c == b'_')
            || !ids.insert(t.id.clone())
            || !["partial", "missing", "reference_only"].contains(&t.status.as_str())
            || !["Constraints", "Construction", "Render", "Poser", "Internal"]
                .contains(&t.category.as_str())
            || [&t.label, &t.left, &t.right, &t.reload, &t.remaining]
                .iter()
                .any(|v| v.trim().is_empty())
        {
            return Err("invalid or duplicate tool definition".into());
        }
    }
    #[derive(Deserialize)]
    struct ReferenceTool {
        id: String,
    }
    let reference: Vec<ReferenceTool> = rows(&path.join("tools.csv"))?;
    let mut expected: BTreeSet<_> = reference
        .iter()
        .map(|t| t.id.strip_prefix("tool_").unwrap_or(&t.id).to_owned())
        .collect();
    expected.insert("freeze".into());
    if ids != expected {
        return Err(
            "source_tools must cover every tools.csv ID plus the custom freeze tool".into(),
        );
    }
    let options: Vec<ToolOption> = rows(&path.join("source_tool_options.csv"))?;
    let mut keys = BTreeSet::new();
    for o in &options {
        if !ids.contains(&o.tool)
            || o.key.is_empty()
            || !o
                .key
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_')
            || !keys.insert((&o.tool, &o.key))
            || !o.accepts(&o.default)
            || !o.min.is_finite()
            || !o.max.is_finite()
            || !o.step.is_finite()
            || o.min > o.max
            || (o.kind == "number" && o.min == o.max)
            || o.step <= 0.
        {
            return Err("invalid tool option".into());
        }
    }
    let materials: Vec<PhysicsMaterial> = rows(&path.join("source_physics_materials.csv"))?;
    let mut names = BTreeSet::new();
    for m in &materials {
        if !names.insert(&m.id)
            || !m.friction.is_finite()
            || !(0.0..=10.0).contains(&m.friction)
            || !m.restitution.is_finite()
            || !(0.0..=1.0).contains(&m.restitution)
        {
            return Err("invalid physics material approximation".into());
        }
    }
    for option in options
        .iter()
        .filter(|o| o.tool == "physprop" && o.key == "material")
    {
        if option
            .choices
            .split('|')
            .any(|id| !materials.iter().any(|m| m.id == id))
        {
            return Err(
                "physical material choice is not authored in source_physics_materials".into(),
            );
        }
    }
    Ok(ToolCatalog {
        gun,
        tools,
        options,
        materials,
    })
}
pub fn generate(c: &ToolCatalog) -> String {
    format!("pub fn compiled_tool_catalog()->sandbox_catalog::toolgun::ToolCatalog {{ serde_json::from_str({:?}).expect(\"build-validated tool catalog\") }}\n", serde_json::to_string(c).unwrap())
}
