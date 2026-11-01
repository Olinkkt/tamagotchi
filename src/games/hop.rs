//! Hop: crates slide towards the pet along the floor. Press space (or click) to jump over each
//! one. Five crates, about 15 seconds in all; one button.

use super::*;

pub const ROUNDS: u8 = 5;
const PET_X: i32 = 70;
const FLOOR: i32 = GH - 14;
const CRATE: i32 = 16;
/// Pixels a second, from the right edge to the pet.
const SPEED: i64 = 100;
const START_X: i32 = GW + 8;
const JUMP_MS: u64 = 700;
const JUMP_H: i32 = 34;
/// Clear the crate by being at least this high as it passes.
const CLEAR_H: i32 = CRATE - 2;
/// How long after the last crate before the game ends.
const END_MS: u64 = 1200;

pub struct Hop {
    /// When each crate sets off from the right edge, in milliseconds on the game's clock.
    spawn: Vec<u64>,
    results: Vec<bool>,
    /// When the current jump began.
    jump: Option<u64>,
    /// When the last crate was resolved, and a stumble's end.
    stumble_until: u64,
    look: Box<Pet>,
    t0: Instant,
}

/// When a crate that set off at `spawn` is level with the pet.
fn crosses(spawn: u64) -> u64 {
    spawn + ((START_X - PET_X) as i64 * 1000 / SPEED) as u64
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
        let mut at = 900;
        let spawn = (0..ROUNDS)
            .map(|_| {
                let s = at;
                at += 2300 + pet.rand(10) * 100;
                s
            })
            .collect();
        Hop { spawn, results: Vec::new(), jump: None, stumble_until: 0, look: Box::new(pet.clone()), t0: Instant::now() }
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
        if self.jump.is_none_or(|j| now >= j + JUMP_MS) {
            self.jump = Some(now);
        }
    }

    fn height(&self, now: u64) -> i32 {
        self.jump.map_or(0, |j| jump_height(now.saturating_sub(j)))
    }

    pub fn tick(&mut self) -> Outcome {
        self.advance(self.now())
    }

    /// Settle every crate that has reached the pet by `now`.
    fn advance(&mut self, now: u64) -> Outcome {
        while let Some(&s) = self.spawn.get(self.results.len()) {
            if now < crosses(s) {
                break;
            }
            // The jump is judged at the moment the crate passes, using where the pet was then.
            let cleared = self.height(crosses(s)) >= CLEAR_H;
            self.results.push(cleared);
            if !cleared {
                self.stumble_until = now + 600;
            }
        }
        if self.results.len() == self.spawn.len() && now >= crosses(*self.spawn.last().unwrap()) + END_MS {
            return Outcome::Done { wins: self.results.iter().filter(|&&r| r).count() as u8, rounds: ROUNDS };
        }
        Outcome::Continue
    }

    pub fn hint(&self) -> &'static str {
        if self.now() < self.stumble_until { "OOF! KEEP GOING" } else { "PRESS SPACE (OR CLICK) TO JUMP OVER EACH CRATE" }
    }

    pub fn draw(&self) -> Frame {
        let mut f = backdrop();
        let now = self.now();
        let h = self.height(now);
        let mood = if now < self.stumble_until {
            Expr::Sad
        } else if h > 0 {
            Expr::Happy
        } else {
            Expr::Idle
        };
        draw_pet(&mut f, &self.look, mood, PET_X, FLOOR - h);
        draw_rounds(&mut f, &self.results, self.results.len() as u8 + 1, ROUNDS);
        for (i, &s) in self.spawn.iter().enumerate() {
            if now < s || i < self.results.len() && now > crosses(s) + 700 {
                continue;
            }
            let x = START_X - (now.saturating_sub(s) as i64 * SPEED / 1000) as i32;
            let top = FLOOR - CRATE;
            let (fill, edge) = match self.results.get(i) {
                Some(true) => (Pal::Green, Pal::Forest),
                Some(false) => (Pal::Red, Pal::Maroon),
                None => (Pal::Tan, Pal::Brown),
            };
            f.boxed(x - CRATE / 2, top, CRATE, CRATE, fill, Pal::Ink);
            f.rect(x - CRATE / 2 + 2, top + 2, CRATE - 4, 2, edge);
            f.rect(x - CRATE / 2 + 2, top + CRATE - 4, CRATE - 4, 2, edge);
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

    #[test]
    fn a_jump_is_an_arc_that_peaks_in_the_middle_and_lands() {
        assert_eq!(jump_height(0), 0);
        assert_eq!(jump_height(JUMP_MS / 2), JUMP_H);
        assert_eq!(jump_height(JUMP_MS), 0);
        assert!(jump_height(JUMP_MS / 4) > CLEAR_H / 2 && jump_height(JUMP_MS / 4) < JUMP_H);
    }

    #[test]
    fn jumping_as_the_crate_arrives_clears_it_and_standing_still_does_not() {
        let mut p = pet();
        let mut g = Hop::new(&mut p);
        let first = crosses(g.spawn[0]);
        g.press(first - JUMP_MS / 2);
        g.advance(first);
        assert_eq!(g.results, vec![true], "in the air at the right moment");
        let second = crosses(g.spawn[1]);
        g.advance(second);
        assert_eq!(g.results, vec![true, false], "no jump, no clear");
    }

    #[test]
    fn jumping_far_too_early_or_too_late_misses() {
        let mut p = pet();
        let mut g = Hop::new(&mut p);
        let first = crosses(g.spawn[0]);
        g.press(first - JUMP_MS - 50);
        g.advance(first);
        assert_eq!(g.results, vec![false], "already landed");
        let second = crosses(g.spawn[1]);
        g.press(second);
        g.advance(second);
        assert_eq!(g.results, vec![false, false], "just taking off is not high enough");
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
    fn the_game_ends_after_the_fifth_crate_with_the_cleared_count() {
        let mut p = pet();
        let mut g = Hop::new(&mut p);
        for i in 0..ROUNDS as usize {
            if i % 2 == 0 {
                g.press(crosses(g.spawn[i]) - JUMP_MS / 2);
            }
            let at = crosses(g.spawn[i]);
            assert!(matches!(g.advance(at), Outcome::Continue));
        }
        let last = crosses(*g.spawn.last().unwrap());
        assert!(matches!(g.advance(last + END_MS - 1), Outcome::Continue));
        assert!(matches!(g.advance(last + END_MS), Outcome::Done { wins: 3, rounds: ROUNDS }));
    }

    #[test]
    fn the_crates_are_spaced_so_the_whole_game_is_short() {
        let mut p = pet();
        let g = Hop::new(&mut p);
        assert!(g.spawn.windows(2).all(|w| w[1] - w[0] >= 2300 && w[1] - w[0] <= 3200));
        assert!(crosses(*g.spawn.last().unwrap()) + END_MS < 30_000);
    }
}
