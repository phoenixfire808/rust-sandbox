use crate::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutConfig {
    pub border_factor: f32,
    pub min_margin: f32,
    pub max_margin: f32,
    pub reference_width: f32,
    pub reference_height: f32,
    pub tool_width: f32,
    pub small_tool_width: f32,
    pub tree_width: f32,
    pub small_tree_width: f32,
    pub tool_list_width: f32,
    pub divider: f32,
    pub icon_size: f32,
    pub icon_gap: f32,
    pub row_height: f32,
    pub tab_height: f32,
    pub font_size: f32,
    pub panel_padding: f32,
    pub scroll_step: f32,
    pub hold_q: bool,
    pub beam_range: f32,
    pub blend_seconds: f32,
    pub move_threshold: f32,
    pub run_threshold: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationState {
    pub hold: String,
    pub idle: String,
    pub walk: String,
    pub run: String,
    pub jump: String,
    pub crouch_idle: String,
    pub crouch_walk: String,
    pub swim_idle: String,
    pub swim: String,
    pub reload: String,
    pub attack: String,
    pub walk_speed: f32,
    pub run_speed: f32,
}
pub fn load(path: &Path) -> Result<(LayoutConfig, Vec<AnimationState>)> {
    let mut rows: Vec<LayoutConfig> = csv::Reader::from_path(path.join("source_layout.csv"))?
        .deserialize()
        .collect::<std::result::Result<_, _>>()?;
    if rows.len() != 1 {
        return Err("source_layout.csv requires one row".into());
    }
    let c = rows.remove(0);
    for v in [
        c.border_factor,
        c.min_margin,
        c.max_margin,
        c.reference_width,
        c.reference_height,
        c.tool_width,
        c.small_tool_width,
        c.tree_width,
        c.small_tree_width,
        c.tool_list_width,
        c.divider,
        c.icon_size,
        c.icon_gap,
        c.row_height,
        c.tab_height,
        c.font_size,
        c.panel_padding,
        c.scroll_step,
        c.beam_range,
        c.blend_seconds,
        c.move_threshold,
        c.run_threshold,
    ] {
        if !v.is_finite() || v <= 0. || v > 8192. {
            return Err("invalid presentation parameter".into());
        }
    }
    if c.min_margin > c.max_margin
        || c.border_factor > 1.
        || c.blend_seconds > 2.
        || c.icon_size < 16.
        || c.font_size < 6.
        || c.run_threshold <= c.move_threshold
        || c.tool_list_width + c.divider >= c.small_tool_width
    {
        return Err("inconsistent presentation dimensions".into());
    }
    let states: Vec<AnimationState> =
        csv::Reader::from_path(path.join("source_animation_states.csv"))?
            .deserialize()
            .collect::<std::result::Result<_, _>>()?;
    let mut holds = BTreeSet::new();
    for s in &states {
        if !holds.insert(s.hold.as_str())
            || !["physgun", "pistol", "revolver", "smg1", "ar2", "shotgun", "crossbow", "rpg", "grenade", "melee", "melee2"].contains(&s.hold.as_str())
            || s.reload.len() > 200
            || s.attack.len() > 200
            || !s.walk.contains("{direction}")
            || !s.run.contains("{direction}")
            || !s.crouch_walk.contains("{direction}")
            || !s.swim.contains("{direction}")
            || [
                &s.idle,
                &s.walk,
                &s.run,
                &s.jump,
                &s.crouch_idle,
                &s.crouch_walk,
                &s.swim_idle,
                &s.swim,
            ]
            .iter()
            .any(|s| s.is_empty() || s.len() > 200)
            || !s.walk_speed.is_finite()
            || !s.run_speed.is_finite()
            || s.walk_speed <= 0.
            || s.run_speed < s.walk_speed
        {
            return Err("invalid animation state mapping".into());
        }
    }
    if !holds.contains("physgun") || !holds.contains("pistol") {
        return Err("base physgun and pistol hold types require animation mappings".into());
    }
    Ok((c, states))
}
pub fn generate(c: &LayoutConfig, states: &[AnimationState]) -> String {
    format!("pub fn compiled_layout_config()->sandbox_catalog::presentation::LayoutConfig {{ serde_json::from_str({:?}).expect(\"build-validated layout\") }}\npub fn compiled_animation_states()->Vec<sandbox_catalog::presentation::AnimationState> {{ serde_json::from_str({:?}).expect(\"build-validated animation states\") }}\n", serde_json::to_string(c).unwrap(), serde_json::to_string(states).unwrap())
}
