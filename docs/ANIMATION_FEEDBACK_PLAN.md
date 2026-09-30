# Animation feedback implementation notes

## Feedback themes

The reviewed private feedback raised three themes: third-person firearm reloads are visually absent, swimming looks unconvincing or lacks a swim pose, and melee swings should read more naturally. Private report bodies and identifiers remain in ignored local storage and are not copied here.

Reference imagery reviewed during the work shows first-person tool presentation and a model-viewer swimming pose. It does not establish third-person melee/reload timing or in-game swimming behavior.

## Read-only original-source research

Installed `models/m_anm.mdl` metadata was inspected with the metadata-only helper. No proprietary asset or extracted script was copied into the repository.

- The player model contains `@swim_idle_all` and directional swim clips named `a_swimming_swimming_<hold>_<direction>`. Authored rows use these names with the supported hold families and the eight horizontal directions. The physgun hold explicitly uses the original `gravgun` swim family because `a_swimming_swimming_physgun_*` does not exist. Final metadata review caught and corrected that startup-breaking name assumption. The metadata names confirm clip existence, not gameplay timing or that the current water-volume approximation matches Source.
- Installed player-model metadata contains non-delta full-body reload candidates `@reload_pistol_original`, `@reload_revolver_original`, `@reload_smg1_original`, `@reload_ar2_original`, and `@reload_shotgun_original`. Their recorded lengths are 41, 111, 66, 66, and 81 frames respectively. Reload candidates are mapped for pistol, revolver, SMG1, AR2, and shotgun. Other families intentionally leave this action blank.
- `@seq_meleeattack01` is a non-delta 32-frame sequence candidate. The crowbar/stunstick viewmodels have their own hit/miss clips, but those clips are not player-body gestures. The shared player sequence is a generic substitution, not evidence of weapon-specific crowbar or stunstick swing parity.
- The installed crowbar and stunstick scripts distinguish their viewmodel/world models and animation prefixes. This evidence does not establish a matching third-person player action sequence.
- Public upstream activity reference: [Garry's Mod `animations.lua`](https://raw.githubusercontent.com/Facepunch/garrysmod/master/garrysmod/gamemodes/base/gamemode/animations.lua). It was used as activity/gesture context, not as proof that this prototype reproduces engine gesture layering.

## Implemented scope

- `sheets/source_animation_states.csv` authors locomotion and swim mappings for eleven hold families: physgun, pistol, revolver, SMG1, AR2, shotgun, crossbow, RPG, grenade, melee, and melee2. `sheets/source_weapons.csv` assigns enabled weapons to those families. The toolgun intentionally uses the existing pistol family as a generic fallback.
- Player scene creation deduplicates clip loads. Swimming uses a separate playback state at water level 2 or 3, selects the authored directional loop while moving and swim idle while stationary, and leaves noclip, vehicles, shallow water, and dry movement on their existing paths.
- Third-person reload events sample the mapped full-body candidate and blend the upper-body bone subset over the current locomotion pose with a short fade envelope. This is a prototype bone-mask blend, not native Source sequence-layer evaluation. It is triggered only while the matching weapon's reload interval is active and the player is grounded, standing, not swimming, and not in a vehicle or noclip.
- Melee attacks use the mapped `@seq_meleeattack01` candidate for the `melee` hold family. The same generic clip is used by crowbar and stunstick. It is sampled from the accepted melee fire timestamp as a single non-looping gesture and blended over locomotion; it is not repeated and does not replace lower-body movement. Like reload, it is suppressed while airborne, crouched, swimming, in a vehicle, or in noclip.
- Action clips are optional at load time. If a metadata candidate cannot be sampled, the player scene remains available and that action is skipped with a diagnostic.
- `source_player::finish_walk` is exposed `pub(crate)` so fixed-step footstep cadence can run after movement writeback.

## Limits and verification boundary

Swimming still depends on the project's rendered-water-surface approximation. The classifier requires a sampled horizontal water surface and valid ray checks; it is not Source water-volume/BSP-content equivalence. No water-depth guard was relaxed for this visual change. Vertical-only movement continues to use swim idle because only horizontal directional templates are authored.

The action blend uses bone-name matching (`spine`, `neck`, `head`, clavicle/shoulder, arm, hand, finger) and a sampled absolute candidate. Native Source upper-body sequence-layer behavior, weapon-specific melee swing identity, and timing parity remain unproven. Drew owns all manual acceptance. No build, test, screenshot, gameplay, or input validation was performed for this change.

Manual cases and their ownership/status are recorded in `sheets/animation_feedback_cases.csv`; every case remains `Drew,not_run`.
