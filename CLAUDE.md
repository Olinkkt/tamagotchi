# Versioning

`MAJOR.MINOR.PATCH` in `Cargo.toml`. On every change, by default and without being asked: bump the version, build and test, then commit the work and tag it `vX.Y.Z`. Reset lower numbers to 0.

- **MAJOR**: breaks existing saves or config, or changes the whole frontend (terminal to window).
- **MINOR**: new player-visible feature or command.
- **PATCH**: fixes, balance, visual polish.

# Changelog

Every version gets an entry in `CHANGELOG.md`, in the same commit as the version bump and the code, so the changelog and git never disagree.

- Newest first. Heading is `## X.Y.Z`, one blank line before and after.
- Bullets are written for the player: what they now see or can do, not which files changed. Say the visible effect ("the egg cracks in five stages"), not the code.
- Say what was removed or changed too, not only what was added. Note any save or config impact.
- One to three bullets per version. Wrap long bullets at ~100 chars with a two-space indent.
- Internal-only changes (refactor, tests, docs) get one short bullet, or fold into the nearest player-visible entry.
- Versions before 1.0.0 are folded into one `## 0.N.x` entry per minor.

# Pixel art

- Only the 24 colours in `src/palette.rs`. Never add a colour elsewhere.
- Characters, the room and other bodies are drawn in shapes in `tools/spritegen` and built with `cargo art`; never hand-place pixels for a whole body. Faces, cracks and icons are pixel grids.
- After changing a sprite, look at its 8x preview in `assets/preview/` and fix what is off before moving on.
- The scene is 240x160 at one pixel size; text is drawn at full window resolution with `src/font.rs`.

# Commands

```sh
cargo build --release
cargo test                      # all unit tests (no window needed)
cargo test egg::                # one module; or `cargo test <test_name>`
cargo art                       # alias: rebuild every sprite into assets/sprites/ (+ assets/preview/)
cargo dev                       # alias: run --release -- --dev   (separate test save, Tab = dev panel)
cargo fast                      # alias: run --release -- --fast  (60x time, separate test save)
cargo install --path .          # install `tama`; `tama --update` does the same from the source folder
```

No lint config; `cargo clippy` is plain defaults. Rust edition 2024, MSRV 1.86 (`fontdue` is pinned in `Cargo.toml` for that reason; don't bump it).

# Architecture

Single binary `tama` (`src/main.rs`) plus the `tools/spritegen` workspace member.

- **Headless first.** `main` calls `cli::run` / `cli::flags` first; only if neither handles the args does it open the macroquad window. Subcommands (`status`, `feed`, `watch`, `event`, `--gallery`) must never touch graphics code.
- **Simulation is separate from drawing.** `pet.rs` is the rules (needs, growth, evolution, trait, shop items) advanced one `tick()` per second. `app.rs` owns game state and input and is window-independent; `window.rs` is the only macroquad glue (scaling, input, full-resolution text). `scene.rs`/`hud.rs`/`panel.rs`/`screens.rs`/`games.rs` draw into a palette `Frame` (`frame.rs`) which is converted to RGBA once per frame.
- **Offline catch-up.** The pet lives while the game is closed: on load, elapsed real time is replayed as ticks (capped by `MAX_CATCHUP_TICKS`) under the slower, safe "away" rules. The same replay runs in CLI commands. A lock file stops CLI care commands while the game is open.
- **Shell events** (`tama hook zsh` -> `tama event commit|fail`) append to a queue file; the game or the next CLI command applies it (capped by `MAX_QUEUED_EVENTS`).
- **The egg** (`egg.rs`) is a pure function of age and a seed (the lay time, derived from the save), so its 30 s of shaking is identical across restarts. Keep it deterministic.
- **Persistence** (`save.rs`): atomic writes, a `SAVE_VERSION` field, unreadable saves moved aside. Save, profile (coins/achievements/collection, survives pet death), graveyard and lock live in the user data dir; config in `~/.config/tamagotchi/config.json` (`config.rs`). `--dev`/`--fast` use a separate save. A save-format change needs a `SAVE_VERSION` bump and counts as a MAJOR version.
- **Sprite pipeline.** `src/palette.rs`, `faces.rs` and `cast.rs` are compiled into both the game and `tools/spritegen` (hence `#[allow(dead_code)]` in `main.rs`). `cast.rs` defines the sheet layout (one row of square cells: body, then every frame of every animated part), so the tool and the game cannot disagree. spritegen draws bodies on a 128x128 canvas, downsamples with nearest-neighbour, snaps to the palette, and writes `assets/sprites/*.png`, which `art.rs` embeds with `include_bytes!`. After `cargo art`, commit `assets/sprites/`; `assets/preview/` is gitignored.
- **Releases.** Pushing a `vX.Y.Z` tag runs `.github/workflows/release.yml`, which builds the Linux binary and attaches it as `tama-linux-x86_64` to a GitHub release; `install.sh` and `tama --update` download that asset. `update.rs` picks the newest of the local source folder and the latest release. Push only the newest tag, so "latest" is right.
- **Docs to keep in sync.** `README.md` documents keys, commands, config options and balance numbers; update it with player-visible changes and add a `CHANGELOG.md` entry per version. `EGG_PLAN.md` is a working plan for reworking the egg-crack visuals.
