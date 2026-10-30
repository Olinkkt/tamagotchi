//! Slots: three reels spin; stop them one at a time, left to right, and get a pair or better
//! to win the round. Three rounds, with the reels faster each time.

use std::time::Instant;

use super::*;

pub const ROUNDS: u8 = 3;
const SYMBOLS: [icons::Grid; 5] = [icons::STAR, icons::COIN, icons::HEART, icons::APPLE, icons::CANDY];
const REEL_W: i32 = 44;
const REEL_H: i32 = 44;
const REEL_GAP: i32 = 12;
const REEL_Y: i32 = 20;
/// How long each reel stays on one symbol, in ms, before round-speed scaling.
const REEL_MS: [u128; 3] = [120, 90, 150];

enum Phase {
    Spin,
    /// Until when the result shows.
    Reveal(Instant),
}

pub struct Slots {
    round: u8,
    wins: u8,
    results: Vec<bool>,
    phase: Phase,
    /// The symbol (0 to 4) each reel shows, and whether it has been stopped.
    shown: [usize; 3],
    stopped: [bool; 3],
    /// Last time each reel moved on.
    moved: [Instant; 3],
    jackpot: bool,
    look: Box<Pet>,
    born: Instant,
}

impl Slots {
    pub fn new(pet: &mut Pet) -> Self {
        let mut g = Slots {
            round: 1,
            wins: 0,
            results: Vec::new(),
            phase: Phase::Spin,
            shown: [0; 3],
            stopped: [false; 3],
            moved: [Instant::now(); 3],
            jackpot: false,
            look: Box::new(pet.clone()),
            born: Instant::now(),
        };
        g.start_round(pet);
        g
    }

    fn start_round(&mut self, pet: &mut Pet) {
        self.shown = [0; 3].map(|_| pet.rand(SYMBOLS.len() as u64) as usize);
        self.stopped = [false; 3];
        self.moved = [Instant::now(); 3];
        self.jackpot = false;
        self.phase = Phase::Spin;
    }

    fn reel_ms(&self, reel: usize) -> u128 {
        REEL_MS[reel] * (10 - self.round as u128 + 1) / 10
    }

    /// Stop the next reel from the left.
    pub fn key(&mut self, code: Key) -> Outcome {
        if !matches!((&self.phase, code), (Phase::Spin, Key::Char(' '))) {
            return Outcome::Continue;
        }
        if let Some(reel) = self.stopped.iter().position(|&s| !s) {
            self.stopped[reel] = true;
        }
        if self.stopped.iter().all(|&s| s) {
            let [a, b, c] = self.shown;
            let right = a == b || b == c || a == c;
            self.jackpot = a == b && b == c;
            self.results.push(right);
            if right {
                self.wins += 1;
            }
            self.phase = Phase::Reveal(after_ms(if self.jackpot { 2000 } else { 1500 }));
        }
        Outcome::Continue
    }

    pub fn tick(&mut self, pet: &mut Pet) -> Outcome {
        let now = Instant::now();
        match self.phase {
            Phase::Spin => {
                for reel in 0..3 {
                    if !self.stopped[reel] && self.moved[reel].elapsed().as_millis() >= self.reel_ms(reel) {
                        self.shown[reel] = (self.shown[reel] + 1) % SYMBOLS.len();
                        self.moved[reel] = now;
                    }
                }
            }
            Phase::Reveal(until) if now >= until => {
                if self.round >= ROUNDS {
                    return Outcome::Done { wins: self.wins, rounds: ROUNDS };
                }
                self.round += 1;
                self.start_round(pet);
            }
            Phase::Reveal(_) => {}
        }
        Outcome::Continue
    }

    pub fn hint(&self) -> &'static str {
        match self.phase {
            Phase::Spin => "PRESS SPACE OR CLICK TO STOP EACH REEL - MATCH TWO",
            Phase::Reveal(_) if self.jackpot => "JACKPOT!",
            Phase::Reveal(_) if self.results.last() == Some(&true) => "A MATCH!",
            Phase::Reveal(_) => "NO MATCH",
        }
    }

    fn reel_x(reel: i32) -> i32 {
        (GW - (3 * REEL_W + 2 * REEL_GAP)) / 2 + reel * (REEL_W + REEL_GAP)
    }

    pub fn draw(&self) -> Frame {
        let t = anim_clock(self.born);
        let mut f = backdrop();
        let revealing = matches!(self.phase, Phase::Reveal(_));
        let (mood, win) = match (revealing, self.results.last()) {
            (true, Some(true)) => (Expr::Happy, true),
            (true, Some(false)) => (Expr::Sad, false),
            _ => (Expr::Idle, false),
        };
        // The machine: a dark frame behind the reels.
        let left = Slots::reel_x(0) - 6;
        f.boxed(left, REEL_Y - 6, 3 * REEL_W + 2 * REEL_GAP + 12, REEL_H + 12, Pal::Maroon, Pal::Ink);
        f.rect(left + 1, REEL_Y - 5, 3 * REEL_W + 2 * REEL_GAP + 10, 1, Pal::Red);
        for reel in 0..3usize {
            let x = Slots::reel_x(reel as i32);
            let spinning = matches!(self.phase, Phase::Spin) && !self.stopped[reel];
            // In a pair the matching reels glow gold.
            let part_of_pair = revealing && win && self.shown.iter().filter(|&&s| s == self.shown[reel]).count() >= 2;
            let border = if part_of_pair { Pal::Yellow } else if self.stopped[reel] || revealing { Pal::White } else { Pal::Slate };
            f.boxed(x, REEL_Y, REEL_W, REEL_H, if spinning { Pal::Night } else { Pal::Cream }, border);
            let g = SYMBOLS[self.shown[reel]];
            let (gw, gh) = (g[0].len() as i32 * 3, g.len() as i32 * 3);
            let shake = if spinning { [-1, 1][((t + reel as u64) % 2) as usize] } else { 0 };
            f.grid_scaled(g, x + (REEL_W - gw) / 2, REEL_Y + (REEL_H - gh) / 2 + shake, 3, |p| if spinning && p != Pal::Ink { Pal::Mist } else { p });
        }
        // The pet watches from the floor, hopping on a win.
        let hop = if win { [0, -4, -7, -4, 0, 0][(t % 6) as usize] } else { 0 };
        draw_pet(&mut f, &self.look, mood, GW / 2, GH - 14 + hop);
        draw_rounds(&mut f, &self.results, self.round);
        // Reels stopped so far, as pips on the floor.
        if matches!(self.phase, Phase::Spin) {
            for i in 0..3i32 {
                f.boxed(GW / 2 - 16 + i * 12, GH - 9, 8, 4, if self.stopped[i as usize] { Pal::Yellow } else { Pal::Mist }, Pal::Ink);
            }
        }
        f
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pet() -> Pet {
        let mut p = Pet::new(12345);
        p.stage = crate::pet::Stage::Baby;
        p
    }

    #[test]
    fn each_press_stops_the_next_reel_and_the_third_scores_the_round() {
        let mut p = pet();
        let mut g = Slots::new(&mut p);
        g.shown = [1, 1, 3];
        g.key(Key::Char('x'));
        assert_eq!(g.stopped, [false; 3], "only space stops a reel");
        g.key(Key::Char(' '));
        g.key(Key::Char(' '));
        assert_eq!(g.stopped, [true, true, false]);
        assert!(matches!(g.phase, Phase::Spin));
        g.key(Key::Char(' '));
        assert_eq!((g.wins, g.results.clone(), g.jackpot), (1, vec![true], false));
        assert!(matches!(g.phase, Phase::Reveal(_)));
    }

    #[test]
    fn three_different_symbols_lose_and_three_alike_is_a_jackpot() {
        let mut p = pet();
        let mut g = Slots::new(&mut p);
        g.shown = [0, 1, 2];
        (0..3).for_each(|_| _ = g.key(Key::Char(' ')));
        assert_eq!((g.wins, g.results.clone()), (0, vec![false]));

        let mut g = Slots::new(&mut p);
        g.shown = [4, 4, 4];
        (0..3).for_each(|_| _ = g.key(Key::Char(' ')));
        assert_eq!((g.wins, g.jackpot), (1, true));
    }

    #[test]
    fn reels_move_on_only_while_spinning_and_the_game_ends_after_three_rounds() {
        let mut p = pet();
        let mut g = Slots::new(&mut p);
        g.moved = [Instant::now() - std::time::Duration::from_secs(1); 3];
        let before = g.shown;
        g.stopped[1] = true;
        g.tick(&mut p);
        assert_eq!(g.shown[0], (before[0] + 1) % SYMBOLS.len());
        assert_eq!(g.shown[1], before[1], "a stopped reel stays put");
        for round in 1..=ROUNDS {
            (0..3).for_each(|_| _ = g.key(Key::Char(' ')));
            g.phase = Phase::Reveal(Instant::now());
            let out = g.tick(&mut p);
            assert!(matches!(out, Outcome::Done { .. }) == (round == ROUNDS));
        }
    }
}
