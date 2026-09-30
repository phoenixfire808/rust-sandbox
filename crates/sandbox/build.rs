fn main() {
    let sheets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets");
    for name in [
        "props.csv",
        "world.csv",
        "scene.csv",
        "systems.csv",
        "tools.csv",
        "sources.csv",
        "source_maps.csv",
        "source_play.csv",
        "source_player.csv",
        "source_effects.csv",
        "source_layout.csv",
        "source_animation_states.csv",
        "source_performance.csv",
        "source_toolgun.csv",
        "source_frontend.csv",
        "source_main_menu.csv",
        "source_release_notes.csv",
        "source_tools.csv",
        "source_tool_options.csv",
        "source_physics_materials.csv",
        "spawn_reference.csv",
        "source_creation_tabs.csv",
        "spawn_capabilities.csv",
        "source_weapons.csv",
        "source_weapon_handling.csv",
        "source_vehicles.csv",
        "source_vehicle_wheels.csv",
        "source_gameplay.csv",
        "source_model_categories.csv",
        "source_water.csv",
        "source_npcs.csv",
        "source_npc_rules.csv",
        "parity_gaps.csv",
    ] {
        println!("cargo:rerun-if-changed={}", sheets.join(name).display());
    }
    let content = sandbox_catalog::content::load(&sheets).expect("invalid spreadsheet content");
    let mut generated =
        sandbox_catalog::content::generate_rust(&content).expect("code generation failed");
    let behaviors =
        sandbox_catalog::behavior::load(&sheets).expect("invalid behavior spreadsheets");
    generated.push_str(
        &sandbox_catalog::behavior::generate(&behaviors).expect("behavior generation failed"),
    );
    let presentation =
        sandbox_catalog::presentation::load(&sheets).expect("invalid presentation sheets");
    generated.push_str(&sandbox_catalog::presentation::generate(
        &presentation.0,
        &presentation.1,
    ));
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    generated.push_str(&sandbox_catalog::spawn::generate(
        &sandbox_catalog::spawn::load(&sheets).expect("invalid spawn catalog spreadsheets"),
    ));
    generated.push_str(&sandbox_catalog::frontend::generate(
        &sandbox_catalog::frontend::load(&sheets).expect("invalid frontend spreadsheet"),
    ));
    generated.push_str(&sandbox_catalog::toolgun::generate(
        &sandbox_catalog::toolgun::load(&sheets).expect("invalid tool spreadsheets"),
    ));
    generated.push_str(&sandbox_catalog::performance::generate(
        &sandbox_catalog::performance::load(&sheets).expect("invalid performance spreadsheet"),
    ));
    generated.push_str(&sandbox_catalog::effects::generate(
        &sandbox_catalog::effects::load(&sheets).expect("invalid effects spreadsheet"),
    ));
    generated.push_str(&sandbox_catalog::player::generate(
        &sandbox_catalog::player::load(&sheets).expect("invalid player spreadsheet"),
    ));
    generated.push_str(&sandbox_catalog::play::generate(
        &sandbox_catalog::play::load(&sheets).expect("invalid gameplay spreadsheet"),
    ));
    generated.push_str(&sandbox_catalog::source_maps::generate(
        &sandbox_catalog::source_maps::load(&sheets).expect("invalid Source map spreadsheet"),
    ));
    std::fs::write(out.join("content_generated.rs"), generated)
        .expect("cannot write generated Rust");
}
