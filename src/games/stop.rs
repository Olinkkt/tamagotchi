//! Stop: a marker slides back and forth along a track. Press space (or click) to stop it, and
//! win the round if it stops inside the green. Five rounds; the green shrinks and the marker
//! speeds up each time. One button, about 20 seconds in all.

use super::*;

pub const ROUNDS: u8 = 5;
const TRACK_X: i32 = 20;
const TRACK_W: i32 = 200;
const TRACK_Y: i32 = 82;
const TRACK_H: i32 = 14;
const MARK_W: i32 = 4;
/// The width of the green zone and the marker's speed (pixels a second), by round.
const ZONE_W: [i32; 5] = [40, 32, 26, 22, 18];
const SPEED: [i64; 5] = [110, 135, 160, 185, 200];
/// A marker left running this long counts as a miss, so a game never hangs.
const TIMEOUT_MS: u64 = 7000;
const RESULT_MS: u64 = 1100;

enum Phase {
    /// Sliding since this many milliseconds on the game's clock.
    Slide(u64),
    /// Stopped at this track position (`None` if it timed out), hit or miss, until then.
    Stopped(Option<i32>, bool, u64),
}

pub struct Stop {
    round: u8,
    wins: u8,
    results: Vec<bool>,
    phase: Phase,
    /// Left edge of the green zone, on the track.
    zone_x: i32,
    look: Box<Pet>,
    t0: Instant,
}

/// Where the marker's left edge is, `elapsed` milliseconds after it set off: it bounces
/// between the two ends of the track.
fn marker_x(elapsed: u64, speed: i64) -> i32 {
    let span = (TRACK_W - MARK_W) as i64;
    let p = (elapsed as i64 * speed / 1000) % (2 * span);
    (if p > span { 2 * span - p } else { p }) as i32
}

impl Stop {
    pub fn new(pet: &mut Pet) -> Self {
        let mut g = Stop { round: 1, wins: 0, results: Vec::new(), phase: Phase::Slide(0), zone_x: 0, look: Box::new(pet.clone()), t0: Instant::now() };
        g.zone_x = g.pick_zone(pet);
        g
    }

    fn pick_zone(&self, pet: &mut Pet) -> i32 {
        let w = ZONE_W[self.round as usize - 1];
        pet.rand((TRACK_W - w) as u64) as i32
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

    /// Stop the marker where it is `now`.
    fn press(&mut self, now: u64) {
        let Phase::Slide(since) = self.phase else { return };
        let x = marker_x(now - since, SPEED[self.round as usize - 1]);
        let w = ZONE_W[self.round as usize - 1];
        let middle = x + MARK_W / 2;
        let hit = (self.zone_x..self.zone_x + w).contains(&middle);
        self.score(hit);
        self.phase = Phase::Stopped(Some(x), hit, now + RESULT_MS);
    }

    fn score(&mut self, hit: bool) {
        self.results.push(hit);
        self.wins += hit as u8;
    }

    pub fn tick(&mut self, pet: &mut Pet) -> Outcome {
        self.advance(self.now(), pet)
    }

    fn advance(&mut self, now: u64, pet: &mut Pet) -> Outcome {
        match self.phase {
            Phase::Slide(since) if now - since >= TIMEOUT_MS => {
                self.score(false);
                self.phase = Phase::Stopped(None, false, now + RESULT_MS);
            }
            Phase::Stopped(_, _, until) if now >= until => {
                if self.round >= ROUNDS {
                    return Outcome::Done { wins: self.wins, rounds: ROUNDS };
                }
                self.round += 1;
                self.zone_x = self.pick_zone(pet);
                self.phase = Phase::Slide(now);
            }
            _ => {}
        }
        Outcome::Continue
    }

    pub fn hint(&self) -> &'static str {
        match self.phase {
            Phase::Slide(_) => "PRESS SPACE (OR CLICK) TO STOP THE MARKER IN THE GREEN",
            Phase::Stopped(_, true, _) => "RIGHT IN THE GREEN!",
            Phase::Stopped(None, _, _) => "TOO SLOW - NEXT ONE",
            Phase::Stopped(_, false, _) => "MISSED - NEXT ONE",
        }
    }

    pub fn draw(&self) -> Frame {
        let mut f = backdrop();
        let round = self.round as usize - 1;
        let (x, mood, hit) = match self.phase {
            Phase::Slide(since) => (marker_x(self.now() - since, SPEED[round]), Expr::Idle, None),
            Phase::Stopped(x, hit, _) => (x.unwrap_or(0), if hit { Expr::Happy } else { Expr::Sad }, Some(hit)),
        };
        draw_pet(&mut f, &self.look, mood, GW / 2, 66);
        draw_rounds(&mut f, &self.results, self.round, ROUNDS);
        f.boxed(TRACK_X - 2, TRACK_Y - 2, TRACK_W + 4, TRACK_H + 4, Pal::Night, Pal::Ink);
        let zone = ZONE_W[round];
        f.rect(TRACK_X + self.zone_x, TRACK_Y, zone, TRACK_H, Pal::Green);
        f.rect(TRACK_X + self.zone_x, TRACK_Y, zone, 2, Pal::Lime);
        let colour = match hit {
            Some(true) => Pal::White,
            Some(false) => Pal::Red,
            None => Pal::Yellow,
        };
        f.boxed(TRACK_X + x - 1, TRACK_Y - 5, MARK_W + 2, TRACK_H + 10, colour, Pal::Ink);
        f
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pet() -> Pet {
        let mut p = Pet::new(99);
        p.stage = crate::pet::Stage::Baby;
        p
    }

    #[test]
    fn the_marker_bounces_between_the_ends_of_the_track() {
        let end = (TRACK_W - MARK_W) as i64;
        assert_eq!(marker_x(0, 100), 0);
        assert_eq!(marker_x(500, 100), 50);
        assert_eq!(marker_x((end * 10) as u64, 100), end as i32, "at the far end after the track's length");
        assert_eq!(marker_x((end * 10) as u64 + 500, 100), end as i32 - 50, "then back again");
        assert!((0..40_000).all(|ms| (0..=TRACK_W - MARK_W).contains(&marker_x(ms, 170))));
    }

    #[test]
    fn stopping_inside_the_green_wins_and_outside_loses() {
        let mut p = pet();
        let mut g = Stop::new(&mut p);
        // Park the green right where the marker is after half a second.
        let x = marker_x(500, SPEED[0]);
        g.zone_x = x - 10;
        g.press(500);
        assert_eq!((g.wins, g.results.clone()), (1, vec![true]));
        g.press(900);
        assert_eq!(g.results.len(), 1, "a second press in the same round does nothing");

        g.advance(500 + RESULT_MS, &mut p);
        assert_eq!(g.round, 2);
        g.zone_x = 0;
        g.press(500 + RESULT_MS + 1500);
        assert_eq!(g.results, vec![true, false], "the marker is far from the green");
    }

    #[test]
    fn a_marker_left_running_is_a_miss_and_five_rounds_end_the_game() {
        let mut p = pet();
        let mut g = Stop::new(&mut p);
        let mut now = 0;
        loop {
            now += TIMEOUT_MS;
            g.advance(now, &mut p);
            now += RESULT_MS;
            if let Outcome::Done { wins, rounds } = g.advance(now, &mut p) {
                assert_eq!((wins, rounds), (0, ROUNDS));
                break;
            }
            assert!(g.round <= ROUNDS);
        }
        assert_eq!(g.results, vec![false; ROUNDS as usize]);
    }

    #[test]
    fn the_green_always_fits_on_the_track_and_shrinks() {
        let mut p = pet();
        let mut g = Stop::new(&mut p);
        for round in 1..=ROUNDS {
            g.round = round;
            for _ in 0..200 {
                let z = g.pick_zone(&mut p);
                assert!(z >= 0 && z + ZONE_W[round as usize - 1] <= TRACK_W);
            }
        }
        assert!(ZONE_W.windows(2).all(|w| w[0] > w[1]) && SPEED.windows(2).all(|w| w[0] < w[1]));
    }
}
