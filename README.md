# Rust Workshop

Independent **Bevy 0.16.1 + Rapier 0.30** sandbox foundation, with spreadsheet-authored data and a Garry's Mod parity research catalog.

**This is not a complete Garry's Mod replacement, a Source port, or a Lua-compatible engine.** It is a working first implementation plus an explicit roadmap. No proprietary assets or Valve/Facepunch implementation code are bundled.

## Continue development

- **[Download the complete review workbook](spreadsheets/rust-sandbox-catalog-20260929-presentation.xlsx)**: includes the reference placement ledger, animation mappings and layout parameters. Use GitHub's **Download raw file** button for Excel/LibreOffice.
- **[Spreadsheet contributor guide](spreadsheets/README.md)**: where to edit, validate, rebuild and regenerate the workbook.
- **[Remaining feature inventory](sheets/parity_gaps.csv)**: 169 missing/partial/unverified entries covering every cataloged system and tool, with priorities and acceptance checks. Includes nonmatching menus and remaining physgun fidelity gaps.
- **[Ordered implementation queue](sheets/work_queue.csv)**: dependency-ordered work packages assign every known gap exactly once. The first effects slice is implemented, not reference-equivalent.
- **[Authored CSV sheets](sheets/)** and **[public metadata catalogs](catalogs/stock-20260929/)**: all workbook inputs are tracked. No private local files or original game payloads are needed to inspect or export the data.

The gap inventory covers the current stock-game discovery catalog, not every possible native option or community addon. A reference row is not proof of implemented behavior or one-to-one parity.

### Current presentation repair

[Detailed plan](docs/PRESENTATION_PLAN.md), [source/reference ledger](sheets/presentation_references.csv), [layout dimensions](sheets/source_layout.csv), and [animation state mappings](sheets/source_animation_states.csv). The current build adds eight-direction velocity-driven movement clips, non-looping jump poses and crossfades, a physgun firing beam even without a grabbed prop, and reference-sized simultaneous browser/icon/tool panels. These changes are built and launched for Drew, not claimed visually verified or one-to-one.

## Start

On this machine, double-click `play.cmd`, or from Windows cmd:

```bat
cd /d C:\Users\Drew\Projects\rust-sandbox
set "CARGO_TARGET_DIR=D:\jcode-build\rust-sandbox-20260929"
call play.cmd
rem Or choose the other original installed map:
call play.cmd gm_flatgrass
```

The launcher builds first, so spreadsheet edits cannot silently leave you running an old binary. First build requires Rust, MSVC build tools, dependency downloads, and a supported graphics adapter. The selected versions are a deliberately verified compatible pair, not a claim to be the latest Bevy release.

**Normal launch now loads the original installed `gm_construct` in Bevy**, not the box demo. It reads BSP geometry/displacements/lightmaps, VPK-mounted VMT/VTF textures, six sky faces, and placed MDL/VVD/VTX static models. Set `GMOD_DIR` if your Steam `GarrysMod` root differs from `D:\SteamLibrary\steamapps\common\GarrysMod`. First content decode takes a short time, with per-asset progress in the console. No Steam files are changed.

Click to capture the mouse. **WASD walks**, **Space jumps**, **Shift runs**, **F4 toggles third person**, and **V toggles noclip** (Space/Ctrl moves vertically only in noclip). Esc releases the mouse and F10 quits. The player uses the original Kleiner model, original idle/walk/run clips, bone-merged first-person hands, and attached world weapons. Grounded movement uses Rapier, not an exact Source physics solver.

The physgun now has mounted additive attachment glows and a scrolling textured beam with a release/switch-cleaned endpoint flare. Color and presentation parameters come from `sheets/source_effects.csv`. These are approximations, not matched stock lighting, claw animation or sound.

**Hold Q** to open the original-model build menu and release it to close. Click the search field to retain keyboard focus (Enter ends editing, Esc closes). The left browser and icon area stay alongside the right tool/options panels. **1/2** selects physgun/toolgun. Left mouse fires the physgun beam with or without a grabbed prop. **Z** undoes, and **F5/F6** saves/loads local prop scenes. `play-prototype.cmd` preserves the earlier physics sandbox separately. The detailed fidelity checklist and unimplemented one-to-one requirements are in [PLAYER_PARITY.md](docs/PLAYER_PARITY.md). Run `cargo run -p rust-sandbox --bin source-player-check` to validate locally installed player/weapon assets, or `call play.cmd --smoke` to capture both views and an interior-wall check.

## Delivered files

| Location | Purpose |
|---|---|
| `sheets/props.csv` | Authored procedural prop sizes, mass, friction, bounce, color |
| `sheets/source_maps.csv` | Original installed map references, scale, FOV and inspection settings |
| `sheets/source_player.csv` | Original player/hands/weapon references, movement dimensions and camera settings |
| `sheets/source_play.csv` | Prop menu, grabbing, gravity and sandbox parameters |
| `sheets/source_effects.csv` | Mounted physgun sprites and beam texture, color, widths, pulse and scroll |
| `sheets/work_queue.csv` | Ordered implementation work packages and explicit remaining acceptance |
| `sheets/source_pipeline.csv` | Import capabilities and remaining fidelity gaps |
| `sheets/world.csv` | Gravity, physics tick rate, floor size, movement, distances, prop limit |
| `sheets/scene.csv` | Initial prop instances and frozen state |
| `sheets/systems.csv` | 64 major system specifications and parity gaps |
| `sheets/tools.csv` | 37 installed stock tool specifications, including internal/example entries |
| `sheets/sources.csv` | Research citations and evidence limitations |
| `sheets/milestones.csv` | Sequenced implementation plan and acceptance gates |
| `local/inventory-20260929/` | Installed file and VPK metadata, Lua declaration candidates, stock tool list |
| `local/gmod-research.xlsx` | Generated Excel review workbook, when exported |
| `local/content_generated.rs` | Inspectable generated Rust example, when generated |
| `docs/PLAN.md` | Architecture, parity definition, phases and risks |
| `docs/SCHEMA.md` | Spreadsheet contract and units |
| `docs/RESEARCH.md` | Sources, decisions, observed installation scope |
| `docs/VALIDATION.md` | Executed checks and untested boundaries |

CSV sheets are the **authoritative editable source**. Open them in Excel or LibreOffice and save as UTF-8 CSV. XLSX is an exported review snapshot, not a second source of truth. To use edits made in the workbook, export the relevant tab back to its named CSV before rebuilding. No Excel formulas are evaluated by the build.

## Preserved prototype gameplay (`play-prototype.cmd`)

- Lit 3D workshop with four original procedural prop types and a spreadsheet-built spawn menu.
- Fixed-step rigid-body gravity and collisions, mass, friction, restitution and CCD.
- Free-fly camera, ray-targeted grabbing, distance adjustment, simple rotation, freeze/unfreeze.
- Single-prop duplication/removal, 32-step scene-snapshot undo, reset.
- Versioned JSON save/load with validation before live-world replacement. Saves get new filenames.

Controls are displayed in the game. **Middle mouse** looks around. **WASD / Space / Left Shift** flies. Click a menu entry or press **Enter** to spawn. **Tab / 1-4** selects a prop. **Left mouse** holds a targeted prop, **wheel** adjusts distance, **E** rotates, **right mouse / F** freezes, **R** unfreezes, **X** removes, **C** duplicates, **Z** undoes, **F9** resets, **F5** saves, **F6** loads the latest save, **Esc** exits.

## Spreadsheet to Rust

```mermaid
flowchart LR
    CSV[Authored CSV tables] --> Validate[Typed schemas and validation]
    Validate --> Build[Cargo build.rs]
    Build --> Rust[Generated Rust struct literals]
    Rust --> ECS[Bevy entities and resources]
    ECS --> Physics[Rapier fixed-step simulation]
    ECS --> Render[Bevy meshes UI and input]
    Research[System and tool sheets] --> Registry[Typed BehaviorSpec registry]
    Registry --> Build
    Steam[Local Steam files read-only] --> Inventory[Asset and Lua metadata CSV]
    Inventory --> XLSX[Excel review workbook]
    CSV --> XLSX
```

Descriptions become typed **specification metadata**, not magic executable behavior. Systems still need implementation and tests. The prototype runtime consumes prop/world/scene data. The compiled behavior registry records what remains.

```bat
cargo run -p sandbox-catalog -- validate sheets
cargo run -p sandbox-catalog -- generate sheets local\content_generated-new.rs
cargo run -p sandbox-catalog -- inventory "D:\SteamLibrary\steamapps\common\GarrysMod" "C:\Users\Drew\Projects\rust-sandbox\local\inventory-new"
cargo run -p sandbox-catalog --features workbook -- workbook sheets local\inventory-20260929 local\gmod-research-new.xlsx
cargo test --workspace --all-features
cargo run -p rust-sandbox -- --headless-smoke
cargo run -p rust-sandbox -- --smoke
```

The inventory and export commands refuse existing destinations. Inventory reads directory metadata, not an entire asset payload into spreadsheet cells. Paths, sizes, archive location, declared CRC and provenance are cataloged. **CRC is recorded, not independently verified.** Binary models/textures are not converted into Rust or redistributed.

## Important gaps

The Source importer is partial, not renderer parity. Grounded walking, a third-person player, selected original locomotion clips, attached weapons and visible brush submodels are implemented. Brush entity simulation, complete animation blending/IK, accurate static prop lighting, HDR, material blending/proxies, reflection/refraction, overlays, 3D sky scaling and original player physics remain incomplete. Unsupported assets are listed in `local/<map>-import-report.json`. Ragdolls, full constraint tools, multiplayer, NPCs, vehicles, audio, Lua runtime, Workshop integration and GMod save/addon compatibility are not implemented. Rapier is not Source VPhysics. The reference game has not been run through a comparative parity harness. Research rows are a structured baseline, not proof that every behavior or addon has been enumerated.
