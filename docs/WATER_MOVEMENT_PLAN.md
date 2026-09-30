# Water movement continuation

## Research and evidence

- A private user feedback report requested additional water-physics research. It was reviewed locally and remains private; it is treated as a prioritization signal, not an implementation specification.
- The [Garry's Mod Entity:WaterLevel reference](https://wiki.facepunch.com/gmod/Entity:WaterLevel) documents levels 0 (not in water), 1 (feet), 2 (waist), and 3 (fully submerged). This establishes the observable level categories, not Source's classification algorithm or movement forces.
- Existing `source_assets::WaterSurfaces` exposes rendered horizontal triangle surfaces and a height query. The actual playable `source-map` setup inserts this resource from above-water rendered triangles. Those triangles are not BSP contents volumes, and can't identify water depth when stacked boundaries, tunnels, blockers, or unseen volumes are involved.
- Current player movement already keeps separate horizontal and vertical velocity, applies gravity, and submits Rapier character-controller motion. The scoped addition uses these existing paths and sheet-authored swim speed, drag, gravity scale and upward speed. These values are independent prototype choices, not Source constants or measured equivalents.

## Implemented subset

- Sample feet, waist and eye positions using the active standing/crouch hull and interpolated eye height to expose a 0-3 `PlayerState.water_level`. Swimming uses the waist-level threshold (level 2+); feet-only level 1 keeps ordinary ground speed and jumping.
- Before accepting the rendered surface as a water column, require an unoccluded vertical ray to the surface and a solid bottom within sheet-authored `swim_max_depth`, with the player hull above that floor. This rejects obvious solid-overhead columns and positions below the floor. It remains an approximation based on available Rapier colliders.
- In detected swimming, use tuned horizontal speed and exponential drag. Space applies upward swim velocity and Control applies downward swim velocity. Ground snap/autostep are disabled while swimming, and both half-steps of gravity use the configured scale unless vertical input is held.
- WASD swimming remains yaw-horizontal. Looking up or down changes aim/camera but does not incline the swim direction; pitch-driven swim direction is not implemented.
- Noclip bypasses water classification and retains its existing full 3D movement. Dry-ground friction, ground acceleration, ordinary air movement, crouch and jump paths remain selected when water level is 0.
- Tuning is authored in `sheets/source_player.csv`, validated by catalog bounds, and compiled through the existing generated-player-config pipeline.

## Known gaps and manual acceptance

- This is still surface-derived classification, not true BSP contents. Ray checks can reject obvious occlusion, absent/too-distant floors, and positions below an observed floor, but cannot prove that a point above a floor is water. It cannot represent all underwater tunnels, water blockers, overlapping volumes, or native swim/launch behavior. Do not claim Source physics equivalence.
- There is no underwater fog, post-processing, audio, water entry/exit effect, drowning, swimming animation, buoyancy model or native jump-out impulse curve here.
- `sheets/water_movement_cases.csv` lists Drew-owned manual cases. They have not been run. Drew owns all tests and gameplay acceptance.
