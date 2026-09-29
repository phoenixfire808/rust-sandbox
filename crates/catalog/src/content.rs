use crate::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Write as _, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PropDef {
    pub id: String,
    pub label: String,
    pub size_x: f32,
    pub size_y: f32,
    pub size_z: f32,
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorldDef {
    pub id: String,
    pub gravity: f32,
    pub fixed_hz: u32,
    pub ground_half_size: f32,
    pub move_speed: f32,
    pub spawn_distance: f32,
    pub grab_distance: f32,
    pub max_props: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub prop_id: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub frozen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Content {
    pub props: Vec<PropDef>,
    pub world: WorldDef,
    pub scene: Vec<Placement>,
}

fn rows<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)?;
    reader
        .deserialize()
        .enumerate()
        .map(|(i, row)| row.map_err(|e| format!("{} row {}: {e}", path.display(), i + 2).into()))
        .collect()
}

fn bounded(value: f32, min: f32, max: f32, field: &str) -> Result<()> {
    if !value.is_finite() || value < min || value > max {
        return Err(format!("{field}: expected finite value in {min}..={max}, got {value}").into());
    }
    Ok(())
}

pub fn load(directory: &Path) -> Result<Content> {
    let props = rows(&directory.join("props.csv"))?;
    let mut worlds = rows::<WorldDef>(&directory.join("world.csv"))?;
    if worlds.len() != 1 {
        return Err("world.csv must contain exactly one world".into());
    }
    let content = Content {
        props,
        world: worlds.remove(0),
        scene: rows(&directory.join("scene.csv"))?,
    };
    content.validate()?;
    Ok(content)
}

impl Content {
    pub fn validate(&self) -> Result<()> {
        if self.props.is_empty() || self.props.len() > 10000 {
            return Err("props.csv must have 1..=10000 rows".into());
        }
        let mut ids = HashSet::new();
        for p in &self.props {
            if p.id.is_empty() || !p.id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
                return Err(format!("invalid prop id {:?}", p.id).into());
            }
            if !ids.insert(p.id.as_str()) {
                return Err(format!("duplicate prop id {}", p.id).into());
            }
            if p.label.trim().is_empty() || p.label.len() > 100 {
                return Err(format!("{}: label must have 1..=100 bytes", p.id).into());
            }
            if p.provenance != "original_procedural" {
                return Err(format!(
                    "{}: prototype only permits original_procedural assets",
                    p.id
                )
                .into());
            }
            for (name, value) in [
                ("size_x", p.size_x),
                ("size_y", p.size_y),
                ("size_z", p.size_z),
            ] {
                bounded(value, 0.05, 100., &format!("{}.{name}", p.id))?;
            }
            bounded(p.mass, 0.01, 10000., &format!("{}.mass", p.id))?;
            bounded(p.friction, 0., 2., &format!("{}.friction", p.id))?;
            for (name, value) in [
                ("restitution", p.restitution),
                ("red", p.red),
                ("green", p.green),
                ("blue", p.blue),
            ] {
                bounded(value, 0., 1., &format!("{}.{name}", p.id))?;
            }
        }
        let w = &self.world;
        bounded(w.gravity, -100., 0., "world.gravity")?;
        bounded(w.ground_half_size, 5., 1000., "world.ground_half_size")?;
        for (name, value) in [
            ("move_speed", w.move_speed),
            ("spawn_distance", w.spawn_distance),
            ("grab_distance", w.grab_distance),
        ] {
            bounded(value, 0.1, 100., name)?;
        }
        if !(30..=240).contains(&w.fixed_hz) {
            return Err("fixed_hz must be 30..=240".into());
        }
        if !(1..=10000).contains(&w.max_props) {
            return Err("max_props must be 1..=10000".into());
        }
        if self.scene.len() > w.max_props {
            return Err("scene exceeds max_props".into());
        }
        for (i, p) in self.scene.iter().enumerate() {
            if !ids.contains(p.prop_id.as_str()) {
                return Err(
                    format!("scene row {} references missing prop {}", i + 2, p.prop_id).into(),
                );
            }
            for v in [p.x, p.y, p.z] {
                bounded(v, -10000., 10000., "scene coordinate")?;
            }
        }
        Ok(())
    }
}

/// Generate actual typed Rust struct literals, not executable spreadsheet formulas.
/// Debug formatting safely escapes every user-authored string.
pub fn generate_rust(c: &Content) -> Result<String> {
    c.validate()?;
    let mut s = String::from("// Generated from sheets. DO NOT EDIT.\npub fn compiled_content() -> sandbox_catalog::content::Content {\nuse sandbox_catalog::content::*;\nContent { props: vec![\n");
    for p in &c.props {
        writeln!(s,"PropDef {{ id: {:?}.into(), label: {:?}.into(), size_x: {:?}, size_y: {:?}, size_z: {:?}, mass: {:?}, friction: {:?}, restitution: {:?}, red: {:?}, green: {:?}, blue: {:?}, provenance: {:?}.into() }},",p.id,p.label,p.size_x,p.size_y,p.size_z,p.mass,p.friction,p.restitution,p.red,p.green,p.blue,p.provenance)?;
    }
    let w = &c.world;
    writeln!(s,"], world: WorldDef {{ id: {:?}.into(), gravity: {:?}, fixed_hz: {}, ground_half_size: {:?}, move_speed: {:?}, spawn_distance: {:?}, grab_distance: {:?}, max_props: {} }}, scene: vec![",w.id,w.gravity,w.fixed_hz,w.ground_half_size,w.move_speed,w.spawn_distance,w.grab_distance,w.max_props)?;
    for p in &c.scene {
        writeln!(
            s,
            "Placement {{ prop_id: {:?}.into(), x: {:?}, y: {:?}, z: {:?}, frozen: {} }},",
            p.prop_id, p.x, p.y, p.z, p.frozen
        )?;
    }
    s.push_str("] }\n}\n");
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Content {
        load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets")).unwrap()
    }
    #[test]
    fn authored_sheets_validate() {
        fixture().validate().unwrap();
    }
    #[test]
    fn duplicates_rejected() {
        let mut c = fixture();
        c.props.push(c.props[0].clone());
        assert!(c.validate().unwrap_err().to_string().contains("duplicate"));
    }
    #[test]
    fn nonfinite_rejected() {
        let mut c = fixture();
        c.props[0].mass = f32::NAN;
        assert!(c.validate().is_err());
    }
    #[test]
    fn broken_references_rejected() {
        let mut c = fixture();
        c.scene[0].prop_id = "missing".into();
        assert!(c.validate().is_err());
    }
    #[test]
    fn rust_strings_are_escaped() {
        let mut c = fixture();
        c.props[0].label = "quoted \"name\"\nline".into();
        let s = generate_rust(&c).unwrap();
        assert!(s.contains("quoted \\\"name\\\"\\nline"));
    }
    #[test]
    fn invalid_physics_rejected() {
        let mut c = fixture();
        c.props[0].size_x = -1.;
        assert!(c.validate().is_err());
    }
    #[test]
    fn generation_is_deterministic() {
        let c = fixture();
        assert_eq!(generate_rust(&c).unwrap(), generate_rust(&c).unwrap());
    }
}
