# NPC equipment continuation, 2026-09-30

## Researched boundary

- The checked-in stock snapshot is metadata only: `catalogs/stock-20260929/assets.csv` inventories mounted model/sound paths and `lua_symbols.csv` records lexical script declarations, not semantic behavior. It does not prove per-NPC weapon attachment transforms or provide a complete weapon assignment table.
- The current enabled ranged identities are `npc_npc_combine_s`, `npc_combineelite`, and `npc_combineprison`. Their authored attack clip candidate is `models/combine_soldier_anims.mdl::@shootAR2s`, which supports naming the prototype firing event AR2, but does not prove native ammo/reload timing.
- The stock AR2 single-fire sound path `sound/weapons/ar2/fire1.wav` is reused by the scoped audio event `npc.ar2.fire`. This is the weapon sound payload, not evidence that NPC animation/equipment/schedules are equivalent.

## Delivered slice

- `sheets/source_npc_equipment.csv` explicitly assigns only the three already-enabled ranged Combine actors to `weapon_ar2` and authors magazine capacity, reload seconds, burst size and burst spacing. Values are bounded prototype settings, not native measurements.
- `source_npc_equipment.rs` owns transient per-actor magazine, reload, and timed-burst state. Each due shot consumes one round and returns one damage event. Empty magazines begin a reload; burst shots use the authored short spacing and then return to the NPC's normal authored attack interval.
- `source_npcs.rs` attaches state only for ranged actors and routes accepted shots through the existing health/damage lifecycle. Every shot reacquires a visible target and checks its current range/line of sight before consuming ammunition. Paused/disabled AI does not fire. `npc.ar2.fire` is emitted only after a round is accepted.
- Scene restore intentionally initializes a fresh full magazine. Live magazine/burst/reload state is not serialized; this is a documented partial lifecycle policy, not exact save parity.

## Explicit remaining work

- No held weapon mesh or muzzle attachment is rendered. The checked-in reference metadata does not establish reliable local attachment transforms; do not infer these from the shot animation alone.
- Burst, magazine size, reload time, damage and timing are prototypes, not Source-equivalent behavior. Native schedules, NPC weapon AI, hitgroups, ammo pickups, sounds beyond the one shot cue, reactions, and combat gameplay remain incomplete.
- No other NPC is enabled by this work. A new class needs original model, compatible idle/walk/attack clips and distinct behavior evidence before registration.

## Manual acceptance owned by Drew

See `npc_magazine`, `npc_burst_los`, and `npc_ar2_audio` in `sheets/npc_cases.csv`. Also repeat existing `npc_sight`, `npc_thinking`, `npc_ignore`, `npc_save`, and `npc_invalid_save` after the coordinator's consolidated build. All remain `not_run` until Drew tests the game.

No builds, tests, benchmarks, screenshots, or input automation were run for this change.
