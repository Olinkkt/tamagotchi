//! The iris wipe between scenes: the picture closes to a black circle around the pet, the game
//! swaps scenes while it is shut, and the circle opens again around the pet in the new scene.
//! It runs on real time, so the game speed does not change it.

use std::time::Instant;

use crate::frame::Frame;
use crate::palette::Pal;

const CLOSE_SECS: f32 = 0.45;
const OPEN_SECS: f32 = 0.45;

/// Where the wipe takes the player.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Next {
    /// Into the washroom, to this booth.
    Toilet(usize),
    /// Back to the room.
    Room,
}

pub struct Wipe {
    pub(crate) started: Instant,
    pub next: Next,
    swapped: bool,
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The circle's radius `secs` into the wipe, for a picture whose far corner is `max` away.
fn radius(secs: f32, max: f32) -> f32 {
    if secs < CLOSE_SECS {
        max * (1.0 - smooth(secs / CLOSE_SECS))
    } else {
        max * smooth((secs - CLOSE_SECS) / OPEN_SECS)
    }
}

impl Wipe {
    pub fn new(next: Next) -> Wipe {
        Wipe { started: Instant::now(), next, swapped: false }
    }

    fn age(&self) -> f32 {
        self.started.elapsed().as_secs_f32()
    }

    /// True once, the moment the picture is fully shut: time to change scenes.
    pub fn swap_now(&mut self) -> bool {
        let due = !self.swapped && self.age() >= CLOSE_SECS;
        self.swapped |= due;
        due
    }

    pub fn done(&self) -> bool {
        self.age() >= CLOSE_SECS + OPEN_SECS
    }

    /// Black out everything outside the circle around `centre`.
    pub fn cover(&self, f: &mut Frame, (cx, cy): (i32, i32)) {
        let far = |x: i32, y: i32| (((x - cx).pow(2) + (y - cy).pow(2)) as f32).sqrt();
        let max = far(0, 0).max(far(f.w, 0)).max(far(0, f.h)).max(far(f.w, f.h));
        let r2 = radius(self.age(), max).powi(2);
        for y in 0..f.h {
            for x in 0..f.w {
                if (((x - cx).pow(2) + (y - cy).pow(2)) as f32) > r2 {
                    f.set(x, y, Pal::Ink);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_circle_closes_to_nothing_then_opens_past_the_corners() {
        assert!((radius(0.0, 100.0) - 100.0).abs() < 0.01);
        assert!(radius(CLOSE_SECS, 100.0) < 0.01);
        assert!(radius(CLOSE_SECS + OPEN_SECS, 100.0) > 99.9);
        assert!(radius(CLOSE_SECS / 2.0, 100.0) > 30.0 && radius(CLOSE_SECS / 2.0, 100.0) < 70.0);
    }

    #[test]
    fn a_shut_picture_is_all_ink_and_an_open_one_is_untouched() {
        let mut wipe = Wipe::new(Next::Room);
        let mut f = Frame::new(40, 30, Pal::White);
        wipe.started -= std::time::Duration::from_secs_f32(CLOSE_SECS);
        wipe.cover(&mut f, (20, 15));
        assert_eq!(f.get(0, 0), Some(Pal::Ink));
        assert_eq!(f.get(39, 29), Some(Pal::Ink));
        assert!(wipe.swap_now() && !wipe.swap_now(), "the swap fires once");
        let mut open = Frame::new(40, 30, Pal::White);
        wipe.started -= std::time::Duration::from_secs_f32(OPEN_SECS);
        assert!(wipe.done());
        wipe.cover(&mut open, (20, 15));
        assert_eq!(open.get(0, 0), Some(Pal::White));
    }
}
