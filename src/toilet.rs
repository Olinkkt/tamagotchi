//! The washroom visit: the pet sits on the toilet in one of the booths, the door swings shut,
//! a few seconds go by with nothing shown, and the door opens again. It runs on real time, so
//! the game speed does not change it.

use std::time::Instant;

use crate::app::App;
use crate::art;
use crate::cast;
use crate::frame::Frame;
use crate::icons;
use crate::palette::Pal;
use crate::scene;

/// The whole visit, in seconds.
pub const VISIT_SECS: f32 = 5.7;
/// The pet sits with the door open, then the door swings shut, stays shut, and swings open.
const SETTLE: f32 = 0.8;
const SWING: f32 = 0.7;
const OPEN_AT: f32 = 4.2;

pub struct Visit {
    started: Instant,
    booth: usize,
}

impl Visit {
    pub fn new(booth: usize) -> Visit {
        Visit { started: Instant::now(), booth: booth % cast::BOOTHS }
    }

    fn age(&self) -> f32 {
        self.started.elapsed().as_secs_f32()
    }

    pub fn done(&self) -> bool {
        self.age() >= VISIT_SECS
    }
}

/// How far the door has swung shut at `t` seconds: 0 is open, 1 is shut.
fn shut(t: f32) -> f32 {
    if t < SETTLE {
        0.0
    } else if t < SETTLE + SWING {
        (t - SETTLE) / SWING
    } else if t < OPEN_AT {
        1.0
    } else {
        (1.0 - (t - OPEN_AT) / SWING).max(0.0)
    }
}

/// A door drawn `swing` of the way shut, hinged on its left edge: it narrows as it opens.
fn draw_door(f: &mut Frame, door: &art::Sprite, x: i32, y: i32, swing: f32, lock: Pal) {
    let full = door.w;
    let width = ((full as f32 * swing).round() as i32).max(8);
    for dx in 0..width {
        let src = dx * full / width;
        for sy in 0..door.h {
            if let Some(p) = door.get(src, sy) {
                f.set(x + dx, y + sy, p);
            }
        }
    }
    if width == full {
        f.boxed(x + full - 17, y + 24, 8, 5, lock, Pal::Ink);
    }
}

fn booth_x(booth: usize) -> i32 {
    cast::BOOTH_X + cast::BOOTH_PITCH * booth as i32
}

/// Where the master image lands in a `w` x `h` area centred on a booth: the left edge of the
/// slice shown, and the shift from master pixels to area pixels.
fn view(booth: usize, w: i32, h: i32) -> (i32, (i32, i32)) {
    let master = art::washroom();
    let (cw, ch) = (w.min(master.w), h.min(master.h));
    let sx = (booth_x(booth) - cw / 2).clamp(0, master.w - cw);
    (sx, ((w - cw) / 2 - sx, h - ch - (master.h - ch)))
}

/// The middle of the pet on its toilet, in area pixels.
pub fn spot(app: &App, visit: &Visit, w: i32, h: i32) -> (i32, i32) {
    let (_, (ox, oy)) = view(visit.booth, w, h);
    let body = art::body(&scene::character(&app.pet));
    (booth_x(visit.booth) + ox, cast::TOILET_TOP + oy + 7 - (body.bottom() - body.top()) / 2)
}

/// The washroom, centred on the pet's booth and as large as the area allows.
pub fn area(app: &App, visit: &Visit, w: i32, h: i32) -> Frame {
    let t = visit.age();
    let master = art::washroom();
    let mut f = Frame::new(w, h, Pal::Ink);
    let (cw, ch) = (w.min(master.w), h.min(master.h));
    let (sx, (ox, oy)) = view(visit.booth, w, h);
    f.region(master, (sx, master.h - ch, cw, ch), (w - cw) / 2, h - ch);
    let mid = booth_x(visit.booth) + ox;
    let toilet_y = cast::TOILET_TOP + oy;

    let c = scene::character(&app.pet);
    let body = art::body(&c);
    f.sprite(&scene::seated(app), mid - c.size as i32 / 2, toilet_y + 7 - body.bottom());
    if t < 0.4 && (t * 20.0) as i32 % 2 == 0 {
        f.grid(icons::SPARKLE, mid - 18, toilet_y - body.bottom() / 2 - 8);
        f.grid(icons::SPARKLE, mid + 12, toilet_y - body.bottom() / 2);
    }
    f.sprite(art::toilet(), mid - cast::TOILET.0 as i32 / 2, toilet_y);

    let door = art::door();
    for booth in 0..cast::BOOTHS {
        let mine = booth == visit.booth;
        let (swing, lock) = if mine { (shut(t), Pal::Red) } else { (1.0, Pal::Lime) };
        draw_door(&mut f, door, booth_x(booth) + ox - door.w / 2, cast::DOOR_TOP + oy, swing, lock);
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_booth_draws_at_every_window_size() {
        let app = crate::app::tests::app_with(crate::pet::Stage::Adult, |_| {});
        for booth in 0..cast::BOOTHS {
            for (w, h) in [(160, 116), (320, 160), (544, 244)] {
                let f = area(&app, &Visit::new(booth), w, h);
                assert_eq!((f.w, f.h), (w, h));
            }
        }
    }

    #[test]
    fn the_door_swings_shut_waits_and_opens() {
        assert_eq!(shut(0.0), 0.0);
        assert!(shut(SETTLE + SWING / 2.0) > 0.4 && shut(SETTLE + SWING / 2.0) < 0.6);
        assert_eq!(shut(2.0), 1.0);
        assert!(shut(OPEN_AT + SWING / 2.0) < 0.6);
        assert_eq!(shut(VISIT_SECS), 0.0);
    }
}
