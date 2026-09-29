# Spreadsheet handoff for contributors

Download [rust-sandbox-catalog-20260929.xlsx](rust-sandbox-catalog-20260929.xlsx) for the combined Excel/LibreOffice review workbook. GitHub does not render XLSX files inline, so use **Download raw file**. All source tables are also available as CSV for review and version control.

## Where to work

| Location | Role |
|---|---|
| [`../sheets/`](../sheets/) | Authoritative authored configuration and research tables |
| [`../sheets/parity_gaps.csv`](../sheets/parity_gaps.csv) | Detailed missing/partial/unverified feature backlog and acceptance checks |
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

Evidence keys: `source_play` and `source_play_beam` refer to `crates/sandbox/src/source_play.rs` (including `beam` and `rebuild_menu`). `source_player`, `source_animation`, `source_assets`, and `source_models` refer to the corresponding Rust modules. `feature_ledger` means `docs/PLAYER_PARITY.md` plus the existing system catalog. `tools_catalog` means `sheets/tools.csv` and its source R9. `user_report*` records the 2026-09-29 reports of missing physgun lighting, nonmatching menus and unreliable launch feedback. These are observations and inspection references, not original-game numerical measurements.

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

Builds consume authored configuration through `crates/sandbox/build.rs`. Asset catalogs and `parity_gaps.csv` are research metadata, not runtime logic. The catalog test checks gap IDs, schema, priorities and coverage of every known system/tool. Runtime data equality tests verify compiled player/play/map configuration.

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
