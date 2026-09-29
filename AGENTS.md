# Spreadsheet-driven Bevy sandbox

- Author gameplay parameters in `sheets/`, not generated Rust.
- Keep researched behavior separate from implemented and tested behavior.
- Do not modify the Steam installation or commit proprietary assets, extracted scripts, or user saves.
- Inventory is read-only. Local manifests and saves belong in ignored `local/`.
- Use a private target directory on D: because C: has limited free space.
- Do not claim Source physics equivalence or Lua compatibility based on prototype tests.
- Record research in `docs/RESEARCH.md` and verification in `docs/VALIDATION.md`.
