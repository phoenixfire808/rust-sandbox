//! Private feedback snapshots captured before opening the editor, never at submission time.
use super::*;
use bevy::ui::RelativeCursorPosition;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub(super) struct EditorSlot;

#[derive(Component)]
pub(crate) struct ContextTarget {
    label: String,
    priority: u16,
}
#[derive(Resource, Default)]
pub(super) struct Request(pub Option<String>);
#[derive(Default, Serialize, Deserialize)]
pub(super) struct Draft {
    #[serde(default)]
    pub fields: [String; 4],
    #[serde(default)]
    pub body: Option<String>,
    pub category: usize,
    pub context: Option<serde_json::Value>,
}
pub(super) fn body(draft: &Draft) -> String {
    draft.body.clone().unwrap_or_else(|| {
        ["Title", "Observed", "Expected", "Notes"]
            .iter()
            .zip(&draft.fields)
            .filter(|(_, text)| !text.is_empty())
            .map(|(label, text)| format!("{label}: {text}"))
            .collect::<Vec<_>>()
            .join("\n\n")
    })
}
pub(crate) fn tag(world: &mut World, entity: Entity, label: impl Into<String>, priority: u16) {
    world.entity_mut(entity).insert((
        ContextTarget {
            label: label.into(),
            priority,
        },
        RelativeCursorPosition::default(),
    ));
}
pub(super) fn load() -> Draft {
    if let Some(draft) = std::fs::read(project_root().join("local/feedback-context-draft.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<Draft>(&b).ok())
    {
        return Draft {
            category: draft.category.min(3),
            ..draft
        };
    }
    // Keep the original four-field draft file untouched when migrating.
    Draft {
        fields: std::fs::read(project_root().join("local/feedback-draft.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default(),
        ..default()
    }
}
pub(super) fn persist(f: &Frontend) -> Result<()> {
    let dir = project_root().join("local");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("feedback-context-draft.json");
    let temp = dir.join(format!("feedback-context-draft-{}.tmp", std::process::id()));
    let draft = Draft {
        fields: f.fields.clone(),
        body: Some(f.feedback_text.clone()),
        category: f.category,
        context: f.feedback_context.clone(),
    };
    use std::io::Write;
    let mut file = std::fs::File::create(&temp)?;
    file.write_all(&serde_json::to_vec_pretty(&draft)?)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(temp, path)?;
    Ok(())
}
fn capture(world: &mut World, f: &Frontend, explicit: Option<String>) -> serde_json::Value {
    let hovered = world
        .query::<(&ContextTarget, &RelativeCursorPosition)>()
        .iter(world)
        .filter(|(_, cursor)| cursor.mouse_over())
        .max_by_key(|(target, _)| target.priority)
        .map(|(target, _)| target.label.clone());
    let play = world.get_resource::<PlayState>();
    let screen = if f.page == Page::Hidden {
        if play.is_some_and(|p| p.menu_open) {
            "spawn_tool_menu".into()
        } else {
            "gameplay".into()
        }
    } else {
        format!("{:?}", f.page)
    };
    let subject = explicit.or(hovered).unwrap_or_else(|| screen.clone());
    let gameplay = play.map(|p| serde_json::json!({
        "menu_open":p.menu_open,"tab":p.tab,"category":p.category,"search":p.search,
        "creation_tab":p.creation_tab,"catalog_selected":p.catalog_selected,
        "catalog_definition":p.spawn_catalog.entries.iter().find(|e|e.id==p.catalog_selected),
        "page":p.page,"selected_model":p.selected,"active_weapon":p.active_weapon,
        "vehicle":world.get_resource::<crate::source_play::vehicles::Occupancy>().map(|v| serde_json::json!({"definition":v.definition,"entity":v.vehicle.map(|e|e.to_bits()),"transition_weight":v.weight,"pose":v.pose})),
        "selected_tool":p.tool,"tool_settings":p.tools.values,
        "tool_stage":p.tools.stage.as_ref().map(|(tool,target,step)| serde_json::json!({"tool":tool,"step":step,"target_entity":target.entity.to_bits(),"point":target.point.to_array(),"normal":target.normal.to_array()})),
        "held_entity":p.held.map(|e|e.to_bits()),"game_status":p.status,
        "fov":p.fov,"sensitivity":p.sensitivity,"noclip_speed":p.speed
    }));
    let player = world.get_resource::<PlayerState>().map(|p| serde_json::json!({
        "eye_position":p.eye.translation.to_array(),"yaw":p.yaw,"pitch":p.pitch,
        "local_velocity":p.local_velocity.to_array(),"grounded":p.grounded,"noclip":p.noclip,"third_person":p.third_person,
        "movement":{"command":p.direction.to_array(),"sprinting":p.running,"horizontal_velocity":p.horizontal_velocity.to_array(),"vertical_velocity":p.vertical,
            "ground_friction":p.config.ground_friction,"ground_acceleration":p.config.ground_acceleration,"stop_speed":p.config.stop_speed,"air_acceleration":p.config.air_acceleration,"air_speed_cap":p.config.air_speed_cap}
    }));
    let window = world.query_filtered::<&Window, With<PrimaryWindow>>().iter(world).next()
        .map(|w|serde_json::json!({"width":w.width(),"height":w.height(),"scale_factor":w.scale_factor(),"focused":w.focused,"cursor_visible":w.cursor_options.visible,"cursor_grab":format!("{:?}",w.cursor_options.grab_mode)}));
    let props = world.query::<&SpawnedProp>().iter(world).count();
    serde_json::json!({"captured_unix_millis":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|v|v.as_millis().to_string()).unwrap_or_default(),
        "screen":screen,"subject":subject,"map":f.map,"selected_map":f.selected,"map_search":f.map_search,
        "popup":f.popup.map(|p|format!("{p:?}")),"gameplay":gameplay,"player":player,"window":window,"prop_count":props})
}
pub(super) fn open(world: &mut World, f: &mut Frontend, subject: Option<String>) {
    if f.page == Page::Feedback {
        return;
    }
    let context = capture(world, f, subject);
    f.feedback_return = f.page;
    f.feedback_return_menu = world
        .get_resource::<PlayState>()
        .is_some_and(|p| p.menu_open);
    f.feedback_candidate = Some(context.clone());
    if f.feedback_context.is_none() || f.feedback_text.trim().is_empty() {
        f.feedback_context = Some(context);
        f.message.clear();
    } else {
        f.message = "Continuing your unsent draft with its original context. Use 'Attach current context' to retarget it.".into();
    }
    f.page = Page::Feedback;
    f.popup = None;
    f.feedback_focus = true;
    f.search_focus = false;
    f.dirty = true;
    f.blocked_frame = true;
    if let Err(error) = persist(f) {
        f.message = format!("Draft retained in memory; save failed: {error}");
    }
}
pub(super) fn close(f: &mut Frontend) {
    f.page = f.feedback_return;
    f.resume_menu = (f.page == Page::Hidden).then_some(f.feedback_return_menu);
    f.blocked_frame = true;
    f.dirty = true;
}
pub(super) fn summary(f: &Frontend) -> String {
    f.feedback_context
        .as_ref()
        .map(|c| {
            format!(
                "Context: {} / {}\nMap: {} | Tool: {} | Model: {}",
                c["screen"].as_str().unwrap_or("unknown"),
                c["subject"].as_str().unwrap_or("unknown"),
                c["map"].as_str().unwrap_or("startup"),
                c["gameplay"]["selected_tool"].as_str().unwrap_or("none"),
                c["gameplay"]["selected_model"].as_str().unwrap_or("none")
            )
        })
        .unwrap_or_else(|| "No context captured".into())
}
