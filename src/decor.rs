//! Things you buy for the room. They do nothing in play and show on the wall, the floor and
//! the windowsill, so a player learns them by seeing them. What you own lives in the profile,
//! so it survives a pet dying. All positions are in the standard 160x116 room.

use crate::frame::Frame;
use crate::icons::Grid;
use crate::palette::Pal;
use crate::profile::{self, Profile};

/// A place in the room that holds one thing; buying another thing for it swaps them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spot {
    Poster,
    FloorLeft,
    FloorRight,
    /// A corner of its own, past the lamp, in the floor the room shows beyond the stage.
    Corner,
}

impl Spot {
    /// The top-left corner of an item of this size on its spot.
    fn at(self, w: i32, h: i32) -> (i32, i32) {
        match self {
            // Left of the window, in the wall the room shows beyond the 160-wide stage.
            Spot::Poster => (-18, 16),
            Spot::Corner => (162, FLOOR - h),
            // The floor pieces stand on the same line, a little way in from the room's edge.
            Spot::FloorLeft => (12, FLOOR - h),
            Spot::FloorRight => (156 - w, FLOOR - h),
        }
    }
}

const FLOOR: i32 = 108;

pub struct Decor {
    pub id: &'static str,
    pub name: &'static str,
    pub blurb: &'static str,
    pub price: u32,
    pub spot: Spot,
    pub icon: Grid,
}

pub const DECOR: [Decor; 4] = [
    Decor { id: "poster", name: "Star poster", blurb: "for the wall", price: 30, spot: Spot::Poster, icon: POSTER },
    Decor { id: "fern", name: "Potted fern", blurb: "left of the rug", price: 40, spot: Spot::FloorLeft, icon: FERN },
    Decor { id: "lamp", name: "Floor lamp", blurb: "right of the rug", price: 60, spot: Spot::FloorRight, icon: LAMP },
    Decor { id: "teddy", name: "Teddy bear", blurb: "on its own cushion", price: 45, spot: Spot::Corner, icon: TEDDY },
];

/// What the memorial plaque costs: cheap enough to buy after every pet.
pub const PLAQUE_PRICE: u32 = 20;
/// Right of the shelf, level with it, in the wall beyond the stage.
const PLAQUE: (i32, i32, i32, i32) = (154, 24, 26, 16);
/// The windowsill, where a keepsake stands: its middle and the line it rests on.
const SILL: (i32, i32) = (34, 57);

pub fn by_id(id: &str) -> Option<&'static Decor> {
    DECOR.iter().find(|d| d.id == id)
}

/// Draw everything the profile has put in the room. `dx`, `dy` move the standard room to where
/// it sits on screen.
pub fn draw_room(f: &mut Frame, p: &Profile, dx: i32, dy: i32) {
    for d in DECOR.iter().filter(|d| p.placed(d.id)) {
        let (w, h) = (d.icon[0].chars().count() as i32, d.icon.len() as i32);
        let (x, y) = d.spot.at(w, h);
        f.grid(d.icon, x + dx, y + dy);
    }
    if let Some(id) = &p.shelf {
        if let Some(k) = profile::KEEPSAKES.iter().find(|k| k.id == id) {
            f.grid(k.icon, SILL.0 - 6 + dx, SILL.1 - 12 + dy);
        }
    }
    if let Some(plaque) = &p.plaque {
        draw_plaque(f, plaque, dx, dy);
    }
}

/// A small wooden plaque: a heart in the colour of the form the pet had, and how long it lived.
fn draw_plaque(f: &mut Frame, plaque: &profile::Plaque, dx: i32, dy: i32) {
    let (x, y, w, h) = (PLAQUE.0 + dx, PLAQUE.1 + dy, PLAQUE.2, PLAQUE.3);
    f.boxed(x, y, w, h, Pal::Night, Pal::Ink);
    f.outline(x + 1, y + 1, w - 2, h - 2, Pal::Tan);
    let heart = crate::scene::lean_colour(&plaque.species);
    for (r, row) in [".#.#.", "#####", "#####", ".###.", "..#.."].iter().enumerate() {
        for (c, ch) in row.chars().enumerate() {
            if ch == '#' {
                f.set(x + 3 + c as i32, y + 4 + r as i32, heart);
            }
        }
    }
    f.text(&plaque.lived, x + 10, y + 5, 1, Pal::Cream);
}

const POSTER: Grid = &[
    ".kkkkkkkkkkkkkk.",
    "knnnnnnnnnnnnnnk",
    "knttttttttttttnk",
    "kntsssssssssstnk",
    "kntsscssssssstnk",
    "kntsssssssssstnk",
    "kntdddddyddddtnk",
    "kntdddddyddcdtnk",
    "kntddddycydddtnk",
    "kntddyycccyydtnk",
    "kntdddyyyyyddtnk",
    "kntddddyyydddtnk",
    "kntdddyydyyddtnk",
    "kntdcddddddddtnk",
    "kntdddddddddctnk",
    "kntddddddddddtnk",
    "kntddddddddddtnk",
    "knttttttttttttnk",
    "knnnnnnnnnnnnnnk",
    ".kkkkkkkkkkkkkk.",
];

const FERN: Grid = &[
    "................",
    "........kk......",
    "....kk.kggk.kk..",
    "...kggkkggkkggk.",
    "...kgggklgkgggk.",
    "...kglgklgklggk.",
    "..kkkllklgllgkkk",
    ".kggkglglglggkgg",
    ".kgggklllllgkggg",
    "..kgggglllgggggk",
    "...kggglllggggk.",
    "..kggggglggggggk",
    "..kyyyyyyyyyyggk",
    "..kooooooooookk.",
    "...kuuuuuuuuk...",
    "....kuuuuuuk....",
    "....kuuuuuuk....",
    "....kuuuuuuk....",
    ".....kkkkkk.....",
    "................",
];

const LAMP: Grid = &[
    "....kkkkkk....",
    "...kcccccck...",
    "...kccyyyyk...",
    "...kccyyyyk...",
    "..kyccyyyyyk..",
    "..kyyyyyyyyk..",
    "..kyyyyyyyyk..",
    ".kyyyyyyyyyyk.",
    "..kkkkhakkkk..",
    ".....khak.....",
    ".....khak.....",
    ".....khak.....",
    ".....khak.....",
    ".....khak.....",
    ".....khak.....",
    ".....khak.....",
    "...kkkhakkk...",
    "..khhhhhhhhk..",
    "..kaaaaaaaak..",
    "...kkkkkkkk...",
];

const TEDDY: Grid = &[
    "...kk......kk...",
    "..knnkkkkkknnk..",
    ".knnnnnnnnnnnnk.",
    ".knntnnnnnnntnk.",
    ".knnnnnnnnnnnnk.",
    "..knnnknnnknnk..",
    "..knnnttttnnnk..",
    "...knnttktnnk...",
    "...knnnttnnnk...",
    "..knnnnnnnnnnk..",
    ".knnnnttttnnnnk.",
    ".knnnnttttnnnnk.",
    "..knnnttttnnnk..",
    "...knnttttnnk...",
    "....kknnnnkk....",
    "..kkkkkkkkkkkk..",
    ".kbbppppppppppk.",
    "kbppppppppppppvk",
    "kppppppppppppvvk",
    ".kkkkkkkkkkkkkk.",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The least wall the room shows on each side of the 160-wide stage (the smallest window).
    const WALL: i32 = 24;

    #[test]
    fn every_icon_is_a_clean_grid_that_fits_a_shop_row() {
        for d in DECOR.iter() {
            let w = d.icon[0].chars().count();
            assert!(d.icon.iter().all(|r| r.chars().count() == w), "{}", d.name);
            assert!(d.icon.len() <= 20 && w <= 16, "{} is {w}x{}", d.name, d.icon.len());
            assert!(d.icon.iter().all(|r| r.chars().all(|c| c == '.' || crate::faces::colour(c).is_some())), "{}", d.name);
        }
    }

    #[test]
    fn items_stand_inside_the_standard_room_and_prices_are_20_to_60() {
        for d in DECOR.iter() {
            let (w, h) = (d.icon[0].chars().count() as i32, d.icon.len() as i32);
            let (x, y) = d.spot.at(w, h);
            assert!(x >= -WALL && y >= 0 && x + w <= crate::scene::W + WALL && y + h <= crate::scene::H, "{}", d.name);
            assert!((20..=60).contains(&d.price), "{}", d.name);
        }
        assert!(PLAQUE_PRICE <= 25);
        assert!(PLAQUE.0 >= -WALL && PLAQUE.0 + PLAQUE.2 <= crate::scene::W + WALL);
    }
}
