//! Sheet-authored release checklist. Personal checkmarks never assert project acceptance.
use super::*;
use std::collections::BTreeSet;

pub(super) fn load() -> (BTreeSet<String>, String) {
    let path = project_root().join("local/release-checklist.json");
    match std::fs::read(&path) {
        Ok(bytes) => match serde_json::from_slice(&bytes) {
            Ok(checked) => (checked, String::new()),
            Err(e) => (BTreeSet::new(), format!("Checklist could not be read: {e}. Original file preserved; checkmarks are read-only this session.")),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (BTreeSet::new(), String::new()),
        Err(e) => (BTreeSet::new(), format!("Checklist could not be read: {e}. Checkmarks are read-only this session.")),
    }
}
pub(super) fn key(note: &sandbox_catalog::frontend::ReleaseNote) -> String {
    format!("{}@{}", note.id, note.revision)
}
pub(super) fn toggle(f: &mut Frontend, id: &str) -> Result<()> {
    if !f.news_error.is_empty() {
        return Err(f.news_error.clone().into());
    }
    let note = crate::compiled_release_notes()
        .into_iter()
        .find(|n| n.id == id)
        .ok_or("Unknown release checklist entry")?;
    let mut checked = f.news_checked.clone();
    let key = key(&note);
    if !checked.remove(&key) {
        checked.insert(key);
    }
    let dir = project_root().join("local");
    std::fs::create_dir_all(&dir)?;
    let temp = dir.join(format!("release-checklist-{}.tmp", std::process::id()));
    use std::io::Write;
    let mut file = std::fs::File::create(&temp)?;
    file.write_all(&serde_json::to_vec_pretty(&checked)?)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(temp, dir.join("release-checklist.json"))?;
    f.news_checked = checked;
    f.message =
        "Personal checkmark saved locally. This does not mark a feature as passing acceptance."
            .into();
    Ok(())
}
pub(super) fn open(world: &World, f: &mut Frontend) {
    // Keep unsent feedback and its original return target intact.
    if matches!(
        f.page,
        Page::News | Page::Feedback | Page::Loading | Page::Confirm
    ) {
        return;
    }
    f.news_return = f.page;
    f.news_return_menu = world
        .get_resource::<PlayState>()
        .is_some_and(|p| p.menu_open);
    f.page = Page::News;
    f.popup = None;
    f.search_focus = false;
    f.message.clear();
    f.dirty = true;
    f.blocked_frame = true;
}
pub(super) fn close(f: &mut Frontend) {
    f.page = f.news_return;
    f.resume_menu = (f.page == Page::Hidden).then_some(f.news_return_menu);
    f.blocked_frame = true;
    f.dirty = true;
    f.message.clear();
}
pub(super) fn report(world: &mut World, f: &mut Frontend, id: &str) {
    if let Some(note) = crate::compiled_release_notes()
        .into_iter()
        .find(|n| n.id == id)
    {
        feedback::open(
            world,
            f,
            Some(format!(
                "What's New: {} [{}@{}] | Try: {} | Expected: {} | Limits: {}",
                note.title, note.id, note.revision, note.steps, note.expected, note.limits
            )),
        );
    }
}
