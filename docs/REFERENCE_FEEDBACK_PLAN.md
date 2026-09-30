# Reference-backed feedback integration, 2026-09-30

## Goal and evidence boundaries
Review the seven new private feedback reports without publishing their bodies. Fix the reported behavior using authored CSV data and original references. Actual original images are evidence of visible layout and pose only, not proof of timing, physics or our runtime acceptance. Drew owns all gameplay testing. No tests, screenshots, input automation or benchmark runs are authorized.

## Detailed implementation plan
1. **Tool interaction and presentation** (rat + badger): inspect balloon spawn/tether/buoyancy and button input state. Add contextual stage/action and aimed-button hints on the gameplay camera, not the offscreen toolgun screen. Compare title, description and instruction strip with Facepunch's actual Using Tools screenshots. Reuse authored tool labels/actions. Keep native button animation limitations explicit.
2. **Vehicles** (butterfly + coordinator): smooth a shared steering state for tire forces and wheel visuals, review sign and model basis from original attachments. Author per-vehicle response and engine pitch bounds in `source_vehicle_feedback.csv`. Add occupancy-owned original engine loops with cleanup on exit/removal/death/UI/focus changes. Do not claim a native gearbox or all Source vehicle effects.
3. **Footsteps** (shrimp + coordinator): preserve original VMT surface properties on imported world colliders. Author surface-to-left/right-event mappings and cadence. Use measured grounded displacement, suppress unsafe states, and keep unknown surfaces silent rather than pretending to classify them. Resolve original installed sounds through the existing bounded audio cache.
4. **Animations and swimming** (pig + coordinator): add original swim clips to animation-state sheets and select them from water immersion rather than grounded status alone. Author weapon hold families and reuse non-delta original reload clips plus a generic melee sequence through an upper-body mask. Preserve locomotion and suppress these actions in unsupported movement states. Exact native gesture layers and model-specific timing remain open; metadata availability is not runtime animation acceptance.
5. **Consolidation** (coordinator): review each worker diff and all shared integration boundaries. Validate catalogs, compile the real production executable using the private D: target, and export a new workbook. Do not replace the running executable or terminate Drew's game.
6. **Delivery**: update the feedback ledger, remaining-gap records and in-game F7 cards with concrete checks and `not_run` acceptance status. Commit only reviewed paths and publish the completed batch.

## Ownership and asynchronous coordination
Five requested Luna workers have bounded file ownership. Coordinator owns catalog/build hooks, shared audio, imported surface metadata, common records and publication. Workers report API needs before shared edits. No worker builds or commits. `git status`, relevant diffs and `git diff --check` are reviewed before consolidation.

## References actually inspected
- https://wiki.facepunch.com/gmod/Using_Tools
- https://files.facepunch.com/wiki/files/tutorial_tools_3.png (720x405): large top-left Winch title, smaller description, translucent dark instruction strip with cyan information glyph and Click on an object.
- https://files.facepunch.com/wiki/files/tutorial_tools_4.png (1280x720): same hierarchy with stage-specific Click somewhere else. Title near x50/y56 and instruction strip y152..177. Original toolgun screen and cyan beam remain separate from gameplay HUD.
- Reference downloads remain ignored under `local/reference-images/`. These are original public reference images, not screenshots of this application. No claim of pixel-perfect fonts, gradients or animation timing follows from them.

## Integration outcome
The five slices and shared hooks are integrated. The actual production executable compiled, production catalogs validated, and the complete workbook exported with 107927 data rows. F7 has six new focused cards and the five new manual-case tables contain 51 Drew-owned `not_run` rows. [Validation evidence](VALIDATION.md) records the initial build failure, repairs, final artifact and untested boundaries. No running game was terminated or overwritten.

## Acceptance gate
Production compilation and catalog validation demonstrate integration only. All new manual cases remain Drew-owned and `not_run` until he exercises the game. Full Source/Garry's Mod parity is not a deliverable claim for this bounded feedback batch.
