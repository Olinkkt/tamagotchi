//! The washroom: a row of toilet booths, a door and the front of the bowl. Drawn like the room,
//! in master pixels on a 4x canvas, with the layout numbers shared through `cast`.

use crate::cast::{BOOTHS, BOOTH_PITCH, BOOTH_X, DOOR, TOILET, TOILET_TOP, WASHROOM_FLOOR as FLOOR};
use crate::img::Img;
use crate::palette::Pal::{self, *};
use crate::process::{layer, Finish};
use crate::room::{lined, K, MH, MW};
use crate::sketch::Sketch;

fn canvas() -> Sketch {
    Sketch::new((MW as f32 * K) as u32, (MH as f32 * K) as u32).scaled(K)
}

fn flat(s: &Sketch) -> Img {
    layer(&s.to_img(), MW, MH, &Finish { shade: false, outline: None })
}

fn put(img: &mut Img, x: i32, y: i32, p: Pal) {
    img.set(x, y, Some(p));
}

fn centre(i: usize) -> i32 {
    BOOTH_X + BOOTH_PITCH * i as i32
}

/// Tiled wall above, chequered floor below.
fn shell() -> Img {
    let mut s = canvas();
    s.rect(0.0, 0.0, MW as f32, FLOOR as f32, Ice);
    s.rect(0.0, FLOOR as f32 - 30.0, MW as f32, 3.0, Teal);
    s.rect(0.0, FLOOR as f32 - 27.0, MW as f32, 1.0, DeepTeal);
    s.rect(0.0, FLOOR as f32, MW as f32, (MH as i32 - FLOOR) as f32, White);
    let mut img = flat(&s);
    for y in (0..FLOOR - 30).filter(|y| y % 14 == 13) {
        for x in 0..MW as i32 {
            put(&mut img, x, y, Sky);
        }
    }
    for y in (0..FLOOR - 30).filter(|y| y % 14 != 13) {
        for x in (0..MW as i32).filter(|x| (x + (y / 14) * 7) % 28 == 27) {
            put(&mut img, x, y, Sky);
        }
    }
    // Floor tiles, 12 across, in two tones.
    for y in FLOOR..MH as i32 {
        for x in 0..MW as i32 {
            if ((x / 12) + ((y - FLOOR) / 6)) % 2 == 0 {
                put(&mut img, x, y, Ice);
            }
        }
    }
    img
}

/// What is inside a booth: the back wall and the cistern.
fn stalls() -> Img {
    let mut s = canvas();
    for i in 0..BOOTHS {
        let x = centre(i) as f32;
        s.rect(x - 30.0, 134.0, 60.0, FLOOR as f32 - 134.0, Mist);
        s.rect(x - 30.0, 134.0, 60.0, 3.0, Slate);
        s.rect(x - 11.0, 162.0, 22.0, 22.0, White);
        s.rect(x - 12.0, 160.0, 24.0, 3.0, Ice);
        s.rect(x - 2.0, 158.0, 4.0, 2.0, Slate);
        s.rect(x - 11.0, 180.0, 22.0, 4.0, Ice);
        s.ellipse(x, 198.0, 15.0, 5.0, Ice);
    }
    lined(&s)
}

fn partitions() -> Img {
    let mut s = canvas();
    for j in 0..=BOOTHS {
        let x = (centre(0) - 36 + BOOTH_PITCH * j as i32) as f32;
        s.rect(x, 130.0, 6.0, FLOOR as f32 + 4.0 - 130.0, Teal);
        s.rect(x, 130.0, 6.0, 3.0, DeepTeal);
        s.rect(x + 1.0, 133.0, 1.0, FLOOR as f32 + 4.0 - 133.0, Sky);
    }
    lined(&s)
}

/// The front of the bowl: the rim the pet sits in, and the base under it.
pub fn toilet() -> Img {
    let (w, h) = TOILET;
    let mut s = Sketch::new(w as u32 * 4, h as u32 * 4).scaled(4.0);
    s.poly(&[(7.0, 9.0), (33.0, 9.0), (29.0, 18.0), (11.0, 18.0)], White);
    s.rect(11.0, 16.0, 18.0, 2.0, Ice);
    s.ellipse(20.0, 8.0, 16.0, 6.0, White);
    s.ellipse(20.0, 8.0, 12.5, 4.0, Ice);
    layer(&s.to_img(), w, h, &Finish { shade: false, outline: Some(Night) })
}

/// A closed door with a recessed panel and a slatted top; the game paints the lock.
pub fn door() -> Img {
    let (w, h) = DOOR;
    let mut s = Sketch::new(w as u32 * 4, h as u32 * 4).scaled(4.0);
    s.rect(1.0, 1.0, w as f32 - 2.0, h as f32 - 2.0, Teal);
    s.rect(1.0, 1.0, w as f32 - 2.0, 3.0, DeepTeal);
    s.rect(1.0, 4.0, 2.0, h as f32 - 5.0, Sky);
    for k in 0..4 {
        s.rect(8.0, 9.0 + k as f32 * 4.0, w as f32 - 16.0, 2.0, DeepTeal);
    }
    s.rect(8.0, 30.0, w as f32 - 16.0, 28.0, DeepTeal);
    s.rect(10.0, 32.0, w as f32 - 20.0, 24.0, Teal);
    layer(&s.to_img(), w, h, &Finish { shade: false, outline: Some(Night) })
}

pub fn washroom() -> Img {
    let mut img = shell();
    img.blit(&stalls(), 0, 0);
    img.blit(&partitions(), 0, 0);
    let bowl = toilet();
    for i in 0..BOOTHS {
        img.blit(&bowl, centre(i) - TOILET.0 as i32 / 2, TOILET_TOP);
    }
    img
}
