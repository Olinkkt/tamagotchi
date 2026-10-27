//! Commands that work without opening the game: `tama status` for a status bar,
//! `tama feed|snack|clean|medicine|scold|light` to care for the pet from the shell, and
//! `tama watch` to get desktop notifications while the game is closed.

use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::pet::{Pet, Stage, MAX_HEARTS};
use crate::save;

/// Load the real save, caught up to the present, without writing anything back.
/// The simulation is deterministic, so the game reaches the same state when it next opens.
pub fn load_caught_up() -> Option<Pet> {
    let save::Loaded::Pet(mut pet, away) = save::load() else { return None };
    let (cfg, _) = Config::load();
    pet.set_env(cfg, save::tz_offset());
    if pet.clock == 0 {
        pet.clock = (save::now() - away) as i64;
    }
    for _ in 0..away.min(crate::MAX_CATCHUP_TICKS) {
        pet.tick();
    }
    pet.clock = save::now() as i64;
    pet.events.clear();
    Some(pet)
}

fn hearts(n: u8) -> String {
    "♥".repeat(n as usize) + &"♡".repeat((MAX_HEARTS - n) as usize)
}

fn face(p: &Pet) -> &'static str {
    if !p.alive() {
        "x_x"
    } else if p.asleep {
        "-_-"
    } else if p.sick {
        "@_@"
    } else if p.hunger == 0 || p.happy == 0 {
        "T_T"
    } else if p.happy >= 3 {
        "^_^"
    } else {
        "o_o"
    }
}

fn action_word(key: char) -> &'static str {
    match key {
        'f' => "feed",
        'm' => "medicine",
        'd' => "scold",
        'c' => "clean",
        'p' => "play",
        'l' => "light off",
        _ => "new egg",
    }
}

/// One compact line for a status bar, e.g. `(^_^) Mametchi ♥♥♥♡ ♥♥♥♥` or `(T_T) Marutchi ♥♡♡♡ ♥♥♡♡ ! feed`.
pub fn status_line(p: &Pet) -> String {
    match p.stage {
        Stage::Dead => format!("({}) {} has passed away", face(p), p.species),
        Stage::Egg => "(.) egg".to_string(),
        _ => {
            let mut s = format!("({}) {} {} {}", face(p), p.species, hearts(p.hunger), hearts(p.happy));
            if let Some((key, _)) = p.top_need() {
                s.push_str(&format!(" ! {}", action_word(key)));
            }
            s
        }
    }
}

fn status() -> i32 {
    match load_caught_up() {
        Some(p) => {
            println!("{}", status_line(&p));
            0
        }
        None => {
            println!("(no pet yet - run tama)");
            1
        }
    }
}

fn act(cmd: &str) -> i32 {
    if save::game_running() {
        eprintln!("The game is open - care for the pet there (changes made here would be overwritten).");
        return 1;
    }
    let Some(mut pet) = load_caught_up() else {
        eprintln!("No saved pet yet. Run `tama` to start one.");
        return 1;
    };
    let msg = match cmd {
        "feed" => pet.feed_meal(),
        "snack" => pet.feed_snack(),
        "clean" => pet.clean(),
        "medicine" => pet.medicine(),
        "scold" => pet.scold(),
        _ => pet.toggle_light(),
    };
    if !pet.alive() && !pet.buried {
        pet.buried = true;
        let _ = save::bury(&pet);
    }
    if let Err(e) = save::save(&pet) {
        eprintln!("Could not save: {e}");
        return 1;
    }
    println!("{msg}");
    0
}

/// Show a desktop notification. The error says what is missing.
fn notify(title: &str, body: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let result = {
        let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
        Command::new("osascript")
            .args(["-e", &format!("display notification \"{}\" with title \"{}\"", esc(body), esc(title))])
            .status()
    };
    #[cfg(not(target_os = "macos"))]
    let result = Command::new("notify-send").args(["-a", "tama", title, body]).status();
    match result {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("the notifier exited with {s}")),
        Err(e) => Err(format!("could not run the notifier ({e}); on Linux, install notify-send (libnotify)")),
    }
}

const WATCH_EVERY: Duration = Duration::from_secs(30);
/// An unanswered need is announced again after this long.
const RENOTIFY_AFTER: Duration = Duration::from_secs(600);

/// Notify for a new need, or the same one still unanswered after a while.
/// The "it has passed away" notice is only sent once.
fn should_notify(last: Option<(char, Duration)>, key: char) -> bool {
    match last {
        None => true,
        Some((k, since)) => k != key || (key != 'n' && since >= RENOTIFY_AFTER),
    }
}

fn watch(test: bool) -> i32 {
    if test {
        return match notify("tama", "Test notification: the watcher can reach your desktop.") {
            Ok(()) => {
                println!("Sent a test notification.");
                0
            }
            Err(e) => {
                eprintln!("{e}");
                1
            }
        };
    }
    println!("tama watch: checking every 30s and notifying when the pet needs you (only while the game is closed). Ctrl-C to stop.");
    let mut last: Option<(char, Instant)> = None;
    let mut warned = false;
    loop {
        if !save::game_running() {
            match load_caught_up().and_then(|p| p.top_need().map(|(key, text)| (p.species.clone(), key, text))) {
                Some((species, key, text)) => {
                    if should_notify(last.map(|(k, at)| (k, at.elapsed())), key) {
                        match notify(&species, text) {
                            Ok(()) => last = Some((key, Instant::now())),
                            Err(e) if !warned => {
                                eprintln!("{e}");
                                warned = true;
                            }
                            Err(_) => {}
                        }
                    }
                }
                None => last = None,
            }
        }
        thread::sleep(WATCH_EVERY);
    }
}

/// Run a subcommand if `args[1]` is one. Returns the exit code when handled.
pub fn run(args: &[String]) -> Option<i32> {
    let cmd = args.get(1)?.as_str();
    if !matches!(cmd, "status" | "feed" | "snack" | "clean" | "medicine" | "scold" | "light" | "watch") {
        return None;
    }
    if args.iter().any(|a| a == "--dev" || a == "--fast") {
        save::use_test_slot();
    }
    Some(match cmd {
        "status" => status(),
        "watch" => watch(args.iter().any(|a| a == "--test")),
        _ => act(cmd),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_line_shows_hearts_and_the_top_need() {
        let mut p = Pet::new(1);
        p.stage = Stage::Adult;
        p.species = "Mametchi".into();
        p.hunger = 3;
        p.happy = 4;
        assert_eq!(status_line(&p), "(^_^) Mametchi ♥♥♥♡ ♥♥♥♥");
        p.hunger = 0;
        assert_eq!(status_line(&p), "(T_T) Mametchi ♡♡♡♡ ♥♥♥♥ ! feed");
    }

    #[test]
    fn notifications_fire_for_new_needs_and_repeat_slowly() {
        assert!(should_notify(None, 'f'));
        assert!(!should_notify(Some(('f', Duration::from_secs(60))), 'f'));
        assert!(should_notify(Some(('f', RENOTIFY_AFTER)), 'f'));
        assert!(should_notify(Some(('f', Duration::from_secs(5))), 'm'));
        assert!(!should_notify(Some(('n', RENOTIFY_AFTER * 10)), 'n'));
    }

    #[test]
    fn status_line_for_egg_and_dead_pet() {
        let mut p = Pet::new(1);
        assert_eq!(status_line(&p), "(.) egg");
        p.stage = Stage::Dead;
        p.species = "Marutchi".into();
        assert_eq!(status_line(&p), "(x_x) Marutchi has passed away");
    }
}
