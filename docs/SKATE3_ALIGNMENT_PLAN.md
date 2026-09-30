# Skate 3 visual and board-contact correction plan

## Goal
Correct high-confidence presentation and lifecycle mismatches while using Skate 3 as a visual/control reference, not as a claim of exact or one-to-one reverse engineering. All numeric board geometry and suspension values remain sheet-authored.

## Evidence
- RAWG's public Skate 3 screenshot gallery shows deck/rider proximity, coherent rider/board framing and recognizable grounded/airborne/rail poses. Stills cannot establish force curves, animation timing, collision response or input windows.
- The official Xbox manual's indexed Flick-It description calls for directional/pattern input across flips, grinds, manuals and varials. Full manual retrieval failed, so frame timing and exhaustive gestures remain unverified.
- Source audit identified a wheel-height/contact-origin mismatch and blocked dismount state regression. See `RESEARCH.md` for references and calculations.

## Implemented
1. Set board suspension stroke to 0.014 m in `sheets/source_skateboard.csv`. For the 0.028 m wheel radius, place the board origin at two wheel radii plus 0.004 m so visible wheels begin 4 mm above ground.
2. Place four suspension ray origins at the wheel axles. Their length/rest distance is wheel radius plus the authored suspension stroke. This makes ray compression, wheel geometry and contact height share a reference.
3. Replace the gun-hold seated pose with the existing generic melee-hold pose. This avoids a pistol posture but does not provide skate-specific animation.
4. Keep the rider root oriented with the board and regular/goofy yaw during board rotation. This is a rigid pose approximation, not foot IK.
5. If the completed exit becomes blocked, release at the original clear mount point when available; if both positions are obstructed, keep the completed exit state and retry. Never restart entry.
6. Add Drew-owned manual cases for static wheel/deck alignment, trick/rider coupling, blocked exit retry and a normal dismount.

## Validation and boundaries
- Build the actual `source-map` binary using a D-drive Cargo target and output path. Run the real production catalog loader/workbook exporter if the project workflow includes it.
- Do not run tests, test builds, input automation, screenshots, or gameplay acceptance. All manual cases stay `Drew,not_run`.
- Drew should check spawn contact, deck height, regular/goofy stance, pop/flip/catch, and both clear and obstructed dismounts.
- One-to-one Skate 3 physics, foot locking, separate articulated rider/board dynamics, trick recognition, motion-captured animation and exact timing remain outside this change. Source stills and the incomplete manual evidence do not justify inventing these values.

## Diagram
```mermaid
flowchart LR
  CSV[Board geometry and suspension CSV] --> Catalog[Production catalog]
  Catalog --> Spawn[Wheel-bottom-aligned spawn]
  Spawn --> Rays[Axle-origin support rays]
  Rays --> Board[Board body and visible wheels]
  Board --> Rider[Rigid rider-root orientation]
  Rider --> Exit[Persistent blocked exit target]
  Exit --> Cases[Drew-owned manual acceptance]
```
