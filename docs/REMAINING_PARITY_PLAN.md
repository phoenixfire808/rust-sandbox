# Remaining parity implementation plan

The CSV sheets are the source of truth. `parity_gaps.csv` tracks system gaps, `spawn_capabilities.csv` tracks dependencies, and `playable_coverage.csv` is a derived per-registration review view. No broad gap is complete merely because one class has a working subset. All gameplay acceptance remains Drew-owned.

## Current package: NPC lifecycle and reported collision filtering

1. Read native registration, UI and tool code, plus original model/animation metadata. Preserve private feedback locally.
2. Give all 85 NPC registrations an explicit runtime coverage row in `source_npcs.csv`.
3. Enable a bounded ground-actor subset: classic zombie, zombie torso, standard Combine soldier, elite soldier, prison guard and passive Odessa. Use original model and animation resources, not substitute props.
4. Integrate floor/clearance checks, FIFO cancellation, NPC and global spawn limits, kinematic collision, visibility-gated nearest opposing-faction targeting, direct pursuit, cooldown-based melee/ranged damage and original clip playback.
5. Integrate Disable Thinking and Ignore Players controls, player health/death/respawn gates, weapon damage and removal, typed scene identity and health restoration. Reject unsupported prop-tool operations on live NPCs.
6. Repair world-only No Collide player movement filtering, preserving the distinction between pair-only left click and world-only right click.
7. Compile the actual executable, validate production sheets, regenerate the workbook, review and publish. Compilation and metadata extraction do not constitute gameplay acceptance.

```mermaid
flowchart LR
  R[Read-only stock registrations and model metadata] --> S[Authored NPC and rules sheets]
  S --> G[Validated generated catalog]
  G --> Q[Menu and bounded spawn queue]
  Q --> A[Typed animated actors]
  A --> C[Collision and combat]
  A --> P[Versioned scenes and undo]
  C --> D[Drew-owned acceptance cases]
  P --> D
```

## Explicit boundaries of this package

- Six enabled registrations are partial implementations. The other 79 remain disabled with visible missing-work details. Shotgun aliases are not silently mapped to rifle behavior.
- Numeric combat tuning and hulls are authored prototype values, not measured native equivalence. Soldier equipment models, ammo/reload, native firing events, secondary actions, grenade use and sounds are absent.
- Perception is distance plus collider line of sight. Pursuit is direct kinematic motion, not nodegraph/navmesh pathfinding, hearing, memory, cover, squads or native schedules. Passive Odessa has no invented weapon or attack.
- Death removes an NPC rather than creating a fake ragdoll. Health uses the existing weapon damage route without hitgroups. Player health/death/respawn is local and does not imply armor, fall/crush damage, team rules or native loadout parity.
- Required clip failure rejects spawning with an error. Listed clip names were researched, but actual clip decoding, posing and end-user gameplay are not accepted until Drew checks them.
- Save/load preserves NPC identity and remaining health, not native AI task stacks. Legacy scene versions remain supported. Live NPC prop-tool operations are intentionally restricted rather than corrupting the kinematic controller.

## Remaining dependency order

| Package | Authoritative gap families | Completion gate |
| --- | --- | --- |
| Actor foundations | npc_ai, npc_controls, player_life, damage_rules, player_ragdoll, navigation_data | Per-class behavior, damage, death, navigation and lifecycle evidence |
| Vehicles | vehicle_runtime and vehicle_* capabilities | Every powered model and passive seat, actual original tuning, wheels, entry/exit, audio and occupied removal |
| Entity factories and map interaction | entity_callbacks, bsp_entities, map_io_dispatch, map_interactions | Each class has distinct use/touch/think and delayed I/O behavior |
| Player and weapon behavior | move_*, noclip_exit, weapon_*, stock_*, gravity_actions, physgun_* | Reference input/state/animation/resource cases including failure paths |
| Sandbox tools and persistence | ownership_rules, undo_groups, cleanup_groups, dupe_assemblies, save_* | Per-operation undo and mixed typed scene roundtrips without data loss |
| UI parity | menu_*, context_*, derma_widgets, ui_*, main_menu, settings_menu, hud | All stock control locations, behavior, focus and persistence across resolutions |
| Rendering and assets | water_render, materials_*, model_*, anim_*, lighting, BSP/sky, effects, postprocessing | Material/model/map corpus and matched visual acceptance |
| Sound | audio_environment, audio_channels, physgun_audio | Source script event, channel, loop, spatial and environmental behavior |
| Compatibility and multiplayer | lua_*, net_*, nextbot_runtime, multiplayer UI | Explicit versioned API and multi-client conformance, not prototype equivalence |

The full individual work items and remaining requirements stay in the existing spreadsheet rows. This plan orders them, it does not replace or close them.
