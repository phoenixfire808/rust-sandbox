use crate::Result;
use serde::Serialize;
use std::{
    fs::{self, File},
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize)]
pub struct AssetRow {
    pub container: String,
    pub path: String,
    pub kind: String,
    pub bytes: u64,
    pub crc32: String,
    pub archive_index: String,
    pub archive_offset: String,
    pub preload_bytes: u16,
    pub provenance: String,
    pub redistribution: String,
    pub conversion: String,
}
#[derive(Debug, Default, Serialize)]
pub struct Summary {
    pub loose_files: usize,
    pub vpk_entries: usize,
    pub vpk_directories: usize,
    pub lua_symbols: usize,
    pub stock_tools: usize,
    pub skipped_links: usize,
    pub loose_bytes: u64,
    pub scope: String,
}

fn u16le(r: &mut impl Read) -> Result<u16> {
    let mut b = [0; 2];
    r.read_exact(&mut b)?;
    Ok(u16::from_le_bytes(b))
}
fn u32le(r: &mut impl Read) -> Result<u32> {
    let mut b = [0; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}
fn cstring(r: &mut Cursor<&[u8]>) -> Result<String> {
    let mut bytes = Vec::new();
    loop {
        let mut b = [0];
        r.read_exact(&mut b)?;
        if b[0] == 0 {
            break;
        }
        if bytes.len() >= 4096 {
            return Err("VPK string exceeds 4096 bytes".into());
        }
        bytes.push(b[0]);
    }
    Ok(String::from_utf8(bytes)?)
}
fn safe_part(part: &str) -> bool {
    !part.contains('\\')
        && !part.starts_with('/')
        && !part.contains(':')
        && !part.split('/').any(|p| p == ".." || p == ".")
}
pub fn kind(path: &str) -> &'static str {
    match path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "mdl" => "model",
        "vvd" | "vtx" | "phy" => "model_companion",
        "vmt" => "material",
        "vtf" | "png" | "jpg" | "tga" => "texture",
        "bsp" => "map",
        "nav" | "ain" => "navigation",
        "wav" | "mp3" | "ogg" => "audio",
        "lua" => "lua",
        "pcf" => "particles",
        "vpk" | "gma" => "archive",
        "dll" | "exe" => "native_binary",
        "ttf" => "font",
        _ => "other",
    }
}
/// Reads directory metadata only. Payload, archive checksums, signatures and models are NOT decoded.
pub fn read_vpk(reader: &mut impl Read, container: &str) -> Result<Vec<AssetRow>> {
    if u32le(reader)? != 0x55aa1234 {
        return Err("invalid VPK signature".into());
    }
    let version = u32le(reader)?;
    if version != 1 && version != 2 {
        return Err(format!("unsupported VPK version {version}").into());
    }
    let len = u32le(reader)? as usize;
    if len > 64 * 1024 * 1024 {
        return Err("VPK directory exceeds 64 MiB safety limit".into());
    }
    if version == 2 {
        let mut rest = [0; 16];
        reader.read_exact(&mut rest)?;
    }
    let mut tree = vec![0; len];
    reader.read_exact(&mut tree)?;
    let mut r = Cursor::new(tree.as_slice());
    let mut rows = Vec::new();
    loop {
        let ext = cstring(&mut r)?;
        if ext.is_empty() {
            break;
        }
        loop {
            let dir = cstring(&mut r)?;
            if dir.is_empty() {
                break;
            }
            loop {
                let name = cstring(&mut r)?;
                if name.is_empty() {
                    break;
                }
                if !safe_part(&ext) || !safe_part(&dir) || !safe_part(&name) {
                    return Err("unsafe VPK virtual path".into());
                }
                let crc = u32le(&mut r)?;
                let preload = u16le(&mut r)?;
                let archive = u16le(&mut r)?;
                let offset = u32le(&mut r)?;
                let size = u32le(&mut r)?;
                if u16le(&mut r)? != 0xffff {
                    return Err("invalid VPK entry terminator".into());
                }
                let next = r.position() + u64::from(preload);
                if next > len as u64 {
                    return Err("truncated VPK preload".into());
                }
                r.set_position(next);
                let path = format!(
                    "{}{}{}",
                    if dir == " " {
                        String::new()
                    } else {
                        format!("{dir}/")
                    },
                    name,
                    if ext == " " {
                        String::new()
                    } else {
                        format!(".{ext}")
                    }
                );
                rows.push(AssetRow {
                    kind: kind(&path).into(),
                    container: container.into(),
                    path,
                    bytes: u64::from(size) + u64::from(preload),
                    crc32: format!("{crc:08x}"),
                    archive_index: archive.to_string(),
                    archive_offset: offset.to_string(),
                    preload_bytes: preload,
                    provenance: "installed_game_unknown_rightsholder".into(),
                    redistribution: "not_cleared".into(),
                    conversion: "catalog_only".into(),
                });
            }
        }
    }
    if r.position() != len as u64 {
        return Err("unexpected trailing VPK directory bytes".into());
    }
    Ok(rows)
}

/// Prefix untrusted text that spreadsheet programs might interpret as a formula.
pub fn spreadsheet_text(s: &str) -> String {
    if s.starts_with(['=', '+', '-', '@', '\t', '\r', '\n']) {
        format!("'{s}")
    } else {
        s.into()
    }
}
fn files(root: &Path, out: &mut Vec<PathBuf>, skipped: &mut usize) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_symlink() {
            *skipped += 1;
            continue;
        }
        if ft.is_dir() {
            files(&entry.path(), out, skipped)?;
        } else if ft.is_file() {
            out.push(entry.path());
        }
    }
    Ok(())
}

/// Output must be a fresh directory outside Steam. Never execute or extract game code.
pub fn inventory(root: &Path, output: &Path) -> Result<Summary> {
    let root = root.canonicalize()?;
    if !root.is_dir() {
        return Err("inventory root must be a directory".into());
    }
    if output.exists() {
        return Err(
            "output already exists: use a new directory to preserve previous inventory".into(),
        );
    }
    let mut ancestor = output.parent().ok_or("output needs a parent directory")?;
    while !ancestor.exists() {
        ancestor = ancestor.parent().ok_or("output has no existing ancestor")?;
    }
    if ancestor.canonicalize()?.starts_with(&root) {
        return Err("inventory output must be outside the installation".into());
    }
    let mut summary=Summary {scope:"Loose files and VPK v1/v2 directory entries under the supplied root. No mounted external games, remote Workshop, GMA internals, or BSP embedded pak contents.".into(),..Default::default()};
    let mut paths = Vec::new();
    files(&root, &mut paths, &mut summary.skipped_links)?;
    paths.sort();
    fs::create_dir_all(output)?;
    let mut assets = csv::Writer::from_path(output.join("assets.csv"))?;
    let mut symbols = csv::Writer::from_path(output.join("lua_symbols.csv"))?;
    symbols.write_record(["file", "line", "declaration", "evidence", "port_status"])?;
    let mut tools = csv::Writer::from_path(output.join("stock_tools.csv"))?;
    tools.write_record(["id", "source", "evidence", "port_status"])?;
    for path in paths {
        let rel = path
            .strip_prefix(&root)?
            .to_string_lossy()
            .replace('\\', "/");
        let size = path.metadata()?.len();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        summary.loose_files += 1;
        summary.loose_bytes += size;
        assets.serialize(AssetRow {
            container: "loose".into(),
            path: spreadsheet_text(&rel),
            kind: kind(&rel).into(),
            bytes: size,
            crc32: String::new(),
            archive_index: String::new(),
            archive_offset: String::new(),
            preload_bytes: 0,
            provenance: "installed_game_unknown_rightsholder".into(),
            redistribution: "not_cleared".into(),
            conversion: "catalog_only".into(),
        })?;
        if rel.ends_with("_dir.vpk") {
            for mut row in
                read_vpk(&mut File::open(&path)?, &rel).map_err(|e| format!("{rel}: {e}"))?
            {
                row.path = spreadsheet_text(&row.path);
                row.container = spreadsheet_text(&row.container);
                assets.serialize(row)?;
                summary.vpk_entries += 1;
            }
            summary.vpk_directories += 1;
        }
        if ext.eq_ignore_ascii_case("lua") && size <= 8 * 1024 * 1024 {
            let source = fs::read(&path)?;
            let source = String::from_utf8_lossy(&source);
            for (line, text) in source.lines().enumerate() {
                let text = text.trim();
                if text.starts_with("function ")
                    || text.starts_with("local function ")
                    || (!text.starts_with("--") && text.contains("= function("))
                {
                    let declaration: String = text.chars().take(300).collect();
                    symbols.write_record([
                        spreadsheet_text(&rel),
                        (line + 1).to_string(),
                        spreadsheet_text(&declaration),
                        "lexical_candidate_not_semantic_analysis".into(),
                        "not_ported".into(),
                    ])?;
                    summary.lua_symbols += 1;
                }
            }
        }
        if rel.starts_with("garrysmod/gamemodes/sandbox/entities/weapons/gmod_tool/stools/")
            && ext == "lua"
            && rel.matches('/').count() == 7
        {
            let id = path.file_stem().unwrap_or_default().to_string_lossy();
            tools.write_record([
                spreadsheet_text(&id),
                spreadsheet_text(&rel),
                "installed_file".into(),
                "not_ported".into(),
            ])?;
            summary.stock_tools += 1;
        }
    }
    assets.flush()?;
    symbols.flush()?;
    tools.flush()?;
    fs::write(
        output.join("summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(version: u32) -> Vec<u8> {
        let mut tree = b"mdl\0models\0crate\0".to_vec();
        tree.extend(123u32.to_le_bytes());
        tree.extend(2u16.to_le_bytes());
        tree.extend(0x7fffu16.to_le_bytes());
        tree.extend(0u32.to_le_bytes());
        tree.extend(10u32.to_le_bytes());
        tree.extend(0xffffu16.to_le_bytes());
        tree.extend([1, 2, 0, 0, 0]);
        let mut bytes = 0x55aa1234u32.to_le_bytes().to_vec();
        bytes.extend(version.to_le_bytes());
        bytes.extend((tree.len() as u32).to_le_bytes());
        if version == 2 {
            bytes.extend([0; 16]);
        }
        bytes.extend(tree);
        bytes
    }
    #[test]
    fn vpk_versions_and_preload() {
        for v in [1, 2] {
            let rows = read_vpk(&mut fixture(v).as_slice(), "fixture").unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].path, "models/crate.mdl");
            assert_eq!(rows[0].bytes, 12);
        }
    }
    #[test]
    fn every_truncation_fails() {
        let b = fixture(2);
        for n in 0..b.len() {
            assert!(read_vpk(&mut &b[..n], "fixture").is_err(), "truncation {n}");
        }
    }
    #[test]
    fn oversized_header_rejected() {
        let mut b = fixture(1);
        b[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(read_vpk(&mut b.as_slice(), "x").is_err());
    }
    #[test]
    fn unsafe_paths_rejected() {
        assert!(!safe_part("../evil"));
        assert!(!safe_part("C:/evil"));
        assert!(!safe_part("/evil"));
    }
    #[test]
    fn formula_text_escaped() {
        assert_eq!(spreadsheet_text("=1+1"), "'=1+1");
        assert_eq!(spreadsheet_text("models/a.mdl"), "models/a.mdl");
    }
}
