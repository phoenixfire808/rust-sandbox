# Contextual feedback, cursor and Source-style movement

## Delivered implementation scope

1. Research the actual public Source movement functions and installed Sandbox player class. Record sources and separate native engine uncertainty.
2. Keep acceleration, friction, air cap and jump boost in source_player.csv and generated PlayerConfig. Retain planar momentum and implement the standing Sandbox jump rule without claiming exact Source collision behavior.
3. Synchronize cursor capture after frontend/game menu updates. Closing a menu no longer requires a second capture click. While Q or another interactive UI is open, the cursor intentionally remains available.
4. Capture per-window/control feedback before opening the editor. Preserve drafts and original context, allow deliberate retargeting, save private reports and return to the previous interface.
5. Export all existing tables plus movement_reference and context_feedback_cases. Compile the actual executable without replacing or terminating a live game. Drew owns runtime acceptance.

## Giving direct context from the game

- Hover a tool setting, model, map card or menu button and press **F8**. For whole-window notes use **Feedback on this browser/panel** in Q. In gameplay F8 records the current tool, selected model and scene context.
- Enter a title and observed behavior. Expected behavior and reproduction notes are optional. Tab changes field and Enter adds a line. Category is saved with the draft.
- An existing unsent draft retains its original context. **Attach current context** explicitly replaces it with the snapshot from this opening of the editor.
- Choose **Save feedback locally**, then F8/Esc or Return to go back. Reports live in ignored `local/feedback/*.json`. Ask Jcode **review my local feedback** in the terminal to request a review. The game does not upload reports or contact an agent automatically.
- Text editing is currently append/backspace/multiline, not a full native text editor with selection/clipboard/IME. Arbitrary addon windows are not supported because addon UI itself is not implemented.

## Movement expectations and limits

Fresh Space presses jump. Holding Space is not auto-bhop. Ground acceleration/friction, air-strafe projection and bounded standing Sandbox boost now replace direct planar velocity assignment. Numerical constants are source-derived baselines rather than measured trajectories. Crouch and its reduced boost, Source hull clipping/steps, surfing, water/ladder/platform behavior, slope/surface handling, ceiling response and command timing remain open. See movement_reference.csv for individual acceptance cases. No gameplay tests or visual comparisons were performed by the agent.

## Remaining broader parity

The 40 menu work packages and 169 detailed parity gaps remain the governing backlog. This change does not complete native Options, new-game effects, all Q/C panels, stock tool semantics, networking, Workshop, Lua/addons or Source rendering/physics. Existing reference inventories remain intact. A compiled implementation is not acceptance of one-to-one appearance or behavior.
