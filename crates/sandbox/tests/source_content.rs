use rust_sandbox::{compiled_source_maps, source_assets};
use std::path::{Path, PathBuf};

#[test]
fn player_and_play_spreadsheets_reach_compiled_runtime() {
    let sheets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets");
    let player = sandbox_catalog::player::load(&sheets).unwrap();
    assert_eq!(
        sandbox_catalog::effects::load(&sheets).unwrap(),
        rust_sandbox::compiled_effects_config()
    );
    assert_eq!(
        serde_json::to_value(player).unwrap(),
        serde_json::to_value(rust_sandbox::compiled_player_config()).unwrap()
    );
    assert_eq!(
        sandbox_catalog::play::load(&sheets).unwrap(),
        rust_sandbox::compiled_play_config()
    );
}

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
        if def.id == "gm_construct" {
            assert!(loaded.report["brush_faces"].as_u64().unwrap() > 0);
            let room = loaded
                .surfaces
                .iter()
                .find(|s| s.name == "gm_construct/color_room")
                .unwrap();
            // Original brush 3: origin z=156, lower face z=-4. From inside
            // the room this ceiling must face down, not disappear by backface culling.
            let ceiling: Vec<_> = room
                .geometry
                .positions
                .chunks_exact(3)
                .zip(room.geometry.normals.chunks_exact(3))
                .filter(|(positions, _)| {
                    let center = positions
                        .iter()
                        .map(|p| bevy::prelude::Vec3::from_array(*p))
                        .sum::<bevy::prelude::Vec3>()
                        / (3. * def.unit_scale);
                    // Wall tops share this height but lie outside the room interior.
                    (2633. ..4567.).contains(&center.x)
                        && (809. ..3287.).contains(&center.z)
                        && positions
                            .iter()
                            .all(|p| (p[1] - 152. * def.unit_scale).abs() < 0.001)
                })
                .collect();
            assert!(
                !ceiling.is_empty(),
                "original white-room ceiling is missing"
            );
            for (positions, normals) in ceiling {
                assert!(
                    normals.iter().all(|n| n[1] < -0.99),
                    "ceiling faces away from room: positions={positions:?}, normals={normals:?}"
                );
                let a = bevy::prelude::Vec3::from_array(positions[0]);
                let b = bevy::prelude::Vec3::from_array(positions[1]);
                let c = bevy::prelude::Vec3::from_array(positions[2]);
                assert!(
                    (b - a).cross(c - a).y < 0.,
                    "ceiling winding faces away from room"
                );
            }
        }
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
