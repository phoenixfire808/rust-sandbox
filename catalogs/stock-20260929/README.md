# Stock installation metadata snapshot

Date: 2026-09-29 UTC. Garry's Mod Steam app 4000, recorded build ID 25375506. This is a read-only research inventory from one base installation, not a redistributable asset pack or an exhaustive addon inventory.

| Table | Rows | Meaning |
|---|---:|---|
| assets.csv | 80,955 | 2,904 loose-file records plus 78,051 entries from nine VPK directory archives |
| lua_symbols.csv | 5,792 | Lexically detected declaration references with relative source path and line |
| stock_tools.csv | 37 | Installed stock tool source entries including internal/example/legacy entries |
| models.csv | 6,343 | Unique model virtual paths in the selected mounted roots, with companion-file presence |
| menu_settings_references.csv | 1,166 | Lexical UI/convar API references found across 596 scanned Lua files |

The first three tables are exact copies of the inputs used by the previous local research workbooks. The final two are exact copies of the later mounted-content catalog. Copy integrity was checked using SHA-256. All five inputs were reviewed for absolute local paths, account IDs, common credential signatures, email addresses and user-save/screenshot/data paths before publication. No absolute paths, account IDs, credential signatures or email addresses were found. Two generic `steam_autocloud.vdf` filename/size records under `dupes/` and `saves/` are retained as metadata only. Their contents are not published. No personal save or screenshot filenames were identified. This is a scoped audit, not a general guarantee for future inventories.

## Limitations

- These tables contain identifiers, file metadata, declaration signatures and short reference arguments. They do **not** contain model/texture/audio payloads, full extracted scripts, personal save contents, binaries or credentials.
- File names and recorded signatures do not grant redistribution rights to the referenced assets. The `not_cleared` field concerns the original payload, not an assertion that a catalog entry is a bundled asset.
- Archive CRCs are declared values, not independently verified payload hashes.
- The asset table preserves container duplicates. It is not a count of unique artistic assets or unique storage bytes.
- The mounted model/UI catalog has a different scope from the recursive loose/VPK inventory. Its original audit counted 77,785 mounted paths and 6,256 models with VVD/VTX companions, with no failed Lua reads. Do not expect these counts to equal the broader inventory totals.
- `not_ported`, `not_yet_decoded` and `unverified` are original snapshot annotations. They do not override the current implementation evidence or certify that each model can render. Current work is tracked in `sheets/parity_gaps.csv` and `docs/VALIDATION.md`.
- Lexical extraction can miss dynamic registrations, single-quoted arguments, multiline syntax and native engine APIs. It can also include false positives. A reference row is not a complete behavioral specification.
- External mounted games, remote Workshop, GMA internals and complete BSP-pak enumeration are not covered by this snapshot.

## Reproduction

Use the repository's catalog CLI `inventory` command on your own legally installed game into a **new ignored local directory**. The mounted model/UI audit is available via `source-map --export-catalog`; it currently writes `local/playable-catalog` and refuses an existing destination. That command loads map content and is not required to build or read these published CSVs.

Review any new snapshot for private paths/content before committing it. Do not remove the ignore rule for `local/`. For the combined workbook export command, see `spreadsheets/README.md`.
