# Research record

Research date: 2026-09-29 UTC. Citation IDs are maintained in `sheets/sources.csv`.

## Observed local baseline

- Steam app 4000 installation found through `libraryfolders.vdf` at `D:\SteamLibrary\steamapps\common\GarrysMod`.
- Manifest build ID 25375506. The installation was read only. No game files, saves, authentication or Steam settings were changed.
- First successful inventory: 2,904 loose files, 78,051 VPK entries across nine directory archives, 5,792 lexical Lua declaration candidates, 37 direct stock tool Lua files, zero skipped filesystem links.
- Combined asset records: **80,955**, not 80,955 unique artistic assets. Companion files, archive containers and duplicated virtual paths are separate records.
- Local gamemode directories: `base`, `sandbox`, `terrortown`. This does not mean those modes are implemented here.
- Tool declarations were inspected for categories, callback names and `TOOL.ClientConVar` defaults. High-level semantics in `tools.csv` remain summaries, not full line-by-line verified specifications.

## Public source scope

The Facepunch repository README says it contains Lua/text/config extensions and that binary sources are not public. Valve Source SDK 2013 README identifies HL2, HL2DM and TF2 game code. Its license grants specific Source 1 mod-related rights and distribution conditions, not unrestricted ownership of all code or assets. `gmod-module-base` exposes a native module interface. `garrysmod_common` is a community module-building utility, not the full engine.

These are useful research references. No Valve/Facepunch implementation was copied or mechanically translated into this repository. The installed source inventory remains in ignored local output. Reference behavior is independently approximated where implemented.

## Reuse decisions

1. **Bevy**, requested by Drew, supplies ECS, input, windowing, UI and rendering. Version 0.16.1 was selected with a compatible physics plugin, not because it was assumed newest.
2. **bevy_rapier3d 0.30.0** manifest explicitly targets Bevy 0.16. Its fixed-schedule API is used. This is an existing solver, not a new hand-written physics engine. The project has since moved upstream to the main Rapier repository, so future upgrades should consult that location.
3. **csv / serde / serde_json** handle typed authoring and persistence. Do not implement ad-hoc CSV parsing or silently accept invalid physical values.
4. **rust_xlsxwriter** creates the Excel review snapshot. It is optional so the core game build need not use the workbook feature.
5. **ValvePython/vpk** was consulted for directory header and entry layouts. Our bounded metadata-only parser is independently written. It supports directory versions 1/2 and does not extract, execute or decode payloads. The Valve Developer Wiki format page returned a bot-protection page, so it was not claimed as successfully inspected.
6. Macroquad was initially investigated before Drew requested Bevy. It is not a project dependency or implementation component.

Bevy uses MIT/Apache-2.0 licensing, Rapier uses Apache-2.0, and the serialization/workbook crates have their own permissive terms. Exact dependency versions are locked in Cargo.lock. Review dependency notices and asset rights before any distribution. No broad license over third-party content is asserted by this project.

## What has not been researched to completion

No full native engine implementation, exhaustive Lua API semantics, every game convar, all model/material formats, all stock map I/O entities, Workshop corpus, mounted game corpus, weapon/NPC stat corpus or numerical Source physics baseline has been reconstructed. The 64-system matrix is a discovery map that must be expanded into concrete reference cases during M2. The current runnable prototype is evidence for its own behavior only.
