# Research record

## 2026-09-29 entity and creation-menu discovery

Read installed Facepunch stock NPC/vehicle/item registrations, Sandbox spawn commands, creation/content/context menus, content-icon implementation and scripted entity/weapon declarations. Reused their metadata and documented semantics, not executable Lua. The reproducible read-only extractor is `scripts/catalog-spawn-reference.ps1`. It produced 197 spawn definitions, 1,406 literal menu-control records and 178 source hashes. Existing HTML/CSS/native-menu research remains intact. Detailed findings, bounds and the next vehicle/NPC implementation sequence are in [ENTITY_MENU_SOURCE_OF_TRUTH.md](ENTITY_MENU_SOURCE_OF_TRUTH.md). The sources are the installed counterparts of <https://github.com/Facepunch/garrysmod>. Discovery does not demonstrate native behavior or pixel parity, and conditional mount availability is not evaluated.

Research date: 2026-09-29 UTC. Citation IDs are maintained in `sheets/sources.csv`.

## Reported running drift: correction to the preceding movement pass

Drew reported drifting after the context/movement build. The earlier pass incorrectly selected **generic SDK friction 4** as the GMod runtime baseline. Read installed `lua/autorun/utilities_menu.lua:8-14,21-40,57-61`: the authored GMod server preset has **sv_friction 8**, gravity 600 and sticktoground 1. LoadInConvarDefaults can replace these with engine GetDefault values, so this is direct installed preset evidence, not a dump of live native convars. The spreadsheet now uses 8. This is a correction to our implementation, not a request for Drew to compensate by adjusting controls. Source R14 records the installed/public definition.

Expanded the R12 read to gamemovement.cpp Friction (1610-1677), Accelerate (1820-1852), StayOnGround (1857-1888), WalkMove (1893-2018), FullWalkMove (2023 onward), CheckJumpButton (2352-2527), TryPlayerMove (2550 onward), ClipVelocity (3144-3187) and CategorizePosition (3794-3913). Grounded movement clears vertical velocity before friction and walking and after final gravity. WalkMove stops speeds below 1 HU/s. Position traces and steps do not redefine velocity as displacement/dt. Collision clipping modifies inward velocity components and handles multiple blocking planes separately. Ground categorization uses a 2 HU probe and normal >=0.7. Native step/hull/quadrant tracing remains different from Rapier.

Read installed bevy_rapier3d 0.30.0 `plugin/systems/character_controller.rs` and `plugin/plugin.rs`, plus rapier3d 0.25.1 `control/character_controller.rs` (R15). SyncBackend runs the character controller before transform propagation and body synchronization; output is refreshed and translation consumed there. Effective translation includes normal nudges, stair/snap corrections and possible kinematic-platform translation. It is not final physical velocity. The old feedback path divided that displacement by dt and persisted it as momentum. The revised pre/post-physics phases preserve velocity, clip against blocking contact planes, clear landing/ceiling velocity appropriately, and apply Sandbox FinishMove boost after contacts. Actual displacement remains animation/diagnostic data only.

Both Bevy fixed time and Rapier are already explicitly 1/60s, so no timestep mismatch was found and the tick rate was not changed. Airborne autostep is now disabled. Ground support probing is kept separate from gravitational velocity, using the sheet-authored 2 HU distance with Rapier snapping. Ramp/stair/corner and broad Rapier grounded semantics still require real acceptance. The exact changes, equations, remaining differences and unrun regression cases are in [MOVEMENT_REPAIR.md](MOVEMENT_REPAIR.md). No feedback reports existed on inspection, no active game was terminated, and no tests or automated gameplay were performed.

## Source movement and contextual feedback, 2026-09-29

Read Valve Source SDK 2013 `src/game/shared/gamemovement.cpp` (Friction, AirAccelerate, FullWalkMove and CheckJumpButton), `gamemovement.h` and `movevars_shared.cpp` from the public Valve repository (R12). Ground friction uses max(speed, stop speed); air acceleration limits velocity projected onto the wish direction to 30 Hammer units/s but uses uncapped wish speed in the acceleration term. A successful fresh jump precedes ground friction. Held Space alone is not automatic bunny hopping. Gravity uses half steps around movement. SDK baseline values are acceleration 10, air acceleration 10, friction 4 and stop speed 100 HU/s. These are reference baselines, not measurements of the installed GMod native binary. The Valve Developer Wiki Bunnyhop page returned an anti-bot response and was not used as read evidence.

Read the installed `gamemodes/base/gamemode/player_class/player_default.lua` and `gamemodes/sandbox/gamemode/player_class/player_sandbox.lua` without modifying Steam (R13). Sandbox specifies walk 200/run 400/slow 100 HU/s and inherits jump power 200. Its StartMove/FinishMove jump boost uses standing fraction 0.5 (crouched 0.1), deliberately preserves sprint boost, bounds horizontal speed against moveMaxSpeed*(1+fraction), and reverses the addition for backward travel. This differs from treating bunny hopping as unlimited forward acceleration. Existing scale 0.01905 m/HU yields walk 3.81, run 7.62, jump 3.81, stop speed 1.905 and air projection cap 0.5715 in SI units.

The independent Rust controller now retains collision-resolved planar momentum, applies the researched acceleration/friction rules, uses fresh-press jumps and a pending post-move Sandbox boost. Authored constants remain in source_player.csv. Rapier capsule/contact/step behavior is not Source hull sliding. Crouch, water, ladders, surf, moving platforms, ceiling response, surface friction, exact command timing and native frame-rate parity remain incomplete or unverified. movement_reference.csv separates each implemented rule from remaining work and Drew-owned acceptance.

Cursor inspection found that menu closure previously depended on another click to recapture. A single final cursor synchronization now derives capture from window focus and actual UI state. Contextual feedback reuses Bevy RelativeCursorPosition, existing UI actions and serde JSON. F8 or browser/tool-panel feedback captures context before the editor opens, preserves unsent-draft context, and returns to the originating page/menu. No new service, dependency, upload, agent loop or screenshot capture was added. Reports stay private in ignored local/feedback. No reports were present when reviewed for this request. See CONTEXT_MOVEMENT_PLAN.md.

## Nonworking menu repair and exact layout inventory, 2026-09-29

Drew reported that the previous menu did not work and did not match stock placement. Read installed `html/template/main.html`, `newgame.html`, `menu.html`, `css/menu/PageOptions.css`, `NavBar.css`, `NewGame.css`, `Menu.css`, and `js/menu/control.NewGame.js`. Inspected the actual 288x128 Sandbox logo and one installed 1920x1080 PNG background. Stock main text starts at CSS x=50+20+16=86, not the prior 16px sidebar. Stock footer is 50px high. New Game has an independently anchored white map list and a 226px settings panel. The 46 rules in menu_reference.csv retain locations, selectors, responsive conditions and unimplemented boundaries.

Inspected Bevy 0.16.1 `ui_node.rs` and `focus.rs:298-323`: Text and ImageNode require Node, whose FocusPolicy defaults to Block. The previous button children had no Pass policy, so label/image hits stop before their owning Button. Explicit pass-through decorations reuse the framework's actual hit-test system, not a separate rectangle-click engine. A changed-Interaction collector emits actions only on a fresh physical left press. Q-menu labels and model thumbnails receive the same correction. This diagnosis is from source inspection, not a gameplay test.

The view reuses read-only mounted original PNGs and Windows Arial/Arial Bold (CSS fallback), with existing font defaults if unavailable. No artwork or fonts are copied into the repository. Background choice is authored and aspect-filled, not the native rotating background implementation. Native text letter spacing, blurred shadows, gradients and complete subpages still require work and Drew's comparison.

`scripts/catalog-menu-reference.ps1` additionally inventories **140 literal controls** from the seven installed loose Options*.res files, preserving raw position/size/visibility/tab fields, and **30 Sandbox new-game settings** from sandbox.txt. These are metadata, not copied implementation source. Extraction is lexical for flat blocks and does not resolve inherited controls, proportional coordinates, archive-only resources or dynamic engine values. Native options are no longer an unenumerated generic row, but this is not exhaustive native engine discovery. See MENU_REPAIR_PLAN.md.

## Expanded menu audit and follow-through, 2026-09-29 19:28 UTC

Re-read installed PageOptions.css, NavBar.css, NewGame.css, menu.html and newgame.html. The expanded inventory records 368 literal HTML controls/labels and 1138 CSS layout/style declarations across menu templates including nested creations and all HTML CSS subdirectories, in addition to 140 native controls and 30 Sandbox options. This is an exhaustive pass over the scanner's stated literal files and properties, not complete dynamic native/Lua coverage. Quoted comparison operators are retained while HTML comments are excluded. Source lines are kept to resolve media conditions and inherited/repeated control ancestors.

Corrected missing Problems/Games footer labels and the <=940px compact-label/15px-center-margin rule. Added page-preserving Games/Language/Gamemodes popup anchors from NavBar.css, with explicit unsupported backend notices and outside-click/Escape dismissal. English and Sandbox are the only active choices, not invented compatibility. Reference NewGame.css:7-28 specifies a normal favorite star visible on map hover or saved favorites, changing to add/remove only when hovering the star. The independent view now follows those states using Bevy RelativeCursorPosition and mounted images. No new package, third-party source copy, screenshot or input automation was introduced. No local feedback directory existed at review time.

## Startup, pause, feedback and spawn placement, 2026-09-29

Read the installed `html/css/menu/Menu.css`, `html/css/menu/NewGame.css` and `resource/localization/en/main_menu.properties` under the unmodified Garry's Mod installation. NewGame.css specifies 128px thumbnails, 6px card padding, 2px card margin, 16px outer inset, 190px controls and a 226px game-settings region. Menu.css specifies the Helvetica/Arial family, 32px category titles and a 50px footer exclusion. These observations are references, not proof that the current independent Bevy layout matches them. `source_frontend.csv` authors the reused dimensions alongside independently chosen font size, row height, feedback limit and spawn clearance. Stock typography, map images, footer and full options remain pending.

Reused existing Rapier surface traces and cached decoded model vertices for support-plane spawn placement, existing local scene serialization for menu saves, Bevy UI/input/virtual time, and standard-library child processes for startup/map handoff. No new package or proprietary payload was introduced. Feedback reports are local JSON in ignored `local/feedback`, not an upload or automatic agent turn. See FRONTEND_PLAN.md and frontend_cases.csv for precise implementation and open requirements.

## Toolgun and stock tools, 2026-09-29

Drew requested firing animation, a selected-tool screen, and complete tools/settings. The detailed execution plan and explicit unfinished scope are in [TOOL_IMPLEMENTATION_PLAN.md](TOOL_IMPLEMENTATION_PLAN.md). No claim of full tool or Source parity is made.

Read the installed/public `gmod_tool/shared.lua`, `cl_viewscreen.lua`, `cl_init.lua`, and the axis, ballsocket, colour, elastic, material, nocollide, physprop, remover, rope, slider and weld stool callback bodies. Public root: <https://github.com/Facepunch/garrysmod/tree/master/garrysmod/gamemodes/sandbox/entities/weapons/gmod_tool>. Successful tool callback results trigger the original firing sequence/effects, rather than every input click. Stock hold type is revolver, and stock effects include ToolTracer, selection_indicator and Toolgun.Single. Our first-person fire clip is wired, but the original sound, exact effects and third-person gesture remain missing.

The installed screen code specifies the `models/weapons/v_toolgun/screen` surface, screen_bg background, 256x256 target, 60px Helvetica weight 900, text center y=104, 250px/s scrolling and 64px gap. Current Bevy text uses its default font and lacks the original shadow, so source-derived dimensions do not establish pixel equivalence. Tool-specific DrawToolScreen overrides are not implemented. Read-only original model metadata identifies `@fire01` as a 20-frame toolgun animation. This metadata inspection is not a gameplay firing test.

`scripts/catalog-tool-settings.ps1` inventories 170 literal ClientConVar defaults from the installed loose stools without copying Lua source. It does not evaluate Lua, dynamic registrations or full panel definitions. Missing-tool action descriptions in source_tools are planning summaries, not fully verified callback specifications. The 39 authored runtime settings distinguish enabled operations from pending options. Material ranges and Rapier physical coefficients are independently authored approximations, not measured Source values.

Reused Bevy's version-matched UI render-target approach: <https://github.com/bevyengine/bevy/blob/v0.16.1/examples/ui/render_ui_to_texture.rs>. Reused the existing original model/clip decoder, mounted material/texture path, effect pool, UI controls, Rapier GenericJoint/ImpulseJoint child-body support and serde scene serialization. Rapier API source inspected from installed bevy_rapier3d 0.30.0 (`dynamics/joint.rs`, `generic_joint.rs`, `rope_joint.rs`, `spring_joint.rs`). No new package or proprietary payload was added. Tool physics, save documents and dispatch are independent Rust implementations, not copied Lua or native compatibility.

## Spawn responsiveness repair, 2026-09-29

Drew reported continuing lag, especially spawning an item. Static inspection found unoptimized engine/physics/decoder dependencies in the dev build, synchronous model/VTF/mipmap/convex-hull preparation inside the exclusive game update, deep geometry copies on every model-cache hit, idle aim queries, and unchanged HUD/projection/config writes. These are observed code paths, not measured proportions of frame cost.

Reused Bevy's documented optimized dependency profile (`[profile.dev.package."*"] opt-level = 3`), its existing `AsyncComputeTaskPool`, `Task`, and `block_on(poll_once(...))` completion pattern, and standard-library `Arc` for immutable mounted assets and cached geometry. Sources: <https://bevy.org/learn/quick-start/getting-started/setup/> and the version-matched example <https://github.com/bevyengine/bevy/blob/v0.16.1/examples/async_tasks/async_compute.rs>. No new dependency or agent worker was added. One bounded game-internal preparation task avoids concurrent decode storms. Exact duplicate hull input points are removed, not simplified into a different collision shape. Physics tick rate, mass, gravity, CCD and render detail are unchanged.

`source_performance.csv` supplies queue bounds and passive diagnostic settings through the normal typed build pipeline. `performance_cases.csv` records scenario-level implementation and remaining acceptance. GPU upload, icon loading, CPU skinning and original collision-solid parity remain explicit boundaries. See [PERFORMANCE_PLAN.md](PERFORMANCE_PLAN.md). No measured speedup or complete one-to-one parity is claimed.

## Reference-driven presentation repair, 2026-09-29

Drew's play feedback identified incorrect walking/jumping, missing beam and nonmatching menu layout. Read Facepunch's public `gamemodes/base/gamemode/animations.lua`, Sandbox `spawnmenu/spawnmenu.lua`, `creationmenu.lua`, `toolpanel.lua`, `creationmenu/content/content.lua`, `lua/vgui/spawnicon.lua`, and the `GM:DrawPhysgunBeam` wiki page. Exact URLs, observed rules, approximation labels and remaining work are in `sheets/presentation_references.csv`. The implementation plan is [PRESENTATION_PLAN.md](PRESENTATION_PLAN.md).

Reused existing Bevy `RelativeCursorPosition`, `ScrollPosition`, flex layout and `StandardMaterial`, Rapier's ray query and actual controller displacement, and the project's original-asset skeleton/clip decoder. No additional packages were introduced and no Valve/Facepunch implementation source was copied. Clip identifiers were taken from the previously inspected local animation metadata. The new runtime sheets are `source_layout.csv` and `source_animation_states.csv`.

The old animation path selected forward clips from global elapsed time regardless of airborne state. The old effects path required a valid held entity before drawing any beam. The old menu used unrelated 3%/5%/94%/82% placement and a 145px card grid. These code-level findings explain specific reported mismatches, but the corrected version remains subject to Drew's visual testing. Source activity layers, pose parameters and original skin rendering are still incomplete.

## Observed local baseline

- Steam app 4000 installation found through `libraryfolders.vdf` at `D:\SteamLibrary\steamapps\common\GarrysMod`.
- Manifest build ID 25375506. The installation was read only. No game files, saves, authentication or Steam settings were changed.
- First successful inventory: 2,904 loose files, 78,051 VPK entries across nine directory archives, 5,792 lexical Lua declaration candidates, 37 direct stock tool Lua files, zero skipped filesystem links.
- Combined asset records: **80,955**, not 80,955 unique artistic assets. Companion files, archive containers and duplicated virtual paths are separate records.
- Local gamemode directories: `base`, `sandbox`, `terrortown`. This does not mean those modes are implemented here.
- Tool declarations were inspected for categories, callback names and `TOOL.ClientConVar` defaults. High-level semantics in `tools.csv` remain summaries, not full line-by-line verified specifications.

## Public source scope

The Facepunch repository README says it contains Lua/text/config extensions and that binary sources are not public. Valve Source SDK 2013 README identifies HL2, HL2DM and TF2 game code. Its license grants specific Source 1 mod-related rights and distribution conditions, not unrestricted ownership of all code or assets. `gmod-module-base` exposes a native module interface. `garrysmod_common` is a community module-building utility, not the full engine.

These are useful research references. No Valve/Facepunch implementation was copied or mechanically translated into this repository. The installed source inventory remains in ignored local output. Reference behavior is independently approximated where implemented.

## Reuse decisions

The original installed-content pipeline and additional decoder research are recorded in [SOURCE_ASSETS.md](SOURCE_ASSETS.md). This replaces the earlier procedural-only visual target, not the preserved testbed or its tests.

1. **Bevy**, requested by Drew, supplies ECS, input, windowing, UI and rendering. Version 0.16.1 was selected with a compatible physics plugin, not because it was assumed newest.
2. **bevy_rapier3d 0.30.0** manifest explicitly targets Bevy 0.16. Its fixed-schedule API is used. This is an existing solver, not a new hand-written physics engine. The project has since moved upstream to the main Rapier repository, so future upgrades should consult that location.
3. **csv / serde / serde_json** handle typed authoring and persistence. Do not implement ad-hoc CSV parsing or silently accept invalid physical values.
4. **rust_xlsxwriter** creates the Excel review snapshot. It is optional so the core game build need not use the workbook feature.
5. **ValvePython/vpk** was consulted for directory header and entry layouts. Our bounded metadata-only parser is independently written. It supports directory versions 1/2 and does not extract, execute or decode payloads. The Valve Developer Wiki format page returned a bot-protection page, so it was not claimed as successfully inspected.
6. Macroquad was initially investigated before Drew requested Bevy. It is not a project dependency or implementation component.

Bevy uses MIT/Apache-2.0 licensing, Rapier uses Apache-2.0, and the serialization/workbook crates have their own permissive terms. Exact dependency versions are locked in Cargo.lock. Review dependency notices and asset rights before any distribution. No broad license over third-party content is asserted by this project.

## Physgun presentation research, 2026-09-29

- Read installed `materials/sprites/physbeam.vmt` and `materials/sprites/blueflare1_noz_gmod.vmt` through the existing read-only mount. The former references `sprites/physbeam_white`; the latter requests additive sprite rendering. Diagnostic text stays in ignored `local/effects-research.log`.
- Inspected original `models/weapons/w_physics.mdl` metadata. Its `core` and `fork*t` attachments provide effect positions. The viewmodel's available muzzle/fork attachments are transformed by the same sampled skeleton as its mesh. This reuses the existing MDL decoder and coordinate conversion rather than introducing a second asset pipeline.
- Reused Bevy 0.16 `StandardMaterial` additive blending, billboard quad meshes, render layers and mutable mesh assets. A fixed pool holds four weapon glow slots, one endpoint and one ribbon. Beam UVs scroll on the existing mesh instead of allocating a new GPU asset every frame.
- `sheets/source_effects.csv` authors texture references, color, widths, pulse and scroll parameters. These are independently chosen presentation parameters, not measured GMod constants. The beam's viewmodel start is adjusted for the separate world and viewmodel camera FOVs.
- No native renderer code was copied. Claw transitions, sound, configurable player color, exact grab anchors, illumination and paired original-game visual acceptance remain open. An additive sprite does not establish that the weapon illuminates nearby geometry.

```mermaid
flowchart LR
    CSV[source_effects.csv] --> Validate[Catalog validation]
    Validate --> Build[Build-generated Rust config]
    Build --> FX[Bevy effect pool]
    Mount[Read-only installed VTF textures] --> FX
    Pose[Sampled weapon attachments] --> FX
    Hold[Real pickup and release state] --> FX
    FX --> Views[World and viewmodel render layers]
```

## What has not been researched to completion

No full native engine implementation, exhaustive Lua API semantics, every game convar, all model/material formats, all stock map I/O entities, Workshop corpus, mounted game corpus, weapon/NPC stat corpus or numerical Source physics baseline has been reconstructed. The 64-system matrix is a discovery map that must be expanded into concrete reference cases during M2. The current runnable prototype is evidence for its own behavior only.

## Player and map repair research, 2026-09-29

- Inspected installed dependency sources for `vmdl 0.2.0`: `src/lib.rs`, `src/vvd/{mod,raw}.rs`, `src/mdl/raw/{header,bones,animation,mod}.rs`, and `src/compressed_vector.rs`. Geometry, VVD fixups and material lookup remain reused. Observed that its weight iterator divides by influence count, external animation blocks are unimplemented, compressed samples are unsigned, and frame indices narrow to u8.
- Added independently implemented bounded skeletal/selected-clip sampling using those format layouts. It preserves signed i16 runs, frame indices above 255, raw quaternion formats, external ANI blocks, section tables, bind matrices and attachment matrices. Full Source sequence layering, IK, flexes and procedural bones remain outside this implementation.
- Original local metadata shows Kleiner's player model contains its ragdoll animation, while stock locomotion and hold clips live in `models/m_anm.mdl` and its ANI blocks. Actual idle/walk/run physgun and pistol clips are used, not synthesized walking poses.
- Reused Rapier 0.30's KinematicCharacterController, autostep and grounded output. Its capsule/sliding behavior is not Source's box-hull movement solver. Models and movement dimensions are authored in `sheets/source_player.csv`.
- Inspected `vbsp 0.9.1` face triangulation and normals. Final visual review disproved the earlier face-side correction: the actual indexed planes already carry signed normals, so multiplying by `face.side` again inverted negative-axis surfaces and hid the white-room ceiling. Removed the duplicate flip. A real installed-map regression first failed with `ceiling faces away from room`. Local gm_construct has 12 non-world brush submodels, including interior func_brush walls, reflective glass, vehicle clips and illusionary surfaces. Diagnostic vertices confirm visible submodels use entity-local coordinates. Rendering only model 0 omitted this geometry.
- Original hands are bone-merged by name. World weapons use the authored `anim_attachment_RH` matrix, including its local orientation. Visual review caught the initial bone-only attachment pointing the gun down.
- Teeth/Eyes shaders are missing from vmt-parser. Their base textures now load through a VertexLitGeneric approximation, not equivalent lighting or eye animation.
- Full fidelity acceptance inventory is in [PLAYER_PARITY.md](PLAYER_PARITY.md). Proprietary payloads and diagnostic dumps remain local/ignored.
