# Remaining tool implementation plan

## Scope and evidence, 2026-09-30

Reviewed the entire 38-row inventory and all 18 private reports. Starting inventory: 13 partial implementations, 22 missing tools, 3 reference-only internal tools. Read the installed read-only original stool scripts for all 22 missing tools, including ClientConVar defaults, LeftClick/RightClick/Reload entry points, entity constructors, key callbacks, constraints and posing APIs. Native scripts remain private Steam content. This is an independent runtime, not execution of those scripts.

## Delivery order

1. Establish persistent typed construction-device configuration and bounded key controls. Preserve old version 2/3/4 scenes using optional new fields. Validate all settings and assets before replacement. Reuse original model spawning, whole-scene undo, connected-assembly removal and duplication.
2. Add balloon lift, thruster forward/reverse force, hoverball altitude control, momentary/toggle buttons, delayed dynamite, bounded spark emitter, point light and spotlight. Left-click attaches where appropriate, right-click creates unattached devices or updates as described by the tool row. Surface placement and forces use authored configuration. Do not invent audio or native particle fidelity.
3. Add motor, hydraulic, muscle and winch constraints driven by configurable keypad controls. Retain local anchors and actual physics joints, persistent configuration, current lengths and group-key behavior. Add motorized wheel construction through the same joint path. Native force-break, solver and cable fidelity remain separate gaps.
4. Add four-anchor pulley routing with actual coupled-length forces and rendered segments. Implement actual mechanics rather than showing a rope with no force.
5. Add persistent trail and paint configuration with bounded runtime geometry, actual materials, remove/reset actions, undo and save/restore. Entity editing must select real supported device settings rather than fake a generic network-property editor.
6. Camera now has independent remote-view/input ownership and optional fixed-point aiming, with moving-target tracking and native locking still open. Four poser tools still require model eye material targeting, flex vertex decoding, finger-bone poses and nonuniform bone deformation. Do not mark these complete by moving a rigid model or displaying a placeholder. Their exact missing prerequisites stay visible in the source sheet until implemented.
7. Audit all tools and controls again, update tool-by-tool manual cases and F7 cards, compile the actual game and validate/export the authoritative sheets. Drew owns gameplay acceptance. No tests, screenshots, benchmarks or input automation. Preserve active game processes.

```mermaid
flowchart TD
    A[Original tool contracts and sheet controls] --> B[Validated persistent device data]
    B --> C[Key groups and construction entities]
    C --> D[Powered joints and motor wheels]
    D --> E[Coupled pulley mechanics]
    B --> F[Persistent paint and trail rendering]
    G[Eye flex and bone deformation support] --> H[Poser tools]
    I[Remote view and input ownership] --> J[Camera tool]
    C --> K[Undo duplication removal and scene restore]
    D --> K
    E --> K
    F --> K
    K --> L[Drew-owned action-by-action acceptance]
```

## Acceptance boundaries

Every advertised left/right/reload action and enabled setting needs a concrete runtime consumer. All new status values stay partial until full native behavior and Drew's acceptance exist. Old saves and private feedback must remain intact. Unsupported targets must reject clearly, and lifecycle failures must not destroy the previous scene.

Current delivery: 31 partial rows including custom Freeze, four missing posers and three reference-only entries. The 169 option rows and 45 manual cases are authored in the existing sheets. Version-5 scenes add typed devices and paint while retaining legacy readers. Transient device activation/fuses reset to authored start-on state on restore, trail history restarts and muscle phase is not persisted. These choices avoid accidentally detonating a loaded scene and are not native save parity.
