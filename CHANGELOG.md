# Changelog

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
