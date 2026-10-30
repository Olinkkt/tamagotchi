//! The pet's room, drawn in room pixels on a 4x canvas and shrunk like a sprite.
//! Two versions: daytime, and asleep in the dark.
//!
//! Everything is authored in "stage" coordinates, the standard 160x116 room. The image made is
//! the larger master of `cast::ROOM_MASTER`: the stage sits at its bottom centre, the wall and
//! floor patterns carry on around it, and the lamp cord reaches up to the top edge.

use crate::img::Img;
use crate::palette::Pal::{self, *};
use crate::process::{layer, Finish};
use crate::sketch::Sketch;

use crate::cast::{ROOM_MASTER, ROOM_STAGE};

pub const W: usize = ROOM_STAGE.0;
pub const H: usize = ROOM_STAGE.1;
pub(crate) const MW: usize = ROOM_MASTER.0;
pub(crate) const MH: usize = ROOM_MASTER.1;
/// Where the stage's top-left corner is in the master.
pub(crate) const SX: i32 = ((MW - W) / 2) as i32;
pub(crate) const SY: i32 = (MH - H) as i32;
const FLOOR: f32 = 84.0;
pub(crate) const K: f32 = 4.0;
/// Stage coordinates covered by the master.
const XS: std::ops::Range<i32> = -SX..(W as i32 + SX);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Light {
    Day,
    Dark,
}

fn canvas() -> Sketch {
    Sketch::new((MW as f32 * K) as u32, (MH as f32 * K) as u32).scaled(K).shifted(SX as f32, SY as f32)
}

/// Set a pixel at a stage position.
fn put(img: &mut Img, x: i32, y: i32, p: Pal) {
    img.set(x + SX, y + SY, Some(p));
}

fn flat(s: &Sketch) -> Img {
    layer(&s.to_img(), MW, MH, &Finish { shade: false, outline: None })
}

/// Furniture gets a soft dark outline rather than the characters' ink, so the pet stands out.
pub(crate) fn lined(s: &Sketch) -> Img {
    layer(&s.to_img(), MW, MH, &Finish { shade: false, outline: Some(Night) })
}

fn wall() -> Img {
    let mut s = canvas();
    let (x0, w, top) = (-SX as f32, MW as f32, -SY as f32);
    s.rect(x0, top, w, FLOOR - top, Dusk);
    // Lower panelling below a wooden rail.
    s.rect(x0, 60.0, w, FLOOR - 60.0, Night);
    for x in XS.filter(|x| (x - 8).rem_euclid(16) == 0) {
        s.rect(x as f32, 64.0, 1.0, 14.0, Dusk);
    }
    s.rect(x0, 58.0, w, 2.0, Tan);
    s.rect(x0, 60.0, w, 1.0, Brown);
    s.rect(x0, 79.0, w, 5.0, Brown);
    s.rect(x0, 79.0, w, 1.0, Tan);
    let mut img = flat(&s);
    // Wallpaper: a lattice of small darker diamonds, on rows 8 apart and columns 12 apart.
    for y in (-SY..56).filter(|y| (y - 7).rem_euclid(8) == 0) {
        let shift = (y - 7).div_euclid(8).rem_euclid(2) * 6;
        for x in XS.filter(|x| (x - 4 - shift).rem_euclid(12) == 0) {
            for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
                put(&mut img, x + dx, y + dy, Night);
            }
        }
    }
    img
}

/// What is seen through the glass. Kept separate so it stays bright when the room goes dark.
fn view(light: Light) -> Img {
    let mut s = canvas();
    let day = light == Light::Day;
    s.rect(17.0, 15.0, 37.0, 33.0, if day { Sky } else { Night });
    if day {
        s.ellipse(46.0, 21.0, 3.5, 3.5, Yellow);
        for (x, y, rx) in [(26.0, 25.0, 6.0), (31.0, 23.0, 4.5), (44.0, 38.0, 7.0), (39.0, 39.0, 4.0)] {
            s.ellipse(x, y, rx, rx * 0.55, White);
        }
    } else {
        s.ellipse(44.0, 23.0, 5.0, 5.0, Cream);
        s.ellipse(46.5, 21.5, 4.5, 4.5, Night);
    }
    let mut img = flat(&s);
    if day {
        // A dithered haze towards the horizon.
        for y in 43..48 {
            for x in 17..54 {
                if (x + y) % 2 == 0 || y >= 46 {
                    put(&mut img, x, y, Ice);
                }
            }
        }
    } else {
        for (x, y, p) in [(21, 19, White), (29, 27, Ice), (24, 37, White), (36, 18, Ice), (50, 41, White), (33, 43, Ice), (39, 31, White)] {
            put(&mut img, x, y, p);
        }
    }
    img
}

fn window() -> Img {
    let mut s = canvas();
    // Frame with a cross of glazing bars, and the sill below.
    s.rect(14.0, 12.0, 43.0, 3.0, Tan);
    s.rect(14.0, 48.0, 43.0, 2.0, Tan);
    s.rect(14.0, 12.0, 3.0, 38.0, Tan);
    s.rect(54.0, 12.0, 3.0, 38.0, Tan);
    s.rect(34.5, 15.0, 2.0, 33.0, Tan);
    s.rect(17.0, 30.0, 37.0, 2.0, Tan);
    s.rect(11.0, 50.0, 49.0, 3.0, Tan);
    s.rect(11.0, 52.0, 49.0, 1.0, Brown);
    lined(&s)
}

fn curtains() -> Img {
    let mut s = canvas();
    s.rect(6.0, 9.0, 59.0, 2.0, Brown);
    s.ellipse(6.0, 10.0, 2.0, 2.0, Tan);
    s.ellipse(65.0, 10.0, 2.0, 2.0, Tan);
    for flip in [false, true] {
        let x = |v: f32| if flip { 71.0 - v } else { v };
        s.poly(&[(x(8.0), 11.0), (x(20.0), 11.0), (x(20.0), 33.0), (x(18.5), 37.0), (x(20.0), 41.0), (x(20.0), 57.0), (x(8.0), 57.0)], Pink);
        s.rect(x(12.0).min(x(13.0)), 13.0, 1.0, 42.0, Violet);
        s.rect(x(16.0).min(x(17.0)), 13.0, 1.0, 22.0, Violet);
        s.rect(x(16.0).min(x(17.0)), 40.0, 1.0, 15.0, Violet);
        s.rect(x(8.0).min(x(18.0)), 36.0, 10.0, 2.0, Yellow);
    }
    lined(&s)
}

fn shelf() -> Img {
    let mut s = canvas();
    for (x, y, w, p) in [(107.0, 24.0, 4.0, Red), (111.0, 26.0, 4.0, Sky), (115.0, 23.0, 3.0, Yellow), (118.0, 27.0, 4.0, Green)] {
        s.rect(x, y, w, 36.0 - y, p);
        s.rect(x, y + 2.0, w, 1.0, p.highlight());
    }
    s.poly(&[(122.0, 36.0), (125.0, 36.0), (129.0, 27.0), (126.0, 26.0)], Violet);
    // A potted plant.
    s.poly(&[(132.0, 30.0), (143.0, 30.0), (141.0, 36.0), (134.0, 36.0)], Rust);
    s.rect(131.0, 29.0, 13.0, 2.0, Orange);
    s.ellipse(134.0, 24.0, 4.0, 2.0, Green);
    s.ellipse(141.0, 23.0, 4.0, 2.0, Green);
    s.ellipse(137.5, 20.0, 2.0, 5.0, Lime);
    s.ellipse(137.5, 26.0, 3.0, 3.0, Forest);
    // The plank and its brackets.
    s.rect(103.0, 36.0, 47.0, 3.0, Tan);
    s.rect(103.0, 38.0, 47.0, 1.0, Brown);
    s.poly(&[(107.0, 39.0), (111.0, 39.0), (107.0, 44.0)], Brown);
    s.poly(&[(146.0, 39.0), (142.0, 39.0), (146.0, 44.0)], Brown);
    lined(&s)
}

fn clock() -> Img {
    let mut s = canvas();
    s.ellipse(127.0, 13.0, 7.0, 7.0, Tan);
    s.ellipse(127.0, 13.0, 5.0, 5.0, White);
    s.line(&[(127.0, 13.0), (127.0, 9.5)], 1.0, Ink);
    s.line(&[(127.0, 13.0), (129.5, 13.0)], 1.0, Ink);
    s.rect(126.5, 12.5, 1.0, 1.0, Red);
    lined(&s)
}

fn floor() -> Img {
    let mut s = canvas();
    s.rect(-SX as f32, FLOOR, MW as f32, H as f32 - FLOOR, Brown);
    let mut img = flat(&s);
    // Boards get taller towards the viewer; joints are staggered from board to board.
    let seams = [84, 88, 93, 99, 106, 114];
    for (i, w) in seams.windows(2).enumerate() {
        let seam = if w[0] == seams[0] { Night } else { Maroon };
        for x in XS {
            put(&mut img, x, w[0], seam);
        }
        let (i, step) = (i as i32, 36 + i as i32 * 6);
        let first = (i * 23) % step;
        for x in XS.filter(|x| (x - first).rem_euclid(step) == 0) {
            for y in w[0] + 1..w[1] {
                put(&mut img, x, y, Maroon);
            }
            // A streak of grain just after each joint.
            let gy = (w[0] + w[1]) / 2;
            for gx in x + 4..x + 4 + 5 + i {
                put(&mut img, gx, gy, Tan);
            }
        }
    }
    img
}

fn rug() -> Img {
    let mut s = canvas();
    s.ellipse(80.0, 101.0, 50.0, 11.0, Violet);
    s.ellipse(80.0, 101.0, 46.5, 9.2, Pink);
    s.ellipse(80.0, 101.0, 43.5, 7.2, Violet);
    s.ellipse(80.0, 101.0, 26.0, 4.2, Pink);
    s.ellipse(80.0, 101.0, 23.0, 2.6, Violet);
    lined(&s)
}

fn lamp() -> Img {
    let mut s = canvas();
    // The cord reaches up to the top of the master, however tall the room is cropped.
    s.rect(79.5, -SY as f32, 1.0, 8.0 + SY as f32, Ink);
    s.poly(&[(75.0, 8.0), (85.0, 8.0), (90.0, 16.0), (70.0, 16.0)], Yellow);
    s.poly(&[(72.5, 12.0), (87.5, 12.0), (90.0, 16.0), (70.0, 16.0)], Orange);
    s.ellipse(80.0, 17.5, 2.5, 2.0, Mist);
    lined(&s)
}

fn inside(pts: &[(f32, f32)], x: f32, y: f32) -> bool {
    let mut odd = false;
    let mut j = pts.len() - 1;
    for i in 0..pts.len() {
        let ((xi, yi), (xj, yj)) = (pts[i], pts[j]);
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            odd = !odd;
        }
        j = i;
    }
    odd
}

/// A colour where light falls on it. Seams and outlines warm up a step instead of turning red.
fn lit(p: Pal) -> Pal {
    match p {
        Maroon => Brown,
        Night => Dusk,
        Ink => Night,
        other => other.highlight(),
    }
}

/// Recolour every pixel where `hit` (given stage coordinates) is true.
fn light_patch(img: &mut Img, hit: impl Fn(f32, f32) -> bool, f: impl Fn(Pal) -> Pal) {
    for y in 0..MH as i32 {
        for x in 0..MW as i32 {
            if hit((x - SX) as f32 + 0.5, (y - SY) as f32 + 0.5) {
                if let Some(p) = img.get(x, y) {
                    img.set(x, y, Some(f(p)));
                }
            }
        }
    }
}

/// Where light from the window lands on the floor.
const WINDOW_PATCH: [(f32, f32); 4] = [(20.0, 85.0), (52.0, 85.0), (66.0, 98.0), (30.0, 98.0)];

pub fn room(light: Light) -> Img {
    let view = view(light);
    let front = [window(), curtains(), shelf(), clock(), floor(), rug(), lamp()];
    let mut img = wall();
    img.blit(&view, 0, 0);
    for part in &front {
        img.blit(part, 0, 0);
    }
    match light {
        Light::Day => light_patch(&mut img, |x, y| inside(&WINDOW_PATCH, x, y), lit),
        Light::Dark => {
            let moonlit: Vec<bool> =
                (0..MW * MH).map(|i| inside(&WINDOW_PATCH, (i % MW) as f32 - SX as f32 + 0.5, (i / MW) as f32 - SY as f32 + 0.5)).collect();
            for (p, moon) in img.px.iter_mut().zip(moonlit) {
                *p = p.map(|c| if moon { lit(c.dark()) } else { c.dark() });
            }
            // The night sky through the glass keeps its own light.
            for y in 0..MH as i32 {
                for x in 0..MW as i32 {
                    if view.get(x, y).is_some() && front.iter().all(|l| l.get(x, y).is_none()) {
                        img.set(x, y, view.get(x, y));
                    }
                }
            }
        }
    }
    img
}
