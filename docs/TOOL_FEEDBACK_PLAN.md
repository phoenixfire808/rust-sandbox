# Tool feedback continuation plan

## Feedback and scope

Private feedback says construction buttons should animate on use, the tools should show a small use guide, and the balloon tool did not work. This slice fixes the balloon's authored rope/gravity setup and adds context-sensitive help text. It does not claim complete Garry's Mod tool or physics parity.

## Original behavior research

Read-only inspection of the installed sandbox scripts found:

- `gamemodes/sandbox/entities/weapons/gmod_tool/stools/balloon.lua` creates the balloon and calls `constraint.Rope` with the configured rope length. Its free-balloon right-click path skips the rope.
- `entities/entities/gmod_balloon.lua` disables gravity, wakes physics, then adds its configured upward force in `PhysicsSimulate`. This is distinct from the ordinary gravity-affected prop path.
- `stools/button.lua` and `entities/entities/gmod_button.lua` implement toggle/momentary use and drive the model's `switch` pose parameter while changing state.
- The installed default button model is `models/maxofs2d/button_05.mdl`. The current renderer loads static geometry and does not evaluate model pose parameters, so native visible switch travel is not implemented here.

Installed files were only read. No proprietary assets were copied or modified.

## Implemented behavior

- Balloon entities now persist `Properties.gravity = false` and use `GravityScale(0)`, while their existing authored upward-force behavior remains active. A later PhysProp gravity operation can still update the persisted property and physics scale normally.
- Balloon rope length is treated as the configured total length instead of adding the initial anchor separation again. Free balloons remain unattached.
- The device input system caches a button only when it is actually aimed at, in range, and normal device use is available. Blocked/unfocused input clears that cached target, and momentary buttons release on those transitions while toggle state remains latched.
- The read-only HUD hint shows tool instructions only with the Tool Gun active, prioritizes the second-target instruction for staged constraints, and adds aimed-button use/output/state guidance regardless of selected tool or active weapon. The HUD integration owner is responsible for hiding the panel for frontend/menu/death/focus/remote-view/vehicle states.

## Manual acceptance cases

`sheets/tool_feedback_cases.csv` records eight Drew-owned cases, all `not_run`: balloon tether to world and prop, free balloon, reconfiguration, toggle and momentary buttons, cross-weapon targeted-button guidance, and the button switch pose limitation. Drew performs runtime checks.

## Known limitations

- The button's functional switch state and generated control output change, but its original `switch` pose parameter cannot currently animate in the static model rendering path. Do not describe this as visually animated.
- Balloon buoyancy is the project's existing force approximation, not Source `PhysicsSimulate` equivalence.
- No tests, compilation, Clippy, benchmarks, screenshots, input automation, or gameplay checks were run. Consolidated build and all manual verification remain with the coordinator and Drew.
