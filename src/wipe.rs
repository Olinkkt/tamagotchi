//! The pixel dissolve between scenes: blocks of ink pop in all over the picture until it is shut,
//! the game swaps scenes, and the blocks pop out again in the new scene.
//! It runs on real time, so the game speed does not change it.

use std::time::Instant;

use crate::frame::Frame;
use crate::palette::Pal;

const CLOSE_SECS: f32 = 0.3;
const OPEN_SECS: f32 = 0.3;
/// Side of one block of ink, in scene pixels.
const BLOCK: i32 = 8;

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

/// How much of the picture is inked `secs` into the wipe: 0 open, 1 shut.
fn shut(secs: f32) -> f32 {
    if secs < CLOSE_SECS {
        secs / CLOSE_SECS
    } else {
        1.0 - (secs - CLOSE_SECS) / OPEN_SECS
    }
    .clamp(0.0, 1.0)
}

/// A fixed, scattered 0..1 rank for one block, so each block always pops in at the same moment.
fn rank(bx: i32, by: i32) -> f32 {
    let mut h = (bx as u32).wrapping_mul(0x9E37_79B1) ^ (by as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h % 1000) as f32 / 1000.0
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

    /// Ink over the blocks that have popped in so far.
    pub fn cover(&self, f: &mut Frame) {
        let amount = shut(self.age());
        for by in 0..(f.h + BLOCK - 1) / BLOCK {
            for bx in 0..(f.w + BLOCK - 1) / BLOCK {
                // `<=` so a fully shut picture has no block left open.
                if rank(bx, by) <= amount && amount > 0.0 {
                    f.rect(bx * BLOCK, by * BLOCK, BLOCK, BLOCK, Pal::Ink);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_blocks_fill_in_then_clear_again() {
        assert_eq!(shut(0.0), 0.0);
        assert!((shut(CLOSE_SECS) - 1.0).abs() < 0.001);
        assert_eq!(shut(CLOSE_SECS + OPEN_SECS), 0.0);
        assert!(shut(CLOSE_SECS / 2.0) > 0.4 && shut(CLOSE_SECS / 2.0) < 0.6);
    }

    #[test]
    fn a_shut_picture_is_all_ink_and_an_open_one_is_untouched() {
        let mut wipe = Wipe::new(Next::Room);
        let mut f = Frame::new(40, 30, Pal::White);
        wipe.started -= std::time::Duration::from_secs_f32(CLOSE_SECS);
        wipe.cover(&mut f);
        assert_eq!(f.get(0, 0), Some(Pal::Ink));
        assert_eq!(f.get(39, 29), Some(Pal::Ink));
        assert!(wipe.swap_now() && !wipe.swap_now(), "the swap fires once");
        let mut open = Frame::new(40, 30, Pal::White);
        wipe.started -= std::time::Duration::from_secs_f32(OPEN_SECS);
        assert!(wipe.done());
        wipe.cover(&mut open);
        assert_eq!(open.get(0, 0), Some(Pal::White));
    }
}
