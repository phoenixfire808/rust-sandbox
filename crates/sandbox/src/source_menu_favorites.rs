//! Bounded, local spawn-menu favorites. Keys are `model:<virtual-path>` and
//! `entry:<catalog-id>` so model paths cannot collide with catalog IDs.
use bevy::prelude::{default, Resource, World};
use std::collections::BTreeSet;

const MAX_BYTES: u64 = 64 * 1024;
const MAX_FAVORITES: usize = 512;
const FILE_NAME: &str = "menu-favorites.json";

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedFavorites {
    version: u8,
    favorites: Vec<String>,
}

#[derive(Resource, Default)]
pub(crate) struct FavoritesState {
    ids: BTreeSet<String>,
    read_only: bool,
    warning: Option<String>,
}

impl FavoritesState {
    pub(crate) fn load() -> Self {
        let path = crate::project_root().join("local").join(FILE_NAME);
        let metadata = match std::fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() => metadata,
            Ok(_) => {
                return Self {
                    read_only: true,
                    warning: Some(
                        "Favorites path is not a regular file; it was left untouched".into(),
                    ),
                    ..default()
                };
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Self::default();
            }
            Err(error) => {
                return Self {
                    read_only: true,
                    warning: Some(format!(
                        "Favorites path could not be inspected and was left untouched: {error}"
                    )),
                    ..default()
                };
            }
        };
        if metadata.len() > MAX_BYTES {
            return Self {
                read_only: true,
                warning: Some(format!(
                    "Favorites file exceeds {} KiB and was left untouched",
                    MAX_BYTES / 1024
                )),
                ..default()
            };
        }
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                return Self {
                    read_only: true,
                    warning: Some(format!(
                        "Favorites could not be read and were left untouched: {error}"
                    )),
                    ..default()
                };
            }
        };
        let saved = match serde_json::from_slice::<SavedFavorites>(&bytes) {
            Ok(saved) if saved.version == 1 && saved.favorites.len() <= MAX_FAVORITES => saved,
            _ => {
                return Self {
                    read_only: true,
                    warning: Some(
                        "Favorites file is invalid; it was preserved and favorites are read-only"
                            .into(),
                    ),
                    ..default()
                };
            }
        };
        let mut ids = BTreeSet::new();
        for id in saved.favorites {
            if !valid_id(&id) {
                return Self {
                    read_only: true,
                    warning: Some("Favorites file contains an invalid ID; it was preserved and favorites are read-only".into()),
                    ..default()
                };
            }
            ids.insert(id);
            if ids.len() > MAX_FAVORITES {
                return Self {
                    read_only: true,
                    warning: Some(
                        "Favorites file contains too many unique entries; it was preserved".into(),
                    ),
                    ..default()
                };
            }
        }
        Self { ids, ..default() }
    }
}

fn valid_id(id: &str) -> bool {
    let Some((namespace, value)) = id.split_once(':') else {
        return false;
    };
    !value.is_empty()
        && value.len() <= 240
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'/'))
        && match namespace {
            "entry" => !value.contains('/') && !value.starts_with('.') && !value.ends_with('.'),
            "model" => {
                value.starts_with("models/")
                    && value.to_ascii_lowercase() == value
                    && value.ends_with(".mdl")
                    && value
                        .split('/')
                        .all(|part| !part.is_empty() && part != "." && part != "..")
            }
            _ => false,
        }
}

pub(crate) fn contains(world: &World, id: &str) -> bool {
    world
        .get_resource::<FavoritesState>()
        .is_some_and(|state| state.ids.contains(id))
}

pub(crate) fn warning(world: &World) -> Option<String> {
    world
        .get_resource::<FavoritesState>()
        .and_then(|state| state.warning.clone())
}

pub(crate) fn toggle(world: &mut World, id: String) {
    if !valid_id(&id) {
        world.resource_mut::<super::PlayState>().status =
            "Favorite not saved: invalid model or catalog identifier".into();
        return;
    }
    let snapshot = world.get_resource::<FavoritesState>().map(|state| {
        (
            state.ids.clone(),
            state.read_only.then(|| {
                state
                    .warning
                    .clone()
                    .unwrap_or_else(|| "Favorites are read-only".into())
            }),
        )
    });
    let Some((candidate, read_only_warning)) = snapshot else {
        world.resource_mut::<super::PlayState>().status =
            "Favorites are unavailable because their state was not initialized".into();
        return;
    };
    let mut candidate = candidate;
    if let Some(warning) = read_only_warning {
        world.resource_mut::<super::PlayState>().status = warning;
        return;
    }
    if !candidate.remove(&id) {
        if candidate.len() >= MAX_FAVORITES {
            world.resource_mut::<super::PlayState>().status =
                format!("Favorites limit reached ({MAX_FAVORITES})");
            return;
        }
        candidate.insert(id);
    }
    let saved = SavedFavorites {
        version: 1,
        favorites: candidate.iter().cloned().collect(),
    };
    let bytes = match serde_json::to_vec_pretty(&saved) {
        Ok(bytes) if bytes.len() as u64 <= MAX_BYTES => bytes,
        Ok(_) => {
            world.resource_mut::<super::PlayState>().status =
                "Favorites not saved: serialized data exceeds the 64 KiB limit".into();
            return;
        }
        Err(error) => {
            world.resource_mut::<super::PlayState>().status =
                format!("Favorites not saved: {error}");
            return;
        }
    };
    let root = world.resource::<super::PlayState>().storage_root.clone();
    let path = root.join(FILE_NAME);
    let result = (|| -> std::io::Result<()> {
        std::fs::create_dir_all(&root)?;
        write_atomically(&path, &bytes)
    })();
    if let Err(error) = result {
        world.resource_mut::<super::PlayState>().status =
            format!("Favorite not changed because saving failed: {error}");
        return;
    }
    if let Some(mut state) = world.get_resource_mut::<FavoritesState>() {
        state.ids = candidate;
    }
    let mut play = world.resource_mut::<super::PlayState>();
    play.dirty = true;
    play.page = 0;
    play.status = "Favorite updated".into();
}

fn write_atomically(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "favorites path has no parent",
        )
    })?;
    let file_name = path
        .file_name()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "favorites path has no file name",
            )
        })?
        .to_string_lossy();
    let mut last_error = None;
    for _ in 0..8 {
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(
            ".{file_name}.{}.{}.tmp",
            std::process::id(),
            serial
        ));
        let mut file = match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                last_error = Some(error);
                continue;
            }
            Err(error) => return Err(error),
        };
        let write_result = file.write_all(bytes).and_then(|()| file.sync_all());
        drop(file);
        if let Err(error) = write_result {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
        if let Err(error) = std::fs::rename(&temporary, path) {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
        return Ok(());
    }
    Err(last_error.unwrap_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "could not reserve a temporary favorites file",
        )
    }))
}
