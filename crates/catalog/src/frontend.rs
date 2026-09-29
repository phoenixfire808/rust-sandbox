use crate::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrontendConfig {
    pub margin: f32,
    pub sidebar_width: f32,
    pub settings_width: f32,
    pub map_icon: f32,
    pub font_size: f32,
    pub row_height: f32,
    pub feedback_limit: usize,
    pub spawn_clearance: f32,
    pub footer_height: f32,
    pub main_padding: f32,
    pub main_list_margin: f32,
    pub main_item_padding: f32,
    pub main_font: f32,
    pub compact_font: f32,
    pub compact_width: f32,
    pub short_height: f32,
    pub short_font: f32,
    pub short_line: f32,
    pub logo_width: f32,
    pub logo_height: f32,
    pub map_right: f32,
    pub map_grid_left: f32,
    pub card_padding: f32,
    pub card_margin: f32,
    pub caption_font: f32,
    pub category_height: f32,
    pub category_gap: f32,
    pub header_font: f32,
    pub body_font: f32,
    pub start_inset: f32,
    pub start_font: f32,
    pub background: String,
    pub footer_label_breakpoint: f32,
    pub footer_compact_margin: f32,
    pub popup_right: f32,
    pub popup_width: f32,
    pub popup_games_right: f32,
    pub popup_games_width: f32,
    pub popup_language_right: f32,
    pub popup_max_height: f32,
    pub tiny_height: f32,
    pub narrow_width: f32,
    #[serde(skip)]
    pub entries: Vec<MenuEntry>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuEntry {
    pub id: String,
    pub label: String,
    pub visibility: String,
    pub kind: String,
    pub action: String,
}
pub fn load(path: &Path) -> Result<FrontendConfig> {
    let rows: Vec<FrontendConfig> = csv::Reader::from_path(path.join("source_frontend.csv"))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?;
    if rows.len() != 1 {
        return Err("source_frontend needs exactly one row".into());
    }
    let mut c = rows.into_iter().next().unwrap();
    let value = serde_json::to_value(&c)?;
    for (key, v) in value.as_object().unwrap() {
        if let Some(n) = v.as_f64() {
            if key != "feedback_limit" && (!n.is_finite() || n <= 0. || n > 8192.) {
                return Err(format!("invalid frontend dimension {key}").into());
            }
        } else if v.is_null() {
            return Err(format!("non-finite frontend dimension {key}").into());
        }
    }
    if !(256..=32768).contains(&c.feedback_limit) || c.spawn_clearance > 1. {
        return Err("invalid feedback limit or spawn clearance".into());
    }
    if !c.background.starts_with("backgrounds/")
        || c.background.contains("..")
        || c.background.contains(['\\', ':'])
        || !c.background.ends_with(".png")
    {
        return Err("frontend background must be a safe installed PNG".into());
    }
    c.entries = csv::Reader::from_path(path.join("source_main_menu.csv"))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?;
    let mut ids = BTreeSet::new();
    for e in &c.entries {
        if !ids.insert(&e.id)
            || !matches!(e.visibility.as_str(), "all" | "game")
            || !matches!(e.kind.as_str(), "logo" | "gap" | "link")
            || !matches!(
                e.action.as_str(),
                "none"
                    | "resume"
                    | "maps"
                    | "servers"
                    | "addons"
                    | "dupes"
                    | "saves"
                    | "demos"
                    | "options"
                    | "disconnect"
                    | "quit"
            )
            || (e.kind == "link" && (e.label.is_empty() || e.action == "none"))
        {
            return Err(format!("invalid main-menu entry {}", e.id).into());
        }
    }
    if c.entries.is_empty() {
        return Err("empty main menu".into());
    }
    Ok(c)
}
pub fn generate(c: &FrontendConfig) -> String {
    format!("pub fn compiled_frontend_config()->sandbox_catalog::frontend::FrontendConfig {{ serde_json::from_str({:?}).expect(\"build-validated frontend\") }}\npub fn compiled_main_menu()->Vec<sandbox_catalog::frontend::MenuEntry> {{ serde_json::from_str({:?}).expect(\"build-validated menu\") }}\n",serde_json::to_string(c).unwrap(),serde_json::to_string(&c.entries).unwrap())
}
