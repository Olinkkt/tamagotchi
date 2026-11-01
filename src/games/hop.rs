//! Hop: an endless-runner like the dinosaur game. The floor scrolls and crates come at your pet,
//! faster and faster. Press space (or click) to jump. Touch a crate and the run is over. Every
//! three crates you clear fills one of five boxes; clear all fifteen and the run ends on its own.

use super::*;

pub const ROUNDS: u8 = 5;
/// Crates cleared for each box.
const PER_BOX: usize = 3;
const OBSTACLES: usize = PER_BOX * ROUNDS as usize;
const PET_X: i32 = 70;
/// The pet's hit box is this wide either side of its centre.
const PET_HALF: i32 = 6;
const FLOOR: i32 = GH - 14;
const CRATE_W: i32 = 14;
const CRATE_H: i32 = 16;
const TALL_H: i32 = 26;
/// A crate only hurts if the pet's feet are lower than its top by more than this.
const FORGIVE: i32 = 2;
/// Pixels a second at the start, how much faster it gets each second, and the top speed.
const START_SPEED: f64 = 130.0;
const ACCEL: f64 = 4.0;
const MAX_SPEED: f64 = 220.0;
const JUMP_MS: u64 = 650;
const JUMP_H: i32 = 34;
/// Where the first crate stands, and the smallest gap between two (pixels, from the pet's side).
const FIRST_AT: i32 = 330;
const MIN_GAP: i32 = 170;
/// Collisions are checked this often, so a slow frame cannot skip over a crate.
const STEP_MS: u64 = 10;
/// How long the pet slumps after a crash, or runs on after the last crate.
const CRASH_MS: u64 = 900;
const FINISH_MS: u64 = 600;

#[derive(Clone, Copy)]
struct Crate {
    /// Left edge, in pixels along the course (the pet is at 0 when the run starts).
    at: i32,
    w: i32,
    h: i32,
}

pub struct Hop {
    crates: Vec<Crate>,
    /// When the current jump began.
    jump: Option<u64>,
    /// How far the game's clock has been checked for crashes.
    checked: u64,
    cleared: usize,
    crashed: Option<u64>,
    /// When the last crate was passed.
    finished: Option<u64>,
    look: Box<Pet>,
    t0: Instant,
}

/// How far the course has scrolled `ms` milliseconds in: it speeds up steadily to a top speed.
fn course(ms: u64) -> i32 {
    let t = ms as f64 / 1000.0;
    let ramp = (MAX_SPEED - START_SPEED) / ACCEL;
    let d = if t <= ramp { START_SPEED * t + ACCEL * t * t / 2.0 } else { START_SPEED * ramp + ACCEL * ramp * ramp / 2.0 + MAX_SPEED * (t - ramp) };
    d as i32
}

/// How high the pet is `since` milliseconds after the jump began (a smooth arc).
fn jump_height(since: u64) -> i32 {
    if since >= JUMP_MS {
        return 0;
    }
    let u = since as f32 / JUMP_MS as f32;
    (4.0 * JUMP_H as f32 * u * (1.0 - u)).round() as i32
}

impl Hop {
    pub fn new(pet: &mut Pet) -> Self {
        let mut at = FIRST_AT;
        let crates = (0..OBSTACLES)
            .map(|_| {
                let c = if pet.rand(4) == 0 {
                    Crate { at, w: CRATE_W, h: TALL_H }
                } else {
                    Crate { at, w: CRATE_W * (1 + pet.rand(3) as i32), h: CRATE_H }
                };
                at += c.w + MIN_GAP + pet.rand(10) as i32 * 14;
                c
            })
            .collect();
        Hop { crates, jump: None, checked: 0, cleared: 0, crashed: None, finished: None, look: Box::new(pet.clone()), t0: Instant::now() }
    }

    fn now(&self) -> u64 {
        self.t0.elapsed().as_millis() as u64
    }

    pub fn key(&mut self, code: Key) -> Outcome {
        if code == Key::Char(' ') {
            self.press(self.now());
        }
        Outcome::Continue
    }

    fn press(&mut self, now: u64) {
        if self.crashed.is_none() && self.jump.is_none_or(|j| now >= j + JUMP_MS) {
            self.jump = Some(now);
        }
    }

    fn height(&self, now: u64) -> i32 {
        self.jump.map_or(0, |j| jump_height(now.saturating_sub(j)))
    }

    /// Where a crate's left edge is on screen at `ms`.
    fn screen_x(c: &Crate, ms: u64) -> i32 {
        PET_X + c.at - course(ms)
    }

    pub fn tick(&mut self) -> Outcome {
        self.advance(self.now())
    }

    /// Run the game up to `now`, one small step at a time.
    fn advance(&mut self, now: u64) -> Outcome {
        while self.crashed.is_none() && self.finished.is_none() && self.checked < now {
            self.checked = (self.checked + STEP_MS).min(now);
            let t = self.checked;
            let h = self.height(t);
            while let Some(c) = self.crates.get(self.cleared) {
                let x = Self::screen_x(c, t);
                if x + c.w < PET_X - PET_HALF {
                    self.cleared += 1;
                    continue;
                }
                if x < PET_X + PET_HALF && h < c.h - FORGIVE {
                    self.crashed = Some(t);
                }
                break;
            }
            if self.cleared == self.crates.len() && self.crashed.is_none() {
                self.finished = Some(t);
            }
        }
        let end = self.crashed.map(|t| t + CRASH_MS).or(self.finished.map(|t| t + FINISH_MS));
        if end.is_some_and(|e| now >= e) {
            return Outcome::Done { wins: self.boxes(), rounds: ROUNDS };
        }
        Outcome::Continue
    }

    fn boxes(&self) -> u8 {
        (self.cleared / PER_BOX).min(ROUNDS as usize) as u8
    }

    pub fn hint(&self) -> &'static str {
        if self.crashed.is_some() { "OOF! YOU HIT A CRATE" } else { "SPACE OR CLICK TO JUMP. DON'T HIT A CRATE" }
    }

    pub fn draw(&self) -> Frame {
        let mut f = backdrop();
        // The picture freezes where the pet crashed.
        let now = self.crashed.map_or(self.now(), |c| c.min(self.now()));
        let h = self.height(now);
        let d = course(now);
        for i in 0..GW / 24 + 2 {
            f.rect(i * 24 - d % 24, FLOOR + 5, 6, 1, Pal::Maroon);
        }
        let mood = if self.crashed.is_some() {
            Expr::Sad
        } else if h > 0 {
            Expr::Happy
        } else {
            Expr::Idle
        };
        draw_pet(&mut f, &self.look, mood, PET_X, FLOOR - h);
        let mut results = vec![true; self.boxes() as usize];
        if self.crashed.is_some() && results.len() < ROUNDS as usize {
            results.push(false);
        }
        draw_rounds(&mut f, &results, results.len() as u8 + 1, ROUNDS);
        for c in &self.crates {
            let x = Self::screen_x(c, now);
            if x > GW || x + c.w < 0 {
                continue;
            }
            let top = FLOOR - c.h;
            f.boxed(x, top, c.w, c.h, Pal::Tan, Pal::Ink);
            f.rect(x + 2, top + 2, c.w - 4, 2, Pal::Brown);
            f.rect(x + 2, FLOOR - 4, c.w - 4, 2, Pal::Brown);
            if c.h == TALL_H {
                f.rect(x + 2, top + c.h / 2, c.w - 4, 2, Pal::Brown);
            }
        }
        f
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pet() -> Pet {
        let mut p = Pet::new(3);
        p.stage = crate::pet::Stage::Baby;
        p
    }

    /// The moment the middle of a crate is level with the pet.
    fn level(c: &Crate) -> u64 {
        (0..).step_by(5).find(|&t| PET_X + c.at + c.w / 2 - course(t) <= PET_X).unwrap()
    }

    /// Press so the top of the jump is over the crate.
    fn jump_over(g: &mut Hop, i: usize) {
        let c = g.crates[i];
        g.press(level(&c) - JUMP_MS / 2);
    }

    #[test]
    fn the_course_speeds_up_to_a_top_speed() {
        assert_eq!(course(0), 0);
        let (a, b) = (course(1000) - course(0), course(2000) - course(1000));
        assert!(b > a, "faster each second");
        let (x, y) = (course(41_000) - course(40_000), course(61_000) - course(60_000));
        assert_eq!(x, y, "and then steady");
        assert_eq!(y, MAX_SPEED as i32);
    }

    #[test]
    fn a_jump_is_an_arc_that_peaks_in_the_middle_and_lands() {
        assert_eq!(jump_height(0), 0);
        assert_eq!(jump_height(JUMP_MS / 2), JUMP_H);
        assert_eq!(jump_height(JUMP_MS), 0);
    }

    #[test]
    fn standing_still_runs_into_the_first_crate() {
        let mut p = pet();
        let mut g = Hop::new(&mut p);
        let hit = level(&g.crates[0]);
        assert!(matches!(g.advance(hit - 600), Outcome::Continue));
        assert!(g.crashed.is_none());
        g.advance(hit);
        assert!(g.crashed.is_some());
        assert!(matches!(g.advance(hit + CRASH_MS + 100), Outcome::Done { wins: 0, rounds: ROUNDS }));
    }

    #[test]
    fn jumping_over_a_crate_clears_it() {
        let mut p = pet();
        let mut g = Hop::new(&mut p);
        jump_over(&mut g, 0);
        g.advance(level(&g.crates[0]) + 400);
        assert!(g.crashed.is_none());
        assert_eq!(g.cleared, 1);
    }

    #[test]
    fn a_jump_that_is_too_early_or_too_late_crashes() {
        for shift in [-(JUMP_MS as i64), JUMP_MS as i64 / 2 + 150] {
            let mut p = pet();
            let mut g = Hop::new(&mut p);
            g.press((level(&g.crates[0]) as i64 + shift - JUMP_MS as i64 / 2) as u64);
            g.advance(level(&g.crates[0]) + 400);
            assert!(g.crashed.is_some(), "shift {shift}");
        }
    }

    #[test]
    fn you_cannot_start_a_second_jump_in_mid_air() {
        let mut p = pet();
        let mut g = Hop::new(&mut p);
        g.press(1000);
        g.press(1200);
        assert_eq!(g.jump, Some(1000));
        g.press(1000 + JUMP_MS);
        assert_eq!(g.jump, Some(1000 + JUMP_MS), "after landing it can jump again");
    }

    #[test]
    fn a_slow_frame_cannot_skip_a_crate() {
        let mut p = pet();
        let mut g = Hop::new(&mut p);
        g.advance(level(&g.crates[0]) + 5_000);
        assert!(g.crashed.is_some(), "one huge step still checks every crate on the way");
    }

    #[test]
    fn clearing_every_crate_ends_the_run_with_five_boxes() {
        let mut p = pet();
        let mut g = Hop::new(&mut p);
        for i in 0..OBSTACLES {
            jump_over(&mut g, i);
            g.advance(level(&g.crates[i]) + JUMP_MS);
        }
        assert!(g.crashed.is_none(), "every crate cleared");
        let end = g.finished.unwrap();
        assert!(matches!(g.advance(end + FINISH_MS - 1), Outcome::Continue));
        assert!(matches!(g.advance(end + FINISH_MS), Outcome::Done { wins: 5, rounds: ROUNDS }));
    }

    #[test]
    fn crashing_midway_pays_for_the_boxes_filled_so_far() {
        let mut p = pet();
        let mut g = Hop::new(&mut p);
        for i in 0..4 {
            jump_over(&mut g, i);
            g.advance(level(&g.crates[i]) + JUMP_MS);
        }
        let hit = level(&g.crates[4]);
        g.advance(hit + CRASH_MS + 100);
        assert!(matches!(g.advance(hit + 3_000), Outcome::Done { wins: 1, rounds: ROUNDS }), "four crates is one box");
    }

    #[test]
    fn the_crates_are_spaced_so_every_one_can_be_jumped() {
        let mut p = pet();
        let g = Hop::new(&mut p);
        assert!(g.crates.windows(2).all(|w| w[1].at - (w[0].at + w[0].w) >= MIN_GAP));
        assert!(g.crates.iter().all(|c| c.w <= CRATE_W * 3 && c.h <= TALL_H));
        assert!(level(g.crates.last().unwrap()) < 45_000, "a whole run is well under a minute");
    }
}
