# tama

A tamagotchi that lives in your terminal. Feed it, clean up after it, play games with it, and
keep an eye on it from your status bar. It keeps living while the game is closed.

Pixel-art characters are drawn with half-block characters, so you need a terminal with
24-bit colour and a window of at least **64x22**.

## Install

You need [Rust](https://rustup.rs). From the project folder:

```sh
cargo install --path .
```

This puts `tama` in `~/.cargo/bin`, which must be on your `PATH`. After you change the code,
`tama --update` rebuilds from the same source folder and replaces the installed copy.
(There is no download-based updater yet.)

## Playing

Run `tama`. One pet at a time; when it dies you can hatch a new egg with `n`.

| Key | Action |
|---|---|
| `f` | feed a meal (+1 hunger heart) |
| `s` | snack (+1 happy heart; makes it heavier, and sometimes sick) |
| `p` | play: pick a mini-game (see below) |
| `l` | light on/off (only matters while it sleeps) |
| `m` | medicine (a sick pet needs two doses) |
| `c` | clean up poop |
| `d` | scold, but only when it is being naughty |
| `b` | shop (see below) |
| `a` | awards: achievements and the adults you have raised |
| `g` | graveyard: the pets you have lost |
| `q` | quit (the pet keeps living) |

The line under the room always tells you the most urgent thing to do, the matching key lights
up in the menu, and the pet shows a bubble with an icon for it.

### Mini-games

Any game makes the pet a little lighter, and winning makes it happier. A perfect game forgives
one care mistake.

Both are drawn in the same pixel art as the pet, scores included. When a game ends, a result
screen shows a star per round won and the coins earned for three seconds (any key skips it).

1. **Memory**: four coloured pads light up in a pattern of 3, 4 and 5; repeat it with keys 1 to 4. Your pet watches and cheers or sulks.
2. **Pong**: return the ball five times; one miss ends it. Your pet keeps goal on the other side. A direction key keeps the paddle going until you press the other way or space, so it does not rely on your terminal's key repeat.

### Coins, shop and awards

Winning a mini-game pays **4 coins**, and a perfect game **10**. Spend them in the shop (`b`,
then `1` to `4`); nothing is charged if the item would do nothing right now:

| Item | Price | Effect |
|---|---|---|
| Fancy meal | 8 | +2 hunger, a bit heavy |
| Toy | 6 | +2 happy, burns weight |
| Salad | 5 | +1 hunger, lighter |
| Vitamins | 12 | no sickness for 3 hours |

`a` shows ten achievements (for example "Spotless": reach adulthood with no care mistakes) and
which of the five adult species you have raised. Coins, achievements and the collection belong
to you, not to one pet, so they survive a pet dying; they live in `profile.json` next to the save.
Prices and rewards are first guesses.

### Growing up

```
Egg -30s-> Baby -10 min-> Child -1 h-> Teen -2 h-> Adult -2 days-> old age
```

Care mistakes (counted over its whole life) decide what it grows into:

- Ignoring hunger, unhappiness or sickness for 15 minutes, ignoring a naughty call for
  10 minutes, or leaving the light on while it sleeps for 10 minutes each add one mistake.

| Becoming | Needs |
|---|---|
| Teen: Tonmarutchi | 0 to 2 mistakes |
| Teen: Hashitamatchi | 3 or more |
| Adult: Mametchi | 0 to 1 mistakes **and** 75% discipline |
| Adult: Kuchipatchi | 0 to 2 mistakes |
| Adult: Tarakotchi | 3 to 4 |
| Adult: Ichigotchi | 5 to 6 |
| Adult: Nyorotchi | 7 or more |

Discipline starts at 50% and rises 25 points each time you scold a real naughty call.
The status panel's "growth" section shows what it is on track for.

Each pet also gets a random trait: *greedy* (hungry faster), *playful* (bored faster, +2
happiness from a win), *hardy* (rarely sick), *mischievous* (acts up more) or *easygoing*.
An overweight pet (a heavy snack habit) falls sick three times as often.

The pet sleeps from 22:00 to 07:00 by your clock.

### While the game is closed

The pet is "away". Its needs fall three times slower, hunger and happiness never drop below
one heart, and it cannot fall sick, act up or die. It still grows up. Both rules can be changed
in the config (`away_slowdown`, `away_safe`).

## Commands

```
tama                 play
tama status          one line for a status bar, e.g. (T_T) Marutchi ♡♡♡♡ ♥♥♥♡ ! feed
tama status --waybar JSON for waybar: short text, a hover tooltip, and a class
tama feed|snack|clean|medicine|scold|light
                     care for the pet without opening the game (refused while it is open)
tama hook zsh        print a shell snippet so commits feed the pet and failed commands upset it
tama event commit|fail  tell the pet about a shell event (the hook calls this)
tama watch           desktop notifications when the pet needs you
tama watch --test    send one test notification
tama --animate on|off  show or hide the animated ears, wings, tuft, leaves and tail
tama --update | --version | --help
tama --gallery       print every character
tama --dev | --fast  developer mode / 60x speed, both with a separate test save
```

## Settings

`~/.config/tamagotchi/config.json` is created on first run with every option and its default.
Leave out any you do not want to change. On macOS and Windows the file lives in that system's
usual config folder.

| Option | Meaning |
|---|---|
| `hunger_secs_baby` / `_child` / `_teen` / `_adult` | seconds per lost hunger heart |
| `happy_decay_factor` | happiness falls this many times slower than hunger |
| `poop_min_secs`, `poop_extra_secs` | poop every min + random(extra) seconds |
| `sick_odds`, `naughty_odds` | 1 in N per awake second |
| `overweight_sick_factor` | sickness is this many times likelier when heavy |
| `mistake_after_secs` | how long a need may wait before it counts as a mistake |
| `baby_secs`, `child_secs`, `teen_secs`, `adult_lifespan_secs` | stage lengths |
| `sleep_start_hour`, `sleep_end_hour` | sleeping hours, 24h clock |
| `away_slowdown`, `away_safe` | see "While the game is closed" |
| `commit_meal_gap_secs`, `fails_per_sad_heart` | see "Shell hook" |
| `animate_parts` | show and wiggle the ears, wings, tuft, leaves and tail |

The balance numbers are first guesses and not play-tested over many days.

Saves, the profile, the graveyard and a lock file live in the user data folder (`~/.local/share/tamagotchi`
on Linux). Saves are written atomically, and an unreadable save is moved aside instead of
being overwritten.

## Your machine

The pet reacts to how the computer is doing. This is only for show, with no effect on its needs:

| Machine | Pet |
|---|---|
| CPU busy (1-minute load average at 80% of your cores or more) | sweat drops |
| Memory nearly full (85% or more in use) | dizzy, spinning eyes |
| Battery low (20% or less, and discharging) | sleepy eyes and a battery icon |

The status panel and the status-bar tooltip say what is going on. This reads `/proc` and `/sys`,
so it works on Linux; on other systems the pet just never notices. The thresholds are my
guesses. Your own machine can be tried out with the dev panel (`u`, `r`, `b`).

## Shell hook

Add this to `~/.zshrc` (zsh only for now) and your pet notices what you do in the terminal:

```sh
eval "$(tama hook zsh)"
```

- A successful `git commit` feeds it one hunger heart, at most once per 10 minutes.
- Every 5th failed command (Ctrl-C does not count) costs it one happy heart.

The hook only appends a line to a small queue file, so it is instant and works whether or not
the game is open; the game, or the next `tama feed`-style command, applies the queue. Both
numbers are in the config.

## Status bar and notifications

None of this is required to play.

**waybar**: a custom module that runs `tama status --waybar`:

```json
"custom/tama": {
    "exec": "~/.cargo/bin/tama status --waybar",
    "return-type": "json",
    "interval": 10,
    "tooltip": true
}
```

It shows `(^_^)`, adds `!` when the pet needs something, and the tooltip has the details. The
module gets the CSS classes `ok`, `alert`, `dead` and `none`, e.g. `#custom-tama.alert { color: #ff453a; }`.

**tmux** (not tried): `set -g status-right '#(tama status)'`.

**Notifications**: run `tama watch` in the background (for example `exec-once = tama watch` in
Hyprland, or a systemd user service). It checks every 30 seconds, notifies about the most
urgent need, repeats an unanswered one after 10 minutes, and stays quiet while the game is
open. It uses `notify-send` on Linux (tested) and `osascript` on macOS (written, never run),
and sends them under the app name `tama`. To show only tama's notifications in mako, end its
config with:

```ini
[expiring]
invisible=1

[!expiring]
invisible=1

[app-name=tama]
invisible=0
```

## Development

```sh
cargo test        # unit tests; file tests run in a temporary folder
tama --dev        # separate test save; press Tab for the dev panel
```

The dev panel changes hunger/happy/discipline/weight/mistakes, adds poop, sickness or a naughty
call, adds coins, grows or kills the pet, skips time and sets the clock, changes the speed, and switches the
machine reactions (CPU, RAM, battery) on and off.

Shift+R (then `y`) throws the current pet away and starts a new egg.

Source layout: `pet.rs` (the simulation), `sprites.rs` (pixel art), `main.rs` (UI and game
loop), `games.rs` (mini-games), `profile.rs` (coins, achievements), `sysinfo.rs` (machine load), `cli.rs` (shell commands, status bar, notifications),
`save.rs` (files and lock), `config.rs`.

Versions follow `MAJOR.MINOR.PATCH`; see `CLAUDE.md` for the rules.
