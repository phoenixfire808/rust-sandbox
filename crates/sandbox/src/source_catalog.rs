//! Local metadata audit. It inventories the mounted installation, not hypothetical addon code.
use crate::source_assets::{virtual_path, MountedSource};
use sandbox_catalog::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub model: String,
    pub category: String,
    pub vvd: bool,
    pub vtx: bool,
    pub phy: bool,
    pub icon: bool,
    pub decode_status: String,
}
impl MountedSource {
    pub fn files(&self) -> Result<BTreeSet<String>> {
        let mut out = BTreeSet::new();
        for (root, archives) in &self.mounts.roots {
            for archive in archives {
                out.extend(archive.tree.keys().cloned());
            }
            fn walk(root: &Path, dir: &Path, out: &mut BTreeSet<String>) -> Result<()> {
                for entry in std::fs::read_dir(dir)? {
                    let e = entry?;
                    let t = e.file_type()?;
                    if t.is_symlink() {
                        continue;
                    }
                    if t.is_dir() {
                        walk(root, &e.path(), out)?;
                    } else if t.is_file() {
                        if let Ok(name) =
                            virtual_path(&e.path().strip_prefix(root)?.to_string_lossy())
                        {
                            out.insert(name);
                        }
                    }
                }
                Ok(())
            }
            walk(root, root, &mut out)?;
        }
        Ok(out)
    }
    pub fn models(&self) -> Result<Vec<ModelEntry>> {
        let files = self.files()?;
        Ok(files
            .iter()
            .filter(|p| p.starts_with("models/") && p.ends_with(".mdl"))
            .map(|p| {
                let stem = p.trim_end_matches(".mdl");
                ModelEntry {
                    model: p.clone(),
                    category: p.split('/').nth(1).unwrap_or("other").to_string(),
                    vvd: files.contains(&format!("{stem}.vvd")),
                    vtx: files.contains(&format!("{stem}.dx90.vtx")),
                    phy: files.contains(&format!("{stem}.phy")),
                    icon: files.contains(&format!("materials/spawnicons/{stem}.png")),
                    decode_status: "not_yet_decoded".into(),
                }
            })
            .collect())
    }
    pub fn export_catalog(&self, dir: &Path) -> Result<serde_json::Value> {
        if dir.exists() {
            return Err("catalog output exists: use a new directory".into());
        }
        std::fs::create_dir_all(dir)?;
        let models = self.models()?;
        let files = self.files()?;
        let mut writer = csv::Writer::from_path(dir.join("models.csv"))?;
        for model in &models {
            writer.serialize(model)?;
        }
        writer.flush()?;
        let mut refs = csv::Writer::from_path(dir.join("menu_settings_references.csv"))?;
        refs.write_record([
            "source",
            "line",
            "api",
            "literal_arguments",
            "implementation_status",
            "evidence_kind",
        ])?;
        let apis = [
            "AddCreationTab",
            "AddToolMenuOption",
            "AddToolCategory",
            "AddToolTab",
            "CreateConVar",
            "CreateClientConVar",
            "SetConVar",
            "GetConVar",
            "CheckBox",
            "NumSlider",
            "ComboBox",
            "AddOption",
            "AddControl",
            "AddItem",
            "AddMenuOption",
            "RunConsoleCommand",
        ];
        let mut hits = 0;
        let mut scanned = 0;
        let mut failed = Vec::new();
        for path in files.iter().filter(|p| {
            p.ends_with(".lua") && (p.starts_with("lua/") || p.starts_with("gamemodes/"))
        }) {
            let bytes = match self.mounts.read(&self.bsp, path) {
                Ok(b) => b,
                Err(e) => {
                    failed.push(format!("{path}: {e}"));
                    continue;
                }
            };
            let text = String::from_utf8_lossy(&bytes);
            scanned += 1;
            for (line_index, line) in text.lines().enumerate() {
                if line.trim_start().starts_with("--") {
                    continue;
                }
                for api in apis {
                    if line.contains(api) {
                        let literals = line
                            .split('"')
                            .enumerate()
                            .filter(|(i, _)| i % 2 == 1)
                            .map(|(_, s)| s)
                            .collect::<Vec<_>>()
                            .join(" | ");
                        refs.write_record([
                            path.as_str(),
                            &(line_index + 1).to_string(),
                            api,
                            &literals,
                            "unverified",
                            "lexical_reference_not_runtime_registration",
                        ])?;
                        hits += 1;
                    }
                }
            }
        }
        refs.flush()?;
        let report = serde_json::json!({"mounted_files":files.len(),"unique_models":models.len(),"models_with_mesh_parts":models.iter().filter(|m|m.vvd&&m.vtx).count(),"lua_files_scanned":scanned,"menu_setting_references":hits,"failed_sources":failed,"scope":"Current mounted base installation only. Lexical menu/settings references are not exhaustive dynamic registrations or native engine convars. Models listed are not all tested or animated."});
        std::fs::write(
            dir.join("coverage.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        Ok(report)
    }
}
