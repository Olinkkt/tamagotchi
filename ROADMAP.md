# Roadmap

## Direction

tama is a cute pixel-art pet that lives in a window and your status bar. It is about raising
a pet well and building something that lasts across pets, not about your coding.

**Simple to learn, deep to keep.** A new player should understand the game in a minute: hatch,
feed, play, grow. New content adds things to discover, never things to learn first. Every new
feature must work without a tutorial, and the first hour must stay as easy as it is today.

Three goals, in this order:

1. **Long-term goals.** Give a life more reason to matter: breeding or generations, a growing
   collection, achievements that carry over between pets.
2. **More content.** New species and evolutions, rooms, mini-games and shop items.
3. **More platforms.** macOS and Windows builds next to Linux.

## Evolution by habit and personality (next)

Care mistakes no longer pick a form. What you do most during a stage does, and each teen and
adult form brings its own personality. The birth trait (random) stays for life; a form's
personality replaces the previous form's, so a pet has at most two traits at once.

| Stage | Form | Habit that earns it | Personality |
|---|---|---|---|
| Teen | Tonmarutchi | child stage: mostly meals and snacks | Calm: happiness falls 25% slower |
| Teen | Hashitamatchi | child stage: mostly games and shop | Spirited: +1 coin per game win |
| Adult | Kuchipatchi | teen stage: mostly meals | Gourmet: hunger falls 25% slower |
| Adult | Tarakotchi | teen stage: mostly games | Sporty: +1 extra happy per win |
| Adult | Ichigotchi | teen stage: mostly snacks | Cuddly: happiness falls 40% slower |
| Adult | Mametchi | teen stage: mostly scolds | Disciplined: acts up half as often |
| Adult | Nyorotchi | teen stage: mostly shop | Sturdy: gets sick half as often |

Ties go to the first row; no habits gives the default form. The panel shows what the pet is
leaning towards and the family tree shows each form's habit and personality. Numbers are first
guesses. More teen forms come later (each needs new sprites).

## Dropped

- The shell hook (`tama hook`, `tama event`, commits feeding the pet). Removed in 1.23.0.
- The machine reactions (CPU, RAM and battery). Removed in 1.24.0.

## Next, smallest first

1. Done in 1.23.0: the shell hook was removed.
2. One new mini-game or shop item (MINOR, a few hours).
3. A second collection layer on top of adult species, for example a record of each pet's life.
4. Pick the long-term goal: generations or collection depth.
5. macOS build in the release workflow.
