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
- 2.0: evolution by habit, personalities and the Plain form (Futsutchi); 2.0.3: the leaning tint.
- 2.1: gifts while away and the keepsakes page; 2.2: care badges; 2.3: the Stop and Hop mini-games.

## Next, in order

Times are my own work time, plus your playtesting.

1. **Shop collectibles (MINOR, about 45 minutes).** Details below. A second shelf in the shop
   for things you keep, hung or placed in the room.
2. **More shop items (MINOR, about 30 minutes).** Treats with a clear effect, to go with the
   new prices.
3. **macOS build in the release workflow (MINOR, about 30 minutes).**

## Shop collectibles

The shop gets a second tab for things you keep. They cost coins, do nothing in play, and show in
the room, so a player learns them by seeing them. They are the main coin sink after treats.

- **Memorial plaque.** A wall plaque of your last pet: its species, name or age, and how long it
  lived. Bought once per pet from the graveyard, and it stays on the wall after a new egg
  hatches. This is the first piece of "memory of past pets".
- **Wall and floor pieces.** Posters, a second rug, a plant, a lamp. A few spots in the room,
  one item per spot, swapped by buying another.
- **Keepsakes from gifts** can also be placed, so the two systems feed each other.
- Owned collectibles live in the profile, so they survive a pet's death. Old profiles load as
  before (new fields have defaults).
- Palette rule: only the 24 colours, drawn as pixel grids like the other icons, with a preview
  checked at 8x.
- First guess for prices: 20 to 60 coins, with the plaque cheap enough to buy after every pet.

## Later

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
