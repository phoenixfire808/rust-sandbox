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
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
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
