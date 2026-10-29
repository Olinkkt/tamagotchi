//! A small indexed image: every pixel is a palette entry or transparent.

use std::path::Path;

use crate::palette::Pal;

#[derive(Clone)]
pub struct Img {
    pub w: usize,
    pub h: usize,
    pub px: Vec<Option<Pal>>,
}

impl Img {
    pub fn new(w: usize, h: usize) -> Img {
        Img { w, h, px: vec![None; w * h] }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Pal> {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return None;
        }
        self.px[y as usize * self.w + x as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, p: Option<Pal>) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.px[y as usize * self.w + x as usize] = p;
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, p: Pal) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, Some(p));
            }
        }
    }

    /// Copy `src` on top at (`dx`, `dy`); transparent pixels leave what is below.
    pub fn blit(&mut self, src: &Img, dx: i32, dy: i32) {
        for y in 0..src.h as i32 {
            for x in 0..src.w as i32 {
                if let Some(p) = src.get(x, y) {
                    self.set(x + dx, y + dy, Some(p));
                }
            }
        }
    }

    /// Every pixel scaled up `k` times, on a two-tone checker so transparency is visible.
    pub fn preview(&self, k: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.w * self.h * k * k * 4);
        for y in 0..self.h * k {
            for x in 0..self.w * k {
                let c = match self.px[(y / k) * self.w + x / k] {
                    Some(p) => p.rgb(),
                    None if ((x / k) + (y / k)) % 2 == 0 => (0x3a, 0x3a, 0x44),
                    None => (0x30, 0x30, 0x38),
                };
                out.extend_from_slice(&[c.0, c.1, c.2, 255]);
            }
        }
        out
    }

    pub fn rgba(&self) -> Vec<u8> {
        self.px
            .iter()
            .flat_map(|p| match p {
                Some(p) => {
                    let (r, g, b) = p.rgb();
                    [r, g, b, 255]
                }
                None => [0, 0, 0, 0],
            })
            .collect()
    }
}

pub fn write_png(path: &Path, w: usize, h: usize, rgba: &[u8]) {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).expect("create output folder");
    }
    let file = std::fs::File::create(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().and_then(|mut wr| wr.write_image_data(rgba)).expect("write png");
}

pub fn save(img: &Img, path: &Path) {
    write_png(path, img.w, img.h, &img.rgba());
}

pub fn save_preview(img: &Img, path: &Path, k: usize) {
    write_png(path, img.w * k, img.h * k, &img.preview(k));
}
