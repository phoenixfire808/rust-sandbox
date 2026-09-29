# Validation record

Date: 2026-09-29 UTC. Windows x86_64, Rust 1.97.1. Build artifacts isolated in `D:\jcode-build\rust-sandbox-20260929`. No remote compilation or remote asset upload.

## Observed passing checks

### Original installed-content follow-up

The normal `play.cmd` now starts `source-map`. Earlier prototype evidence below describes the historical launcher, now preserved as `play-prototype.cmd`.

| Original map | World faces | Displacement faces | Lightmapped faces | Textured materials | Static props decoded | Total rendered triangles |
|---|---:|---:|---:|---:|---:|---:|
| gm_construct | 6,952 | 110 | 6,887 | 203 / 205 | 181 / 182 | 105,141 |
| gm_flatgrass | 1,657 | 16 | 1,648 | 16 / 16 | 1 / 1 | 14,448 |

Both rendered six original sky textures and the BSP-authored player spawn. Actual Bevy screenshots are `evidence/gm_construct-source.png` and `evidence/gm_flatgrass-source.png`. They were visually inspected. Original buildings, terrain and textures are visible, not procedural replacements. Material counts include cubemap-specific VMT variants, not unique source textures. Textures now share CPU decode and GPU image handles by base-texture path.

`gm_construct` has two water materials without base textures. These use original VMT fog colors as a documented approximation, not Source water shading. One original model, `models/dav0r/buttons/switch.mdl`, is rejected by vmdl with `String is not null-terminated` and remains absent. Reports contain these exact warnings. No unsupported asset is counted as loaded.

The first visual iteration exposed a missing `sourceengine` mount (only 9 / 187 world materials loaded) and near-black lightmaps from Bevy's default physical exposure. Installed archive paths and Bevy's exposure implementation were inspected. Adding the actual bundled Source content root raised world texture coverage to 185 / 187, and unit exposure restored normalized LDR baked lighting. Subsequent model import raised totals to the table above. Mipmap and anisotropic filtering reduced the prominent high-frequency texture aliasing visible in the earlier screenshot.

The opt-in `installed_stock_maps_load_original_textures_and_geometry` integration test was executed against both real local maps and passed. It calls the same importer used by the renderer and checks sky count, world coverage, at least 95% texture/model coverage, finite positions/UVs, complete triangles, matching attributes and decoded texture mip data. The initial missing-content-mount result would fail its explicit texture-coverage gate. A separate test compares map CSV values to the generated runtime structs. Path traversal, invalid ranges, NaN and duplicate IDs have negative tests.

No pixel-equivalence or Source movement equivalence is established. The renderer remains a noclip asset inspection mode, separate from prototype sandbox physics. Brush entities, model animation, correct model lighting, full material shaders, HDR and 3D sky scaling need further work. OS-level physical mouse/keyboard input has not been automated for this viewer.

The final standard workspace suite passed **31 tests**, with the installed-content test ignored by default. That opt-in test was run separately and passed against both real maps. Strict workspace/all-target/all-feature clippy passed. Final `play.cmd --smoke` and `play.cmd gm_flatgrass --smoke` both exited successfully after actual screenshot capture. The refreshed `local/gmod-original-assets.xlsx` contains **12 sheets and 86,931 data rows** (5,058,294 bytes), independently checked against all CSV sources with no formula elements. The earlier workbook remains untouched.

### Earlier procedural sandbox baseline

| Requirement | Executed check | Result |
|---|---|---|
| Typed spreadsheet ingestion | Catalog tests plus `sandbox-catalog validate sheets` | Four prop definitions, five initial instances, 60 Hz, 101 behavior specs validated |
| Spreadsheet-to-Rust code | Cargo build script generates and compiles struct literals, equality tests compare compiled values to source CSV | Passed for gameplay data and full behavior registry |
| Runtime consumption | `generated_dimensions_and_mass_reach_real_physics_components` | Authored dimensions reach Rapier cuboid half-extents and mass components |
| Physics | Headless Bevy + Rapier application advances 240 fixed-step frames | Initial props settle above the ground with expected count |
| Grab/freeze | Real Bevy integration test advances 180 tracking ticks and 120 frozen ticks | Held body reaches target within 0.3 m and frozen position remains within 0.001 m |
| Editing | Spawn, duplicate, remove, freeze and undo tests | Expected entity counts and fixed-body state observed |
| Persistence | Save two new files, reload through Action, inject malformed JSON | Valid save restores state, malformed load preserves live world, save names differ |
| Input/UI/render path | `play.cmd --smoke` | Keyboard Enter spawn and Z undo, spawn-menu Interaction, F5 save, F9 reset, F6 load all hit expected prop counts in actual windowed app |
| Rendering | Bevy screenshot captured at frame 150, app exits after frame 240 | 1280x800 image shows grid, lit props, shadows, spreadsheet labels, controls and HUD |
| Parser safety | VPK v1/v2 fixtures, every byte-prefix truncation, excessive tree length, path and formula-text checks | Passed |
| Authored-data safety | Duplicate IDs, broken references, invalid size/mass, NaN, escaping, deterministic generation | Passed |
| Real inventory | Catalog CLI on installed game root | 2,904 loose files plus 78,051 VPK entries in nine archives |
| Inventory cross-check | Independent PowerShell recursive file count | 2,904 files, matching loose-file inventory |
| Tool coverage | Compare installed stock tool IDs with tools.csv IDs after prefix normalization | No differences across 37 entries |
| Workbook export | Actual `workbook` CLI | 10 tabs and 86,918 data rows, 5,051,142 bytes |
| Workbook independent check | PowerShell ZIP/XML streaming parser compares every sheet with source CSV | All sheet row counts match, no formula elements |
| Strict lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed after documenting idiomatic Bevy system parameter count |

`cargo test --workspace --all-features`: **25 tests passed**, zero failed (5 sandbox unit tests, 4 Bevy workflow tests, 16 catalog tests). Doc-test targets contain zero examples and are not counted as behavioral coverage.

The grab/freeze integration test was also rerun **five additional times**, with all five passing. This establishes repeatability for that prototype fixture, not Source physics parity.

## Graphical evidence

Local screenshot: `evidence/bevy-smoke.png`. It is ignored in Git because it is a generated artifact, but present on this machine. The final graphics harness uses Bevy input resources and UI Interaction to exercise the real control systems, not OS mouse/keyboard automation. It creates a fresh scene save under `local/saves/`. This verifies application bindings and rendering, but not every physical input-device behavior or long interactive session.

The first Bevy compilation took roughly four minutes for executable/test code generation. A background stall warning occurred while rustc processes were active. It was inspected rather than treated as a failure. The task completed successfully. Compile timing is not a gameplay performance benchmark.

## Workbook composition

| Tab | Data rows |
|---|---:|
| milestones | 12 |
| props | 4 |
| scene | 5 |
| sources | 11 |
| systems | 64 |
| tools | 37 |
| world | 1 |
| assets | 80,955 |
| lua_symbols | 5,792 |
| stock_tools | 37 |

The workbook was structurally and quantitatively verified, not manually opened in Microsoft Excel. It is a generated review snapshot. CSV remains the runtime authoring source.

## Boundaries not verified or delivered

- No direct GMod-versus-Bevy reference trajectory, screenshot, audio, movement or Lua conformance comparison. **No one-to-one parity claim is supported yet.**
- No native GMod engine implementation, complete Lua API reconstruction, model/map conversion, VPK payload CRC validation, addon execution, GMA internals or BSP embedded pak enumeration.
- No multiplayer, Steam/Workshop service integration, NPCs, ragdolls, vehicles, stock weapons or full constraints.
- No performance characterization at the configured 500-prop cap, release build measurements, alternate GPUs or operating systems.
- No full XLSX import. Export edited tabs back to CSV before building.
- Source paths baked into the development binary support this checkout. Relocatable packaged save/config paths are future packaging work.
- Undo restores complete scene snapshots, not GMod's full per-player undo semantics. Grab uses a velocity-following controller and simple rotation, not Source physgun behavior.
- The reference list is a high-level initial stock-systems baseline. Complete behavioral specifications require the planned M2 reference corpus.
