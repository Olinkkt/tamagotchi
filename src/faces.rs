//! Faces and other small details, drawn pixel by pixel because every pixel matters at this size.
//! They are laid over a body sprite at the spot `cast.rs` gives for that character. The sprite
//! tool includes this file too, so its previews show exactly what the game draws.
//!
//! In a grid, `.` is see-through and every other letter is one palette colour (see `colour`).

use crate::palette::Pal;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Expr {
    Idle,
    Happy,
    Sad,
    Sick,
    Sleeping,
    Eating,
    Blink,
}

impl Expr {
    pub const ALL: [Expr; 7] = [Expr::Idle, Expr::Happy, Expr::Sad, Expr::Sick, Expr::Sleeping, Expr::Eating, Expr::Blink];

    pub fn name(self) -> &'static str {
        match self {
            Expr::Idle => "idle",
            Expr::Happy => "happy",
            Expr::Sad => "sad",
            Expr::Sick => "sick",
            Expr::Sleeping => "sleeping",
            Expr::Eating => "eating",
            Expr::Blink => "blink",
        }
    }
}

/// Which set of face grids a character uses. Characters of a similar size share one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FaceStyle {
    Small,
}

pub fn colour(ch: char) -> Option<Pal> {
    Some(match ch {
        'k' => Pal::Ink,
        'w' => Pal::White,
        'p' => Pal::Pink,
        'b' => Pal::Blush,
        'r' => Pal::Red,
        'm' => Pal::Maroon,
        's' => Pal::Sky,
        'i' => Pal::Ice,
        'g' => Pal::Green,
        'l' => Pal::Lime,
        'v' => Pal::Violet,
        'x' => Pal::Night,
        'z' => Pal::Dusk,
        'a' => Pal::Slate,
        'h' => Pal::Mist,
        'u' => Pal::Rust,
        'o' => Pal::Orange,
        'y' => Pal::Yellow,
        'c' => Pal::Cream,
        'f' => Pal::Forest,
        'd' => Pal::DeepTeal,
        'e' => Pal::Teal,
        'n' => Pal::Brown,
        't' => Pal::Tan,
        _ => return None,
    })
}

/// 12 pixels wide, mirrored about the middle; eye glints sit top-left, where the light comes from.
const SMALL_IDLE: [&str; 6] = [
    "..wk....wk..",
    "..kk....kk..",
    "..kk....kk..",
    "............",
    ".....kk.....",
    "............",
];
const SMALL_HAPPY: [&str; 6] = [
    "..kk....kk..",
    ".k..k..k..k.",
    "............",
    "pp........pp",
    "....krrk....",
    ".....kk.....",
];
const SMALL_SAD: [&str; 6] = [
    "..k......k..",
    ".k........k.",
    "..wk....wk..",
    "..kk....kk..",
    "..s..kk.....",
    "..s.k..k....",
];
const SMALL_SICK: [&str; 6] = [
    ".k.k....k.k.",
    "..k......k..",
    ".k.k....k.k.",
    "l..........l",
    "...k.kk.k...",
    "....k..k....",
];
const SMALL_SLEEPING: [&str; 6] = [
    "............",
    ".k..k..k..k.",
    "..kk....kk..",
    "............",
    ".....kk.....",
    "............",
];
const SMALL_EATING: [&str; 6] = [
    "..kk....kk..",
    ".k..k..k..k.",
    "............",
    "....kkkk....",
    "....kmmk....",
    ".....kk.....",
];
const SMALL_BLINK: [&str; 6] = [
    "............",
    "............",
    "..kk....kk..",
    "............",
    ".....kk.....",
    "............",
];

pub fn grid(style: FaceStyle, e: Expr) -> &'static [&'static str] {
    match (style, e) {
        (FaceStyle::Small, Expr::Idle) => &SMALL_IDLE,
        (FaceStyle::Small, Expr::Happy) => &SMALL_HAPPY,
        (FaceStyle::Small, Expr::Sad) => &SMALL_SAD,
        (FaceStyle::Small, Expr::Sick) => &SMALL_SICK,
        (FaceStyle::Small, Expr::Sleeping) => &SMALL_SLEEPING,
        (FaceStyle::Small, Expr::Eating) => &SMALL_EATING,
        (FaceStyle::Small, Expr::Blink) => &SMALL_BLINK,
    }
}

/// Cracks in the egg shell as it gets close to hatching, laid over the egg sprite.
pub const EGG_CRACKS: [[&str; 3]; 2] = [
    [
        "......k.......",
        ".....k.k..k...",
        "....k...kk....",
    ],
    [
        ".k....k.....k.",
        "k.k..k.k..kk.k",
        "...kk...kk....",
    ],
];

/// Spinning rings in place of the eyes while the machine's memory is nearly full, with the
/// idle mouth. `phase` moves the gap in each ring.
pub fn dizzy(style: FaceStyle, phase: usize) -> Vec<(i32, i32, Pal)> {
    let (eyes, mouth) = match style {
        FaceStyle::Small => ([(2, 1), (9, 1)], [(5, 4), (6, 4)]),
    };
    let ring = [(-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)];
    let mut out: Vec<(i32, i32, Pal)> = mouth.iter().map(|&(x, y)| (x, y, Pal::Ink)).collect();
    for (side, (cx, cy)) in eyes.into_iter().enumerate() {
        out.push((cx, cy, Pal::Ink));
        for (i, (dx, dy)) in ring.iter().enumerate() {
            if i != (phase + side * 4) % ring.len() {
                out.push((cx + dx, cy + dy, Pal::Ink));
            }
        }
    }
    out
}

/// The ghost of a pet that has passed away.
pub const GHOST_FACE: [&str; 5] = [
    "kk..kk",
    "kk..kk",
    "......",
    "..kk..",
    "..kk..",
];

/// Every coloured pixel of a grid, as (x, y, colour) relative to its top-left corner.
pub fn pixels<'a>(rows: &'a [&'a str]) -> impl Iterator<Item = (i32, i32, Pal)> + 'a {
    rows.iter().enumerate().flat_map(|(y, row)| {
        row.chars().enumerate().filter_map(move |(x, ch)| colour(ch).map(|p| (x as i32, y as i32, p)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_grid_is_rectangular_and_uses_known_letters() {
        let styles = [FaceStyle::Small];
        let grids = styles.iter().flat_map(|s| Expr::ALL.map(|e| grid(*s, e)));
        for g in grids.chain(EGG_CRACKS.iter().map(|g| g.as_slice())) {
            let w = g[0].len();
            for row in g {
                assert_eq!(row.len(), w, "{g:?}");
                assert!(row.chars().all(|c| c == '.' || colour(c).is_some()), "{row}");
            }
        }
    }

    #[test]
    fn face_grids_are_mirror_images_apart_from_glints() {
        // The sad face's single tear is deliberately one-sided.
        for e in Expr::ALL.into_iter().filter(|e| *e != Expr::Sad) {
            for row in grid(FaceStyle::Small, e) {
                let shape: String = row.chars().map(|c| if c == 'w' { 'k' } else { c }).collect();
                assert_eq!(shape, shape.chars().rev().collect::<String>(), "{e:?}: {row}");
            }
        }
    }
}
