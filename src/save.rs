use crate::pet::Pet;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize, Deserialize)]
struct SaveFile {
    saved_at: u64,
    pet: Pet,
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// The machine's current offset from UTC in seconds.
pub fn tz_offset() -> i64 {
    chrono::Local::now().offset().local_minus_utc() as i64
}

/// Lets tests (and nothing else) point every file at a temporary directory.
static DATA_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// The override directory, if one is set. `config.rs` follows it too, so tests never touch real files.
pub(crate) fn dir_override() -> Option<PathBuf> {
    DATA_DIR.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn dir() -> PathBuf {
    if let Some(d) = dir_override() {
        return d;
    }
    let mut d = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    d.push("tamagotchi");
    d
}

static TEST_SLOT: AtomicBool = AtomicBool::new(false);

/// Use a separate save file so test modes never touch the real pet.
pub fn use_test_slot() {
    TEST_SLOT.store(true, Ordering::Relaxed);
}

fn save_path() -> PathBuf {
    dir().join(if TEST_SLOT.load(Ordering::Relaxed) { "save-test.json" } else { "save.json" })
}

/// Write to a temp file and rename it over the save, so a crash mid-write
/// can never leave a half-written save behind.
pub fn save(pet: &Pet) -> io::Result<()> {
    fs::create_dir_all(dir())?;
    let json = serde_json::to_string_pretty(&SaveFile { saved_at: now(), pet: pet.clone() })?;
    let path = save_path();
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)
}

pub enum Loaded {
    Pet(Pet, u64),
    None,
    /// The save existed but could not be read; it was moved aside to this path.
    Corrupt(PathBuf),
}

/// Returns the saved pet and the real seconds elapsed since it was saved.
pub fn load() -> Loaded {
    let path = save_path();
    let Ok(text) = fs::read_to_string(&path) else { return Loaded::None };
    match serde_json::from_str::<SaveFile>(&text) {
        Ok(f) => Loaded::Pet(f.pet, now().saturating_sub(f.saved_at)),
        Err(_) => {
            let aside = path.with_extension("json.corrupt");
            let _ = fs::rename(&path, &aside);
            Loaded::Corrupt(aside)
        }
    }
}

fn lock_path() -> PathBuf {
    dir().join(if TEST_SLOT.load(Ordering::Relaxed) { "tama-test.lock" } else { "tama.lock" })
}

/// Called by the game on every save so other commands know it is open.
pub fn heartbeat() {
    let _ = fs::create_dir_all(dir());
    let _ = fs::write(lock_path(), now().to_string());
}

/// Called when the game quits cleanly.
pub fn release() {
    let _ = fs::remove_file(lock_path());
}

/// True if a game window has saved within the last 30 seconds.
pub fn game_running() -> bool {
    fs::metadata(lock_path())
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|e| e.as_secs() < 30)
        .unwrap_or(false)
}

fn graveyard_path() -> PathBuf {
    dir().join(if TEST_SLOT.load(Ordering::Relaxed) { "graveyard-test.txt" } else { "graveyard.txt" })
}

fn fmt_date(ts: u64) -> String {
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .map(|d| d.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| ts.to_string())
}

pub fn bury(pet: &Pet) -> io::Result<()> {
    use io::Write;
    fs::create_dir_all(dir())?;
    let mut f = fs::OpenOptions::new().create(true).append(true).open(graveyard_path())?;
    writeln!(
        f,
        "{} | {} | lived {} | {} care mistakes | cause: {}",
        fmt_date(now()),
        pet.species,
        crate::fmt_age(pet.age),
        pet.mistakes,
        pet.cause_of_death.as_deref().unwrap_or("unknown")
    )
}

/// All past pets, newest first.
pub fn graveyard() -> Vec<String> {
    let text = fs::read_to_string(graveyard_path()).unwrap_or_default();
    text.lines().rev().map(str::to_owned).collect()
}

/// Run `f` with every save, lock, graveyard and config file redirected to a fresh temporary
/// directory. Tests that touch files go through here, one at a time.
#[cfg(test)]
pub(crate) fn with_temp_dir<R>(f: impl FnOnce() -> R) -> R {
    static GATE: Mutex<()> = Mutex::new(());
    static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let _gate = GATE.lock().unwrap_or_else(|e| e.into_inner());
    let d = std::env::temp_dir().join(format!(
        "tama-test-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    *DATA_DIR.lock().unwrap_or_else(|e| e.into_inner()) = Some(d.clone());
    TEST_SLOT.store(false, Ordering::Relaxed);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    *DATA_DIR.lock().unwrap_or_else(|e| e.into_inner()) = None;
    let _ = fs::remove_dir_all(&d);
    match result {
        Ok(v) => v,
        Err(e) => std::panic::resume_unwind(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pet::Stage;

    #[test]
    fn save_round_trips_and_leaves_no_temp_file() {
        with_temp_dir(|| {
            let mut p = Pet::new(1);
            p.stage = Stage::Adult;
            p.species = "Mametchi".into();
            p.age = 1234;
            save(&p).unwrap();
            assert!(!dir().join("save.json.tmp").exists());
            match load() {
                Loaded::Pet(q, away) => {
                    assert_eq!(q.species, "Mametchi");
                    assert_eq!(q.age, 1234);
                    assert!(away < 5);
                }
                _ => panic!("expected the saved pet"),
            }
        });
    }

    #[test]
    fn an_unreadable_save_is_moved_aside_not_overwritten() {
        with_temp_dir(|| {
            fs::write(save_path(), "not json at all").unwrap();
            match load() {
                Loaded::Corrupt(aside) => assert!(aside.exists()),
                _ => panic!("expected the corrupt-save path"),
            }
            assert!(matches!(load(), Loaded::None));
        });
    }

    #[test]
    fn the_test_slot_uses_separate_files() {
        with_temp_dir(|| {
            save(&Pet::new(1)).unwrap();
            use_test_slot();
            assert!(matches!(load(), Loaded::None), "the test slot must not see the real save");
            save(&Pet::new(2)).unwrap();
            assert!(dir().join("save-test.json").exists());
            assert!(dir().join("save.json").exists());
        });
    }

    #[test]
    fn the_lock_tracks_whether_a_game_is_running() {
        with_temp_dir(|| {
            assert!(!game_running());
            heartbeat();
            assert!(game_running());
            release();
            assert!(!game_running());
        });
    }

    #[test]
    fn the_graveyard_lists_newest_first() {
        with_temp_dir(|| {
            let mut p = Pet::new(1);
            p.species = "Marutchi".into();
            p.cause_of_death = Some("starvation".into());
            bury(&p).unwrap();
            p.species = "Mametchi".into();
            p.cause_of_death = Some("old age".into());
            bury(&p).unwrap();
            let g = graveyard();
            assert_eq!(g.len(), 2);
            assert!(g[0].contains("Mametchi") && g[0].contains("old age"));
            assert!(g[1].contains("Marutchi") && g[1].contains("starvation"));
        });
    }
}
