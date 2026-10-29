//! What the game needs to know about each character's sprite sheet: its size, where the face
//! goes, and which animated parts it has. The sprite tool includes this file too and lays out
//! the sheets from it, so the two always agree.
//!
//! A sheet is one row of square cells: the body first, then every frame of every part in order.

use crate::faces::FaceStyle;

#[derive(Clone, Copy, Debug)]
pub struct FaceSpot {
    pub style: FaceStyle,
    /// Top-left corner of the face grid, in sprite pixels.
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Part {
    pub name: &'static str,
    pub frames: usize,
    /// Drawn over the body (true) or behind it.
    pub front: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Character {
    pub name: &'static str,
    pub size: usize,
    pub face: Option<FaceSpot>,
    pub parts: &'static [Part],
}

impl Character {
    pub fn cells(&self) -> usize {
        1 + self.parts.iter().map(|p| p.frames).sum::<usize>()
    }

    /// The sheet cell of `frame` of part number `part`.
    pub fn part_cell(&self, part: usize, frame: usize) -> usize {
        1 + self.parts[..part].iter().map(|p| p.frames).sum::<usize>() + frame
    }
}

pub const EGG: Character = Character { name: "egg", size: 32, face: None, parts: &[] };

pub const BABY: Character = Character {
    name: "baby",
    size: 32,
    face: SMALL_FACE_32,
    parts: &[],
};

const SMALL_FACE_32: Option<FaceSpot> = Some(FaceSpot { style: FaceStyle::Small, x: 10, y: 15 });
const fn large_face(size: i32, y: i32) -> Option<FaceSpot> {
    Some(FaceSpot { style: FaceStyle::Large, x: size / 2 - 8, y })
}
const fn part(name: &'static str, front: bool) -> Part {
    Part { name, frames: 3, front }
}

pub const CHILD: Character = Character { name: "marutchi", size: 32, face: SMALL_FACE_32, parts: &[] };
pub const TONMARUTCHI: Character = Character { name: "tonmarutchi", size: 40, face: large_face(40, 19), parts: &[part("ears", false)] };
pub const HASHITAMATCHI: Character = Character { name: "hashitamatchi", size: 40, face: large_face(40, 19), parts: &[] };
pub const MAMETCHI: Character = Character { name: "mametchi", size: 48, face: large_face(48, 19), parts: &[part("ears", false)] };
pub const KUCHIPATCHI: Character = Character { name: "kuchipatchi", size: 48, face: large_face(48, 19), parts: &[part("wings", false)] };
pub const TARAKOTCHI: Character = Character { name: "tarakotchi", size: 48, face: large_face(48, 19), parts: &[part("tuft", true)] };
pub const ICHIGOTCHI: Character = Character { name: "ichigotchi", size: 48, face: large_face(48, 19), parts: &[part("leaves", true)] };
pub const NYOROTCHI: Character = Character { name: "nyorotchi", size: 48, face: large_face(48, 18), parts: &[part("tail", false)] };

pub const POOP: Character = Character { name: "poop", size: 16, face: None, parts: &[] };
pub const GRAVE: Character = Character { name: "grave", size: 32, face: None, parts: &[] };
pub const GHOST: Character = Character { name: "ghost", size: 24, face: None, parts: &[] };

pub const ALL: [Character; 13] =
    [EGG, BABY, CHILD, TONMARUTCHI, HASHITAMATCHI, MAMETCHI, KUCHIPATCHI, TARAKOTCHI, ICHIGOTCHI, NYOROTCHI, POOP, GRAVE, GHOST];

/// The sprite set for a species, if it has one.
pub fn for_species(species: &str) -> Option<Character> {
    let name = species.to_lowercase();
    ALL.into_iter().find(|c| c.name == name || (c.name == "baby" && name == "babytchi"))
}

/// The spot on the egg where the cracks go.
pub const EGG_CRACK_AT: (i32, i32) = (9, 15);
/// Where the ghost's face goes.
pub const GHOST_FACE_AT: (i32, i32) = (9, 8);
