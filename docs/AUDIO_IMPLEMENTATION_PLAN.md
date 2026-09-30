# Mounted audio and impact feedback

## Delivered slice, 2026-09-30

This closes part of the missing sound and prop-impact feedback dependency, not every remaining parity gap. All 19 private feedback reports were reviewed. CSV files remain the authoring source, and gameplay acceptance remains Drew-owned.

1. Reuse Bevy 0.16.1 audio rather than adding a second engine. WAV/MP3 decoders consume bytes from the existing read-only Steam mount layer. No original audio is extracted into the repository or redistributed.
2. Author limits in `source_audio.csv` and 22 event mappings in `source_audio_events.csv`. Nine shot/swing mappings cover pistol, .357, SMG, AR2, shotgun, crossbow, RPG, crowbar and stunstick. Five reload-start cues, one shared empty cue, one independent tool cue, one explosion event and five impact-material/fallback events complete this slice. Metadata inspection found all 27 distinct paths in the existing stock catalog. This does not establish actual decoder or playback acceptance.
3. Preload one bounded asset at a time off the game thread. Reject oversized, empty, unsupported or excessive decoded-sample data before playback. Current authored caps are 4 MiB per asset, 64 MiB aggregate cached compressed bytes, 2,646,000 decoded samples per asset, 24 voices, 48 queued requests and 30-second voice lifetime. The mount layer has its own read bound before the smaller audio check. No asset file is written. First-use events before preload completion are dropped, not replayed later.
4. Emit on accepted attacks and tool actions, not rejected clicks. Clear transient audio on menus, focus loss, death and scene restoration. Stop local weapon/tool tails on equipment changes. Live volume updates and distance falloff apply to current voices. World sounds use Bevy stereo positioning, not Source soundlevel or DSP behavior.
5. Reuse Rapier contact events with pre-step relative normal speed, force thresholds and per-prop cooldowns. Selected Physics Properties materials choose metal, wood or glass variants. Unknown materials use an explicitly generic fallback, not guessed original model surfaces. Decaying bounded visual shake uses distance to the rendered camera and does not alter aim rays or physical movement. Angular-only contact speed is not implemented.
6. Expose audio volume and impact-shake settings in Q > Player with separate local persistence. F7 has two new cards and links into contextual feedback. `audio_cases.csv` contains 14 manual scenarios, all `not_run`.

## Remaining work, not silently completed

- Native physgun loops and acquire/release/freeze cues, vehicle engines and wheels, NPC voices and attacks, footsteps, water, ambient emitters, menus and other weapon mappings remain absent.
- Reload-start cues are not frame-accurate animation events or per-shell handling. Toolgun.Single's waveform mapping was not resolved, so its temporary airboat-shot mapping is explicitly independent. Flechette and grenade-throw cues are not claimed.
- Source pitch variation, channel replacement, sound scripts/operators, HRTF, occlusion, DSP, original surface-pair selection and physical response calibration remain open. Voice overload drops new sounds rather than stealing by native priority.
- Native audio Options/device controls and device-loss recovery are not implemented. Bevy's no-device behavior and hardware playback still need Drew's acceptance. Linux/macOS audio build prerequisites and portability were not exercised.
- Articulated ragdolls, remaining NPC/weapon functionality, full vehicles, menu equality and all other root-gap entries remain tracked in `parity_gaps.csv` and dependency-ordered `work_queue.csv`.

## Integration evidence and next acceptance

Compile the actual source-map executable using the private D: target, validate the production catalog and export a new review workbook. Do not run tests or gameplay automation. Drew should use F7, then the 14 audio cases, beginning with preload/device behavior and the nine mapped weapons. Compile success is not proof that sounds were heard, contact tuning feels right or lifecycle transitions work in play. Results belong in the existing acceptance sheets only after actual user observation.
