# Validation record

Date: 2026-09-29 UTC. Windows x86_64, Rust 1.97.1. Build artifacts isolated in `D:\jcode-build\rust-sandbox-20260929`. No remote compilation or remote asset upload.

## Reference-menu repair and expanded inventory, 2026-09-29 19:28 UTC

- Final `cargo build --locked -p rust-sandbox --bin source-map` succeeded in **1m30s**. The preceding build succeeded in 1m43s. Logs: `local/menu-parity-final-build.log` and `local/menu-parity-build.log`. The pre-existing binrw future-compatibility warning remains. No test, benchmark or Clippy run was performed.
- The catalog exporter compiled in 9.16s and produced `spreadsheets/rust-sandbox-catalog-20260929-menu-parity.xlsx` with **96,763 data rows**, including the recursively expanded 368 HTML control/label and 1138 CSS declaration inventories, 140 native controls, 30 new-game settings, 46 menu reference rules, 40 menu work packages and updated frontend cases. Log: `local/menu-parity-workbook.log`. This is exporter success, not independent XLSX acceptance.
- Launched the final executable normally without any automated inputs or screenshots. `local/menu-parity-launch.log` reported **MENU_ASSETS images=13**, with read-only source mounts. `local/menu-parity-launch.err.log` was empty at inspection. At 19:44 UTC the running binary had PID 37840 and a responding `gm_construct | Bevy local Source map` window. No active game was terminated or replaced by automation. Window/process availability is not a visual or interaction parity test.
- Static review corrected label/icon hit blocking, preserved one action per fresh press, added responsive footer labels and original intrinsic PNG sizes, anchored page-preserving popups with isolated dismissal, favorite hover states, alphabetic map order, search autofocus, and clearing stale double-click history after unrelated actions. `git diff --check` reported no whitespace errors. These checks do not prove actual button behavior.
- Drew retains all interaction/visual acceptance. Every acceptance row remains `not_run`. Full native Options, new-game settings effects, networking, Workshop, other mode/language runtime, full Q/C interfaces, Chromium typography and complete one-to-one menu parity remain open in `menu_work_items.csv` and `MENU_REPAIR_PLAN.md`.

## Spawn clearance and frontend awaiting Drew's testing

`cargo build --locked -p rust-sandbox --bin source-map` succeeded in 56.70s after the final source changes. Log: `local/frontend-executable-build.log`. The first library build caught a JSON macro expression error in feedback category selection, fixed before successful library and executable builds. Static review added degenerate-normal fallback, F10 confirmation routing, page-scroll reset and the spreadsheet-authored map-settings column. These are implementation/build observations, not passing gameplay checks. The existing binrw future-compatibility warning remains.

The normal no-argument executable was launched without automated input. PID 8492 reported a responding `Rust Workshop | Main Menu` window. Logs `local/frontend-launch-20260929-133548.log` and `.err.log` were empty at inspection. No prior running source-map process was present and none was terminated. The startup window existing does not verify map handoff, rendered appearance, prop placement or feedback persistence. Drew owns those acceptance cases.

The catalog exporter build succeeded and produced `spreadsheets/rust-sandbox-catalog-20260929-frontend.xlsx` with **94,981 data rows**, including source_frontend and 38 frontend_cases rows. Earlier workbooks remain. No independent workbook verification, tests, Clippy, benchmarks, screenshots or gameplay automation were run. Full one-to-one menus/settings and the wider parity backlog remain unfinished.

The previous toolgun handoff subsequently completed normally and launched PID 44640 after Drew closed PID 50072. That historical launch supersedes the pending status recorded below, without constituting tool acceptance.

## Toolgun and staged tools awaiting Drew's testing

The current pass implements the original toolgun fire-clip path, selected-title screen render target, authored tool controls, twelve stock-tool subsets, supported Rapier constraints, prop modifiers, connected duplication and version-2 local scene state. Full tools/settings, original effects/audio and one-to-one parity remain unfinished. The exact scope and acceptance scenarios are in TOOL_IMPLEMENTATION_PLAN.md and tool_cases.csv.

`cargo build --locked -p rust-sandbox --lib` succeeded after the final source changes (47.08s). The earlier compile found that ImpulseJoint requires TypedJoint::GenericJoint rather than an implicit GenericJoint conversion. That mismatch was corrected. Static review also corrected missing world-parent visibility for cable children, duplicate joint limits, failure cleanup for weapon actors, repeated held UI clicks and menu-closing click leakage. These are code inspections, not passing runtime tests. Final log: `local/toolgun-build-final.log`. The existing binrw future-compatibility warning remains.

`cargo build --locked -p sandbox-catalog --features workbook` succeeded (37.48s), and the actual exporter produced `spreadsheets/rust-sandbox-catalog-20260929-toolgun.xlsx` with **94,942 data rows**. All prior workbooks remain. No independent workbook verification was run.

At handoff the prior game, PID 50072, was still open and responding. It was not terminated. A one-shot local job waits for Drew to close it, then builds `source-map` and launches it normally with separate timestamped stdout/stderr logs. At this recording the executable build and new launch are **pending**, not claimed successful. The queued script is ignored local state, not a recurring supervisor. No automated tests, Clippy, benchmarks, gameplay input or screenshots were run. Drew owns all functional, visual, persistence and reference acceptance.

## Spawn performance implementation awaiting Drew's testing

The reported spawn lag prompted dependency optimization, a bounded off-thread preparation queue, shared immutable cached geometry, exact duplicate hull-input removal, and reduced idle-frame change processing. Settings come from `source_performance.csv`. `performance_cases.csv` lists implemented and remaining scenarios without passing results.

`cargo build --locked -p rust-sandbox --bin source-map` succeeded. The initial full optimized dependency build exceeded the tool's ten-minute limit and was resumed with cached artifacts. The resumed compile exposed an ambiguous `Real` import from Bevy/Rapier, corrected to `bevy::time::Real` before the successful build. Final output: `local/performance-build-final.log`. Cargo's overall dev profile label remains unoptimized, while explicit package overrides optimize dependencies at level 3 and rust-sandbox at level 2. Existing binrw future-compatibility warning remains.

The existing workbook exporter produced `spreadsheets/rust-sandbox-catalog-20260929-performance.xlsx` with **94,663 data rows**, including both new sheets and the complete existing catalogs. No independent workbook test was run. Earlier snapshots were retained.

The newly built `source-map.exe` was launched normally, with no smoke flag or automated inputs. Process 50072 reported a responding `gm_construct | Bevy local Source map` window. Startup logged the selected player clips and both physgun textures. Logs: `local/performance-launch.log` and `local/performance-launch.err.log`. Passive frame diagnostics start during normal play, but startup/idle samples do not establish a before/after spawn improvement. No tests, benchmarks, Clippy, screenshots or gameplay automation were run. Drew owns performance and visual acceptance. Cold asset upload, menu opening, long sessions, cancellation behavior and complete Source equivalence remain unverified.

## Presentation corrections awaiting Drew's testing

After Drew reported incorrect leg motion/jumps, missing physgun beam and nonmatching Q layout, the implementation was updated from public-source rules recorded in `presentation_references.csv`. `source_pose` adds measured directional gait, takeoff-reset non-looping jump clips and crossfades. `source_play` separates beam firing from held targets, preserves local grab anchors and uses Rapier world traces. `source_menu` consumes compiled dimensions for the browser, 64px icons and tool regions, with hold-Q and focused search.

`cargo build --locked -p rust-sandbox --bin source-map` completed successfully. The first compile exposed private UI system parameter types, which were corrected before the successful build. Log: `local/presentation-build.log`. **No automated tests, Clippy, smoke harness, screenshots or input automation were run for this change**, as requested. Previous test counts below describe earlier commits, not this change. Runtime appearance, complete state transitions and reference equivalence are left to Drew. Full stock UI, animation layers/IK, original physics and the remaining backlog are not claimed complete.

The normal `play.cmd` was launched for Drew, without `--smoke`. Startup output reported the window ready, all 36 body clips loaded (including both 56-frame jump clips and all eight walk/run directions per hold), and both effect textures loaded. Log: `local/presentation-launch.log`. This is startup evidence only, not an automated gameplay or visual test.

## Testing handoff, 2026-09-29

Drew requested to perform all further testing. Automated testing was stopped and no additional test runs should be started without his request.

Before that instruction, the effects implementation passed 44 workspace tests (one installed-content test ignored by default) and strict all-target/all-feature Clippy. Tests include generated effects configuration, invalid CLI input rejection, effect resource reuse and lifecycle cleanup, beam UV orientation, FOV projection, and complete assignment of 169 gap IDs across 23 ordered work packages. Evidence: `local/effects-final-tests.log` and `local/effects-final-clippy.log`.

Earlier graphical attempts exposed a frame-dependent pickup fixture and a sideways beam texture. The fixture now spawns a bounds-centered prop and injects the real pickup input in the same update. Beam UVs now span width on U and scroll along V. A diagnostic capture showed glows and a held beam in both views before the UV fix. Final graphical acceptance of the corrected version is left to Drew, not claimed complete here. Use `play.cmd` for normal interactive testing. Exact original-game appearance, physical-device behavior and reference parity remain unverified.

The refreshed `spreadsheets/rust-sandbox-catalog-20260929-effects.xlsx` was exported and independently checked before the handoff: 19 tabs, 94,635 data rows, no formula cells. Earlier workbooks were retained. The workbook includes the effects configuration and dependency-ordered work queue. These are implementation/research records, not proof of one-to-one parity.

## Observed passing checks

### Public spreadsheet handoff, 2026-09-29

- Published inputs consist of all 12 authored CSVs and five versioned metadata catalog CSVs. The latter were copied exactly from the previously local inventories, with SHA-256 copy checks and a scoped privacy review documented in `catalogs/stock-20260929/README.md`. Original local workbooks and inventories remain untouched.
- `cargo test -p sandbox-catalog --all-features` passed, including the new gap-schema test. It checks unique IDs, required fields, statuses, priorities, known specification links, all 64 system families and all 37 tool entries, plus the specifically reported physgun/menu gaps. This checks inventory structure and family coverage, not original-game parity.
- `sandbox-catalog validate sheets` passed. Actual workbook export from only the published `sheets/` and `catalogs/stock-20260929/` directories produced `spreadsheets/rust-sandbox-catalog-20260929.xlsx` with **17 tabs and 94,611 data rows**.
- `scripts/verify-workbook.ps1` independently checked every worksheet name and row count against CSV inputs, and found no formula cells. The workbook was not manually inspected in Microsoft Excel. No runtime gameplay changes were made in this handoff.

### Grounded player and original animation follow-up

This section supersedes the earlier viewer-only limitations below. The current launcher includes the Source player and limited prop sandbox plugins.

- `cargo test --workspace --all-features`: **38 passed**, zero failed, one installed-content test ignored by default. Includes real Bevy/Rapier grounded-wall and jump/land tests, signed animation-run and coordinate-basis regressions, compiled player/play spreadsheet equality, and actual catalog CLI validation/generation tests. Invalid player paths, NaN, inconsistent hull dimensions and zero prop limits are rejected before output is written. Existing generated files are not overwritten.
- `cargo test -p rust-sandbox --test source_content installed_stock_maps -- --ignored`, with `GMOD_DIR`: **passed against both original installed maps**. Added a gm_construct white-room ceiling orientation/winding gate using original brush coordinates. The first regression run failed with `ceiling faces away from room`. The probe was then narrowed to interior triangles to exclude wall-top/degenerate border triangles sharing the same height. The corrected importer and final probe pass.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed. Cargo still reports the existing upstream `binrw 0.14.2` future-incompatibility notice, not a project lint failure.
- `source-player-check`: eight original clips decoded, including external player animation blocks. Player bind residual 0.00002670288, physgun 0.000015854836 and toolgun 0.000009536743. The acceptance binary uses the same actor/model/skinning pipeline as gameplay.
- Actual windowed `source-map --smoke` completed with `SOURCE_PLAYER_RENDER_OK`. Logged the player grounded in both views and moving in third person. Captures: `evidence/gm_construct-player-first.png`, `-player-third.png`, `-player-walk.png`, `-tool-first.png`, `-tool-third.png`, and `-interior-walls.png`. These are local ignored artifacts, not redistributed original assets.
- Visual review identified a wrong world-weapon orientation and a second BSP plane-normal flip. The attachment now uses the authored right-hand attachment matrix. Removing the duplicate normal flip restored missing building panels and the interior enclosure. Final third-person and interior images were inspected again. The color room remains flat white, not matched Source lighting/material proxies.

Evidence logs: `local/player-final-tests.log`, `player-final-clippy.log`, `player-asset-check.log`, `brush-regression-before.log`, `brush-regression-after.log`, and `player-corrected-capture*.log`. No Steam files were modified. CSV remains authoritative and existing workbooks/saves were preserved.

These checks establish a working partial implementation, **not one-to-one GMod parity**. The graphics harness sets ECS state rather than automating physical input devices. It does not verify every UI action, save/load path in the new Source prop sandbox, all directional/airborne animations, hand IK, all collision contents, camera corner cases, performance or exact reference appearance/trajectories. Older procedural sandbox tests below do not substitute for those Source-workflow checks.

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
