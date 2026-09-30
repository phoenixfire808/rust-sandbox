# Spreadsheet and generated-struct contract

## Release checklist

`source_release_notes.csv` authors ordered in-game cards: unique safe `id`, safe `revision`, `title`, `change`, `steps`, `expected`, `limits`, and `status` (`needs_check`, `partial`, `missing`). Required text is nonempty and bounded to 4096 bytes. The frontend loader rejects malformed entries and an empty checklist. `compiled_release_notes()` is generated from these rows; build.rs watches the sheet. Increment a card's revision whenever its testing expectations change so old personal checks do not imply the revised item was checked.

`release_checklist_cases.csv` records Drew-owned manual acceptance scenarios. In-game checkmarks are private reminders in `local/release-checklist.json`, never edits to these acceptance rows or a substitute for evidence. The exported workbook includes both new sheets.

## Typed NPC lifecycle

- `source_npcs.csv` has one explicit row per NPC reference, including disabled definitions. Enabled kinds are melee, ranged and passive. Faction, model, health, locomotion, attack range/damage/cooldown, sight range, hull dimensions and facing yaw are authored here. Numeric values are prototype parameters, not native-equivalence claims.
- Idle/walk/attack contain candidate names separated by `|`. A candidate may be `models/path.mdl::clip_name` to select a researched included animation library. Required clip failures reject spawning. The one-time initialization script refuses to overwrite an existing CSV; subsequent authorship stays in CSV.
- `source_npc_rules.csv` contains a single validated row for actor limit, gravity, local player maximum health, respawn delay and step height. Build generation watches both sheets. `npc_cases.csv` records acceptance without feeding gameplay tuning.
- Version-4 local scenes add optional NPC identity alongside existing vehicle identity and health. An entity cannot be both, live NPC constraints and frozen state are rejected, and enabled ID/model/health/count checks plus required asset preparation precede replacement. Legacy arrays and versions 2 and 3 remain accepted. Native AI task stacks and player lifecycle persistence are not included.

## Vehicle frames, stock categories and water

source_vehicles.csv adds forward_yaw in radians and eye_height in meters. Forward yaw maps Bevy local -Z to the original converted model nose. New spawn rotation is player_yaw - forward_yaw. Existing saved rotations remain unchanged. Propulsion/grip/ray placement use the same frame; seat_yaw sets body heading. Seat coordinates are model-local imported meters from original feet attachments, not normalized chassis coordinates. Pod feet remain estimated.

source_model_categories.csv stores unique model/category pairs plus original source and line. Categories preserve original parent paths and headers using ` / ` separators. Multiple memberships are intentional. Runtime filters to mounted models; fallback directories never use root model filenames as categories. ModelEntry.categories is runtime-only and skipped by the older flat metadata serializer, whose primary category remains readable. The original versioned discovery catalog is not rewritten.

source_water.csv is one authored row: roughness/reflectance/transmission/tint_mix are 0..1, ior is 1..2, positive thickness and uv_scale are bounded by 10, and signed UV scroll rates are bounded by 1. These values are prototype render tuning, not native convars. Original Water shader classification and fog/normal metadata come from mounted VMTs, not name heuristics. Build and production CLI validate and embed these sheets with the spawn catalog.

vehicle_water_category_cases.csv records 14 Drew-owned acceptance cases. not_run means no gameplay acceptance occurred, even when compilation and data export succeeded.

## Playable weapon and vehicle authority

`source_weapons.csv` has exactly one row per registered weapon. `kind` selects disabled/physgun/toolgun/hitscan/melee/projectile/grenade. Model paths stay read-only mount references. Idle/fire/reload are pipe-separated candidate clip names with fallback diagnostics. Clip/reserve are counts; interval/reload_seconds are seconds; range/speed/blast/gravity use meters and seconds. Damage and spread are prototype values, not verified native units. Automatic is a boolean. Scope and remaining are mandatory honesty boundaries.

`source_vehicles.csv` has exactly one row per registered vehicle. Kind is wheels/airboat/seat. Mass is kg; acceleration and speed use SI; steer_rate and seat_yaw use radians. Seat offsets and suspension are meters. Spring/damping/grip are independent prototype force coefficients. Entry/exit durations are seconds. Pose names are optional mounted animation lookups. Passive seats must have zero engine acceleration. `source_gameplay.csv` is one row of shared numeric budgets and tuning. The production catalog loader checks coverage, known IDs, allowed kinds, finite ranges and safe mounted model paths before generating embedded runtime data.

`playable_coverage.csv` is derived by `powershell -NoProfile -File scripts/export-gameplay-coverage.ps1`, not a second authored runtime registry. It preserves 49 per-registration remaining-work and Drew-owned not_run acceptance rows. Regenerate before exporting the workbook. Capabilities and parity statuses do not automatically enable routes.

Scene format version 3 adds optional vehicle registration IDs and ordinary-prop health. Version 2 and legacy snapshots remain readable. Vehicle/model identity and finite health are validated before scene replacement. Vehicle identity survives snapshots/duplication. Occupancy and projectiles are transient and released on restore. Ammo is session-only, not persisted. Older readers must not silently treat version-3 vehicles as ordinary props.

## Entity and creation-menu authority

`spawn_reference.csv` is reproducibly extracted reference metadata with unique kind-prefixed IDs, original spawn aliases/classes, labels/categories, optional explicit models, icon references, conditions, visibility, admin flags, symbolic defaults JSON and source/line provenance. It is not an implementation registry. `source_creation_tabs.csv` is authored runtime tab order and kind mapping. `spawn_capabilities.csv` is the authored system-level missing/partial/implemented/verified backlog with root `parity_gaps` links, P0..P3 priorities, pipe-separated earlier-row dependencies, source, acceptance and result. Kind groups are relevant system work rather than exact per-class dependency assertions. Build.rs and CLI validate/generate load all three, validate unique IDs, references and dependency order, then emit `compiled_spawn_catalog`. UI status changes never enable executable spawn routes. `menu_lua_reference.csv` and `spawn_menu_reference_scope.csv` are discovery evidence only. `spawn_catalog_cases.csv` remains Drew-owned and not_run. Local feedback now additionally captures creation_tab, catalog_selected and the selected catalog_definition, separate from selected_model. See [ENTITY_MENU_SOURCE_OF_TRUTH.md](ENTITY_MENU_SOURCE_OF_TRUTH.md) for scope and authority rules.

## Frontend and local feedback authoring

`source_frontend.csv` has exactly one row generating FrontendConfig. `margin`, `sidebar_width`, `settings_width`, `map_icon`, `font_size` and `row_height` are logical UI pixels. `spawn_clearance` is meters outside the traced support plane. Reference layout fields also include main-column insets, logo size, normal/compact/short fonts and breakpoints, 50px footer, map grid/card/caption/category geometry and Start Game insets. Numeric dimensions are finite and positive with an upper bound of 8192, while spawn_clearance is separately capped at 1 meter. The safe relative PNG background path is restricted to the installed backgrounds directory. `feedback_limit` is a UTF-8 byte limit per field in 256..32768. Build generation and catalog validate/generate consume the sheet. `source_main_menu.csv` generates ordered MenuEntry records with unique IDs, all/game visibility, logo/gap/link kind and a validated action route. Build.rs watches both runtime sheets. `menu_reference.csv`, `native_menu_controls.csv`, `newgame_reference_options.csv` and `frontend_cases.csv` are non-executable review metadata with Drew-owned acceptance marked not_run. Native control coordinates remain raw source values. `menu_html_controls.csv` records each literal control/label definition with source and line plus handler names and bindings, excluding comments. `menu_css_geometry.csv` records literal layout/style declarations with selectors and source lines; the CSS cascade and enclosing media conditions must still be resolved at those lines. These lexical inventories do not execute scripts or prove runtime coverage. `menu_work_items.csv` provides the 40-package ordered per-screen parity backlog, prerequisites and Drew-owned acceptance. Footer popup offsets/widths, compact-footer breakpoint/margin, and tiny/narrow home breakpoints are authored in FrontendConfig. Local `menu-preferences.json` stores a selected supported map plus a set of favorites, separate from Steam preferences.

Private local feedback uses version-2 JSON retaining the version-1 created_unix_nanos, category, title, observed, expected, steps_and_notes, map, optional eye_position and selected_tool, prop_count, optional game_status and review_status fields. `context_at_open` captures the original screen/hovered control or explicitly selected panel, map selection/search/popup, active tool and all settings, staged target, selected model, held entity, player pose/motion, window/cursor state and prop count before opening the editor. Entity IDs are diagnostic snapshots, not persistent references. Each submission gets a unique file and sync_all before clearing the draft. `local/feedback-context-draft.json` stores four fields, category and captured context via a synced temporary replacement. The old four-field `local/feedback-draft.json` is read only as migration fallback and never deleted. Unsent context is preserved unless the user chooses Attach current context. These files are ignored user data, not workbook input. No automatic upload or agent execution occurs. `context_feedback_cases.csv` is Drew-owned review metadata.

`source_player.csv` additionally authors finite ground_acceleration and air_acceleration (0.1..1000 per second), ground_friction (0..100 per second), stop_speed and air_speed_cap (0.01..30 m/s), and jump_boost (0..1 dimensionless). The corrected GMod preset friction is 8. minimum_move_speed (0.001..0.1 m/s) authors the 1 HU/s stopping threshold, ground_probe (0.001..0.1 m) authors the 2 HU support displacement separately from gravity, and walkable_normal (0.1..1) authors the up-normal threshold used for slope configuration and grounded plane clipping. jump_speed is m/s. These validated fields flow through existing build.rs generation. `movement_reference.csv` records source rules, implementation boundaries and not_run acceptance, not executable movement code. Feedback context player.movement now includes command, sprinting, persistent horizontal/vertical velocity and acceleration/friction/stop/air coefficients for local diagnosis.

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
