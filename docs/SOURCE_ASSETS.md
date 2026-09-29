# Original installed Source content pipeline

Drew's corrected acceptance target is to load the installed original maps and assets, not substitute procedural boxes. The earlier sandbox remains a testbed, not delivery of Garry's Mod parity.

## Data flow

`sheets/source_maps.csv -> catalog validation -> build.rs -> compiled_source_maps() -> read-only local Steam mount -> BSP/VPK/VMT/VTF decoders -> Bevy meshes/images/lightmaps`

The spreadsheet stores references and rendering/player configuration. Binary map, texture, model and sound payloads stay in Steam. No assumption of ownership or redistribution rights is made. Imported data is read into memory. Local import reports and screenshots are ignored by git. `GMOD_DIR` selects the install root without changing Steam.

## Incremental acceptance gates

1. **Original world:** load gm_construct and gm_flatgrass geometry, displacements, texture coordinates and base textures from local loose files, BSP pakfiles and VPKs. Verify original spawn coordinates, actual GPU output, failures and missing-material report. No silent primitive substitution.
2. **Original look:** preserve baked lightmaps, six sky textures, 3D sky camera, brush entity placement, static models, overlays, material transforms/blending, HDR exposure, alpha and water. Capture the same camera in Garry's Mod and Bevy and compare. A recognizable map is necessary but is not visual equivalence.
3. **Original model catalog:** MDL/VVD/VTX meshes, material search paths, skins, bodygroups and PHY collision. Bind spreadsheet inventory identifiers to mounted virtual paths, then replace the testbed prop palette with decoded model handles. Report unsupported format versions explicitly.
4. **Original player feel:** Source unit conversion, player hull, ground acceleration/friction, air control, steps, gravity, jumping, water, ducking and noclip. Record repeatable reference input traces and position/time tolerances. A fly camera does not satisfy this gate.
5. **Original sandbox loop:** physgun/toolgun viewmodels, selection/beam rendering, hold constraints, frozen states, weld/rope/axis, menu icons/categories, sounds, save/undo/duplication. Integration must preserve the spreadsheet pipeline and disk saves.
6. **Remaining engine/game behavior:** map I/O, lighting/shader effects, dynamic entities, NPCs, animation, Lua/addon compatibility and multiplayer prediction. Track explicit implementation and test evidence per system. Never infer completion from an asset count.

## First importer limitations

The native import targets world model 0, its displacement geometry, base textures, LDR lightmaps and static prop models. Shared base textures are decoded/uploaded once and sampled with mipmaps and anisotropic filtering. The mount includes `garrysmod`, `sourceengine`, optional `hl2`, and `platform`. Installed `gameinfo.txt` confirms the bundled HL2 archives live in `sourceengine`, not `hl2` on this installation.

Unsupported assets are listed in the report and missing materials are conspicuous magenta. Source water has no base texture by design, so its VMT fog color is used as an explicitly reported approximation, not counted as a decoded texture. Source prop vertex lighting is not yet decoded. Brush entities, overlays, 3D sky scaling, full material shaders and original player movement remain. The inspection mode is separate from the preserved physics testbed, without its fake grid and blocks.

## Reuse research, 2026-09-29

- `vbsp 0.9.1`, MIT: https://docs.rs/vbsp/0.9.1/vbsp/ . Bsp::read, world model faces, displacement tessellation, Source texture axes, packed asset lookup, entity properties.
- `vpk 0.3.0`, MIT: https://docs.rs/vpk/0.3.0/vpk/ . Directory archive parsing and entry payload reads, including preload bytes.
- `vtf 0.4.1`, MIT: https://docs.rs/vtf/0.4.1/vtf/ . Decode installed compressed texture frames into RGBA. Unsupported image formats must remain diagnostics.
- `vmt-parser 0.2.1`, MIT: https://docs.rs/vmt-parser/0.2.1/vmt_parser/ . Valve material grammar, patch resolution, base texture and alpha/culling metadata. Parsing a shader name does not implement the shader.
- `vmdl 0.2.0`, MIT: https://docs.rs/vmdl/0.2.0/vmdl/ . MDL/VVD/VTX mesh vertices, material paths and skin tables, mounted directly from byte buffers. Original placement transforms come from vbsp. Only static triangle meshes are supported in this pass.
- Existing transitive `image 0.25` is reused directly for mipmap resizing. Bevy image resources deduplicate textures by original VMT base texture name.
- Bevy 0.16 Lightmap component: https://docs.rs/bevy/0.16.1/bevy/pbr/struct.Lightmap.html . Reuse existing second-UV baked-light rendering rather than a separate renderer.
- Investigated https://github.com/icewind1991/vbsp-to-gltf and tf-asset-loader. Not reused: archived converter has no license metadata in its GitHub API response and loader assumes tf/hl2 layout. Native import uses the permissively licensed underlying format parsers and explicit Garry's Mod mounting. No converter implementation was copied.

Parser licenses were checked using `cargo info` and exact resolved dependencies are recorded in Cargo.lock. These library licenses do not grant redistribution rights over installed game assets.
