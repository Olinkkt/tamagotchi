//! The characters, drawn in shapes on a 128x128 canvas. Each function returns the sheet cells
//! in the order `cast.rs` describes: the body, then every part frame.

use crate::cast::Character;
use crate::img::Img;
use crate::palette::Pal::*;
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

pub fn cells(c: &Character) -> Vec<Img> {
    match c.name {
        "egg" => egg(c),
        "baby" => baby(c),
        other => panic!("no drawing for {other}"),
    }
}
