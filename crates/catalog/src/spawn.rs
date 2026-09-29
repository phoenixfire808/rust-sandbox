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
    let mut c = SpawnCatalog {
        entries: rows(dir, "spawn_reference.csv")?,
        tabs: rows(dir, "source_creation_tabs.csv")?,
        capabilities: rows(dir, "spawn_capabilities.csv")?,
    };
    #[derive(Deserialize)]
    struct Gap {
        id: String,
    }
    let gaps: BTreeSet<_> = rows::<Gap>(dir, "parity_gaps.csv")?
        .into_iter()
        .map(|g| g.id)
        .collect();
    unique(c.entries.iter().map(|e| e.id.as_str()))?;
    unique(c.tabs.iter().map(|e| e.id.as_str()))?;
    unique(c.capabilities.iter().map(|e| e.id.as_str()))?;
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
