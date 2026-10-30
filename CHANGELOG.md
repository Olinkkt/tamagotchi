# Changelog

## 1.4.3

- The egg's crack is now one zigzag across its middle that grows outwards from the centre, a bit
  more on each hard shake. When it runs all the way across, the top half lifts and rocks while the
  bottom half stays put.
- The egg ends as two halves with a one-pixel gap, exactly where the burst starts. Flying bits of
  shell are shell-coloured instead of white squares and come off the crack.

## 1.4.2

- Internal: the changelog writing rules now live in `CLAUDE.md`.

## 1.4.1

- Changelog now covers 0.1 to 0.23 too, rebuilt from the git history.

## 1.4.0

- Every egg now follows a 30-second schedule of its own: shaking in five phases, from rare 1px
  nudges to rapid irregular shaking, with a tilt from 15 seconds, jumps from 20 seconds, a
  strongest shake at 29 seconds, a held breath and a white flash at the end.
- The pattern comes from a seed (when the egg was laid, already in the save), so the same egg shakes
  the same way after a restart, and different eggs shake differently.
- Six crack stages (the first is three pixels at 20 seconds), a shell that lifts apart from 24
  seconds and flying shell bits from 26 seconds. The old wobble and the post-hatch shake are gone;
  the burst now happens the moment the egg breaks.

## 1.3.1

- The egg wobble and the final hatch shake are random now: every lurch leans a different way and
  distance, so no two eggs (or two runs) move the same.

## 1.3.0

- Hatching is now an animation: the finished egg shakes hard, flashes white and bursts. The baby
  pops up out of the bowl, the cap is thrown in an arc to one side and the bowl hops to the other,
  then they stay on the floor for a few seconds.
- The yellow glow in the cracks is gone; the cracks are plain dark lines, and the last stage has a
  dark chip broken out of the shell.

## 1.2.0

- The egg cracks in five detailed stages (the last two glow, with a chip breaking open), instead of
  a single line.
- The nearer the egg is to hatching, the harder it wobbles: wider swings, faster, then hops in the
  last couple of seconds.
- Hatching leaves the two halves of the shell on the floor for a few seconds.
- `--gallery` shows all six egg stages.

## 1.1.4

- Dev panel: `1` sets the speed straight back to 1x.

## 1.1.3

- Dev panel: the speed key (`f`) now steps 1x, 2x, 5x, 10x, 60x, 600x.

## 1.1.2

- The egg wobble is back; the pet still only bobs up and down.

## 1.1.1

- The pet no longer wanders side to side (and the egg and the scolding no longer shake it sideways); it only bobs up and down.

## 1.1.0

- F11 toggles fullscreen.
- The picture now fills the whole window at any shape (a whole-number pixel scale, so pixels stay
  the same size): the room gets bigger, the status panel taller and the buttons stay on the
  bottom edge. Cards and games are centred. Only a window bigger than the room art can fill gets bars.

## 1.0.0

The terminal game becomes a window game. Saves, profile and config are unchanged.

- Dev panel: ← and → step through the forms (`[` and `]` still work).

- tama now opens a pixel-art window (macroquad) instead of drawing in the terminal. The scene is
  240x160 pixels, scaled to fit by whole numbers.
- All characters, the room and the mini-games are redrawn in far more detail with a fixed
  24-colour palette, built by a sprite tool (`cargo art`).
- Every action is a clickable button as well as a key; the shop and game picker are clickable too.
- Pong's paddle moves while a key is held, instead of keeping going after one press.
- `--gallery` writes `tama-gallery.png` instead of printing to the terminal.
- The terminal UI is gone. Saves, profile, config and all shell commands are unchanged.

Before 1.0.0, rebuilt from the git history (patch releases folded into their minor).

## 0.23.x

- Terminal UI removed; the window is the only frontend.
- Window screens: status panel, growth, hint, shop, awards, graveyard, reset.
- Pixel-art pipeline with a fixed palette; every child, teen and adult form is drawn.
- Dev: arrow keys step through forms (0.23.1).

## 0.22.x

- Mini-games get a result screen and drop text headings.

## 0.21.x

- Snake, reaction, catch and dodge removed. Memory and pong stay, drawn in pixel art with fair hitboxes.

## 0.20.x

- Snake and dodge added (later removed in 0.21). Dev panel opens with Tab.

## 0.19.x

- Two more mini-games, and clipped game screens fixed.
- Saves carry a format version (0.19.6).
- Internal: code split into modules, clippy clean, evolution tests.

## 0.18.x

- Coins, a shop, achievements and a collection of adults.

## 0.17.x

- The pet reacts to CPU load, RAM use and a low battery.

## 0.16.x

- Shell hook: commits feed the pet, failed commands upset it.

## 0.15.x

- Gentler behaviour while the game is closed; small terminal windows handled; README added.

## 0.14.x

- `tama status --waybar` gives short text with a hover tooltip.
- Tests for saves, lock, config, shell commands and mini-games; a second game window is refused.

## 0.13.x

- `tama watch` sends desktop notifications while the game is closed.

## 0.12.x

- `tama status` and shell care commands.

## 0.11.x

- `tama --animate on|off`; the gallery follows the config.

## 0.10.x

- `animate_parts` setting, off by default.

## 0.9.x

- Personality traits, weight that matters, evolution preview, animated sprite parts.

## 0.8.x

- `tama --update`, `--version` and `--help`.

## 0.7.x

- Config file, real-clock sleep, three more mini-games, graveyard, dev mode, `tama` command.

## 0.6.x

- Reset key with confirmation; smarter hints and menu chips.

## 0.5.x

- Hint line, speech bubble, context-aware menu.

## 0.4.x

- Room scene, scaling sprites, restyled UI.

## 0.3.x

- Pixel-art sprites, effects, side status panel.

## 0.2.x

- `cargo fast`: 60x test mode with a separate save.

## 0.1.x

- Terminal tamagotchi: pet simulation, ASCII sprites, saves.
