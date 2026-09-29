# Spreadsheet and generated-struct contract

## Frontend and local feedback authoring

`source_frontend.csv` has exactly one row generating FrontendConfig. `margin`, `sidebar_width`, `settings_width`, `map_icon`, `font_size` and `row_height` are logical UI pixels. `spawn_clearance` is meters outside the traced support plane. All are finite and positive with an upper bound of 1024. `feedback_limit` is a UTF-8 byte limit per field in 256..32768. Build generation and catalog validate/generate consume the sheet. `frontend_cases.csv` is non-executable review metadata with Drew-owned acceptance marked not_run.

Private local feedback uses version-1 JSON with created_unix_nanos, category, title, observed, expected, steps_and_notes, map, optional eye_position and selected_tool, prop_count, optional game_status and review_status. Each submission gets a unique file and sync_all before clearing the draft. `local/feedback-draft.json` retains four string fields but not category. These files are ignored user data, not workbook input. No automatic upload or agent execution occurs.

## Toolgun and tool authoring

`source_toolgun.csv` has one ToolgunConfig row. It authors fire_clip, safe relative screen material/background paths, power-of-two screen_size (64..1024), finite positive font/scroll/center/tracer/range values, and max_constraints (1..10000). Screen units are texture pixels, tracer dimensions/range are meters and tracer duration is seconds. The stock-derived values and independently chosen approximations are distinguished in RESEARCH.md.

`source_tools.csv` defines every tools.csv ID without its tool_ prefix plus the explicitly custom freeze entry. It authors label, category, partial/missing/reference_only status, current left/right/reload descriptions and remaining work. IDs must be unique and exactly cover the reference set. Categories are Constraints, Construction, Render, Poser or Internal. No partial entry means complete GMod behavior.

`source_tool_options.csv` authors tool/key/label, number/bool/choice kind, default, finite min/max/positive step, pipe-separated choices and enabled flag. Defaults must be accepted, keys unique and tool IDs known. Numeric ranges must have nonzero width. Only enabled controls can be changed by the menu or restored from local preferences. `source_physics_materials.csv` authors unique surface names with bounded friction/restitution and an approximation evidence label. Physical-material choices must resolve to that table. All four runtime sheets are consumed by build.rs and catalog validate/generate. `tool_reference_options.csv` and `tool_cases.csv` are non-executable research/acceptance metadata exported into the workbook.

Source sandbox scene documents now use `{version:2, props:[...], links:[...]}`. Props retain model/position/rotation/frozen plus defaultable colour/material/gravity/physical-material/world-only modifiers. Links store tool kind, prop indices or world endpoints, local anchors/bases and supported settings. Counts, indices, finite values, quaternion lengths, material identities and assets are checked before replacing live props. The previous Source prop-array saves still load with default modifiers. The procedural version-1 format described below is separate. Neither format is compatible with GMod saves, and Source-scene velocities/other entity types remain unsupported. F5 writes a new file rather than overwriting prior saves.

## Spawn performance authoring

`source_performance.csv` has exactly one typed row: `spawn_queue_limit` (1..64 pending requests), `report_seconds` (finite 1..60 seconds), and `frame_budget_ms` (finite 1..1000 milliseconds). Build generation and catalog validate/generate consume it. The authored 16.667 ms budget is a proposed 60 Hz frame target, not a measured GMod baseline or an achieved result. It does not alter simulation time. `performance_cases.csv` is a review-only scenario ledger, included in workbook export. The dependency optimization profile is build configuration in Cargo.toml, not gameplay data.

## Original installed Source maps

`source_maps.csv` generates typed `SourceMapDef` values at build time, validated by both build.rs and the catalog CLI. Columns: `id`, `bsp` (must equal `maps/<id>.bsp`), finite positive `unit_scale` (Bevy units per Hammer unit), `fly_speed`, `eye_height_units`, and `fov_degrees`. Duplicate IDs, traversal paths, unknown columns and invalid numeric ranges fail validation. `GMOD_DIR` selects a local installation path, not authored spreadsheet data.

`source_pipeline.csv` is review metadata describing implemented import stages and fidelity gaps. It is exported to XLSX but is not executable shader or physics logic. Binary assets stay in Steam, while the spreadsheet contains identifiers and configuration. `local/<map>-import-report.json` records observed decode counts and unsupported assets.

## General CSV contract

All files are UTF-8 CSV with a required header row. IDs are stable references, not display names. Do not use formulas. The parser handles quoted commas and escapes via the `csv` crate. Typed gameplay rows reject unknown columns, missing values, malformed booleans/numbers, NaN/infinity, duplicate prop IDs and invalid references.

## Presentation authoring

- `source_layout.csv` has one typed row for stock-derived margin reference sizes/factors, browser/tool widths, icon size, dividers, typography, scroll step, hold-Q mode, beam range, blend duration and movement thresholds. Pixel dimensions use Bevy logical UI pixels. Numeric inputs must be finite, positive and bounded. Minimum margin cannot exceed maximum, movement thresholds are ordered, and the tool list/divider must fit inside the narrow tool panel. Font/skin, scroll step, blend duration and beam range remain independently authored approximations as recorded in the reference ledger.
- `source_animation_states.csv` has one row for each supported `physgun` and `pistol` hold. Idle/jump clip identifiers and walk/run `{direction}` templates select mounted original animations. All eight cardinal/diagonal directions are decoded. Nominal clip speeds are meters/second. Both holds are required, duplicate/unknown holds and invalid speeds are rejected.
- These two sheets are registered with Cargo, validated by build.rs and the CLI, and emitted into compiled configuration. `presentation_references.csv` is non-executable evidence metadata with explicit provenance and remaining work. It does not prove pixel or physics equivalence.
- `source_play.csv` now permits 1..256 icons per batch, with 120 authored by default for the scrollable browser. Paging remains a bounded implementation detail, not an exact stock spawnlist implementation.

## Runtime tables

- **source_player.csv -> PlayerConfig:** exactly one row. Mounted `models/*.mdl` references select the player, hands, animation source and held world weapons. Idle clip names select authored weapon poses. Movement speeds are meters/second, hull/eye/step/camera dimensions are meters and viewmodel FOV is degrees. Paths reject traversal, numeric values must be finite and bounded, capsule diameter must be smaller than height, eye height cannot exceed height and run speed cannot be below walk speed. These are approximation settings, not measured Source parity values.
- **source_play.csv -> PlayConfig:** exactly one row. Original first-person weapon/default prop references, spawn distance, hold gain/speed, prop/page limits, sensitivity, gravity, mass and view offsets. Model paths and numeric ranges are validated. Both new sheets are validated by the build script and catalog CLI, emitted by `generate`, and included automatically in new workbook exports. Existing workbooks are snapshots and are not silently overwritten.

- **source_effects.csv -> EffectsConfig:** exactly one row. `glow_texture` and `beam_texture` are mounted `sprites/` texture names, with traversal and drive paths rejected. RGB is finite 0..1 sRGB. Core/tip/endpoint sizes are 0.001..2 meters, beam width 0.001..0.5 meters and beam repeat 0.01..20 meters. Beam scroll is -20..20 texture repeats/second, pulse frequency 0..20 Hz and pulse depth 0..0.9. Build and CLI validation precede generation. These authored values describe the current approximation, not reference-derived constants.

- **props.csv -> PropDef:** `id` is alphanumeric/underscore, `label` is 1..100 bytes. Box dimensions `size_x/y/z` are 0.05..100 meters. `mass` is 0.01..10000 kg. `friction` is 0..2, `restitution` is 0..1. RGB is 0..1 sRGB. `provenance` currently must be `original_procedural`.
- **world.csv -> WorldDef:** exactly one row. Gravity -100..0 m/s², `fixed_hz` 30..240, ground half-size 5..1000 m, movement/spawn/grab distances 0.1..100 in applicable units, `max_props` 1..10000. Physics clock, ground collider, camera speed, spawn distance and initial hold distance consume these values.
- **scene.csv -> Placement:** `prop_id` references an authored prop, positions are finite and within +/-10000 meters, frozen is `true` or `false`. Initial rows cannot exceed the prop limit.

`build.rs` registers all relevant sheet paths with Cargo, validates before writing output, and creates `OUT_DIR/content_generated.rs`. `include!` compiles it into the application. Strings are escaped as Rust string literals. The generated registry does not evaluate user-provided source fragments.

## Research tables

**systems.csv and tools.csv -> BehaviorSpec** share fields:

| Field | Meaning |
|---|---|
| id | Unique specification ID |
| domain | Functional grouping |
| behavior | High-level desired/reference behavior, not a formal complete algorithm |
| authority | Intended reference execution boundary, not current network support |
| dependencies | Pipe-separated specification IDs, or `none` |
| rust_target | Current or proposed Rust implementation name |
| status | `prototype`, `not_started`, `catalog_only`, `reference_only` |
| acceptance | Test objective, not a claim that it has passed |
| source | Citation ID in sources.csv |

IDs, dependency references, source references and status values are validated. Dependencies are a design relationship graph, not a topologically scheduled runtime. `prototype` does not imply all listed dependencies or the full reference behavior exist.

**sources.csv** records title, URL/path, inspected evidence and limitations. **milestones.csv** is planning metadata and is included in the review workbook, not consumed by the game.

**work_queue.csv** assigns every `parity_gaps.csv` ID exactly once through pipe-separated `gap_ids`. It has `id`, `phase`, `gap_ids`, `depends_on`, `status`, `work`, `acceptance`, and `remaining` columns. IDs are unique, fields nonempty, and every dependency must appear earlier in file order. Status is `planned`, `in_progress`, `implemented_partial`, `ongoing`, or `verified`. Tests enforce this structure and full gap coverage. This is a human implementation plan, not an agent scheduler or generated runtime behavior. A partial work package does not close its original-game parity gaps.

## Generated local inventory

**assets.csv -> AssetRow:** container and virtual/relative path form the identity. Duplicate virtual paths in different containers are retained because mount precedence is not resolved. `bytes` is physical file size for loose files and payload length plus preload for VPK entries. Do not sum loose archive sizes and their entry sizes as unique storage. `crc32` is the archive-declared hexadecimal CRC, not a verified hash. Archive index 32767 indicates inline data. Offsets remain archive-format offsets and are not extraction instructions.

All installed rows are conservatively marked `installed_game_unknown_rightsholder`, `not_cleared`, `catalog_only`. This explicitly does not label third-party content as ours. Formula-like text is escaped in CSV exports. XLSX export writes all cells as strings, never formulas or implicit links.

**lua_symbols.csv:** lexical declaration candidates with file and line number. This scanner is not a Lua parser, can include declarations in multiline comments, misses dynamically generated/assigned names, and does not enumerate the native Lua API. Do not treat its count as API coverage.

**stock_tools.csv:** direct `.lua` entries under the installed stock tool directory. Internal `creator`, `example` and `leafblower` records are not evidence of equivalent player-facing tools.

Inventory excludes VPK payload validation, GMA internals, BSP embedded pak entries, mounted external games, remote Workshop and semantic parsing of every asset. Any I/O/parser failure aborts and does not produce a completed summary. The output directory may contain partial CSV files after failure. A fresh output directory is required on retry.

## Save format

JSON version 1 contains prop IDs, translation, unit quaternion rotation, linear/angular velocity and frozen flag. Unknown fields, unknown prop IDs, over-limit scenes, malformed/nonfinite data and unsupported versions fail before world replacement. Each save has a fresh timestamped filename. Source/GMod saves and constrained assemblies are not supported. Undo stores up to 32 whole-scene snapshots and can rewind unrelated ongoing prop motion.
