use crate::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};

/// Asset references, not copies of proprietary data. Physical paths stay local.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SourceMapDef {
    pub id: String,
    pub bsp: String,
    pub unit_scale: f32,
    pub fly_speed: f32,
    pub eye_height_units: f32,
    pub fov_degrees: f32,
}

pub fn load(directory: &Path) -> Result<Vec<SourceMapDef>> {
    let maps: Vec<SourceMapDef> = csv::Reader::from_path(directory.join("source_maps.csv"))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?;
    validate(&maps)?;
    Ok(maps)
}

pub fn validate(maps: &[SourceMapDef]) -> Result<()> {
    let mut ids = HashSet::new();
    if maps.is_empty() {
        return Err("source_maps.csv is empty".into());
    }
    for map in maps {
        if map.id.is_empty()
            || !map
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_')
            || !ids.insert(&map.id)
        {
            return Err("invalid or duplicate Source map id".into());
        }
        if map.bsp != format!("maps/{}.bsp", map.id) {
            return Err("map BSP must be maps/<id>.bsp, without traversal".into());
        }
        for (v, min, max) in [
            (map.unit_scale, 0.001, 1.),
            (map.fly_speed, 0.1, 100.),
            (map.eye_height_units, 1., 128.),
            (map.fov_degrees, 40., 120.),
        ] {
            if !v.is_finite() || !(min..=max).contains(&v) {
                return Err("invalid Source map numeric setting".into());
            }
        }
    }
    Ok(())
}

pub fn generate(maps: &[SourceMapDef]) -> String {
    let mut out = String::from("pub fn compiled_source_maps() -> Vec<sandbox_catalog::source_maps::SourceMapDef> { vec![\n");
    for m in maps {
        out.push_str(&format!("sandbox_catalog::source_maps::SourceMapDef {{ id: {:?}.into(), bsp: {:?}.into(), unit_scale: {:?}, fly_speed: {:?}, eye_height_units: {:?}, fov_degrees: {:?} }},\n", m.id, m.bsp, m.unit_scale, m.fly_speed, m.eye_height_units, m.fov_degrees));
    }
    out.push_str("] }\n");
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn bad_source_map_authoring_is_rejected() {
        let good =
            super::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets"))
                .unwrap();
        let mut maps = good.clone();
        maps.push(maps[0].clone());
        assert!(super::validate(&maps).is_err());
        maps = good.clone();
        maps[0].bsp = "../outside.bsp".into();
        assert!(super::validate(&maps).is_err());
        maps = good.clone();
        maps[0].unit_scale = f32::NAN;
        assert!(super::validate(&maps).is_err());
        maps = good;
        maps[0].fov_degrees = 180.;
        assert!(super::validate(&maps).is_err());
        assert!(super::validate(&[]).is_err());
    }
    #[test]
    fn source_maps_are_valid_and_generated() {
        let maps =
            super::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets"))
                .unwrap();
        assert_eq!(maps[0].id, "gm_construct");
        assert!(super::generate(&maps).contains("maps/gm_construct.bsp"));
    }
}
