# Vehicle steering and engine feedback

## Scope

Address Drew's reports of instantaneous wheel steering, incorrect apparent steering, and absent vehicle sounds. `source_vehicles.csv` remains authoritative for vehicle physical tuning, `source_vehicle_wheels.csv` remains authoritative for wheel attachment/radius/maximum visual angle, and `source_vehicle_feedback.csv` adds a small independently authored steering response and engine-loop mapping. The feedback cases are Drew-owned and all start as `not_run`.

## Research basis

- The Jeep registration points to `scripts/vehicles/jeep_test.txt`; the Jalopy and APC registrations point to `jalopy.txt` and `apc_gmod.txt`. Existing `docs/RESEARCH.md` records read-only inspection of those mounted scripts and model metadata.
- Original wheel attachment names are `wheel_fl`, `wheel_fr`, `wheel_rl`, and `wheel_rr`. `source_position` maps Source +X forward to Bevy -Z and Source +Z up to Bevy +Y; the skeletal basis conversion preserves that yaw sign, matching physical rotation about Bevy +Y. Keep the existing Source-to-Bevy axle basis and do not add a visual-only steering sign.
- The local stock asset inventory lists mounted vehicle audio assets including `sound/vehicles/v8/v8_idle_loop1.wav`, `v8_rev_short_loop1.wav`, `v8_firstgear_rev_loop1.wav`, `v8_stop1.wav`, and `sound/vehicles/apc/apc_turbo_on_loop1.wav`. The inventory is only path/metadata evidence. Assets are not extracted, copied, or committed here. The shared audio catalog owns event loading and playback.
- Steering input is eased with an authored per-vehicle response rate in `source_vehicle_feedback.csv`. For wheeled vehicles, a shared helper maps that normalized occupancy value to one effective front-wheel angle: `source_vehicle_wheels.csv` supplies the low-speed angle, and the existing `source_vehicles.csv` `steer_rate` supplies a high-speed angle cap, reached by the existing speed fraction. Physical tire orientation and visual articulation use the same helper, same signed occupancy input, and same absolute chassis-forward speed. At standstill the physical tire angle is therefore the authored low-speed angle, not zero. The response is a project approximation, not a native script parse.
- The vehicle adapter exposes an occupied powered-wheel engine event and a bounded playback rate to the shared audio owner. This is occupant-local playback, not exterior spatial engine sound. Passive seats and airboats are excluded. Loop creation, occupant-local playback and cleanup on exit, removal, menu/focus loss, or death remain responsibilities of the shared audio system.

## Implemented slice and limits

This is an approximate prototype integration. The shared response curve is exponential easing, not native vehicle-script steering. Engine rate is derived from normalized forward speed and throttle and does not reconstruct Source gearbox bands, native DSP, skid cues, damage, or original sound-script channel semantics. The V8 event shared by Jeep/Jalopy and separate APC event are not claims of per-model mixer parity. Airboat surface support/steering remains on its existing path and receives no new loop.

## Drew-owned manual cases

`sheets/vehicle_feedback_cases.csv` defines the manual checks. Verify smooth directional changes and return-to-center at several speeds, forward and reverse sign/basis, audibility and pitch range on Jeep/Jalopy/APC, loop cleanup on exit/removal/menu/focus loss, and unchanged airboat behavior. Every row is `Drew,not_run` by design. No tests, compile, screenshots, benchmarks, input automation, or gameplay acceptance were performed for this worker task.

## Acceptance boundary

Wheel angle and audio behavior still require Drew's in-game review. These changes do not establish full Source vehicle-physics equivalence, exact Lua/script behavior, native engine mix/pitch behavior, or overall vehicle parity. See `docs/RESEARCH.md`, `docs/VALIDATION.md`, and the cases sheet for evidence and manual verification status.
