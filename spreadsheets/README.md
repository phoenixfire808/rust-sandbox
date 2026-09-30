# Spreadsheet handoff for contributors

The latest flight-aim snapshot includes the complete 18-report review, eight required per-gun handling rows, 20 manual cases and 22 total F7 cards. Edit `source_player.csv` for flight multipliers and `source_weapon_handling.csv` for gun FOV/sensitivity/offsets and recoil. Build-time validation rejects missing enabled-gun rows, duplicate or unknown IDs and invalid numeric ranges. Exact sights and gameplay acceptance remain Drew-owned. Older snapshot descriptions below are historical.

The feedback-repairs snapshot adds ten follow-up rows covering every new note, three new F7 cards (20 total), revised crouch/wheel check revisions and updated root gaps. It accompanies fixes to wheel basis conversion, crouch friction lock, E + mouse rotation, held-weapon bone merging and spread sampling. Audio, universal ADS, general prop-impact shake, articulated ragdolls and 22 missing tool routes remain unfinished. See [the current follow-up sheet](../sheets/feedback_followup.csv).

The impact-crouch snapshot adds 12 authored wheel attachment rows, crouch geometry/timing and clip mappings, bounded impact/crash tuning, four new F7 cards (17 total), six additional feedback priorities and 19 Drew-owned manual cases. Root gaps and per-vehicle coverage remain partial rather than falsely verified. See [the current feedback plan](../docs/FEEDBACK_REVIEW_PLAN.md). CSV remains authoritative. Native vehicle suspension/audio/damage, full decal rendering and exact movement/HUD parity remain open.

The feedback-native snapshot adds nine prioritized review rows and twenty Drew-owned manual cases, updates F7 guidance, and authors original weapon draw/animation mappings. Feedback is now one note with an embedded Windows text-services control, not an external editor or custom speech engine. Browser cards reuse original mounted icons and category grouping. See [the review plan](../docs/FEEDBACK_REVIEW_PLAN.md). Dictation, focus, visuals and gameplay remain unaccepted.

The What's New snapshot adds 13 in-game release cards and 14 manual acceptance cases. Edit `sheets/source_release_notes.csv` for F7 content and increment an item's revision when its testing expectations change. Personal checks stay private in `local/release-checklist.json` and do not modify acceptance results. The feedback and search Space-key repair is included in this source snapshot.

The NPC-lifecycle snapshot adds 85 explicit NPC runtime rows, shared actor/player rules, 21 acceptance cases and 85 additional per-registration coverage rows. Six ground actor routes are partial implementations and 79 are disabled. See [the remaining dependency plan](../docs/REMAINING_PARITY_PLAN.md). Edit `source_npcs.csv` and `source_npc_rules.csv`, regenerate coverage with the existing exporter, then validate and rebuild. All gameplay acceptance remains Drew-owned.

Download [rust-sandbox-catalog-20260930-flight-aim.xlsx](rust-sandbox-catalog-20260930-flight-aim.xlsx) for the combined Excel/LibreOffice review workbook. GitHub does not render XLSX files inline, so use **Download raw file**. All source tables are also available as CSV for review and version control.

The movement-repair snapshot exports **96,805 data rows**. It corrects generic SDK friction 4 to the installed GMod preset 8, separates persistent velocity from step/snap displacement, authors the low-speed stop/support/slope thresholds and expands the movement ledger to **20 rules and acceptance cases**. The [detailed drift investigation](../docs/MOVEMENT_REPAIR.md) records exact source functions, equations, native-engine uncertainty and unrun regression cases. The final executable compiled separately from the still-running prior game and has not been launched or gameplay-tested. All prior menu/tool/context inventories remain included. Acceptance remains Drew-owned and not_run.

The context/movement snapshot exports **96,795 data rows**, retaining the entire menu inventory and adding **12 movement reference rules**, **18 contextual-feedback/cursor acceptance cases**, two research sources and updated authored movement constants/backlog. The [implementation and user workflow](../docs/CONTEXT_MOVEMENT_PLAN.md) covers F8 over individual controls, browser/tool-panel reports, preserved drafts and original context, explicit retargeting and local-only review. Final executable compilation succeeded separately from the live game, which was not terminated. Gameplay, persistence, visual parity and independent workbook acceptance remain Drew-owned and not_run.

The menu-parity snapshot adds **368 HTML control/label definitions**, **1,138 CSS layout/style declarations**, **140 native option controls**, **30 Sandbox settings**, **46 measured menu rules**, and a **40-package screen-by-screen implementation backlog**. Export produced **96,763 data rows**. See [the menu repair plan](../docs/MENU_REPAIR_PLAN.md) and [menu work items](../sheets/menu_work_items.csv). The actual executable compiled and was launched normally. Button behavior, visual equality and workbook acceptance remain Drew-owned and untested by the agent. Full native Options and backend-dependent screens are still unfinished.

The retained frontend snapshot adds typed placement/menu dimensions, 38 detailed frontend and feedback acceptance rows, and updated menu/placement backlog status. Export produced **94,981 data rows**. See [the frontend plan](../docs/FRONTEND_PLAN.md). Startup, pause, map selection and local feedback are implemented as a partial slice, not one-to-one stock parity.

The retained toolgun snapshot adds the typed tool matrix, toolgun presentation configuration, enabled/pending settings, approximate physical materials, 170 literal installed defaults and 24 user-owned acceptance scenarios. Export produced **94,942 data rows**. See [the detailed tool plan](../docs/TOOL_IMPLEMENTATION_PLAN.md). Twelve stock-tool subsets are implemented, not 37 complete tools. Gameplay and workbook acceptance remain with Drew.

The retained performance snapshot adds `source_performance.csv` (typed queue and diagnostic settings) and `performance_cases.csv` (implementation versus user acceptance), and prioritizes spawn lag in the full backlog. Export produced 94,663 data rows. See [the performance plan](../docs/PERFORMANCE_PLAN.md). No measured speedup is claimed.

The retained presentation snapshot added `source_layout.csv` (runtime placement and timing), `source_animation_states.csv` (installed clip mappings), and `presentation_references.csv` (public-source rules versus independent approximation). The [detailed correction plan](../docs/PRESENTATION_PLAN.md) identifies what is implemented and what remains. Earlier snapshots are retained. Drew is performing testing, so this snapshot has not been independently workbook-tested or visually accepted.

The playable-entities snapshot adds all 34 weapon rows, all 15 vehicle rows, shared runtime tuning and 49 derived coverage rows. Thirteen weapon routes include the existing physgun/toolgun plus eleven primary-attack prototypes. Four powered vehicles and eleven passive seats have typed occupancy routes. Native secondary attacks, 21 disabled weapons, NPC AI, native driving/entry animations and audiovisual parity remain open. No gameplay acceptance is claimed.

Before workbook export run `powershell -NoProfile -File scripts/export-gameplay-coverage.ps1`. Project policy reserves gameplay and tests to Drew. The contributor test commands below are not authorization for agent-run tests.

The vehicle/water/category correction snapshot additionally preserves 8,390 original model-category memberships from 43 lists, researched vehicle heading/seat/eye frames, authored water optics and 14 correction acceptance rows. See [the correction plan](../docs/VEHICLE_WATER_CATEGORY_PLAN.md). Original mounted metadata is evidence, not gameplay acceptance. Normal-map/refraction rendering remains an approximation with native reflection and underwater fog pending.

Regenerate category metadata with `powershell -NoProfile -File scripts/catalog-model-categories.ps1` against your own legal installation before rebuilding/exporting when source lists change.

## Where to work

| Location | Role |
|---|---|
| [`../sheets/`](../sheets/) | Authoritative authored configuration and research tables |
| [`../sheets/parity_gaps.csv`](../sheets/parity_gaps.csv) | Detailed missing/partial/unverified feature backlog and acceptance checks |
| [`../sheets/work_queue.csv`](../sheets/work_queue.csv) | Ordered work packages assigning every gap once, with dependencies and remaining work |
| [`../sheets/source_effects.csv`](../sheets/source_effects.csv) | Runtime physgun texture references, color, sizes, pulse and beam scrolling |
| [`../catalogs/stock-20260929/`](../catalogs/stock-20260929/) | Versioned installed-content metadata snapshot, not game assets |
| This workbook | Combined snapshot of every CSV in both directories |
| [`../docs/SCHEMA.md`](../docs/SCHEMA.md) | Column contracts, units, generation and save formats |
| [`../docs/VALIDATION.md`](../docs/VALIDATION.md) | Executed checks and known validation boundaries |

The new combined workbook supersedes the two earlier local review workbooks. It includes their source inventory tables plus the newer player/play configuration, model catalog, UI reference catalog, and gap inventory. Earlier local workbooks were left untouched. No contributor needs those machine-local files to read the published tables or regenerate this workbook.

## Gap inventory conventions

The 169 rows cover all **64 cataloged system families and 37 installed tool entries**, with extra detail for rendering, movement, physgun effects and menus. This is complete coverage of the current discovery catalog, **not proof that every native engine option, script API or community addon has been exhaustively specified**. `reference_options` explicitly tracks the remaining per-option/per-state enumeration. Split broad rows as reference research advances.

- `missing`: no implementation of the stated feature is identified in the current scoped runtime.
- `partial`: an approximation or subset exists. It is not equivalent to stock GMod.
- `unverified`: implementation or expected reference behavior still needs direct measurement. Do not assume absent or correct.
- `reference_only`: installed example/legacy material requiring research, not an advertised player-facing feature.
- `verified`: reserved for a future row with linked passing reference comparisons. None is claimed by this inventory.
- `P0`: reported visual/input mismatch or prerequisite reference work. `P1`: core playability/fidelity. `P2`: broader stock functionality. `P3`: compatibility extensions or research-only entries.
- `spec_id` links to `systems.csv` or `tools.csv`. `acceptance` describes a check to perform, not a result already achieved.

Evidence keys: `source_play` refers to `crates/sandbox/src/source_play.rs` (including pickup and `rebuild_menu`). `source_play_beam` is the historical gizmo-beam reference, now superseded by `source_effects` in `crates/sandbox/src/source_effects.rs`. `source_player`, `source_animation`, `source_assets`, and `source_models` refer to the corresponding Rust modules. `feature_ledger` means `docs/PLAYER_PARITY.md` plus the existing system catalog. `tools_catalog` means `sheets/tools.csv` and its source R9. `user_report*` records the 2026-09-29 reports of missing physgun lighting, nonmatching menus and unreliable launch feedback. These are observations and inspection references, not original-game numerical measurements.

The older `systems.csv` and `tools.csv` statuses describe the initial implementation/research baseline. For the latest detailed gaps use `parity_gaps.csv` and the validation record. The catalog's 37 tool entries include internal/example/legacy entries and do not imply 37 fully specified player-facing tools.

## Edit, validate, build

1. Pick a gap and capture the corresponding behavior in a legally installed reference game. Record build, map, model, settings, input sequence, timing and tolerances.
2. Edit UTF-8 CSV files in `sheets/`. When editing the workbook, export the changed tab back to its matching CSV. XLSX is not a second executable authoring source.
3. Implement the behavior in Rust. Research descriptions do not automatically become engine code.
4. Run these commands from the repository root:

```text
cargo run -p sandbox-catalog -- validate sheets
cargo test -p sandbox-catalog --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Builds consume authored configuration through `crates/sandbox/build.rs`. Asset catalogs and `parity_gaps.csv` are research metadata, not runtime logic. The catalog test checks gap IDs, schema, priorities and coverage of every known system/tool. Runtime data equality tests verify compiled player/play/map/effects configuration. The work-queue test rejects missing or duplicate gap assignments and requires dependencies to occur earlier in the file. `implemented_partial` means the scoped work package has code, not that its linked parity gaps are verified.

Use Rust/MSVC prerequisites on Windows and set `GMOD_DIR` to your own Steam GarrysMod root for original content. Set `CARGO_TARGET_DIR` to a suitable private build directory rather than relying on the developer's D: drive. `play-prototype.cmd` runs the independent procedural testbed without original game content. Stock content and Steam are not required just to read or export these tables.

## Regenerate the workbook

Choose a **new output filename** because the exporter refuses to overwrite an existing workbook:

```text
cargo run -p sandbox-catalog --features workbook -- workbook sheets catalogs/stock-20260929 spreadsheets/rust-sandbox-catalog-next.xlsx
```

On Windows, verify all worksheet names, row counts and absence of formulas:

```powershell
powershell -NoProfile -File scripts/verify-workbook.ps1 -Workbook spreadsheets/rust-sandbox-catalog-next.xlsx -Sheets sheets -Inventory catalogs/stock-20260929
```

Review CSV changes and workbook together before submitting a pull request. Do not upload Steam payloads, extracted game scripts, credentials, personal saves or screenshots containing private data. New catalog snapshots need a provenance and privacy review. File names and declaration references are not permission to redistribute the referenced assets.
