use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// Gameplay timings, loaded from `~/.config/tamagotchi/config.json`.
/// Every field is optional in the file; missing ones keep these defaults.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    /// Seconds per lost hunger heart, by life stage.
    pub hunger_secs_baby: u64,
    pub hunger_secs_child: u64,
    pub hunger_secs_teen: u64,
    pub hunger_secs_adult: u64,
    /// Happiness decays this many times slower than hunger.
    pub happy_decay_factor: f64,
    /// A new poop appears every `poop_min_secs` + random(0..`poop_extra_secs`).
    pub poop_min_secs: u64,
    pub poop_extra_secs: u64,
    /// Chance of random sickness is 1 in N per awake second.
    pub sick_odds: u64,
    /// Chance of a naughty call is 1 in N per awake second.
    pub naughty_odds: u64,
    /// Seconds a need can go unanswered before it counts as a care mistake.
    pub mistake_after_secs: u64,
    /// How long each stage lasts, in seconds.
    pub baby_secs: u64,
    pub child_secs: u64,
    pub teen_secs: u64,
    pub adult_lifespan_secs: u64,
    /// The pet sleeps from this local hour to the end hour (24h clock).
    pub sleep_start_hour: i64,
    pub sleep_end_hour: i64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hunger_secs_baby: 240,
            hunger_secs_child: 480,
            hunger_secs_teen: 600,
            hunger_secs_adult: 720,
            happy_decay_factor: 1.25,
            poop_min_secs: 900,
            poop_extra_secs: 900,
            sick_odds: 20_000,
            naughty_odds: 1800,
            mistake_after_secs: 900,
            baby_secs: 600,
            child_secs: 3600,
            teen_secs: 7200,
            adult_lifespan_secs: 2 * 86_400,
            sleep_start_hour: 22,
            sleep_end_hour: 7,
        }
    }
}

fn path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("tamagotchi").join("config.json"))
}

impl Config {
    /// Load the config; write the defaults on first run so the file is easy to find and edit.
    /// Returns the config plus a warning if the file exists but could not be parsed.
    pub fn load() -> (Config, Option<String>) {
        let Some(p) = path() else { return (Config::default(), None) };
        match fs::read_to_string(&p) {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(c) => (c, None),
                Err(e) => (Config::default(), Some(format!("config.json ignored ({e}); using defaults"))),
            },
            Err(_) => {
                if let Some(dir) = p.parent() {
                    let _ = fs::create_dir_all(dir);
                }
                if let Ok(json) = serde_json::to_string_pretty(&Config::default()) {
                    let _ = fs::write(&p, json);
                }
                (Config::default(), None)
            }
        }
    }
}
