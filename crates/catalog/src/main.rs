use sandbox_catalog::{behavior, content, inventory, Result};
use std::path::Path;
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        #[cfg(feature="workbook")]
        [cmd,sheets,inventory,out] if cmd=="workbook" => {
            let n=sandbox_catalog::workbook::export(Path::new(sheets),Path::new(inventory),Path::new(out))?;
            println!("Exported {n} data rows into {out}");
        }
        [cmd,dir] if cmd=="validate" => {
            let c=content::load(Path::new(dir))?;
            let specs=behavior::load(Path::new(dir))?;
            println!("VALID: {} prop definitions, {} scene instances, {} Hz, {} behavior specifications",c.props.len(),c.scene.len(),c.world.fixed_hz,specs.len());
        }
        [cmd,dir,out] if cmd=="generate" => {
            if Path::new(out).exists(){return Err("output already exists: choose a new filename".into());}
            let c=content::load(Path::new(dir))?;
            let mut code=content::generate_rust(&c)?;
            code.push_str(&behavior::generate(&behavior::load(Path::new(dir))?)?);
            std::fs::write(out,code)?;
            println!("Generated typed Rust structs: {out}");
        }
        [cmd,root,out] if cmd=="inventory" => {
            let s=inventory::inventory(Path::new(root),Path::new(out))?;
            println!("{}",serde_json::to_string_pretty(&s)?);
        }
        _=>return Err("usage: sandbox-catalog validate SHEETS | generate SHEETS NEW_OUTPUT.rs | inventory INSTALL_ROOT NEW_OUTPUT_DIRECTORY | workbook SHEETS INVENTORY NEW_OUTPUT.xlsx (requires --features workbook)".into())
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}
