# Playable weapons and vehicles integration plan

Requested 2026-09-29. GitHub first: existing nine commits pushed through 143dd5c before gameplay edits.

## Ordered implementation, one integrated build after code review

1. Replace catalog-only execution boundary with validated per-registration runtime routes. Include all 34 weapon and 15 vehicle rows, even unsupported ones. Unsupported does not mean equip a misleading substitute. Preserve Inspect and F8 separately from primary click.
2. Generalize active weapon identity while preserving existing physgun and toolgun behavior. Separate weapon input from tools, block gameplay input during UI/focus changes, retain ammo by weapon, time firing/reload, add real ray/projectile collision and damage to supported destructible props. Keep explicit secondary-action gaps.
3. Extend original view/world weapon renderer with route-specific model and animation candidates. Failed asset loading must roll back equip, never erase current weapon or retry expensive loads every frame. Renderer can use an explicitly reported bind-pose fallback when a clip is missing. Use existing mounted MDL skeleton/hands/attachment pipeline, not new external assets.
4. Carry typed vehicle IDs through existing bounded asynchronous spawn queue. Attach vehicle behavior only after model preparation succeeds. Preserve typed identity in saves/undo/duplication and reject unknown or mismatched definitions before scene replacement.
5. Add vehicle occupancy and E entry/exit. Suppress pedestrian movement/weapon actions while occupied. Disable the passenger hull against its chassis, follow authored seat transform, restore hull safely on exit, and recover if vehicle is removed or scene changes. Reject blocked exit positions. No unrequested save overwrite.
6. Add fixed-step wheel-ray suspension/traction/steering/brake/reverse for wheeled classes and separate airboat support. Standalone seats and prisoner pod have occupancy but no engine, as in stock. Author dimensions/tuning explicitly as prototype values until native script import and user comparison are complete. Never describe common chassis control as exact Source vehicle physics.
7. Load observed mounted seated animation clips: drive_jeep_center, drive_airboat_center, @drive_pd, @sit. Blend from pedestrian pose into seated pose and interpolate entry/exit placement. Generic transition blending is not a source-authored entry/exit cinematic. Full vehicle-specific sequence/door animation/IK remains a distinct gap until found and implemented.
8. Review primary-click routes, hold-Q close, weapon switching, empty ammo/reload, vehicle spawn failures, occupancy deletion, undo/load, collision filtering, input focus and renderer failure rollback before final build. Export one new full workbook and compile one normal game executable. Correct compiler failures before packaging. Do not run tests or automation under current project instruction.
9. Update README and spreadsheet guide to current workbook and controls, record observed evidence versus not_run cases, commit and push final reviewed paths. Never force-push or publish local feedback/assets/saves.

## Existing implementation and source research

- Reuse existing Rapier ray queries and fixed scheduling, model queue, prop GPU cache, skinning, tool constraints and versioned scene snapshot pipeline.
- Installed base_vehicles.lua distinguishes three wheeled vehicles, airboat, pod and ten standalone seats. Vehicle class, script keyvalues, occupancy and animation are not implied by a model.
- Existing local mounted animation inventory contains drive_jeep_center/left/right, drive_airboat_center/left/right, @drive_pd and @sit. It does not establish native entry/exit sequence equivalence.
- Mounted model inventory confirms HL2 c_ and w_ weapon assets. Installed weapon_flechettegun.lua uses c_smg1/w_smg1, 0.1 second cadence and projectile speed 2000HU/s. Native weapon tuning not inspected in this pass must be labeled prototype authoring, not exact reference values.
- Existing private report remains preserved and reviewed. Water fidelity remains separate open work.

## Acceptance ownership and coverage

Every new runtime row has implementation scope and remaining behavior. Build success does not mark a row verified. Drew owns click/equip/fire/reload, all vehicle entry/drive/exit and animation review. Unsupported HL1/special weapons and any missing vehicle fidelity remain explicitly tracked, not quietly mapped to a generic firearm or fake driving animation.
