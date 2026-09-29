# Spreadsheet and generated-struct contract

## Original installed Source maps

`source_maps.csv` generates typed `SourceMapDef` values at build time, validated by both build.rs and the catalog CLI. Columns: `id`, `bsp` (must equal `maps/<id>.bsp`), finite positive `unit_scale` (Bevy units per Hammer unit), `fly_speed`, `eye_height_units`, and `fov_degrees`. Duplicate IDs, traversal paths, unknown columns and invalid numeric ranges fail validation. `GMOD_DIR` selects a local installation path, not authored spreadsheet data.

`source_pipeline.csv` is review metadata describing implemented import stages and fidelity gaps. It is exported to XLSX but is not executable shader or physics logic. Binary assets stay in Steam, while the spreadsheet contains identifiers and configuration. `local/<map>-import-report.json` records observed decode counts and unsupported assets.

## General CSV contract

All files are UTF-8 CSV with a required header row. IDs are stable references, not display names. Do not use formulas. The parser handles quoted commas and escapes via the `csv` crate. Typed gameplay rows reject unknown columns, missing values, malformed booleans/numbers, NaN/infinity, duplicate prop IDs and invalid references.

## Runtime tables

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

## Generated local inventory

**assets.csv -> AssetRow:** container and virtual/relative path form the identity. Duplicate virtual paths in different containers are retained because mount precedence is not resolved. `bytes` is physical file size for loose files and payload length plus preload for VPK entries. Do not sum loose archive sizes and their entry sizes as unique storage. `crc32` is the archive-declared hexadecimal CRC, not a verified hash. Archive index 32767 indicates inline data. Offsets remain archive-format offsets and are not extraction instructions.

All installed rows are conservatively marked `installed_game_unknown_rightsholder`, `not_cleared`, `catalog_only`. This explicitly does not label third-party content as ours. Formula-like text is escaped in CSV exports. XLSX export writes all cells as strings, never formulas or implicit links.

**lua_symbols.csv:** lexical declaration candidates with file and line number. This scanner is not a Lua parser, can include declarations in multiline comments, misses dynamically generated/assigned names, and does not enumerate the native Lua API. Do not treat its count as API coverage.

**stock_tools.csv:** direct `.lua` entries under the installed stock tool directory. Internal `creator`, `example` and `leafblower` records are not evidence of equivalent player-facing tools.

Inventory excludes VPK payload validation, GMA internals, BSP embedded pak entries, mounted external games, remote Workshop and semantic parsing of every asset. Any I/O/parser failure aborts and does not produce a completed summary. The output directory may contain partial CSV files after failure. A fresh output directory is required on retry.

## Save format

JSON version 1 contains prop IDs, translation, unit quaternion rotation, linear/angular velocity and frozen flag. Unknown fields, unknown prop IDs, over-limit scenes, malformed/nonfinite data and unsupported versions fail before world replacement. Each save has a fresh timestamped filename. Source/GMod saves and constrained assemblies are not supported. Undo stores up to 32 whole-scene snapshots and can rewind unrelated ongoing prop motion.
