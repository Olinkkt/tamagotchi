//! Turns a big flat drawing into a finished sprite layer: shade it, shrink it with
//! nearest-neighbour, tidy stray pixels, and put a 1px outline around it.

use crate::img::Img;
use crate::palette::Pal;

/// Light comes from the top-left, a little more from above, and towards the viewer.
const LIGHT: (f32, f32, f32) = (-0.5, -0.85, 1.0);
/// Below this much light a pixel is in shadow; above `LIT` it is highlighted.
const SHADE: f32 = 0.5;
const LIT: f32 = 0.95;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tone {
    Shadow,
    Base,
    Light,
}

/// How lit each filled pixel is, treating the silhouette as a rounded cushion: flat in the
/// middle, curving away near the edges. Different colours inside one silhouette share the
/// same curve, so a belly patch shades as part of the body.
pub fn tones(img: &Img) -> Vec<Tone> {
    let (w, h) = (img.w as i32, img.h as i32);
    let filled = |x: i32, y: i32| img.get(x, y).is_some();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for y in 0..h {
        for x in 0..w {
            if filled(x, y) {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
        }
    }
    let r = (((x1 - x0).min(y1 - y0)) as f32 * 0.22).clamp(4.0, 40.0);
    let ri = r.ceil() as i32;
    // Distance from every filled pixel to the nearest empty one, up to `r`.
    let mut dist = vec![0.0f32; img.w * img.h];
    for y in 0..h {
        for x in 0..w {
            if !filled(x, y) {
                continue;
            }
            let mut best = r;
            for dy in -ri..=ri {
                for dx in -ri..=ri {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d < best && !filled(x + dx, y + dy) {
                        best = d;
                    }
                }
            }
            dist[(y * w + x) as usize] = best;
        }
    }
    // A quarter-circle profile: steep at the rim, level once `r` in from it.
    let height = |x: i32, y: i32| -> f32 {
        if x < 0 || y < 0 || x >= w || y >= h {
            return 0.0;
        }
        let u = 1.0 - dist[(y * w + x) as usize] / r;
        r * (1.0 - u * u).max(0.0).sqrt()
    };
    let l = {
        let n = (LIGHT.0 * LIGHT.0 + LIGHT.1 * LIGHT.1 + LIGHT.2 * LIGHT.2).sqrt();
        (LIGHT.0 / n, LIGHT.1 / n, LIGHT.2 / n)
    };
    let mut out = vec![Tone::Base; img.w * img.h];
    for y in 0..h {
        for x in 0..w {
            if !filled(x, y) {
                continue;
            }
            let gx = (height(x + 1, y) - height(x - 1, y)) / 2.0;
            let gy = (height(x, y + 1) - height(x, y - 1)) / 2.0;
            let n = (gx * gx + gy * gy + 1.0).sqrt();
            let lit = (-gx * l.0 - gy * l.1 + l.2) / n;
            out[(y * w + x) as usize] = if lit < SHADE {
                Tone::Shadow
            } else if lit > LIT {
                Tone::Light
            } else {
                Tone::Base
            };
        }
    }
    out
}

/// Shrink to `w` x `h` by taking the source pixel under each target pixel's centre.
fn downsample<T: Copy>(src: &[T], sw: usize, sh: usize, w: usize, h: usize) -> Vec<T> {
    let mut out = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let sx = ((x as f32 + 0.5) * sw as f32 / w as f32) as usize;
            let sy = ((y as f32 + 0.5) * sh as f32 / h as f32) as usize;
            out.push(src[sy.min(sh - 1) * sw + sx.min(sw - 1)]);
        }
    }
    out
}

/// A lone shadow or highlight pixel reads as dirt at this size, so it goes back to base.
fn despeckle(img: &Img, tone: &mut [Tone]) {
    let (w, h) = (img.w as i32, img.h as i32);
    let before = tone.to_vec();
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if before[i] == Tone::Base {
                continue;
            }
            let alike = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                nx >= 0 && ny >= 0 && nx < w && ny < h && before[(ny * w + nx) as usize] == before[i] && img.get(nx, ny) == img.get(x, y)
            });
            if !alike {
                tone[i] = Tone::Base;
            }
        }
    }
}

/// Any see-through pixel touching a filled one (sideways or up/down) becomes `colour`.
pub fn outline(img: &mut Img, colour: Pal) {
    let edge: Vec<(i32, i32)> = (0..img.h as i32)
        .flat_map(|y| (0..img.w as i32).map(move |x| (x, y)))
        .filter(|&(x, y)| img.get(x, y).is_none() && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| img.get(x + dx, y + dy).is_some()))
        .collect();
    for (x, y) in edge {
        img.set(x, y, Some(colour));
    }
}

pub struct Finish {
    pub shade: bool,
    pub outline: Option<Pal>,
}

pub const BODY: Finish = Finish { shade: true, outline: Some(Pal::Ink) };

/// The whole pipeline for one layer: `src` (already palette-snapped) becomes `w` x `h`.
pub fn layer(src: &Img, w: usize, h: usize, finish: &Finish) -> Img {
    let tone = if finish.shade { tones(src) } else { vec![Tone::Base; src.w * src.h] };
    let mut img = Img { w, h, px: downsample(&src.px, src.w, src.h, w, h) };
    let mut tone = downsample(&tone, src.w, src.h, w, h);
    despeckle(&img, &mut tone);
    for (p, t) in img.px.iter_mut().zip(tone) {
        *p = p.map(|c| match t {
            Tone::Shadow => c.shadow(),
            Tone::Light => c.highlight(),
            Tone::Base => c,
        });
    }
    if let Some(c) = finish.outline {
        outline(&mut img, c);
    }
    img
}

/// Layers stacked bottom to top.
pub fn stack(layers: &[Img]) -> Img {
    let mut out = Img::new(layers[0].w, layers[0].h);
    for l in layers {
        out.blit(l, 0, 0);
    }
    out
}
