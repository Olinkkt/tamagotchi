# Versioning

`MAJOR.MINOR.PATCH` in `Cargo.toml`. On every change, by default and without being asked: bump the version, build and test, then commit the work and tag it `vX.Y.Z`. Reset lower numbers to 0.

- **MAJOR**: breaks existing saves or config.
- **MINOR**: new player-visible feature or command.
- **PATCH**: fixes, balance, visual polish.

# Pixel art

- Only the 24 colours in `src/palette.rs`. Never add a colour elsewhere.
- Characters, the room and other bodies are drawn in shapes in `tools/spritegen` and built with `cargo art`; never hand-place pixels for a whole body. Faces, cracks and icons are pixel grids.
- After changing a sprite, look at its 8x preview in `assets/preview/` and fix what is off before moving on.
- The scene is 240x160 at one pixel size; text is drawn at full window resolution with `src/font.rs`.
