use crate::pet::Pet;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

#[derive(Serialize, Deserialize)]
struct SaveFile {
    saved_at: u64,
    pet: Pet,
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn dir() -> PathBuf {
    let mut d = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    d.push("tamagotchi");
    d
}

fn save_path() -> PathBuf {
    dir().join("save.json")
}

pub fn save(pet: &Pet) -> io::Result<()> {
    fs::create_dir_all(dir())?;
    let json = serde_json::to_string_pretty(&SaveFile { saved_at: now(), pet: pet.clone() })?;
    fs::write(save_path(), json)
}

/// Returns the saved pet and the real seconds elapsed since it was saved.
pub fn load() -> Option<(Pet, u64)> {
    let text = fs::read_to_string(save_path()).ok()?;
    let f: SaveFile = serde_json::from_str(&text).ok()?;
    Some((f.pet, now().saturating_sub(f.saved_at)))
}

pub fn bury(pet: &Pet) -> io::Result<()> {
    use io::Write;
    fs::create_dir_all(dir())?;
    let mut f = fs::OpenOptions::new().create(true).append(true).open(dir().join("graveyard.txt"))?;
    writeln!(
        f,
        "{} - lived {} as a {}, {} care mistakes, cause: {}",
        now(),
        crate::fmt_age(pet.age),
        pet.species,
        pet.mistakes,
        pet.cause_of_death.as_deref().unwrap_or("unknown")
    )
}
