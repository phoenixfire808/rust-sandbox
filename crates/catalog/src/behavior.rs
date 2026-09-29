use crate::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Write as _, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BehaviorSpec {
    pub id: String,
    pub domain: String,
    pub behavior: String,
    pub authority: String,
    pub dependencies: String,
    pub rust_target: String,
    pub status: String,
    pub acceptance: String,
    pub source: String,
}
pub fn load(dir: &Path) -> Result<Vec<BehaviorSpec>> {
    let mut specs = Vec::new();
    for name in ["systems.csv", "tools.csv"] {
        for row in csv::Reader::from_path(dir.join(name))?.deserialize() {
            specs.push(row?);
        }
    }
    validate(&specs)?;
    let mut source_reader = csv::Reader::from_path(dir.join("sources.csv"))?;
    let sources: HashSet<String> = source_reader
        .records()
        .map(|r| r.map(|r| r[0].to_string()))
        .collect::<std::result::Result<_, _>>()?;
    for spec in &specs {
        if !sources.contains(&spec.source) {
            return Err(format!("{}: unknown research source {}", spec.id, spec.source).into());
        }
    }
    Ok(specs)
}
pub fn validate(specs: &[BehaviorSpec]) -> Result<()> {
    let mut ids = HashSet::new();
    for s in specs {
        if s.id.is_empty() || !ids.insert(s.id.as_str()) {
            return Err(format!("empty or duplicate behavior id {}", s.id).into());
        }
        if !["prototype", "not_started", "catalog_only", "reference_only"]
            .contains(&s.status.as_str())
        {
            return Err(format!("{}: invalid status {}", s.id, s.status).into());
        }
        if s.behavior.is_empty() || s.acceptance.is_empty() || s.source.is_empty() {
            return Err(format!(
                "{}: behavior requires description acceptance and evidence source",
                s.id
            )
            .into());
        }
    }
    for s in specs {
        for dep in s.dependencies.split('|') {
            if dep != "none" && !ids.contains(dep) {
                return Err(format!("{}: unknown dependency {dep}", s.id).into());
            }
        }
    }
    Ok(())
}
pub fn generate(specs: &[BehaviorSpec]) -> Result<String> {
    validate(specs)?;
    let mut text=String::from("\npub fn compiled_behaviors() -> Vec<sandbox_catalog::behavior::BehaviorSpec> {\nuse sandbox_catalog::behavior::BehaviorSpec;\nvec![\n");
    for s in specs {
        writeln!(text,"BehaviorSpec {{id:{:?}.into(),domain:{:?}.into(),behavior:{:?}.into(),authority:{:?}.into(),dependencies:{:?}.into(),rust_target:{:?}.into(),status:{:?}.into(),acceptance:{:?}.into(),source:{:?}.into()}},",s.id,s.domain,s.behavior,s.authority,s.dependencies,s.rust_target,s.status,s.acceptance,s.source)?;
    }
    text.push_str("]\n}\n");
    Ok(text)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn research_tables_have_valid_links() {
        let v = load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets")).unwrap();
        assert_eq!(v.len(), 101);
    }
    #[test]
    fn missing_dependency_rejected() {
        let mut v = load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets")).unwrap();
        v[0].dependencies = "missing".into();
        assert!(validate(&v).is_err());
    }
}
