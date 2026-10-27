use crate::pet::Pet;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
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

fn dir() -> PathBuf {
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
