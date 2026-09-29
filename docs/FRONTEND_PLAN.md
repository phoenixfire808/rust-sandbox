# Spawn, feedback and frontend implementation plan

Requested 2026-09-29. Drew owns all testing. The target remains one-to-one reference behavior, but this delivery is a functional slice, not complete parity.

## Execution and evidence

1. **Placement:** use the existing camera/physics trace. After asynchronous geometry preparation, project every cached model vertex on the hit normal and offset the model origin so its minimum support lies 0.025m outside the hit plane. Preserve bounded spawning and Z cancellation. This corrects the origin-at-ground failure mechanism. Native PHY solids, corner overlap resolution, moving hit targets and exact stock spawn rules remain open.
2. **Feedback:** expose F8 and a visible feedback button. Capture category, title, observed behavior, expected behavior and reproduction notes. Retain a local draft while editing. Save each submitted report to a unique synchronized JSON file under ignored `local/feedback`, with map, eye position, selected tool, prop count and current status. No upload, agent worker or recurring turn. Future requested development reads this inbox without deleting originals. Append/backspace/newline/Tab editing is implemented, not a full native text editor.
3. **Frontend:** no-argument launch opens a lightweight main menu before decoding a map. Start New Game lists authored supported maps. Escape opens a separate pause menu rather than the Q browser. Resume, options, local scenes, detailed feedback, disconnect and quit have explicit routes. Quit/F10/map change/disconnect offer scene saving or cancellation. Gameplay input and virtual time are blocked while the modal is open. Queued spawns are not committed during pause.
4. **Session transitions:** start a normal child game process using the selected map and project working directory. Keep the old window/scene until a unique local ready marker is emitted by the child after three updates. Child startup errors retain the parent and point to logs. This is not a GPU-render verification or a timeout/recovery guarantee. Native window close still follows OS behavior.
5. **Authored data:** source_frontend.csv feeds the catalog and normal build-generated typed configuration. frontend_cases.csv tracks 38 acceptance scenarios and unfinished work. Existing parity_gaps and work_queue remain the overall scope. No passing gameplay result is inferred from compilation.
6. **Handoff:** build the actual source-map executable, export a new workbook snapshot, review scoped changes and commit. Launch normally for Drew, never terminate an active scene, and inspect only startup logs. No automated tests, screenshots, benchmarks, Clippy or input automation.

## Ordered remaining frontend work

- Compare Drew's placement and input reports first, including off-center barrels/furniture, slopes, pause/resume, focused feedback and map handoff.
- Match original mounted map thumbnails, backgrounds, fonts, hover/selection colors, menu positioning, footer and scrollbars at reference resolution.
- Add map category hierarchy, search, favorites and supported map discovery.
- Inventory and implement every Sandbox new-game option with actual runtime effect, limits and persistence. Do not expose no-op controls.
- Complete startup and in-game options, key bindings, graphics/audio apply/revert and reset semantics.
- Complete local save selection/thumbnails and safe map association before any native/Workshop save compatibility claim.
- Add robust loading cancellation/timeouts and detailed errors while preserving the old scene.
- Extend text editing, draft crash safety and saved feedback browsing. Feedback remains local unless Drew requests sharing.
- Multiplayer, addons, demos, dupes, mounts and localization depend on genuine underlying runtime support. Labels are explicitly pending, not fake functional buttons.

Reference CSS measurements are recorded in RESEARCH.md. Exact stock appearance, complete settings and all tools remain unfinished. `not_run` acceptance rows are intentional and must only change based on Drew's reported testing.
