use crate::{content::Content, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SavedProp {
    pub prop_id: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub linear_velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub frozen: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub version: u32,
    pub props: Vec<SavedProp>,
}
impl Scene {
    pub fn from_content(c: &Content) -> Self {
        Self {
            version: 1,
            props: c
                .scene
                .iter()
                .map(|p| SavedProp {
                    prop_id: p.prop_id.clone(),
                    position: [p.x, p.y, p.z],
                    rotation: [0., 0., 0., 1.],
                    linear_velocity: [0.; 3],
                    angular_velocity: [0.; 3],
                    frozen: p.frozen,
                })
                .collect(),
        }
    }
    pub fn validate(&self, c: &Content) -> Result<()> {
        if self.version != 1 {
            return Err("unsupported scene version".into());
        }
        if self.props.len() > c.world.max_props {
            return Err("scene exceeds prop limit".into());
        }
        for p in &self.props {
            if !c.props.iter().any(|d| d.id == p.prop_id) {
                return Err(format!("unknown saved prop {}", p.prop_id).into());
            }
            for v in p
                .position
                .iter()
                .chain(&p.linear_velocity)
                .chain(&p.angular_velocity)
            {
                if !v.is_finite() || v.abs() > 10000. {
                    return Err("invalid saved coordinate or velocity".into());
                }
            }
            let norm = p.rotation.iter().map(|v| v * v).sum::<f32>();
            if !norm.is_finite() || (norm - 1.).abs() > 0.001 {
                return Err("invalid saved quaternion".into());
            }
        }
        Ok(())
    }
    pub fn save_new(&self, c: &Content, dir: &Path) -> Result<PathBuf> {
        self.validate(c)?;
        let data = serde_json::to_vec_pretty(self)?;
        fs::create_dir_all(dir)?;
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = dir.join(format!("scene-{stamp}.json"));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(&data)?;
        file.sync_all()?;
        Ok(path)
    }
    pub fn load(path: &Path, c: &Content) -> Result<Self> {
        if path.metadata()?.len() > 8 * 1024 * 1024 {
            return Err("scene exceeds 8 MiB limit".into());
        }
        let scene: Self = serde_json::from_slice(&fs::read(path)?)?;
        scene.validate(c)?;
        Ok(scene)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Content {
        crate::content::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets")).unwrap()
    }
    #[test]
    fn roundtrip() {
        let c = fixture();
        let s = Scene::from_content(&c);
        s.validate(&c).unwrap();
        let d: Scene = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        assert_eq!(s, d);
    }
    #[test]
    fn reject_bad_version_and_quaternion() {
        let c = fixture();
        let mut s = Scene::from_content(&c);
        s.version = 2;
        assert!(s.validate(&c).is_err());
        s.version = 1;
        s.props[0].rotation = [0.; 4];
        assert!(s.validate(&c).is_err());
    }
}
