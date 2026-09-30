# Validation record

## Original-data poser integration, 2026-09-30 05:02 UTC

- Reviewed all 19 private reports. Added four partial poser routes with nine authored settings, per-instance meshes/materials and version-6 scene state. Current inventory is 35 partial tool entries including custom Freeze and three reference-only entries. The 178 options, 52 tool cases and 29 F7 cards were counted from the production CSV files. All manual results remain `not_run`.
- The first actual executable build rejected an invalid option kind (`toggle`). Corrected it to the catalog's supported `bool` type. The repaired game compiled in 58.53 seconds. Final review added active-flex accumulation, supported-descriptor/eye validation, aggregate mesh bounds, iris metadata panic containment, pose hull preflight, material modifier preservation and cancellation on weapon switching. These are code-inspection findings, not exercised gameplay cases.
- Final actual executable compilation succeeded in **53.20 seconds** using private `CARGO_TARGET_DIR=D:\jcode-build\rust-sandbox-20260929` and `cargo rustc --offline --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-posers.exe -C link-arg=/DEBUG:NONE -C link-arg=/INCREMENTAL:NO`. Read `local/posers-build.log`: only the output-name/out-dir warnings and existing binrw future-compatibility notice remain. The resulting executable is 72,082,432 bytes. Subsequent Rust edit only restored unrelated formatting in a pre-existing cfg(test) block.
- Production catalog validation succeeded: 197 spawn references, eight creation tabs, 56 capabilities, player/sandbox configuration, two map references, four prop definitions, five scene instances and 101 behavior specifications. Production workbook export wrote **107,693 data rows** to `spreadsheets/rust-sandbox-catalog-20260930-posers.xlsx`. CSV is the authoring source and XLSX is a review snapshot. Workbook application acceptance was not run.
- No tests, test compilation, Clippy, benchmarks, screenshots, automated input, model compatibility execution or gameplay acceptance were run. No game was terminated or save overwritten. Process inspection at 05:02 UTC found no active source-map process. The new executable has not been launched in this pass.
- Unverified boundaries include raw MDL compatibility, eye texture projection, finger-axis calibration, grouped scale inheritance, collision hull behavior, supported-material restoration, scene undo/duplication and legacy loads. Native ragdolls, facial controller rules, automatic facial animation, alternate skeleton maps, eye shaders, animated NPC posing, sounds and the full parity backlog remain incomplete. A successful build does not establish any of those behaviors.


## Construction tools integration, 2026-09-30 04:42 UTC

- Reviewed all 38 tool rows and 18 private reports. Added 18 independent runtime subsets, leaving 31 partial rows including custom Freeze, four missing posers and three reference-only entries. Added 21 manual cases for a total of 45, and four F7 cards for a total of 26. All gameplay acceptance remains Drew-owned and not run.
- The first actual executable build failed because a private Remote resource type appeared in the public player-input system signature used by the binary. Corrected the visibility. Static review also moved toggle edges out of FixedUpdate, isolated remote-view physgun and vehicle entry, rejected unsafe wheel-axis edits, and initialized restored powered joints from saved target length/force limits. These are inspection-backed fixes, not exercised gameplay tests.
- The repaired actual game compiled in 50.33 seconds. After final sheet and restore changes, the final actual game compiled in **49.40 seconds**. Command: `cargo rustc --offline --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-construction-tools.exe -C link-arg=/DEBUG:NONE -C link-arg=/INCREMENTAL:NO`, with private `CARGO_TARGET_DIR=D:\jcode-build\rust-sandbox-20260929`. Read both build logs. Only explicit output-name/out-dir warnings and the existing binrw future-compatibility notice remained.
- Production `sandbox-catalog --features workbook -- validate sheets` succeeded, including player/sandbox configuration, 197 spawn references, eight tabs, 56 capabilities, two maps and 101 base behavior specifications. The production workbook exporter wrote **107673 data rows** to `spreadsheets/rust-sandbox-catalog-20260930-construction-tools.xlsx`. CSV remains authoritative. Excel was not manually opened.
- Read-only installed metadata confirms all ten construction-model paths and VVD/VTX companions, plus the six paint and two trail VMT paths. Successful decode, model alignment, button/keypad behavior, light appearance, device forces, restore/undo/duplication and bounded rendering have not been accepted in gameplay.
- No tests, test compilation, Clippy, benchmarks, screenshots or input automation were run. No game process was terminated and no saves or private feedback were modified. No `source-map*` process was present at the final process inspection. The new executable was not launched during this integration.
- Four poser tools, full native settings, sounds, projected decals/lamps, native pulley mechanics, force breaking and broader Source parity remain unfinished. A passing build and catalog export do not close these gaps.

## Complete feedback review: flight, aim and recoil, 2026-09-30 03:40 UTC

- Reviewed all 18 local reports, including legacy observed/expected/notes fields and full latest context. Private originals remain unchanged and unpublished. Added an 18-row engineering review, eight required handling definitions, and 20 Drew-owned manual cases. All new acceptance results remain `not_run`.
- First actual game build succeeded in **1m07s**. Final static review found that resetting all handling at immediate auto-reload would erase a one-round weapon's recoil on its next frame. Changed reload to cancel aim while preserving natural punch/kick decay, and rebuilt the actual executable successfully in **1m02s**. This is inspection-backed correction, not an exercised gameplay regression test.
- Final command: `cargo rustc --offline --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-flight-aim.exe -C link-arg=/DEBUG:NONE -C link-arg=/INCREMENTAL:NO`, with private `CARGO_TARGET_DIR=D:\jcode-build\rust-sandbox-20260929`. Reviewed local/flight-aim-final-build.log. Only the two explicit output-path warnings and existing binrw 0.14.2 future-compatibility notice remained.
- Production catalog validation succeeded through the new required handling loader: 197 spawn references, 8 tabs, 56 capabilities, player/sandbox configuration, 2 map references and 101 base behavior specifications. Regenerated 134 per-registration coverage rows. No gameplay acceptance was performed by either operation.
- Final combined command exited 1 only because the workbook exporter correctly refused an existing output file after the build and catalog validation had succeeded. Preserved this turn's intermediate snapshot under ignored local storage, then exported the revised workbook successfully with **107,518 data rows** to spreadsheets/rust-sandbox-catalog-20260930-flight-aim.xlsx. Logs: local/flight-aim-final-catalog.log and local/flight-aim-final-workbook.log. Previously published workbooks remain untouched. Export is not independent spreadsheet-app acceptance.
- Reviewed input/focus/menu latches, finite tuning/ID coverage, world-up and normalized flight direction, actual-shot-only recoil, once-per-shot versus pellet placement, bounded recovery, current-eye firing, single world-FOV ownership, shared weapon/hands transforms, restore cleanup and remaining-work wording. F7 contains 22 cards. No native behavior or visual alignment is marked verified.
- At 03:38 UTC the older feedback-repairs process PID 21632 was responding. It was preserved. The new executable has not been launched. No tests, test compilation, Clippy, benchmarks, screenshots or input automation were run. Actual gun sight calibration, recoil feel, flight ratios, all lifecycle/input boundaries and broader parity gaps remain Drew-owned checks. `git diff --check` passed before recording this entry.

## Feedback regression repairs, 2026-09-30 03:01 UTC

- Reviewed seven newer private reports. Preserved originals without publishing their bodies or filenames. Ten authored follow-up rows capture repaired subsets, dependencies and remaining work, all with Drew-owned `not_run` acceptance.
- Actual executable compilation succeeded in **1m00s** using private `CARGO_TARGET_DIR=D:\jcode-build\rust-sandbox-20260929`: `cargo rustc --offline --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-feedback-repairs.exe -C link-arg=/DEBUG:NONE -C link-arg=/INCREMENTAL:NO`. Reviewed local/feedback-repairs-build.log. Remaining diagnostics were the two explicit-output warnings and existing binrw 0.14.2 future compatibility notice.
- Production catalog validation succeeded: 197 spawn references, 8 creation tabs, 56 capabilities, player/sandbox configuration, 2 map references and 101 base behavior specifications. Workbook export succeeded with **107,470 data rows** to spreadsheets/rust-sandbox-catalog-20260930-feedback-repairs.xlsx. Reviewed local/feedback-repairs-catalog.log. Export is not independent spreadsheet-app acceptance.
- Reviewed the wheel basis against the shared converter, crouch friction arithmetic, held-anchor rotation and input guards, named bone merging with attachment fallback, and bounded centered spread sampling. These inspections and successful compilation do not prove visible gameplay correctness or native equivalence.
- F7 now contains 20 cards, including repair checks and unfinished scope. All-tools coverage, native secondary/aim behavior, audio, general heavy-prop impact feedback and articulated ragdolls remain incomplete. No acceptance status was promoted to verified.
- No tests, test compilation, Clippy, benchmarks, screenshots or input automation were run. The new executable was not launched in this pass, and no active game was terminated. Gameplay acceptance remains with Drew. `git diff --check` passed before this documentation update.

## Motion and impact feedback integration, 2026-09-30 02:22 UTC

- Reviewed four additional private reports and original crouch/wheel/material metadata. Public source findings are in RESEARCH.md. Reports and original asset payloads remain private/read-only.
- The first actual executable compilation failed with E0603 because the shared standing-hull clearance helper was private. Exposed it crate-locally and rebuilt successfully. Reviewed appearance restoration and excluded impact meshes from prop material tools before the final build.
- Final `cargo rustc --offline --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-impact-crouch.exe -C link-arg=/DEBUG:NONE -C link-arg=/INCREMENTAL:NO` succeeded in **46.31 seconds**, using the private D: target. Remaining warnings concern the explicit output filename/out-dir combination and existing `binrw 0.14.2` future compatibility. No tests or test compilation were run.
- Production `cargo run --offline --locked -p sandbox-catalog --features workbook -- validate sheets` succeeded: 197 spawn references, 8 creation tabs, 56 capabilities, player/sandbox configuration, 2 map references and the base 101 behavior specifications.
- Regenerated 134 per-registration coverage rows through `scripts/export-gameplay-coverage.ps1`. Production workbook export succeeded with **107,457 data rows** to `spreadsheets/rust-sandbox-catalog-20260930-impact-crouch.xlsx`. Earlier workbooks were preserved. Export is not independent Excel acceptance or gameplay testing.
- New workbook content includes 12 wheel rows, 19 manual scenarios, 15 total feedback priorities and 17 F7 cards. All manual results remain Drew-owned `not_run`. Root gaps stay partial/missing as appropriate, not verified.
- Normal no-argument frontend launch was requested only after finding no active game. PID 36456 was observed responding and the log reported 13 original menu images mounted read-only. No input, screenshot, test harness, benchmark or gameplay automation was performed. No game was terminated. Startup does not establish crouch, wheel animation, marks, collision effects or HUD visual correctness.
- `git diff --check` passed. Live gameplay, actual clip decoding, low-ceiling/jump timing, wheel axes, decal rendering, contact thresholds, saved appearance, DPI/focus and performance still need the [19 manual cases](../sheets/motion_impact_cases.csv). Native Source/GMod equivalence remains unproven.


## Single-note feedback and presentation review, 2026-09-30 01:55 UTC

- Reviewed all four private feedback reports without modifying or publishing them. Added nine prioritized public engineering rows and twenty Drew-owned manual cases. All new acceptance results remain `not_run`. F7 cards now describe the single-note editor, weapon clip revisions and categorized mounted icons.
- First actual build compiled Rust but failed at linking with LNK1318 PDB LIMIT (12), recorded in local/feedback-native-build.log. Rebuilt the final executable without the optional oversized debug-symbol output: `cargo rustc --offline --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-feedback-native.exe -C link-arg=/DEBUG:NONE -C link-arg=/INCREMENTAL:NO`. Private CARGO_TARGET_DIR remained on D:. Final build passed in **1m18s**, with only the two expected separate-output warnings and existing binrw future-incompatibility notice. Log: local/feedback-native-final-build.log. This delivery executable does not carry a PDB.
- Production `sandbox-catalog validate sheets` passed: 197 spawn references, eight creation tabs, 56 capabilities, two map references and 101 behavior specifications. It consumed the new required draw column and enabled-combat idle/fire/draw validation. Full workbook export produced **107,416 data rows** at spreadsheets/rust-sandbox-catalog-20260930-feedback-native.xlsx. Logs: local/feedback-native-sheets.log and local/feedback-native-workbook.log. Prior workbooks were retained. Export is not independent spreadsheet-app acceptance.
- Static review covered legacy body migration, local context retention, main-thread HWND lifetime, system-only DLL loading, explicit TSF opt-in, mirror-before-save ordering, hide/focus handling, literal sheet animation mappings, failed weapon load rollback, and icon fallback/context tags. Windows bindings reuse cached versions, adding only two direct Cargo.lock dependency edges.
- No game was active at the prelaunch check. The new executable was launched normally without input automation. At 01:55 UTC PID **52344** was responding; startup logs showed read-only mounts and `MENU_ASSETS images=13`, with no panic in the inspected output. Logs: local/feedback-native-launch.log and local/feedback-native-launch.err.log. No F8 or Win+H interaction was performed. Native field creation, dictation, text editing, child-window rendering, DPI/focus, persistence and gameplay remain **unaccepted**. No tests, test compilation, Clippy, benchmarks, screenshots or input automation were run. No active session was terminated.


## What's New checklist and feedback typing, 2026-09-30 01:07 UTC

- Reviewed the existing private feedback inbox without modifying or uploading reports. Source inspection identified missing `Key::Space` handling in feedback, map search and Q-menu search. All three now preserve named spaces within existing byte limits. Frontend typing rejects unfocused input; frontend processing is explicitly ordered before spawn-menu search and player input. Additional static review repaired automatic-weapon held-click leakage across UI close using the existing equip-release latch.
- Added 13 sheet-authored release cards with changes, concrete player checks, expectations and explicit limitations. F7 and visible startup/pause/HUD buttons open the checklist. Previous-screen return, card-specific contextual feedback, revision-keyed private checkmarks and corrupt-file preservation are implemented. They have not been interaction-tested. No personal checkmark alters source acceptance status.
- Final actual executable build succeeded in **1m03s**: `cargo rustc --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-whats-new.exe`. Log: local/whats-new-final-build.log. The preceding integrated build also passed. Only the two expected separate-output-path warnings and existing binrw future-incompatibility notice were reported.
- Production `sandbox-catalog validate sheets` passed through the updated frontend loader, alongside 197 spawn references, eight creation tabs, 56 capabilities and 101 behavior specifications. Log: local/whats-new-sheets.log. Workbook export succeeded with **107,387 data rows**, including 13 release notes and 14 new acceptance cases, at `spreadsheets/rust-sandbox-catalog-20260930-whats-new.xlsx`. Log: local/whats-new-workbook.log. CSV remains authoritative; export is not independent workbook acceptance.
- Static review covered named/character space exclusivity, UTF-8-safe Backspace, limits, focus and input ordering, menu pause/return state, held mouse release, generated data loading, revision keys, failed-save rollback and retained feedback context. All 14 new cases in release_checklist_cases.csv remain **not_run**, owned by Drew. Actual typing, rendering/scrolling, persistence/restart, animation and gameplay acceptance were not run. This package does not establish that all broader gameplay gaps work or are complete.
- At 01:06 UTC the prior `source-map-npc-lifecycle.exe` remained responding as PID 42772. It was not terminated, overwritten or restarted. The separate new executable is built but not launched. No tests or test compilation, Clippy, benchmarks, screenshots or input automation were performed. Saves, original content, authentication and private feedback were preserved.

## Typed NPC lifecycle and NoCollide filtering, 2026-09-30 00:49 UTC

- Final actual executable build succeeded in 1m07s: `cargo rustc --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-npc-lifecycle.exe`. Log: local/npc-lifecycle-final-build.log. The preceding integrated build also passed. Only the two deliberate explicit-output-path warnings and existing binrw future-incompatibility notice were reported.
- Production `sandbox-catalog validate sheets` passed, reporting 197 spawn references, eight creation tabs, 56 capabilities and 101 behavior specifications. New NPC definitions and rules pass the actual catalog loading/generation path. Log: local/npc-lifecycle-sheets.log.
- Workbook export succeeded with **107,360 data rows** at `spreadsheets/rust-sandbox-catalog-20260930-npc-lifecycle.xlsx`. Log: local/npc-lifecycle-workbook.log. Derived registration coverage contains 134 rows, including all 85 NPC registrations. CSV remains authoritative and earlier workbooks are preserved. Export success is not independent workbook acceptance.
- Read-only original model metadata research initially failed in vmdl external animation-block parsing. The metadata inspector was corrected to bypass animation decoding, matching the existing runtime metadata approach, and subsequent extraction succeeded. Names and included libraries were observed, but actual runtime clip decoding, posing and visual alignment were not exercised. See RESEARCH.md.
- Static review covered sheet validation, six typed actor routes, required-clip rejection, blocked placement, collision/controller masks, AI toggles, damage/death, paused or blocked respawn, scene version 4 preflight, undo ordering and unsupported-tool rejection. Player NoCollide repair follows the original tool behavior and Rapier controller query semantics, not an instrumented gameplay reproduction.
- This is a partial integration, not closure of the full parity backlog. Six NPC registrations have prototype routes and 79 remain disabled. Native schedules/navigation, equipment, sounds, hitgroups, ragdolls and class-specific behaviors remain open. Player health/respawn is local and incomplete relative to native player lifecycle. All 21 new gameplay cases and all registration acceptance rows remain not_run and Drew-owned.
- No tests or test compilation, Clippy, benchmarks, screenshots or gameplay/input automation were performed. No game was launched or terminated in this pass. Saves, private feedback, Steam content and earlier executables were preserved. Compilation is not gameplay acceptance.

## Vehicle orientation, model categories and water, 2026-09-29 23:19 UTC

- Read-only installed-content research extracted attachment frames for all 15 registered vehicles, 8,390 model/category memberships from 43 stock spawn lists, and original above/below-water material parameters. Metadata extraction is not gameplay acceptance. Research sources and approximation boundaries are recorded in RESEARCH.md and VEHICLE_WATER_CATEGORY_PLAN.md.
- Final actual executable build succeeded in 1m53s: `cargo rustc --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-vehicle-water-repair.exe`. Log: local/vehicle-water-repair-final.log. The preceding integrated build also passed. Only the two deliberate explicit-output-path warnings and existing binrw future-incompatibility notice were reported.
- Production `sandbox-catalog validate sheets` passed, including generated vehicle frames, model categories and water configuration. It reported 197 spawn references, eight creation tabs, 56 capabilities and 101 behavior specifications. Log: local/vehicle-water-sheet-validation.log.
- Workbook export succeeded with **107,168 data rows** at `spreadsheets/rust-sandbox-catalog-20260929-vehicle-water-repair.xlsx`. Log: local/vehicle-water-workbook.log. CSV remains authoritative, earlier snapshots are preserved, and export success is not independent workbook acceptance.
- Static review covered spawn orientation, steering and suspension axes, seat/camera configuration, mounted-model category intersection and parent filtering, parsed-water collision exclusion, original linear normal maps and single-boundary rendering. Native chassis parameters, entry cinematics, exact seat pose alignment, native menu interaction parity, planar water reflections, animated VTF frames and underwater optics remain incomplete or unverified.
- No tests or test compilation, Clippy, benchmarks, screenshots or gameplay automation were performed. All 14 new acceptance cases remain not_run and Drew-owned. Compilation does not establish that the reported visual or handling defects pass gameplay acceptance.
- The prior game was observed responding as PID 48788 during this pass. No game was terminated or replaced, saves and private feedback were preserved, and this separate new executable has not been launched.

## Playable weapons and typed vehicles, 2026-09-29 22:40 UTC

- Published the preceding nine local commits to origin/master through 143dd5c immediately on request. This section records the subsequent gameplay integration separately.
- First integrated build failed with two compiler errors: RapierContext lacks singular intersection_with_shape, and the asset-validation caller still passed a bool after weapon creation changed to an explicit ID. Corrected both. Repaired executable build succeeded, followed by final input/projectile safety changes.
- Final actual executable build succeeded in 52.78s: `cargo rustc --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-playable-entities.exe`. Log: local/build-playable-entities-final.log. Only the two deliberate explicit-output-path warnings and existing binrw future-incompatibility notice were reported.
- Production `sandbox-catalog validate sheets` passed. It loads the new 34 weapon, 15 vehicle and shared tuning rows through the actual generation path, with the existing 197 references/eight tabs/56 capabilities. Coverage export generated 49 rows. Workbook export succeeded with 98,763 data rows at `spreadsheets/rust-sandbox-catalog-20260929-playable-entities.xlsx`. Earlier snapshots remain intact. Export success is not independent workbook acceptance.
- Static review covered explicit equip/spawn routing, post-menu click blocking, projectile budget before ammo consumption, resting grenade contacts, fixed-step focus/UI gates, camera/held-weapon occupancy behavior, safe endpoint checks and versioned scene identity. git diff --check passed. Original model inventory paths were checked, but actual mounted weapon clip decoding and seat alignment were not exercised.
- No tests or test compilation, Clippy, benchmarks, screenshots, gameplay automation or native reference capture were run. No game was launched or terminated in this integration pass. Saves and private feedback were not changed. All 49 per-registration acceptance rows remain not_run and Drew-owned.
- This is a partial integration: 13 weapon routes include the existing physgun/toolgun plus 11 primary-attack prototypes. Fifteen vehicle registrations include four powered prototypes and eleven passive seats. Native driving parameters, secondary attacks, sounds, 21 disabled weapons, NPC AI and exact entry/exit cinematics are not implemented. Generic seated pose interpolation is not native entry animation. Vehicle drive feel, hull clearance, scene restore, assets and all menu/visual parity still require real gameplay acceptance.

## Entity/menu source-of-truth integration, 2026-09-29

- Read-only reference extraction completed: 197 spawn definitions, 1,406 literal UI rows and 178 hashed files. An attempted property scan initially used a nonexistent directory; corrected to installed `lua/autorun/properties` and regenerated successfully. This is metadata generation, not Lua or gameplay execution.
- Production `sandbox-catalog validate sheets` passed with 197 references, eight creation tabs and 56 dependency-linked capabilities, alongside existing sheet validation. No test binary was compiled or run.
- Workbook export succeeded: `spreadsheets/rust-sandbox-catalog-20260929-entity-source-of-truth.xlsx`, 98,664 data rows. CSV remains the editable source of truth. All 14 new interaction cases remain `not_run`.
- Final actual game build: `cargo rustc --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-entity-menu.exe`, exit0 in1m22s. Artifact inspected: 68,761,088 bytes. Compiler reported the two expected explicit-output-path warnings and existing binrw future-incompatibility notice, no project code warnings. The preceding integration build also passed, then an obsolete action variant and scroll-reset handling were corrected before this final build.
- No tests, Clippy, benchmarks, screenshots, gameplay input automation or native reference capture were performed. Runtime search/selection/feedback, small-window layout and source parity are unverified. No active game was terminated, saves and private feedback were preserved. Full cars/NPC/entity simulation remains missing as documented in the sheets and [ENTITY_MENU_SOURCE_OF_TRUTH.md](ENTITY_MENU_SOURCE_OF_TRUTH.md).

Date: 2026-09-29 UTC. Windows x86_64, Rust 1.97.1. Build artifacts isolated in `D:\jcode-build\rust-sandbox-20260929`. No remote compilation or remote asset upload.

## Running drift repair, 2026-09-29 20:32 UTC

- Drew reported drifting in the preceding build. Source review found generic SDK friction 4 instead of the installed GMod preset 8, plus displacement-to-velocity feedback and grounded gravity/low-speed stopping differences. The implementation and remaining native boundaries are documented in MOVEMENT_REPAIR.md and 20 movement_reference rows. This is a source-backed diagnosis, not an instrumented reproduction of Drew's exact trajectory.
- Final `cargo rustc --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-movement-repair.exe` succeeded in **1m44s** after research-sheet changes. The preceding source build succeeded in 1m37s. Logs: `local/movement-repair-final-build.log` and `local/movement-repair-build.log`. Expected separate-output rustc warnings and the existing binrw future-compatibility notice remain. No application-source compile error was reported.
- The exporter compiled in **10.90s** and produced `spreadsheets/rust-sandbox-catalog-20260929-movement-repair.xlsx` with **96,805 data rows**. Log: `local/movement-repair-workbook.log`. Export success is not independent workbook acceptance. Prior snapshots remain intact.
- At 20:32 UTC the prior game `source-map-context.exe`, PID **48604**, was still responding. It was not terminated, overwritten or replaced. The new separate executable is built but **not launched**. After Drew closes the old game, launch `D:\jcode-build\rust-sandbox-20260929\source-map-movement-repair.exe` or use normal `play.cmd`. No automatic restart job was created.
- Static review checked the production pre/post-physics order, ground/air branches, clipping normals, stair-face exclusion, persistent velocity ownership, noclip reset and generated configuration. `git diff --check` reported no whitespace errors. Two regression tests were added and the existing headless harness updated to include both production phases, but tests were **not run or test-compiled**. No Clippy, benchmarks, screenshots, automated gameplay or workbook verification ran. All movement acceptance rows remain **not_run**, owned by Drew, including stopping/turning feel, ramps/stairs/corners, ceiling impacts and repeated frame-rate comparisons.

## Contextual feedback, cursor and movement, 2026-09-29 20:05 UTC

- `cargo build --locked -p rust-sandbox --lib` succeeded in 1m37s. Following final control-tagging and formatting changes, `cargo rustc --locked -p rust-sandbox --bin source-map -- -o D:\jcode-build\rust-sandbox-20260929\source-map-context.exe` succeeded in **1m50s**. Logs: `local/context-movement-build.log` and `local/context-movement-final-build.log`. The final separate executable exists at the specified path (68,509,184 bytes). Rustc reported expected output-name adaptation/out-dir warnings for the deliberate separate output, plus the existing binrw future-compatibility notice. No application-source compiler warning was reported.
- The separately compiled executable was **not launched** while the prior game remained active. PID 37840 at `debug/source-map.exe` was responding during inspection. No live process was terminated and no active scene was replaced. After Drew closes the prior game, he can run the separate `source-map-context.exe` directly, or use the normal repository `play.cmd` to build/run the current source. The active game does not hot-reload these Rust changes.
- The catalog exporter compiled in 8.97s and exported **96,795 data rows** to `spreadsheets/rust-sandbox-catalog-20260929-context-movement.xlsx`. Log: `local/context-movement-workbook.log`. Earlier workbooks remain intact. Export success is not independent workbook acceptance.
- Static diff review covered cursor ownership, frontend input blocking/return, original-context capture, legacy draft fallback, local report preservation, sheet-to-generated movement configuration and collision-resolved planar velocity. `git diff --check` reported no whitespace errors. No local feedback reports existed to process. No tests, Clippy, benchmarks, gameplay input automation, screenshots or independent workbook verification were run, per project instructions.
- All 12 movement-reference and 18 contextual-feedback/cursor acceptance cases remain **not_run**, owned by Drew. Native command timing, collision/hull/step/surf behavior, crouch/water/ladder/platform movement, all menu visual equivalence and the broader existing parity backlog remain incomplete or unverified. Compilation is not a claim that these player-visible workflows pass.

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
