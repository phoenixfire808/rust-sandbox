# Entity and menu source of truth

**Historical discovery milestone:** the later [playable entities integration](PLAYABLE_ENTITIES_PLAN.md) adds primary weapon and vehicle routes. Current executable scope is `source_weapons.csv`, `source_vehicles.csv` and `playable_coverage.csv`; the reference-only runtime description below records the earlier milestone, not current route availability.

## Authority and current delivery

The editable source is CSV in `sheets/`. XLSX is a review/export artifact, not a second database. Export edited workbook tabs back to their corresponding CSV files before compiling. Do not edit generated Rust.

- `parity_gaps.csv`: existing 169 root gaps across the whole game. This remains the cross-system ledger.
- `work_queue.csv`: existing root-gap work packages. No duplicate root assignment was added.
- `spawn_capabilities.csv`: 56 finer system requirements with root gap links, priority, dependencies, acceptance, source and result. These refine, not replace, the root ledger. Rows are topologically ordered.
- `spawn_reference.csv`: 197 discovered definitions, NOT 197 working implementations: 85 NPC registrations, 15 vehicle registrations, 34 weapons, 42 visibly spawnable entities, and 21 internal/inherited-only entities. Aliases are separate rows because their defaults differ.
- `source_creation_tabs.csv`: eight authored runtime tab definitions with source order retained separately from deterministic display order. NPCs/Entities and Dupes/Saves have tied stock order values.
- `menu_lua_reference.csv`: 1,406 literal UI metadata rows. Together with the earlier HTML/CSS/native-control inventories this is a source-location index, not a screenshot or evaluated layout.
- `spawn_menu_reference_scope.csv`: 178 inspected source-file hashes and extraction coverage descriptions.
- `spawn_catalog_cases.csv`: 14 Drew-owned interaction cases, all `not_run`.

The normal game build and catalog validate/generate commands load and validate the spawn references, tabs and capability sheets. Build generation embeds typed records. Bad identifiers, unknown root gaps, invalid kinds/statuses, malformed defaults JSON, duplicate tab positions, and missing/forward/cyclic capability dependencies fail generation. States do not authorize executable actions: editing a status cell cannot turn a missing native entity into a prop or execute Lua.

Runtime now uses the eight sheet-authored creation tabs. Weapons/NPCs/Entities/Vehicles have category filtering, name/class/category search, paging, entry inspection, source location and system-work details. Internal-only entries remain in the workbook but not in the visible creation catalog. Post Process/Dupes/Saves show their remaining work rather than invented user content. F8 captures the selected definition and creation tab independently from the last selected prop model. Existing model spawning and right-side tools are preserved.

**This delivery implements the source-of-truth catalog and navigation. It does not implement drivable cars, native NPC simulation, generic scripted entities, stock weapon inventory, native save/dupe compatibility, or one-to-one Derma visuals.** The new non-model browser deliberately says reference catalog and offers Inspect, not Spawn. Physgun/toolgun continue through their existing controls. Native spawning requires real class implementations, not model substitution.

## Reused sources and concrete findings

Read-only installed sources under `garrysmod/`, also available in [Facepunch/garrysmod](https://github.com/Facepunch/garrysmod):

1. `lua/autorun/base_npcs.lua`: AddNPC registrations retain alias/class distinctions, category, model/skin if explicit, flags, keyvalues, health, weapon restrictions, floor/ceiling placement, offsets and no-drop rules. Rebel/Medic/Refugee and hacked combat classes are not interchangeable. HL1, Portal and Lost Coast definitions are conditional. Native models omitted by registration remain unknown, not guessed.
2. `lua/autorun/base_vehicles.lua`: Jeep uses `prop_vehicle_jeep_old`, Jalopy uses `prop_vehicle_jeep`, Airboat uses `prop_vehicle_airboat`, chairs/seats use `prop_vehicle_prisoner_pod`. Registered vehicle script paths, seat animations and offsets matter. APC has a health override. Fifteen registrations do not imply fifteen separate physics engines or unrestricted passenger capacity.
3. `lua/autorun/game_hl2.lua`: item placement defaults and alias overrides differ from SWEP spawn/equip behavior. Native pickups, chargers, mines, grenades and Portal cores need real entity behavior. NPCUsableWeapons is an equipment list, not another spawnable-weapon list.
4. Sandbox `gamemode/commands.lua`: trace/spawn routines distinguish props, ragdolls, effects, NPCs, SENTs, SWEPs and vehicles. Vehicle spawning uses class/model, scripts/keyvalues, Spawn then Activate, creator/player hooks, health overrides, class aliases, Members, ownership, undo and cleanup. Jeep positioning has a special physics-safety exception. Generic prop enqueue is not an equivalent route.
5. Sandbox `gamemode/spawnmenu/spawnmenu.lua`: divider 6, tool-panel minimum 300 or 460 at width >=1024, left minimum one third of divider width. Margins derive from `(screen - 1024x768) * spawnmenu_border`, clamped 25..256, with zero margins on small screens. Collapse button is 16x16, inset6. Cookie-restored splitter, remembered mouse position, Q hold/toggle/HangOpen, and Q/C exclusion require behavioral parity, not only matching colors.
6. `contextmenu.lua`: active tool control reparenting/restoration; tool canvas width320, height capped at 80% screen, placed 50px from right and bottom. Desktop icon buttons80x82 with64x64 icons and8px inset/spacing. Registered centered singleton windows and world-clicker behavior are missing.
7. `lua/vgui/contenticon.lua`: content icons128x128 with122px image inset3, stock normal/hover materials, depressed border8, admin shield and NPC-weapon badges16, DermaDefault caption near bottom9. Copy spawnname, custom right-click options, read-only removal, selection and drag/drop require actions. Current reference rows are not this icon rendering implementation.
8. NPC content type: sorted names/categories plus lower settings drawer open150 and padding15/10/15/10. Four settings are AI disabled, ignore players, server ragdolls and citizen auto-squad. Weapon default, none and concrete NPC-usable weapons are distinct.
9. Scripted entity and weapon declarations retain literal Spawnable/AdminOnly/Base/Type. Missing Spawnable is not assumed true. Internal tool entities and the nonspawnable NextBot example must not become user-facing generic spawn entries.

## Coverage limits, explicitly open

This is a bounded static discovery pass, not a claim to enumerate every possible game state, addon, native engine class or Lua-generated control. The scanner recognizes specified registration helpers and literal ENT/SWEP fields, masks ordinary comments, balances declaration calls and retains symbolic values without executing them. It is not a Lua parser/interpreter. Long-bracket variants, inheritance, dynamic registrations, mount evaluation, addon hooks, native defaults and computed conditionals need further work. Conditional labels are explicitly scoped to the inspected stock files. Icon paths are references, not verified asset availability. English labels come from installed localization.

UI extraction captures named methods and literal arguments on one line, with nested calls bounded. Multiline calls, expressions, docking, CSS cascade, inheritance, callbacks and runtime-created controls require review at the recorded source line. Numeric atoms are not final screen coordinates. Hashes cover listed source files, not the entire Steam installation or localization tree. No proprietary scripts or assets are copied as executable implementations.

Vehicle script tuning and native suspension/engine solver code have NOT been reverse-engineered in this pass. Detailed NPC engine schedules, navigation and damage semantics likewise remain research/implementation tasks. Per-class acceptance must be added as each real class is implemented. Capability kind membership describes relevant systems, not proof that every vehicle needs every special behavior.

## Next implementation sequence

1. Shared typed entity identity/factory/lifecycle and trace placement, with ownership and failure cleanup. Define persistent IDs before adding object references.
2. Seat/pod vertical slice: valid spawn, E entry, occupancy, camera/view restrictions, safe exit, removal while occupied, save/undo/cleanup. This must work before declaring any chair supported.
3. Jeep slice: read registered vehicle script, validate units and tuning, suspension/wheel contacts, engine/gears, steering/brake/reverse, animation/audio. Then airboat and special vehicles. A static buggy model is never counted as a car implementation.
4. NPC slice: class defaults and placement, animation and navigation, senses/relationships/schedules, equipment, damage/death. Implement representative distinct classes before broadening the 85-row matrix. Wire stock AI settings only when they affect simulation.
5. Typed pickups/chargers/hazards/editors and weapon inventory. Then lifecycle persistence, native dupe/save formats and remaining creation/context controls.
6. Continue existing root-gap work in parallel in priority order, especially water stability, movement acceptance and renderer fidelity. The one local feedback report was reviewed and retained privately; no original text or user settings are included here.

Drew owns testing. Compilation is not gameplay acceptance. Do not mark any `not_run` case passed based on code inspection or the presence of a source reference.
