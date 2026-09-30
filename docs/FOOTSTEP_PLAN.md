# Surface-dependent footsteps

## Scope and implementation status

This is a bounded grounded-footstep approximation, not Source audio, animation, or surface-physics parity. The implementation reads collider `SurfaceMaterial.surface_prop`, tracks observed player transform displacement, and submits one-shots through the existing `source_play::audio::emit` path. The catalog is authored in `sheets/source_footsteps.csv`; event IDs and path variants are proposed in `sheets/footstep_audio_reference.csv` for integration into the shared audio event catalog. Unknown and empty properties are silent. There is no wildcard/default mapping.

The system emits only when the player is grounded, translating above the authored minimum speed, and not dead, in a vehicle, noclip, searching/typing, in a menu, or at any detected water level. Water and wade events are deliberately excluded because the current player state does not distinguish validated shallow-water footstep conditions. Crouched movement uses a longer cadence. Large frame gaps and discontinuous movement are discarded to avoid teleport/respawn footsteps.

## Research

Read-only references consulted:

- Facepunch Garry's Mod public `scripts/surfaceproperties.txt`: <https://github.com/Facepunch/garrysmod/blob/master/garrysmod/scripts/surfaceproperties.txt>. Surface properties define `stepleft`/`stepright` sound-event names for material families including concrete, solid metal, metal grate, dirt, grass, tile, snow, water and wade.
- Facepunch Garry's Mod public `scripts/sounds/hl2_game_sounds_physics.txt`: <https://github.com/Facepunch/garrysmod/blob/master/garrysmod/scripts/sounds/hl2_game_sounds_physics.txt>. These event definitions provide the alternating waveform groups represented in the reference sheet. Concrete, metal, metal grate, dirt, grass, tile, and snow are included; water and wade are intentionally deferred.
- Facepunch hook references describe step events and timing but do not define this implementation's cadence: <https://wiki.facepunch.com/gmod/GM:PlayerFootstep> and <https://wiki.facepunch.com/gmod/GM:PlayerStepSoundTime>.

Installed audio inventory metadata in `catalogs/stock-20260929/assets.csv` confirms the listed concrete, metal, metal-grate, dirt, grass, tile, and snow WAV references are present in the mounted fallback catalog. The inventory is used read-only. No proprietary sound payloads or extracted scripts are copied into this repository.

## Authored approximation

The sheet uses 0.52-second walking and 0.36-second running intervals, with a 5.7 units/second run threshold aligned below the project's authored 7.62 run speed. Crouched cadence multiplies that interval by the authored 1.3 factor. These are tuning values, not claimed native constants. Gain is 0.55, and the shared event rows use zero cooldown because footsteps own their cadence. The selected side alternates on each emitted event.

At runtime a downward ray identifies the ground collider. Only its `surface_prop` is used, normalized to lowercase and matched against explicit catalog keys. The `material` VMT path is retained as metadata but not used as a fallback mapping. The system does not infer material from texture appearance.

## Known gaps

- Ground-only: no validated shallow-water/wading step types, underwater footsteps, ladder steps, landing sounds, or airborne movement.
- Only seven explicit surface properties are mapped. Other Source surface properties remain silent until their sound events and installed payload paths are verified.
- A single ray and collider surface property do not reproduce native foot contact selection on seams, blended surfaces, moving platforms, or multi-material collision meshes.
- Cadence uses current planar displacement and authored thresholds, not Source animation-cycle notifications, gait/stance events, surface-property sound scripts, pitch randomization, or native mixer behavior.
- There is no ambient/AI hearing or multiplayer replication path.
- Manual acceptance remains Drew-owned in `sheets/footstep_cases.csv`; all cases are `not_run` until integrated and exercised.

## Integration and verification

The catalog loader checks unique normalized surface keys, bounded tuning, running interval not exceeding walking interval, and that both left/right event IDs exist in the shared audio catalog. The catalog generator, source-map plugin registration, and shared audio event rows are integrated. No tests, builds, Clippy, benchmarks, gameplay, screenshots, input automation, or launch were performed, per project instructions. Drew performs acceptance testing.
