//! Read-only diagnostics for locally mounted model data.
use rust_sandbox::source_assets::Mounts;
use sandbox_catalog::Result;
fn main() -> Result<()> {
    let install = std::env::var_os("GMOD_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| r"D:\SteamLibrary\steamapps\common\GarrysMod".into());
    let mounts = Mounts::open(&install)?;
    let bsp = vbsp::Bsp::read(&std::fs::read(
        install.join("garrysmod/maps/gm_construct.bsp"),
    )?)?;
    for (i, model) in bsp.models().enumerate() {
        println!(
            "BRUSH {i}: {} faces first={:?}",
            model.faces().count(),
            model
                .faces()
                .next()
                .and_then(|f| f.vertex_positions().next())
        );
        if i > 0 && i < 9 {
            for face in model.faces() {
                println!(
                    "BRUSH_FACE {i} side={} normal={:?} first={:?} material={}",
                    face.side,
                    face.normal(),
                    face.vertex_positions().next(),
                    face.texture().name()
                );
            }
        }
    }
    for e in bsp.entities.iter().filter(|e| e.prop("model").is_some()) {
        println!(
            "ENTITY {:?} model={:?} origin={:?} angles={:?}",
            e.prop("classname"),
            e.prop("model"),
            e.prop("origin"),
            e.prop("angles")
        );
    }
    for path in std::env::args().skip(1) {
        if path.ends_with(".vmt") {
            println!(
                "MATERIAL {path}\n{}",
                String::from_utf8_lossy(&mounts.read(&bsp, &path)?)
            );
            continue;
        }
        println!("MODEL {path}");
        let bytes = mounts.read(&bsp, &path)?;
        let int = |o: usize| i32::from_le_bytes(bytes[o..o + 4].try_into().unwrap()) as usize;
        let text = |o: usize| {
            String::from_utf8_lossy(&bytes[o..])
                .split('\0')
                .next()
                .unwrap()
                .to_owned()
        };
        for i in 0..int(336) {
            let o = int(340) + i * 8;
            println!("INCLUDE {}", text(o + int(o + 4)));
        }
        for i in 0..int(180) {
            let o = int(184) + i * 100;
            println!(
                "RAWANIM {} frames={} block={} sections={}",
                text(o + int(o + 4)),
                int(o + 16),
                int(o + 52),
                int(o + 84)
            );
        }
        if path.contains("_anm") {
            continue;
        }
        let skeleton = rust_sandbox::source_animation::Skeleton::read(&bytes)?;
        for clip in if path.contains("superphys") {
            vec!["@idle"]
        } else if path.contains("toolgun") {
            vec!["@idle01"]
        } else {
            vec!["@idle_physgun", "a_walking_walk_physgun_N"]
        } {
            let source = if path.contains("player/") {
                "models/m_anm.mdl"
            } else {
                &path
            };
            match rust_sandbox::source_animation::load_clip(&mounts, &bsp, source, clip, &skeleton)
            {
                Ok(c) => println!("DECODED {} {} frames", c.name, c.frames.len()),
                Err(e) => println!("CLIP_ERROR {clip}: {e}"),
            }
        }
        let mdl = vmdl::Mdl::read(&bytes)?;
        println!(
            "TEXTURES {:?} SKINS {:?}",
            mdl.textures.iter().map(|t| &t.name).collect::<Vec<_>>(),
            mdl.skin_table
        );
        println!(
            "bounds {:?} bones {}",
            mdl.header.bounding_box,
            mdl.bones.len()
        );
        for (i, b) in mdl.bones.iter().enumerate() {
            println!(
                "BONE {i} {} parent={} pos={:?} rot={:?}",
                b.name, b.parent, b.pos, b.quaternion
            );
        }
        for a in &mdl.local_animations {
            println!(
                "ANIM {} frames={} fps={} bones={}",
                a.name,
                a.frame_count,
                a.fps,
                a.animations.len()
            );
        }
        for a in &mdl.attachments {
            println!("ATTACH {} bone={} {:?}", a.name, a.local_bone, a.local);
        }
    }
    Ok(())
}
