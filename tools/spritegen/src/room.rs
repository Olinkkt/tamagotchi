//! The pet's room, 160x116, drawn in room pixels on a 4x canvas and shrunk like a sprite.
//! Three versions: daytime, asleep with the lamp on, and asleep in the dark.

use crate::img::Img;
use crate::palette::Pal::{self, *};
use crate::process::{layer, Finish};
use crate::sketch::Sketch;

pub const W: usize = 160;
pub const H: usize = 116;
const FLOOR: f32 = 84.0;
const K: f32 = 4.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Light {
    Day,
    Lamp,
    Dark,
}

fn canvas() -> Sketch {
    Sketch::new((W as f32 * K) as u32, (H as f32 * K) as u32).scaled(K)
}

fn flat(s: &Sketch) -> Img {
    layer(&s.to_img(), W, H, &Finish { shade: false, outline: None })
}

/// Furniture gets a soft dark outline rather than the characters' ink, so the pet stands out.
fn lined(s: &Sketch) -> Img {
    layer(&s.to_img(), W, H, &Finish { shade: false, outline: Some(Night) })
}

fn wall() -> Img {
    let mut s = canvas();
    s.rect(0.0, 0.0, W as f32, FLOOR, Dusk);
    s.rect(0.0, 0.0, W as f32, 3.0, Night);
    // Lower panelling below a wooden rail.
    s.rect(0.0, 60.0, W as f32, FLOOR - 60.0, Night);
    for x in (8..W).step_by(16) {
        s.rect(x as f32, 64.0, 1.0, 14.0, Dusk);
    }
    s.rect(0.0, 58.0, W as f32, 2.0, Tan);
    s.rect(0.0, 60.0, W as f32, 1.0, Brown);
    s.rect(0.0, 79.0, W as f32, 5.0, Brown);
    s.rect(0.0, 79.0, W as f32, 1.0, Tan);
    let mut img = flat(&s);
    // Wallpaper: a lattice of small darker diamonds.
    for (row, y) in (7..56).step_by(8).enumerate() {
        for x in (4 + (row % 2) * 6..W).step_by(12) {
            for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
                img.set(x as i32 + dx, y as i32 + dy, Some(Night));
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
                    img.set(x, y, Some(Ice));
                }
            }
        }
    } else {
        for (x, y, p) in [(21, 19, White), (29, 27, Ice), (24, 37, White), (36, 18, Ice), (50, 41, White), (33, 43, Ice), (39, 31, White)] {
            img.set(x, y, Some(p));
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
    s.rect(0.0, FLOOR, W as f32, H as f32 - FLOOR, Brown);
    let mut img = flat(&s);
    // Boards get taller towards the viewer; joints are staggered from board to board.
    let seams = [84, 88, 93, 99, 106, 114];
    for (i, w) in seams.windows(2).enumerate() {
        let seam = if w[0] == seams[0] { Night } else { Maroon };
        for x in 0..W as i32 {
            img.set(x, w[0], Some(seam));
        }
        let step = 36 + i as i32 * 6;
        for x in ((i as i32 * 23) % step..W as i32).step_by(step as usize) {
            for y in w[0] + 1..w[1] {
                img.set(x, y, Some(Maroon));
            }
            // A streak of grain just after each joint.
            let gy = (w[0] + w[1]) / 2;
            for gx in x + 4..x + 4 + 5 + i as i32 {
                img.set(gx, gy, Some(Tan));
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

fn lamp(on: bool) -> Img {
    let mut s = canvas();
    s.rect(79.5, 0.0, 1.0, 8.0, Ink);
    s.poly(&[(75.0, 8.0), (85.0, 8.0), (90.0, 16.0), (70.0, 16.0)], Yellow);
    s.poly(&[(72.5, 12.0), (87.5, 12.0), (90.0, 16.0), (70.0, 16.0)], Orange);
    s.ellipse(80.0, 17.5, 2.5, 2.0, if on { Cream } else { Mist });
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

/// Recolour every pixel inside `area` (or inside the ellipse) with `f`.
fn light_patch(img: &mut Img, hit: impl Fn(f32, f32) -> bool, f: impl Fn(Pal) -> Pal) {
    for y in 0..H as i32 {
        for x in 0..W as i32 {
            if hit(x as f32 + 0.5, y as f32 + 0.5) {
                if let Some(p) = img.get(x, y) {
                    img.set(x, y, Some(f(p)));
                }
            }
        }
    }
}

/// Where light from the window lands on the floor.
const WINDOW_PATCH: [(f32, f32); 4] = [(20.0, 85.0), (52.0, 85.0), (66.0, 98.0), (30.0, 98.0)];

fn in_lamp_pool(x: f32, y: f32) -> bool {
    let (dx, dy) = ((x - 80.0) / 28.0, (y - 101.0) / 7.0);
    dx * dx + dy * dy <= 1.0
}

fn near_bulb(x: f32, y: f32) -> bool {
    let (dx, dy) = (x - 80.0, y - 18.5);
    (dx * dx + dy * dy).sqrt() <= 4.0 && y > 16.0
}

pub fn room(light: Light) -> Img {
    let view = view(light);
    let front = [window(), curtains(), shelf(), clock(), floor(), rug(), lamp(light == Light::Lamp)];
    let mut img = wall();
    img.blit(&view, 0, 0);
    for part in &front {
        img.blit(part, 0, 0);
    }
    match light {
        Light::Day => light_patch(&mut img, |x, y| inside(&WINDOW_PATCH, x, y), lit),
        Light::Lamp => {
            light_patch(&mut img, in_lamp_pool, lit);
            light_patch(&mut img, near_bulb, |p| if p == Cream { p } else { lit(p) });
        }
        Light::Dark => {
            let moonlit: Vec<bool> = (0..W * H).map(|i| inside(&WINDOW_PATCH, (i % W) as f32 + 0.5, (i / W) as f32 + 0.5)).collect();
            for (p, moon) in img.px.iter_mut().zip(moonlit) {
                *p = p.map(|c| if moon { lit(c.dark()) } else { c.dark() });
            }
            // The night sky through the glass keeps its own light.
            for y in 0..H as i32 {
                for x in 0..W as i32 {
                    if view.get(x, y).is_some() && front.iter().all(|l| l.get(x, y).is_none()) {
                        img.set(x, y, view.get(x, y));
                    }
                }
            }
        }
    }
    img
}
