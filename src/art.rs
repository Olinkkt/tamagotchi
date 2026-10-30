//! The sprite sheets made by tools/spritegen, embedded in the binary and decoded on first use.

use std::sync::OnceLock;

use macroquad::prelude::{Image, ImageFormat};

use crate::cast::{self, Character};
use crate::palette::Pal;

#[derive(Clone, Debug)]
pub struct Sprite {
    pub w: i32,
    pub h: i32,
    px: Vec<Option<Pal>>,
}

impl Sprite {
    fn decode(bytes: &[u8]) -> Sprite {
        let img = Image::from_file_with_format(bytes, Some(ImageFormat::Png)).expect("embedded sprites are valid PNGs");
        let px = img.bytes.chunks(4).map(|c| (c[3] >= 128).then(|| Pal::nearest((c[0], c[1], c[2])))).collect();
        Sprite { w: img.width as i32, h: img.height as i32, px }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Pal> {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return None;
        }
        self.px[(y * self.w + x) as usize]
    }

    pub fn empty(size: i32) -> Sprite {
        Sprite { w: size, h: size, px: vec![None; (size * size) as usize] }
    }

    /// Lay `top` (the same size) over this sprite.
    pub fn over(&mut self, top: &Sprite) {
        for (a, b) in self.px.iter_mut().zip(&top.px) {
            if b.is_some() {
                *a = *b;
            }
        }
    }

    pub fn put(&mut self, x: i32, y: i32, p: Pal) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = Some(p);
        }
    }

    /// Cell `i` of a sheet of square `size` cells laid out in one row.
    fn cell(&self, i: usize, size: i32) -> Sprite {
        let mut px = Vec::with_capacity((size * size) as usize);
        for y in 0..size {
            for x in 0..size {
                px.push(self.get(i as i32 * size + x, y));
            }
        }
        Sprite { w: size, h: size, px }
    }

    /// Turned `deg` degrees clockwise around `(px, py)`, a pixel at a time (nearest, so no blur).
    pub fn tilted(&self, deg: i32, px: f32, py: f32) -> Sprite {
        if deg == 0 {
            return self.clone();
        }
        let (sin, cos) = (deg as f32).to_radians().sin_cos();
        let mut out = Sprite { w: self.w, h: self.h, px: vec![None; self.px.len()] };
        for y in 0..self.h {
            for x in 0..self.w {
                let (rx, ry) = (x as f32 + 0.5 - px, y as f32 + 0.5 - py);
                let (sx, sy) = (rx * cos + ry * sin + px, -rx * sin + ry * cos + py);
                out.px[(y * self.w + x) as usize] = self.get(sx.floor() as i32, sy.floor() as i32);
            }
        }
        out
    }

    /// Cut along a ragged line into the pixels above it and the rest, both the size of the whole.
    /// `cut(x)` is the first row of the lower part in column `x`.
    pub fn halves(&self, cut: impl Fn(i32) -> i32) -> (Sprite, Sprite) {
        let mut top = Sprite { w: self.w, h: self.h, px: vec![None; self.px.len()] };
        let mut bottom = top.clone();
        for y in 0..self.h {
            for x in 0..self.w {
                let i = (y * self.w + x) as usize;
                if y < cut(x) { top.px[i] = self.px[i] } else { bottom.px[i] = self.px[i] }
            }
        }
        (top, bottom)
    }

    /// Moved up `by` pixels (down if negative), keeping the size.
    pub fn raised(&self, by: i32) -> Sprite {
        let mut out = Sprite { w: self.w, h: self.h, px: vec![None; self.px.len()] };
        for y in 0..self.h {
            for x in 0..self.w {
                out.px[(y * self.w + x) as usize] = self.get(x, y + by);
            }
        }
        out
    }

    /// The highest row with anything in it.
    pub fn top(&self) -> i32 {
        (0..self.h).find(|&y| (0..self.w).any(|x| self.get(x, y).is_some())).unwrap_or(0)
    }

    /// One past the lowest row with anything in it, so sprites can stand on the floor.
    pub fn bottom(&self) -> i32 {
        (0..self.h).rev().find(|&y| (0..self.w).any(|x| self.get(x, y).is_some())).map_or(0, |y| y + 1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Light {
    Day,
    Lamp,
    Dark,
}

struct Art {
    sheets: Vec<(&'static str, Vec<Sprite>)>,
    rooms: [Sprite; 3],
}

fn sheet_bytes(name: &str) -> &'static [u8] {
    match name {
        "egg" => include_bytes!("../assets/sprites/egg.png"),
        "baby" => include_bytes!("../assets/sprites/baby.png"),
        "marutchi" => include_bytes!("../assets/sprites/marutchi.png"),
        "tonmarutchi" => include_bytes!("../assets/sprites/tonmarutchi.png"),
        "hashitamatchi" => include_bytes!("../assets/sprites/hashitamatchi.png"),
        "mametchi" => include_bytes!("../assets/sprites/mametchi.png"),
        "kuchipatchi" => include_bytes!("../assets/sprites/kuchipatchi.png"),
        "tarakotchi" => include_bytes!("../assets/sprites/tarakotchi.png"),
        "ichigotchi" => include_bytes!("../assets/sprites/ichigotchi.png"),
        "nyorotchi" => include_bytes!("../assets/sprites/nyorotchi.png"),
        "poop" => include_bytes!("../assets/sprites/poop.png"),
        "grave" => include_bytes!("../assets/sprites/grave.png"),
        "ghost" => include_bytes!("../assets/sprites/ghost.png"),
        "shell" => include_bytes!("../assets/sprites/shell.png"),
        other => panic!("no sprite sheet for {other}"),
    }
}

fn art() -> &'static Art {
    static ART: OnceLock<Art> = OnceLock::new();
    ART.get_or_init(|| Art {
        sheets: cast::ALL
            .iter()
            .map(|c| {
                let sheet = Sprite::decode(sheet_bytes(c.name));
                (c.name, (0..c.cells()).map(|i| sheet.cell(i, c.size as i32)).collect())
            })
            .collect(),
        rooms: [
            Sprite::decode(include_bytes!("../assets/sprites/room_day.png")),
            Sprite::decode(include_bytes!("../assets/sprites/room_lamp.png")),
            Sprite::decode(include_bytes!("../assets/sprites/room_dark.png")),
        ],
    })
}

/// Every cell of a character's sheet: the body first, then its part frames.
pub fn cells(c: &Character) -> &'static [Sprite] {
    art().sheets.iter().find(|(n, _)| *n == c.name).map(|(_, s)| s.as_slice()).expect("every cast member has a sheet")
}

pub fn body(c: &Character) -> &'static Sprite {
    &cells(c)[0]
}

pub fn room(light: Light) -> &'static Sprite {
    &art().rooms[light as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sheet_decodes_to_palette_colours_at_the_right_size() {
        for c in cast::ALL {
            let cells = cells(&c);
            assert_eq!(cells.len(), c.cells(), "{}", c.name);
            assert!(cells.iter().all(|s| s.w == c.size as i32 && s.h == c.size as i32));
            assert!(cells[0].bottom() > c.size as i32 / 2, "{} is mostly empty", c.name);
        }
        for light in [Light::Day, Light::Lamp, Light::Dark] {
            let r = room(light);
            assert_eq!((r.w, r.h), (cast::ROOM_MASTER.0 as i32, cast::ROOM_MASTER.1 as i32));
            assert!((0..r.h).all(|y| (0..r.w).all(|x| r.get(x, y).is_some())), "the room has no holes");
        }
    }

    #[test]
    fn every_egg_crack_pixel_lies_on_the_shell() {
        let egg = body(&cast::EGG);
        let (ox, oy) = cast::EGG_CRACK_AT;
        for (stage, grid) in crate::faces::EGG_CRACKS.iter().enumerate() {
            for (x, y, _) in crate::faces::pixels(grid) {
                assert!(egg.get(ox + x, oy + y).is_some(), "stage {stage} has a crack pixel off the egg at {x},{y}");
            }
        }
    }

    #[test]
    fn sheets_use_only_exact_palette_colours() {
        let img = Image::from_file_with_format(sheet_bytes("baby"), Some(ImageFormat::Png)).unwrap();
        for c in img.bytes.chunks(4).filter(|c| c[3] >= 128) {
            assert!(Pal::ALL.iter().any(|p| p.rgb() == (c[0], c[1], c[2])), "{c:?} is not a palette colour");
        }
    }
}
