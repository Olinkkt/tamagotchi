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
    face: Some(FaceSpot { style: FaceStyle::Small, x: 10, y: 15 }),
    parts: &[],
};

pub const POOP: Character = Character { name: "poop", size: 16, face: None, parts: &[] };
pub const GRAVE: Character = Character { name: "grave", size: 32, face: None, parts: &[] };
pub const GHOST: Character = Character { name: "ghost", size: 24, face: None, parts: &[] };

pub const ALL: [Character; 5] = [EGG, BABY, POOP, GRAVE, GHOST];

/// The spot on the egg where the cracks go.
pub const EGG_CRACK_AT: (i32, i32) = (9, 15);
/// Where the ghost's face goes.
pub const GHOST_FACE_AT: (i32, i32) = (9, 8);
