//! The characters, drawn in shapes on a 128x128 canvas. Each function returns the sheet cells
//! in the order `cast.rs` describes: the body, then every part frame.

use crate::cast::Character;
use crate::img::Img;
use crate::palette::Pal::{self, *};
use crate::process::{layer, stack, BODY};
use crate::sketch::Sketch;

const SRC: u32 = 128;

fn finish(c: &Character, s: &Sketch) -> Img {
    layer(&s.to_img(), c.size, c.size, &BODY)
}

pub fn egg(c: &Character) -> Vec<Img> {
    let mut s = Sketch::new(SRC, SRC);
    // Narrow at the top, widest a little below the middle.
    s.shape(
        |p| {
            p.move_to(64.0, 18.0);
            p.cubic_to(86.0, 18.0, 100.0, 50.0, 100.0, 76.0);
            p.cubic_to(100.0, 100.0, 86.0, 116.0, 64.0, 116.0);
            p.cubic_to(42.0, 116.0, 28.0, 100.0, 28.0, 76.0);
            p.cubic_to(28.0, 50.0, 42.0, 18.0, 64.0, 18.0);
        },
        White,
    );
    for (x, y, r) in [(66.0, 38.0, 7.0), (46.0, 90.0, 8.0), (80.0, 98.0, 7.0), (88.0, 82.0, 4.5), (56.0, 106.0, 4.5)] {
        s.ellipse(x, y, r, r * 0.85, Teal);
    }
    vec![finish(c, &s)]
}

pub fn baby(c: &Character) -> Vec<Img> {
    let mut feet = Sketch::new(SRC, SRC);
    feet.ellipse(42.0, 109.0, 12.0, 9.0, Pink);
    feet.ellipse(86.0, 109.0, 12.0, 9.0, Pink);

    let mut body = Sketch::new(SRC, SRC);
    body.ellipse(64.0, 78.0, 43.0, 35.0, Pink);
    // A little curl of hair on top.
    body.line(&[(62.0, 48.0), (59.0, 37.0), (65.0, 28.0), (75.0, 30.0)], 11.0, Pink);
    body.ellipse(64.0, 99.0, 19.0, 10.0, Blush);

    vec![stack(&[finish(c, &feet), finish(c, &body)])]
}

/// The swing of an animated part across its three frames, in degrees.
fn swing(c: &Character, amount: f32) -> [f32; 3] {
    debug_assert!(c.parts.iter().all(|p| p.frames == 3));
    [-amount, 0.0, amount]
}

/// One finished layer per angle, each drawn by `draw` with that angle.
fn frames(c: &Character, amount: f32, draw: impl Fn(&mut Sketch, f32)) -> Vec<Img> {
    swing(c, amount)
        .iter()
        .map(|&a| {
            let mut s = Sketch::new(SRC, SRC);
            draw(&mut s, a);
            finish(c, &s)
        })
        .collect()
}

fn feet(c: &Character, y: f32, spread: f32, rx: f32, p: Pal) -> Img {
    let mut s = Sketch::new(SRC, SRC);
    s.ellipse(64.0 - spread, y, rx, 8.0, p);
    s.ellipse(64.0 + spread, y, rx, 8.0, p);
    finish(c, &s)
}

pub fn marutchi(c: &Character) -> Vec<Img> {
    let mut ears = Sketch::new(SRC, SRC);
    ears.ellipse(36.0, 42.0, 13.0, 15.0, Yellow);
    ears.ellipse(92.0, 42.0, 13.0, 15.0, Yellow);
    let mut body = Sketch::new(SRC, SRC);
    body.ellipse(64.0, 76.0, 45.0, 39.0, Yellow);
    body.ellipse(64.0, 96.0, 24.0, 13.0, Cream);
    vec![stack(&[finish(c, &ears), feet(c, 111.0, 21.0, 14.0, Orange), finish(c, &body)])]
}

pub fn tonmarutchi(c: &Character) -> Vec<Img> {
    let mut body = Sketch::new(SRC, SRC);
    body.ellipse(64.0, 77.0, 48.0, 41.0, Sky);
    body.ellipse(64.0, 97.0, 27.0, 14.0, Ice);
    let mut cells = vec![stack(&[feet(c, 113.0, 24.0, 15.0, Sky), finish(c, &body)])];
    // Two long ears that wiggle in opposite directions from their roots.
    cells.extend(frames(c, 9.0, |s, a| {
        s.turn(-a, 44.0, 46.0);
        s.ellipse(40.0, 26.0, 10.0, 22.0, Sky);
        s.turn(a, 84.0, 46.0);
        s.ellipse(88.0, 26.0, 10.0, 22.0, Sky);
    }));
    cells
}

pub fn hashitamatchi(c: &Character) -> Vec<Img> {
    let mut spikes = Sketch::new(SRC, SRC);
    spikes.poly(&[(38.0, 50.0), (46.0, 16.0), (60.0, 42.0)], Violet);
    spikes.poly(&[(52.0, 40.0), (64.0, 6.0), (76.0, 40.0)], Violet);
    spikes.poly(&[(68.0, 42.0), (82.0, 16.0), (90.0, 50.0)], Violet);
    let mut body = Sketch::new(SRC, SRC);
    body.ellipse(64.0, 77.0, 48.0, 41.0, Violet);
    body.ellipse(64.0, 97.0, 27.0, 14.0, Blush);
    vec![stack(&[finish(c, &spikes), feet(c, 113.0, 24.0, 15.0, Violet), finish(c, &body)])]
}

pub fn mametchi(c: &Character) -> Vec<Img> {
    let mut arms = Sketch::new(SRC, SRC);
    arms.ellipse(19.0, 80.0, 9.0, 14.0, Green);
    arms.ellipse(109.0, 80.0, 9.0, 14.0, Green);
    let mut body = Sketch::new(SRC, SRC);
    body.ellipse(64.0, 72.0, 43.0, 47.0, Green);
    body.ellipse(64.0, 94.0, 25.0, 19.0, Cream);
    let mut cells = vec![stack(&[finish(c, &arms), feet(c, 117.0, 20.0, 15.0, Green), finish(c, &body)])];
    cells.extend(frames(c, 8.0, |s, a| {
        s.turn(-a, 46.0, 36.0);
        s.ellipse(42.0, 20.0, 10.0, 16.0, Green);
        s.turn(a, 82.0, 36.0);
        s.ellipse(86.0, 20.0, 10.0, 16.0, Green);
    }));
    cells
}

pub fn kuchipatchi(c: &Character) -> Vec<Img> {
    let mut body = Sketch::new(SRC, SRC);
    body.ellipse(64.0, 76.0, 44.0, 41.0, Yellow);
    body.line(&[(64.0, 36.0), (60.0, 24.0), (69.0, 18.0)], 9.0, Orange);
    // Big orange lips, where the face's mouth lands.
    body.ellipse(64.0, 70.0, 17.0, 8.0, Orange);
    let mut cells = vec![stack(&[feet(c, 117.0, 24.0, 17.0, Orange), finish(c, &body)])];
    // Little wings flapping from the shoulders.
    cells.extend(frames(c, 16.0, |s, a| {
        s.turn(a, 28.0, 70.0);
        s.ellipse(20.0, 78.0, 10.0, 16.0, Yellow);
        s.turn(-a, 100.0, 70.0);
        s.ellipse(108.0, 78.0, 10.0, 16.0, Yellow);
    }));
    cells
}

pub fn tarakotchi(c: &Character) -> Vec<Img> {
    let mut body = Sketch::new(SRC, SRC);
    body.ellipse(64.0, 76.0, 49.0, 43.0, Orange);
    body.ellipse(64.0, 72.0, 13.0, 7.0, Pink);
    for side in [-1.0f32, 1.0] {
        for dy in [-5.0, 2.0] {
            let x = 64.0 + side * 38.0;
            body.line(&[(x - side * 7.0, 70.0 + dy), (x + side * 5.0, 68.0 + dy * 1.6)], 3.0, Rust);
        }
    }
    let mut cells = vec![stack(&[feet(c, 117.0, 24.0, 16.0, Rust), finish(c, &body)])];
    // A tuft of hair swaying on top.
    cells.extend(frames(c, 10.0, |s, a| {
        s.turn(a, 64.0, 38.0);
        for (x, y) in [(50.0, 14.0), (64.0, 6.0), (78.0, 14.0)] {
            s.line(&[(64.0, 38.0), (x, y)], 9.0, Rust);
        }
    }));
    cells
}

pub fn ichigotchi(c: &Character) -> Vec<Img> {
    let mut body = Sketch::new(SRC, SRC);
    // A strawberry: wide and round on top, narrowing to a soft point.
    body.shape(
        |p| {
            p.move_to(64.0, 30.0);
            p.cubic_to(100.0, 28.0, 116.0, 52.0, 112.0, 78.0);
            p.cubic_to(108.0, 100.0, 84.0, 116.0, 64.0, 116.0);
            p.cubic_to(44.0, 116.0, 20.0, 100.0, 16.0, 78.0);
            p.cubic_to(12.0, 52.0, 28.0, 28.0, 64.0, 30.0);
        },
        Red,
    );
    for (x, y) in [(28.0, 70.0), (100.0, 70.0), (36.0, 92.0), (92.0, 92.0), (52.0, 104.0), (76.0, 104.0), (26.0, 52.0), (102.0, 52.0)] {
        body.ellipse(x, y, 2.5, 3.5, Yellow);
    }
    let mut cells = vec![stack(&[feet(c, 117.0, 18.0, 14.0, Maroon), finish(c, &body)])];
    // Two leaves and a stem, fluttering.
    cells.extend(frames(c, 10.0, |s, a| {
        s.turn(-a, 62.0, 34.0);
        s.ellipse(48.0, 30.0, 15.0, 7.0, Green);
        s.turn(a, 66.0, 34.0);
        s.ellipse(80.0, 30.0, 15.0, 7.0, Green);
        s.turn(a * 0.5, 64.0, 34.0);
        s.line(&[(64.0, 34.0), (64.0, 14.0)], 6.0, Forest);
    }));
    cells
}

pub fn nyorotchi(c: &Character) -> Vec<Img> {
    let mut body = Sketch::new(SRC, SRC);
    body.ellipse(64.0, 74.0, 44.0, 42.0, Teal);
    body.line(&[(64.0, 34.0), (60.0, 20.0), (68.0, 12.0)], 8.0, Teal);
    body.ellipse(64.0, 90.0, 24.0, 21.0, Ice);
    let mut cells = vec![finish(c, &body)];
    // A tail curling out to the right, swishing about its root.
    cells.extend(frames(c, 9.0, |s, a| {
        s.turn(a, 88.0, 104.0);
        s.line(&[(80.0, 108.0), (102.0, 106.0), (114.0, 92.0), (112.0, 76.0), (102.0, 70.0)], 13.0, Teal);
    }));
    cells
}

pub fn poop(c: &Character) -> Vec<Img> {
    // Three coils, each its own layer so the outline separates them.
    let coils = [(64.0, 100.0, 48.0, 20.0), (62.0, 74.0, 36.0, 17.0), (66.0, 50.0, 24.0, 14.0)].map(|(x, y, rx, ry)| {
        let mut s = Sketch::new(SRC, SRC);
        s.ellipse(x, y, rx, ry, Tan);
        if ry < 15.0 {
            s.line(&[(64.0, 44.0), (72.0, 26.0)], 13.0, Tan);
        }
        finish(c, &s)
    });
    vec![stack(&coils)]
}

pub fn grave(c: &Character) -> Vec<Img> {
    let mut stone = Sketch::new(SRC, SRC);
    stone.ellipse(64.0, 52.0, 30.0, 26.0, Slate);
    stone.rect(34.0, 52.0, 60.0, 58.0, Slate);
    // An engraved cross.
    stone.rect(60.0, 50.0, 8.0, 34.0, Dusk);
    stone.rect(50.0, 60.0, 28.0, 7.0, Dusk);
    let mut grass = Sketch::new(SRC, SRC);
    grass.ellipse(64.0, 112.0, 48.0, 9.0, Green);
    vec![stack(&[finish(c, &stone), finish(c, &grass)])]
}

pub fn ghost(c: &Character) -> Vec<Img> {
    let mut s = Sketch::new(SRC, SRC);
    s.ellipse(64.0, 54.0, 38.0, 38.0, White);
    s.rect(26.0, 54.0, 76.0, 36.0, White);
    for x in [39.0, 64.0, 89.0] {
        s.ellipse(x, 92.0, 13.0, 18.0, White);
    }
    vec![finish(c, &s)]
}

/// The two halves an egg leaves behind: the bowl with a ragged rim, and the cap tipped on its side.
pub fn shell(c: &Character) -> Vec<Img> {
    let teeth = |p: &mut tiny_skia::PathBuilder, y: f32, from: f32, to: f32| {
        let n = 8;
        for i in 1..n {
            let x = from + (to - from) * i as f32 / n as f32;
            p.line_to(x, if i % 2 == 1 { y - 12.0 } else { y + 2.0 });
        }
    };
    let mut bowl = Sketch::new(SRC, SRC);
    bowl.shape(
        |p| {
            p.move_to(18.0, 68.0);
            teeth(p, 68.0, 18.0, 110.0);
            p.line_to(110.0, 68.0);
            p.cubic_to(114.0, 100.0, 92.0, 120.0, 64.0, 120.0);
            p.cubic_to(36.0, 120.0, 14.0, 100.0, 18.0, 68.0);
        },
        White,
    );
    bowl.ellipse(44.0, 100.0, 8.0, 7.0, Teal);
    bowl.ellipse(82.0, 108.0, 6.0, 5.0, Teal);
    let mut cap = Sketch::new(SRC, SRC);
    cap.turn(-32.0, 64.0, 80.0);
    cap.shape(
        |p| {
            p.move_to(22.0, 86.0);
            teeth(p, 86.0, 22.0, 106.0);
            p.line_to(106.0, 86.0);
            p.cubic_to(106.0, 40.0, 86.0, 22.0, 64.0, 22.0);
            p.cubic_to(42.0, 22.0, 22.0, 40.0, 22.0, 86.0);
        },
        White,
    );
    cap.ellipse(70.0, 46.0, 8.0, 7.0, Teal);
    vec![finish(c, &bowl), finish(c, &cap)]
}

pub fn cells(c: &Character) -> Vec<Img> {
    match c.name {
        "egg" => egg(c),
        "baby" => baby(c),
        "marutchi" => marutchi(c),
        "tonmarutchi" => tonmarutchi(c),
        "hashitamatchi" => hashitamatchi(c),
        "mametchi" => mametchi(c),
        "kuchipatchi" => kuchipatchi(c),
        "tarakotchi" => tarakotchi(c),
        "ichigotchi" => ichigotchi(c),
        "nyorotchi" => nyorotchi(c),
        "poop" => poop(c),
        "grave" => grave(c),
        "ghost" => ghost(c),
        "shell" => shell(c),
        other => panic!("no drawing for {other}"),
    }
}
