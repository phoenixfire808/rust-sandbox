# Vehicle dynamics and lifecycle continuation

## Implemented slice

`source_vehicles.csv` remains the authority for engine acceleration, brake acceleration, speed cap, steering, suspension and grip. For `kind=wheels`, `grip` is now an explicit dimensionless tire friction multiplier (authored at 1.2 for Jeep/Jalopy and 0.9 for APC), replacing the old 7/6 damping-rate values. The airboat retains its existing proportional damping interpretation. `source_vehicle_wheels.csv` remains the authority for rendered wheel attachment, radius and steering angle. Powered wheeled vehicles apply throttle, brake and lateral tire forces only at supported ray contact points. Per-contact lateral and longitudinal forces share a load-scaled grip limit; front contacts steer and create yaw through their offsets. The chassis still uses approximate ray-supported spring/damper forces. This is a restrained prototype, not native vehicle physics, a wheel-collider solver, or a claim of Source compatibility.

`source_vehicle_visuals.rs` retains the researched Source-to-Bevy axle basis used by attachment-driven wheel skinning. Keep that mapping when extending wheel animation. Airboat surface support remains a separately approximated water-height path. Seat-only entities stay passive.

## Research basis

- The stock catalog identifies the Jeep as `prop_vehicle_jeep_old` with `scripts/vehicles/jeep_test.txt`, and the jalopy/APC registrations identify `jalopy.txt` and `apc_gmod.txt`. Installed mounted files are read-only and are not redistributed.
- Existing `docs/RESEARCH.md` records the mounted Jeep/Jalopy/APC script inspection and original `models/buggy.mdl`, `models/vehicle.mdl`, and `models/combine_apc.mdl` wheel attachment metadata. Those sources informed the existing authored wheel radii, front steering and attachment names.
- Existing Rapier contact-force events are used for crash feedback only. The new traction path uses conservative support rays and `ExternalForce`, not native tire contacts or solver parity.
- Private feedback about movement and wheel orientation was reviewed locally. The already corrected Source-to-Bevy axle basis remains unchanged; private reports and their identifiers are not redistributed.

## Drew-owned manual continuation cases

`sheets/vehicle_continuation_cases.csv` describes Drew-owned acceptance scenarios. Every row is `not_run`. Verify on flat ground, a slope, uneven ground, and while airborne. Confirm a dropped wheel contact reduces available grip instead of retaining chassis-wide traction. Check forward/reverse, braking without brake-induced reversal, low-speed steering, high-speed steering, max-speed throttle, unoccupied rolling, blocked exit, clear exit, occupied vehicle removal, passive seats, and airboat water support. Observe both wheel-spin and steering visuals without changing the established attachment axis.

## Remaining boundaries

- Ray support is not articulated suspension, tire slip curves, differential/gearbox, native friction combine, or full orientation support. Four chassis corner samples can still feel springy or hover on uneven geometry.
- Engine/wheel audio, damage, native vehicle animations/pose parameters, passenger seats/roles, native entry/exit sequences and vehicle weapons remain absent or incomplete.
- Entry and exit are approximate occupancy transitions with endpoint hull clearance, not swept native animations. Manual review must include obstruction, destruction/removal and save/restore lifecycles.
- Drew owns all tests and gameplay/visual acceptance. This worker ran no build, tests, Clippy, screenshots or input automation. The coordinator compiles the integrated production executable separately and records that result in VALIDATION.md.
