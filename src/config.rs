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
    /// After a meal the pet needs the toilet in `urge_after_meal_secs` + random(0..`..extra_secs`).
    pub urge_after_meal_secs: u64,
    pub urge_after_meal_extra_secs: u64,
    /// Out of nowhere it needs the toilet with chance 1 in N per awake second.
    pub urge_odds: u64,
    /// An urge ignored this long ends in an accident.
    pub urge_patience_secs: u64,
    /// Chance of random sickness is 1 in N per awake second.
    pub sick_odds: u64,
    /// Chance of a naughty call is 1 in N per awake second.
    pub naughty_odds: u64,
    /// An overweight pet is this many times more likely to fall sick.
    pub overweight_sick_factor: u64,
    /// Show (and wiggle) the ears, wings, tuft, leaves and tail. When off those parts are not drawn at all.
    pub animate_parts: bool,
    /// While the game is closed the pet's needs fall this many times slower (1 = no change).
    pub away_slowdown: u64,
    /// While the game is closed the pet is looked after loosely: hunger and happiness never drop
    /// below one heart, and it cannot fall sick, act up or die. Growing up still carries on.
    pub away_safe: bool,
    /// A git commit feeds the pet, but at most once per this many seconds (see `tama hook`).
    pub commit_meal_gap_secs: i64,
    /// This many failed shell commands make the pet lose one happy heart.
    pub fails_per_sad_heart: u8,
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
            urge_after_meal_secs: 60,
            urge_after_meal_extra_secs: 120,
            urge_odds: 7200,
            urge_patience_secs: 600,
            sick_odds: 20_000,
            naughty_odds: 1800,
            overweight_sick_factor: 3,
            animate_parts: false,
            away_slowdown: 3,
            away_safe: true,
            commit_meal_gap_secs: 600,
            fails_per_sad_heart: 5,
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
    if let Some(d) = crate::save::dir_override() {
        return Some(d.join("config.json"));
    }
    #[cfg(test)]
    panic!("a test touched the real config folder; wrap it in save::with_temp_dir");
    #[cfg(not(test))]
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

impl Config {
    /// Change one setting in config.json without touching the user's other edits.
    pub fn set_animate_parts(on: bool) -> Result<(), String> {
        let p = path().ok_or("no config directory on this system")?;
        let mut value: serde_json::Value = match fs::read_to_string(&p) {
            Ok(text) => serde_json::from_str(&text).map_err(|e| format!("config.json is not valid JSON ({e}); fix or delete it first"))?,
            Err(_) => serde_json::to_value(Config::default()).map_err(|e| e.to_string())?,
        };
        value["animate_parts"] = serde_json::Value::Bool(on);
        if let Some(dir) = p.parent() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
        fs::write(&p, json).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::with_temp_dir;

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let c: Config = serde_json::from_str(r#"{"sleep_start_hour": 20}"#).unwrap();
        assert_eq!(c.sleep_start_hour, 20);
        assert_eq!(c.hunger_secs_baby, Config::default().hunger_secs_baby);
    }

    #[test]
    fn first_run_writes_defaults_and_a_broken_file_warns() {
        with_temp_dir(|| {
            let (c, warning) = Config::load();
            assert!(warning.is_none());
            assert_eq!(c.sleep_start_hour, Config::default().sleep_start_hour);
            assert!(path().unwrap().exists(), "defaults should be written for the user to edit");

            fs::write(path().unwrap(), "{nope").unwrap();
            let (c, warning) = Config::load();
            assert!(warning.is_some());
            assert_eq!(c.sleep_start_hour, Config::default().sleep_start_hour);
        });
    }

    #[test]
    fn toggling_animate_parts_keeps_the_users_other_edits() {
        with_temp_dir(|| {
            fs::write(path().unwrap(), r#"{"sleep_start_hour": 21}"#).unwrap();
            Config::set_animate_parts(true).unwrap();
            let (c, _) = Config::load();
            assert!(c.animate_parts);
            assert_eq!(c.sleep_start_hour, 21);
            Config::set_animate_parts(false).unwrap();
            assert!(!Config::load().0.animate_parts);
        });
    }

    #[test]
    fn toggling_refuses_to_overwrite_a_broken_file() {
        with_temp_dir(|| {
            fs::write(path().unwrap(), "{nope").unwrap();
            assert!(Config::set_animate_parts(true).is_err());
            assert_eq!(fs::read_to_string(path().unwrap()).unwrap(), "{nope");
        });
    }
}
