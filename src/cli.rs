//! Commands that work without opening the game: `tama status` for a status bar,
//! `tama feed|snack|toilet|medicine|scold|light` to care for the pet from the shell, and
//! `tama watch` to get desktop notifications while the game is closed.

use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::pet::{Pet, Stage, MAX_HEARTS};
use crate::save;

/// Load the real save, caught up to the present, without writing anything back.
/// The simulation is deterministic, so the game reaches the same state when it next opens.
pub fn load_caught_up(consume_events: bool) -> Option<Pet> {
    let save::Loaded::Pet(mut pet, away) = save::load() else { return None };
    let (cfg, _) = Config::load();
    pet.set_env(cfg, save::tz_offset());
    if pet.clock == 0 {
        pet.clock = (save::now() - away) as i64;
    }
    // The time since the last save is "away" unless a game window is open and saving.
    pet.away = !save::game_running();
    for _ in 0..away.min(crate::MAX_CATCHUP_TICKS) {
        pet.tick();
    }
    pet.away = false;
    pet.clock = save::now() as i64;
    // Things that happened in the shell since the last save. Only a caller that saves may use them up.
    let queued = if consume_events { save::take_events() } else { save::peek_events() };
    for e in queued.iter().take(crate::MAX_QUEUED_EVENTS) {
        pet.apply_event(e);
    }
    pet.events.clear();
    pet.load = crate::sysinfo::read();
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
        't' => "toilet",
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

/// The narrowest useful text for a status bar: the face, plus `!` when the pet needs something.
pub fn short_text(p: &Pet) -> String {
    match p.stage {
        Stage::Dead => "(x_x)".to_string(),
        Stage::Egg => "(.)".to_string(),
        _ if p.top_need().is_some() => format!("({}) !", face(p)),
        _ => format!("({})", face(p)),
    }
}

/// The full details, shown as a hover tooltip.
fn tooltip(p: &Pet) -> String {
    if !p.alive() {
        return format!("{} has passed away", p.species);
    }
    let mut s = format!("{} ({})", p.species, p.stage.label());
    if p.stage != Stage::Egg {
        s.push_str(&format!("\nhunger {}   happy {}", hearts(p.hunger), hearts(p.happy)));
    }
    if let Some((_, text)) = p.top_need() {
        s.push_str(&format!("\n{text}"));
    }
    if p.load.busy_cpu {
        s.push_str("\nThe CPU is busy - it is sweating");
    }
    if p.load.low_ram {
        s.push_str("\nMemory is nearly full - it feels dizzy");
    }
    if p.load.low_battery {
        s.push_str("\nThe battery is low - it is sleepy");
    }
    s
}

/// JSON for a waybar custom module with `"return-type": "json"`.
fn waybar_json(p: Option<&Pet>) -> String {
    let (text, tip, class) = match p {
        None => ("(?)".to_string(), "No pet yet - run tama".to_string(), "none"),
        Some(p) => {
            let class = if !p.alive() {
                "dead"
            } else if p.top_need().is_some() {
                "alert"
            } else {
                "ok"
            };
            (short_text(p), tooltip(p), class)
        }
    };
    serde_json::json!({ "text": text, "tooltip": tip, "class": class }).to_string()
}

fn status(waybar: bool) -> i32 {
    let pet = load_caught_up(false);
    if waybar {
        println!("{}", waybar_json(pet.as_ref()));
        return 0;
    }
    match pet {
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
    let Some(mut pet) = load_caught_up(true) else {
        eprintln!("No saved pet yet. Run `tama` to start one.");
        return 1;
    };
    let msg = match cmd {
        "feed" => pet.feed_meal(),
        "snack" => pet.feed_snack(),
        "toilet" => pet.toilet(),
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

/// The arguments for `osascript` that show a notification. Kept free of platform checks
/// so the quoting can be tested on any system.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn osascript_args(title: &str, body: &str) -> Vec<String> {
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    vec!["-e".to_string(), format!("display notification \"{}\" with title \"{}\"", esc(body), esc(title))]
}

/// Show a desktop notification. The error says what is missing.
fn notify(title: &str, body: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let result = Command::new("osascript").args(osascript_args(title, body)).status();
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
            match load_caught_up(false).and_then(|p| p.top_need().map(|(key, text)| (p.species.clone(), key, text))) {
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

/// The shell snippet for `tama hook zsh`: tells tama about git commits and failed commands.
const ZSH_HOOK: &str = r#"# tama: let your pet notice git commits (it gets fed) and failed commands (it gets sad).
# Add this line to ~/.zshrc:   eval "$(tama hook zsh)"
autoload -Uz add-zsh-hook
__tama_preexec() { __tama_cmd="$1" }
__tama_precmd() {
  local code=$?
  case "$__tama_cmd" in
    "") ;;
    git\ commit*|*"&& git commit"*) [ $code -eq 0 ] && { command tama event commit >/dev/null 2>&1 & } ;;
    *) [ $code -ne 0 ] && [ $code -ne 130 ] && { command tama event fail >/dev/null 2>&1 & } ;;
  esac
  __tama_cmd=""
}
add-zsh-hook preexec __tama_preexec
add-zsh-hook precmd __tama_precmd
"#;

fn hook(shell: Option<&str>) -> i32 {
    match shell {
        Some("zsh") => {
            print!("{ZSH_HOOK}");
            0
        }
        Some(other) => {
            eprintln!("No hook for {other} yet; only zsh is supported.");
            1
        }
        None => {
            eprintln!("Usage: tama hook zsh");
            1
        }
    }
}

fn event(kind: Option<&str>) -> i32 {
    match kind {
        Some(k @ ("commit" | "fail")) => {
            save::queue_event(k);
            0
        }
        _ => {
            eprintln!("Usage: tama event commit|fail");
            1
        }
    }
}

/// Run a subcommand if `args[1]` is one. Returns the exit code when handled.
pub fn run(args: &[String]) -> Option<i32> {
    let cmd = args.get(1)?.as_str();
    if !matches!(cmd, "status" | "feed" | "snack" | "toilet" | "medicine" | "scold" | "light" | "watch" | "event" | "hook") {
        return None;
    }
    if args.iter().any(|a| a == "--dev") {
        save::use_test_slot();
    }
    Some(match cmd {
        "status" => status(args.iter().any(|a| a == "--waybar")),
        "watch" => watch(args.iter().any(|a| a == "--test")),
        "event" => event(args.get(2).map(String::as_str)),
        "hook" => hook(args.get(2).map(String::as_str)),
        _ => act(cmd),
    })
}

/// `--gallery`: write a PNG of every form in every expression, without opening a window.
fn gallery() -> i32 {
    let (cfg, _) = Config::load();
    let path = std::path::PathBuf::from("tama-gallery.png");
    match crate::gallery::write(&path, &cfg) {
        Ok(()) => {
            let parts = if cfg.animate_parts { "on" } else { "off (`tama --animate on` to show them)" };
            println!("Wrote {}. Animated parts: {parts}.", path.display());
            0
        }
        Err(e) => {
            eprintln!("Could not write the gallery: {e}");
            1
        }
    }
}

/// GitHub repo (`owner/name`) that `tama --update` installs from.
/// Not set up yet; until it is, `--update` rebuilds from the local source folder instead.
const REPO: Option<&str> = None;

/// `--update`: reinstall the newest version with cargo.
fn update() -> i32 {
    let mut cmd = std::process::Command::new("cargo");
    // --locked builds with the versions in Cargo.lock, which are known to work.
    cmd.args(["install", "--force", "--locked"]);
    match REPO {
        Some(repo) => {
            cmd.args(["--git", &format!("https://github.com/{repo}")]);
        }
        None => {
            let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            if !src.join("Cargo.toml").exists() {
                eprintln!("Source folder {} no longer exists, and no GitHub repo is configured yet.", src.display());
                return 1;
            }
            println!("Rebuilding from {}", src.display());
            cmd.arg("--path").arg(src);
        }
    }
    match cmd.status() {
        Ok(s) if s.success() => 0,
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("Could not run cargo ({e}). Is Rust installed and on your PATH?");
            1
        }
    }
}

/// Handle a `--flag` that prints something and exits. Returns the exit code when handled.
pub fn flags(args: &[String]) -> Option<i32> {
    if args.iter().any(|a| a == "--update") {
        return Some(update());
    }
    if let Some(i) = args.iter().position(|a| a == "--animate") {
        match args.get(i + 1).map(String::as_str) {
            Some(v @ ("on" | "off")) => match Config::set_animate_parts(v == "on") {
                Ok(()) => println!("Animated parts (ears, wings, tuft, leaves, tail) are now {v}. Restart tama to see it."),
                Err(e) => {
                    eprintln!("Could not change the setting: {e}");
                    return Some(1);
                }
            },
            _ => {
                let state = if Config::load().0.animate_parts { "on" } else { "off" };
                println!("Animated parts (ears, wings, tuft, leaves, tail) are {state}. Use `tama --animate on` or `tama --animate off`.");
            }
        }
        return Some(0);
    }
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("tama {}", env!("CARGO_PKG_VERSION"));
        return Some(0);
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("tama - a pixel-art tamagotchi\n\n  tama            play (opens the window)\n  tama status     one line for a status bar (tmux: set -g status-right '#(tama status)'); --waybar prints JSON with a short text and a hover tooltip\n  tama feed|snack|toilet|medicine|scold|light   care for the pet without opening the game\n  tama hook zsh   shell snippet: git commits feed the pet, failed commands make it sad (eval \"$(tama hook zsh)\")\n  tama watch      desktop notifications when the pet needs you (run it in the background; `tama watch --test` sends a test)\n  tama --update   reinstall the newest version\n  tama --animate on|off   show/hide the animated ears, wings, tail\n  tama --version  print the version\n  tama --dev      developer mode (separate test save)\n  tama --gallery  write tama-gallery.png with every character and expression");
        return Some(0);
    }
    if args.iter().any(|a| a == "--gallery") {
        return Some(gallery());
    }
    None
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

    fn saved_adult(hunger: u8) -> Pet {
        let mut p = Pet::new(save::now());
        p.stage = Stage::Adult;
        p.species = "Mametchi".into();
        p.hunger = hunger;
        p.happy = 4;
        p.clock = save::now() as i64;
        p
    }

    #[test]
    fn care_commands_change_the_pet_and_save_it() {
        save::with_temp_dir(|| {
            save::save(&saved_adult(1)).unwrap();
            assert_eq!(act("feed"), 0);
            let p = load_caught_up(false).unwrap();
            assert!(p.hunger >= 2, "feeding should have been saved, hunger is {}", p.hunger);
        });
    }

    #[test]
    fn care_commands_refuse_while_the_game_is_open() {
        save::with_temp_dir(|| {
            save::save(&saved_adult(1)).unwrap();
            save::heartbeat();
            assert_eq!(act("feed"), 1);
            save::release();
            assert_eq!(load_caught_up(false).unwrap().hunger, 1, "nothing should have been changed");
        });
    }

    #[test]
    fn commands_without_a_save_say_so() {
        save::with_temp_dir(|| {
            assert_eq!(act("feed"), 1);
            assert_eq!(status(false), 1);
            let v: serde_json::Value = serde_json::from_str(&waybar_json(None)).unwrap();
            assert_eq!(v["class"], "none");
        });
    }

    #[test]
    fn a_pet_that_died_while_away_is_buried_by_a_care_command() {
        save::with_temp_dir(|| {
            let mut p = saved_adult(2);
            p.dev_kill();
            save::save(&p).unwrap();
            assert_eq!(act("feed"), 0);
            assert_eq!(save::graveyard().len(), 1);
        });
    }

    #[test]
    fn shell_events_are_applied_by_status_but_only_used_up_by_a_care_command() {
        save::with_temp_dir(|| {
            save::save(&saved_adult(2)).unwrap();
            assert_eq!(event(Some("commit")), 0);
            assert_eq!(load_caught_up(false).unwrap().hunger, 3, "a commit feeds it");
            assert_eq!(save::peek_events().len(), 1, "status must leave the queue alone");
            assert_eq!(load_caught_up(false).unwrap().hunger, 3, "...so asking again gives the same answer");
            assert_eq!(act("snack"), 0);
            assert!(save::peek_events().is_empty(), "a care command saves, so it uses the events up");
            assert_eq!(load_caught_up(false).unwrap().hunger, 3, "and the meal was kept");
        });
    }

    #[test]
    fn unknown_events_are_rejected_and_the_hook_snippet_is_printed_for_zsh_only() {
        save::with_temp_dir(|| {
            assert_eq!(event(Some("explode")), 1);
            assert_eq!(event(None), 1);
            assert!(save::peek_events().is_empty());
        });
        assert_eq!(hook(Some("zsh")), 0);
        assert_eq!(hook(Some("fish")), 1);
        assert!(ZSH_HOOK.contains("tama event commit") && ZSH_HOOK.contains("tama event fail"));
    }

    #[test]
    fn osascript_quoting_escapes_quotes_and_backslashes() {
        let args = osascript_args("a\"b", "c\\d");
        assert_eq!(args[0], "-e");
        assert_eq!(args[1], "display notification \"c\\\\d\" with title \"a\\\"b\"");
    }

    #[test]
    fn short_text_is_just_the_face_and_an_alert_mark() {
        let mut p = Pet::new(1);
        p.stage = Stage::Adult;
        p.hunger = 3;
        p.happy = 4;
        assert_eq!(short_text(&p), "(^_^)");
        p.hunger = 0;
        assert_eq!(short_text(&p), "(T_T) !");
    }

    #[test]
    fn the_tooltip_mentions_a_stressed_machine() {
        let mut p = Pet::new(1);
        p.stage = Stage::Adult;
        p.species = "Mametchi".into();
        p.load = crate::sysinfo::Strain { busy_cpu: true, low_ram: true, low_battery: true };
        let t = tooltip(&p);
        assert!(t.contains("sweating") && t.contains("dizzy") && t.contains("sleepy"), "{t}");
    }

    #[test]
    fn waybar_json_has_text_tooltip_and_class() {
        let mut p = Pet::new(1);
        p.stage = Stage::Adult;
        p.species = "Mametchi".into();
        p.hunger = 0;
        let v: serde_json::Value = serde_json::from_str(&waybar_json(Some(&p))).unwrap();
        assert_eq!(v["text"], "(T_T) !");
        assert_eq!(v["class"], "alert");
        assert!(v["tooltip"].as_str().unwrap().contains("Mametchi (Adult)\nhunger"));
        let none: serde_json::Value = serde_json::from_str(&waybar_json(None)).unwrap();
        assert_eq!(none["class"], "none");
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
