use rust_sandbox::{compiled_source_maps, source_assets};
use std::path::{Path, PathBuf};

#[test]
fn map_spreadsheets_reach_compiled_runtime() {
    let authored = sandbox_catalog::source_maps::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets"),
    )
    .unwrap();
    assert_eq!(authored, compiled_source_maps());
}

/// Uses real installed content and the exact loader called by the Bevy renderer.
/// Explicit opt-in keeps proprietary fixtures out of CI and the repository.
#[test]
#[ignore = "requires locally owned Garry's Mod installation via GMOD_DIR"]
fn installed_stock_maps_load_original_textures_and_geometry() {
    let install = PathBuf::from(
        std::env::var_os("GMOD_DIR")
            .expect("set GMOD_DIR to the local Steam GarrysMod installation"),
    );
    for def in compiled_source_maps() {
        let loaded = source_assets::load(&install, &def).unwrap();
        assert_eq!(loaded.sky.len(), 6, "original six sky textures");
        assert!(loaded.report["world_faces"].as_u64().unwrap() > 100);
        let textured = loaded
            .surfaces
            .iter()
            .filter(|s| s.texture.is_some())
            .count();
        assert!(
            textured * 100 >= loaded.surfaces.len() * 95,
            "missing Source content mount: {textured}/{} materials",
            loaded.surfaces.len()
        );
        let placed = loaded.report["loaded_static_props"].as_u64().unwrap();
        let expected = loaded.report["source_static_props"].as_u64().unwrap();
        assert!(
            placed * 100 >= expected * 95,
            "too many missing original models: {placed}/{expected}"
        );
        assert!(loaded.spawn.is_finite());
        for surface in &loaded.surfaces {
            let geo = &surface.geometry;
            assert_eq!(geo.positions.len() % 3, 0);
            assert_eq!(geo.positions.len(), geo.normals.len());
            assert_eq!(geo.positions.len(), geo.uv.len());
            assert_eq!(geo.positions.len(), geo.light_uv.len());
            assert!(geo.positions.iter().flatten().all(|v| v.is_finite()));
            assert!(geo.uv.iter().flatten().all(|v| v.is_finite()));
            assert!(geo.light_uv.iter().flatten().all(|v| v.is_finite()));
            if let Some(name) = &surface.texture {
                let image = &loaded.textures[name];
                assert!(image.width() > 0 && image.height() > 0);
                assert!(image.texture_descriptor.mip_level_count > 1);
                assert!(
                    image.data.as_ref().unwrap().len()
                        >= (image.width() * image.height() * 4) as usize
                );
            }
        }
    }
}
