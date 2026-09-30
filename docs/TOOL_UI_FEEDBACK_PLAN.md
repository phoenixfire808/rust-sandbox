# Tool UI feedback plan

## Request and reference

Private feedback asked for visible click response and a small usage popup like the original game, and reported that the balloon tool appeared not to work. The private report remains local and is not included in source control.

Coordinator reviewed the official Facepunch tutorial image `https://files.facepunch.com/wiki/files/tutorial_tools_4.png` (1280x720). The reference shows a top-left help card with a large tool title, description, and a dark instruction strip. The cyan glyph is an information icon, not a mouse glyph. Other tutorial images vary the instruction text by interaction stage. The implementation follows this content hierarchy and placement as a reference, without claiming pixel parity or the original custom font.

## Implementation

- Retain existing menu pressed/hover color states as interaction feedback.
- Create the HUD card independently of the lazily-created offscreen weapon display, target the gameplay `SourceCamera`, and keep its lifecycle/rendering separate from that display.
- Scale card typography and inset from viewport height using `source_layout.csv`'s `reference_height` as the authored baseline. At the 1280x720 reference size the title is approximately 70px, description 20px, left inset 50px, top 40px; scale is clamped. The card begins below the FPS text row and leaves the FPS readout visible above its background.
- Use the authored `ToolDef.label` as title and `ToolDef.left` as concise description. Use the device worker's dynamic help API for the changing instruction strip, including stage-aware action guidance and aimed/in-range button key/toggle hints.
- Render an information icon in the strip, not a mouse glyph. Do not claim original-font or pixel-identical reproduction.
- Keep wording sourced from authored tool definitions and device behavior, not hard-coded guesses in the renderer.
- Balloon placement/removal implementation belongs to the device worker, separate from what the help text claims.

## Verification

`menu_feedback_cases.csv` records Drew's manual acceptance cases. Status remains `not_run`; no tests, builds, launches, screenshots, or input automation are run by this worker. `source_tool_screen.rs` owns the HUD card and keeps it independent of offscreen screen initialization. The `super::tools::help_hint` implementation and aimed-button snapshot remain owned by the device worker. Validate API integration during normal project build, then Drew performs the in-game checks.
