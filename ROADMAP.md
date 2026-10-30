# Roadmap

## Direction

tama is a cute pixel-art pet that lives in a window and your status bar. It is about raising
a pet well and building something that lasts across pets, not about your coding.

**Simple to learn, deep to keep.** A new player should understand the game in a minute: hatch,
feed, play, grow. New content adds things to discover, never things to learn first. Every new
feature must work without a tutorial, and the first hour must stay as easy as it is today.

**No fail state for being away.** Closing the game never punishes the player. Coming back
should feel like a reward.

Three goals, in this order:

1. **Long-term goals.** Give a life more reason to matter: a growing collection, badges that
   carry over between pets, later generations.
2. **More content.** New forms, rooms, mini-games and shop items.
3. **More platforms.** macOS and Windows builds next to Linux.

## Done

- 1.21 to 1.22: one-line install, in-game updates, shop prices, mini-game payout cooldown.
- 1.23 to 1.24: the shell hook and the CPU, RAM and battery reactions were removed.
- 1.25: the family tree screen (`e`).

## Next, in order

Times are my own work time, plus your playtesting.

1. **Evolution by habit (MAJOR 2.0.0, about 1 hour).** Details below. Changes the save format.
2. **Leaning tint (PATCH, about 15 minutes).** A small tint or accent on the pet that shifts
   towards the form it is heading for. No meter to read.
3. **Gifts while away (MINOR, about 30 minutes).** Details below.
4. **Care badges (MINOR, about 20 minutes).** Achievements for meals, snacks, games and scolds
   (100, 500, 1000). No fail state.
5. **Two new mini-games (MINOR each, about 40 minutes each).** One key or one button, under 30
   seconds a round.
6. **Shop collectibles (MINOR, about 45 minutes).** Details below. A second shelf in the shop
   for things you keep, hung or placed in the room.
7. **More shop items (MINOR, about 30 minutes).** Treats with a clear effect, to go with the
   new prices.
8. **macOS build in the release workflow (MINOR, about 30 minutes).**

## Evolution by habit

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

- **Hidden counters, reset each stage.** Meals, snacks, games, scolds and shop buys are counted
  during a stage; the biggest one picks the next form. Players learn the paths by playing and by
  reading the family tree, not a wiki.
- **Ties** go to the first row. A pet with no habits gets the default form until a neutral
  "Plain" form exists (see Later).
- **Personality as a visible moment.** Each effect is small and clear, with a line of speech
  when it shows, so it feels earned and not like a hidden percentage.
- **Showing the lean.** The panel says what the pet is heading for, the family tree shows each
  form's habit and personality, and the tint (step 2) shows it on the pet itself.
- **Save impact.** New counters with defaults, so old saves load. Pets already past teen keep
  their form and get its personality. The mistakes rule goes away for form choice only; mistakes
  still affect health and the "Spotless" award.

Numbers are first guesses, like the rest of the balance.

## Gifts while away

While the game is closed the pet leaves small gifts, so coming back is a reward and never a
chore.

- One gift for every 2 hours away, up to 5 waiting. First guess: each gift is 2 to 5 coins or a
  keepsake.
- Keepsakes are collectable and appear in a "keepsakes" page next to the awards, shown as
  silhouettes until found.
- A short "welcome back" card shows the gifts when the game opens. No gifts are lost or
  punished if you stay away longer; they just stop piling up at the cap.
- Works with the existing away rules: the pet is safe and slower while the game is closed.

## Shop collectibles

The shop gets a second tab for things you keep. They cost coins, do nothing in play, and show in
the room, so a player learns them by seeing them. They are the main coin sink after treats.

- **Memorial plaque.** A wall plaque of your last pet: its species, name or age, and how long it
  lived. Bought once per pet from the graveyard, and it stays on the wall after a new egg
  hatches. This is the first piece of "memory of past pets".
- **Wall and floor pieces.** Posters, a second rug, a plant, a lamp. A few spots in the room,
  one item per spot, swapped by buying another.
- **Keepsakes from gifts** (see above) can also be placed, so the two systems feed each other.
- Owned collectibles live in the profile, so they survive a pet's death. Old profiles load as
  before (new fields have defaults).
- Palette rule: only the 24 colours, drawn as pixel grids like the other icons, with a preview
  checked at 8x.
- First guess for prices: 20 to 60 coins, with the plaque cheap enough to buy after every pet.

## Later

- **A neutral "Plain" form** for balanced or empty habits. Needs a sprite.
- **More teen forms** so the child stage can branch more than two ways (each needs sprites).
- **Generations:** a new egg inherits a colour or accessory, and the family tree can show
  grandparents. Bigger than anything above, so only after the collection and badges exist.
- **Memory of past pets** with milestones, built on the graveyard and the memorial plaque.
- **Windows build.**

## Dropped

- The shell hook (`tama hook`, `tama event`, commits feeding the pet). Removed in 1.23.0.
- The machine reactions (CPU, RAM and battery). Removed in 1.24.0.

## Where the ideas come from

Research from EDITH (mostly fan wikis, so treat the details as readings, not facts):
Digimon Ver.1 habit counters, Tamagotchi Uni personalities and care icons, Chao evolution tints,
the Neko Atsume Catbook and mementos, Pokémon-Amie affection, Tamagotchi m!x generations. It
could not check Nintendogs, Spiritfarer or classic Connection.
