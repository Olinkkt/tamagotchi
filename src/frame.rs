//! A canvas of palette pixels. The whole scene is drawn into one of these, then turned into
//! RGBA once per frame. Nothing here needs a window, so it is easy to test.

use crate::art::Sprite;
use crate::faces;
use crate::font;
use crate::palette::Pal;

#[derive(Clone, PartialEq, Debug)]
pub struct Frame {
    pub w: i32,
    pub h: i32,
    px: Vec<Pal>,
}

impl Frame {
    pub fn new(w: i32, h: i32, fill: Pal) -> Frame {
        Frame { w, h, px: vec![fill; (w * h) as usize] }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Pal> {
        (x >= 0 && y >= 0 && x < self.w && y < self.h).then(|| self.px[(y * self.w + x) as usize])
    }

    pub fn set(&mut self, x: i32, y: i32, p: Pal) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = p;
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, p: Pal) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, p);
            }
        }
    }

    /// A rectangle with a 1px border.
    pub fn boxed(&mut self, x: i32, y: i32, w: i32, h: i32, fill: Pal, border: Pal) {
        self.rect(x, y, w, h, border);
        self.rect(x + 1, y + 1, w - 2, h - 2, fill);
    }

    pub fn sprite(&mut self, s: &Sprite, x: i32, y: i32) {
        self.sprite_with(s, x, y, |p| p);
    }

    /// Copy the (`sx`, `sy`, `w`, `h`) rectangle of another frame to (`x`, `y`).
    pub fn copy_rect(&mut self, src: &Frame, (sx, sy, w, h): (i32, i32, i32, i32), x: i32, y: i32) {
        for dy in 0..h {
            for dx in 0..w {
                self.set(x + dx, y + dy, src.px[((sy + dy) * src.w + sx + dx) as usize]);
            }
        }
    }

    /// Draw the `w` x `h` part of `s` that starts at (`sx`, `sy`) with its corner at (`x`, `y`).
    pub fn region(&mut self, s: &Sprite, (sx, sy, w, h): (i32, i32, i32, i32), x: i32, y: i32) {
        for dy in 0..h {
            for dx in 0..w {
                if let Some(p) = s.get(sx + dx, sy + dy) {
                    self.set(x + dx, y + dy, p);
                }
            }
        }
    }

    /// Draw a sprite with every colour passed through `f` (for flashes and dark rooms).
    pub fn sprite_with(&mut self, s: &Sprite, x: i32, y: i32, f: impl Fn(Pal) -> Pal) {
        for sy in 0..s.h {
            for sx in 0..s.w {
                if let Some(p) = s.get(sx, sy) {
                    self.set(x + sx, y + sy, f(p));
                }
            }
        }
    }

    /// Draw a pixel grid (see `faces::colour`) with its top-left corner at (`x`, `y`).
    pub fn grid(&mut self, rows: &[&str], x: i32, y: i32) {
        for (dx, dy, p) in faces::pixels(rows) {
            self.set(x + dx, y + dy, p);
        }
    }

    /// A grid with every pixel blown up to a `k` x `k` block and passed through `f`.
    pub fn grid_scaled(&mut self, rows: &[&str], x: i32, y: i32, k: i32, f: impl Fn(Pal) -> Pal) {
        for (dx, dy, p) in faces::pixels(rows) {
            self.rect(x + dx * k, y + dy * k, k, k, f(p));
        }
    }

    /// Text in the pixel font as part of the picture (for scores), `k` scene pixels per font pixel.
    pub fn text(&mut self, text: &str, x: i32, y: i32, k: i32, p: Pal) {
        for (dx, dy) in font::pixels(text) {
            self.rect(x + dx * k, y + dy * k, k, k, p);
        }
    }

    pub fn frame(&mut self, src: &Frame, x: i32, y: i32) {
        for sy in 0..src.h {
            for sx in 0..src.w {
                self.set(x + sx, y + sy, src.px[(sy * src.w + sx) as usize]);
            }
        }
    }

    /// Recolour every pixel in a rectangle.
    pub fn tint(&mut self, x: i32, y: i32, w: i32, h: i32, f: impl Fn(Pal) -> Pal) {
        for yy in y.max(0)..(y + h).min(self.h) {
            for xx in x.max(0)..(x + w).min(self.w) {
                let i = (yy * self.w + xx) as usize;
                self.px[i] = f(self.px[i]);
            }
        }
    }

    pub fn rgba(&self) -> Vec<u8> {
        self.px.iter().flat_map(|p| {
            let (r, g, b) = p.rgb();
            [r, g, b, 255]
        }).collect()
    }
}
