# Reference-driven player, physgun and Q-menu correction

Drew owns all testing. Do not run test suites or smoke automation. Compile the implementation and launch the normal game for him. Public-source research and static review are not runtime acceptance.

## Reference and authoring contract

`presentation_references.csv` distinguishes public-source rules, installed metadata and independent approximations. `source_layout.csv` compiles numeric geometry and presentation parameters. `source_animation_states.csv` maps each implemented hold type to original installed clips. No Steam payload is copied into the repository.

## Ordered implementation

1. **Player presentation:** consume actual Rapier movement instead of requested keys. Select among all eight authored walk/run directions relative to facing. Keep normalized gait phase when changing direction. Enter a non-looping jump clip on takeoff, hold its last frame while airborne, and blend back to grounded locomotion. Crossfade changes rather than sampling every state from the application's global uptime. Keep the same sampled skeleton for the visible body and weapon attachment. Unsupported advanced sequence layers remain explicit gaps.
2. **Physgun:** separate firing from holding. Render a beam on an empty-space/world trace while left mouse is held, not only when pickup succeeds. For held props retain the initial local grab anchor and use it both for hold targeting and the endpoint. Exclude the player's collider from traces, stop on world obstruction, and clear held/effect state on menu entry, release, focus loss and weapon change. Keep first-person FOV correction and world-weapon attachments.
3. **Q menu:** use stock screen-margin formulas, a left creation sheet, 192px browser, central 64px icon flow and simultaneously visible right tool area. Split the tools into a 130px list and control panel. Add scrolling instead of clipped category rows. Use hold-Q by default and isolate focused search from game controls. Keep unsupported creation types and tools visibly unavailable, never pretend placeholders implement them. Retain scene persistence as a clearly labeled independent option rather than presenting it as GMod-compatible saves.
4. **Handoff:** record what changed and what remains in the existing gap inventory/work queue. Build only, fix compiler errors, commit reviewed paths, and start `play.cmd` for Drew. Do not claim visual parity until he reports it.

## Placement examples

At 1280x800 the stock margin formula yields 25.6px left/right and 25px top/bottom. The available width is 1228.8px. A 460px tool region plus 6px divider leaves 762.8px for the creation sheet. Its 192px browser plus 6px divider leaves 564.8px before panel padding for icons. At resolutions below 1024x768 margins are zero and the narrow layout uses 300px tools and 100px browser with bounded shrink behavior.

## Still not equivalent

Stock spawnlist files/tree persistence, icon generation, exact GWEN skin/fonts, complete creation tabs and tools, animation pose parameters/IK/landing gestures, crouch/swim/airwalk, sound, claw animation, exact Source physics and reference-derived beam constants remain open. This implements the reported visible slice, not the entire exhaustive backlog.
