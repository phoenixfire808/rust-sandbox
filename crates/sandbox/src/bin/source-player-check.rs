//! Validate original player assets without opening a window or loading world textures.
use bevy::prelude::*;
use rust_sandbox::{
    source_assets::{MountedSource, Mounts},
    source_play::PlayState,
    source_player,
};
use sandbox_catalog::Result;
fn main() -> Result<()> {
    let install = std::env::var_os("GMOD_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| r"D:\SteamLibrary\steamapps\common\GarrysMod".into());
    let mounts = Mounts::open(&install)?;
    let bsp = vbsp::Bsp::read(&std::fs::read(
        install.join("garrysmod/maps/gm_construct.bsp"),
    )?)?;
    let mut app = App::new();
    app.insert_resource(Assets::<Mesh>::default())
        .insert_resource(Assets::<Image>::default())
        .insert_resource(Assets::<StandardMaterial>::default())
        .insert_resource(PlayState::new(Vec::new(), 75., 10.))
        .insert_resource(MountedSource { mounts, bsp })
        .add_systems(Startup, |mut commands: Commands| {
            source_player::spawn(&mut commands, Vec3::ZERO, Vec3::NEG_Z)
        });
    app.update();
    source_player::validate_assets(app.world_mut())?;
    println!("SOURCE_PLAYER_ASSETS_OK");
    Ok(())
}
