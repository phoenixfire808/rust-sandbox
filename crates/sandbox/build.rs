fn main() {
    let sheets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets");
    for name in [
        "props.csv",
        "world.csv",
        "scene.csv",
        "systems.csv",
        "tools.csv",
        "sources.csv",
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
    std::fs::write(out.join("content_generated.rs"), generated)
        .expect("cannot write generated Rust");
}
