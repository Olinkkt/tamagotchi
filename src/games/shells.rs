//! Shell game: the pet hides under one of three cups, the cups swap places, and you pick the
//! cup it is under. Three rounds, with more and faster swaps each time.

use std::time::Instant;

use super::*;

pub const ROUNDS: u8 = 3;
/// Where the three cups stand (their middles), left to right.
const SLOT_X: [i32; 3] = [60, 120, 180];
const CUP_TOP: i32 = 62;
const CUP_H: i32 = 50;
const CUP_TOP_W: i32 = 34;
const CUP_FOOT_W: i32 = 50;
/// How high a lifted cup rises, enough to clear the pet.
const LIFT: i32 = 52;

enum Phase {
    /// The pet shows itself under its cup before the shuffle.
    Peek(Instant),
    /// Swapping the cups in slots `.0` and `.1`; `i` counts the swaps done.
    Swap { i: usize, start: Instant },
    Pick,
    /// The slot you picked, until when the answer shows.
    Reveal(usize, Instant),
}

pub struct Shells {
    round: u8,
    wins: u8,
    results: Vec<bool>,
    phase: Phase,
    /// The slot each cup (0 to 2) stands in.
    cup_slot: [usize; 3],
    /// The cup the pet hides under.
    hides: usize,
    swaps: Vec<(usize, usize)>,
    swap_ms: u128,
    look: Box<Pet>,
    born: Instant,
}

impl Shells {
    pub fn new(pet: &mut Pet) -> Self {
        let mut g = Shells {
            round: 1,
            wins: 0,
            results: Vec::new(),
            phase: Phase::Pick,
            cup_slot: [0, 1, 2],
            hides: 0,
            swaps: Vec::new(),
            swap_ms: 400,
            look: Box::new(pet.clone()),
            born: Instant::now(),
        };
        // The pet keeps its cup from round to round, so it never jumps between them.
        g.hides = pet.rand(3) as usize;
        g.start_round(pet);
        g
    }

    fn start_round(&mut self, pet: &mut Pet) {
        let count = 3 + 2 * self.round as usize;
        self.swap_ms = [420, 320, 240][self.round as usize - 1];
        self.swaps = (0..count)
            .map(|_| {
                let a = pet.rand(3) as usize;
                (a, (a + 1 + pet.rand(2) as usize) % 3)
            })
            .collect();
        self.phase = Phase::Peek(after_ms(1100));
    }

    fn cup_in(&self, slot: usize) -> usize {
        (0..3).find(|&c| self.cup_slot[c] == slot).unwrap_or(0)
    }

    pub fn key(&mut self, code: Key) -> Outcome {
        let (Phase::Pick, Key::Char(c @ '1'..='3')) = (&self.phase, code) else {
            return Outcome::Continue;
        };
        let slot = (c as u8 - b'1') as usize;
        let right = self.cup_in(slot) == self.hides;
        self.results.push(right);
        if right {
            self.wins += 1;
        }
        self.phase = Phase::Reveal(slot, after_ms(1500));
        Outcome::Continue
    }

    pub fn tick(&mut self, pet: &mut Pet) -> Outcome {
        let now = Instant::now();
        match self.phase {
            Phase::Peek(until) if now >= until => self.phase = Phase::Swap { i: 0, start: now },
            Phase::Swap { i, start } if start.elapsed().as_millis() >= self.swap_ms => {
                let (a, b) = self.swaps[i];
                let (ca, cb) = (self.cup_in(a), self.cup_in(b));
                self.cup_slot.swap(ca, cb);
                self.phase = if i + 1 >= self.swaps.len() { Phase::Pick } else { Phase::Swap { i: i + 1, start: now } };
            }
            Phase::Reveal(_, until) if now >= until => {
                if self.round >= ROUNDS {
                    return Outcome::Done { wins: self.wins, rounds: ROUNDS };
                }
                self.round += 1;
                self.start_round(pet);
            }
            _ => {}
        }
        Outcome::Continue
    }

    /// The slot (0 to 2) of the cup under this point of the picture, once it is your turn.
    pub fn slot_at(&self, x: i32, y: i32) -> Option<usize> {
        if !matches!(self.phase, Phase::Pick) || !(CUP_TOP..CUP_TOP + CUP_H).contains(&y) {
            return None;
        }
        (0..3).find(|&s| (SLOT_X[s] - CUP_FOOT_W / 2..SLOT_X[s] + CUP_FOOT_W / 2).contains(&x))
    }

    pub fn hover_rect(&self, x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {
        self.slot_at(x, y).map(|s| (SLOT_X[s] - CUP_FOOT_W / 2, CUP_TOP, CUP_FOOT_W, CUP_H))
    }

    pub fn hint(&self) -> &'static str {
        match self.phase {
            Phase::Peek(_) => "REMEMBER WHICH CUP I AM UNDER",
            Phase::Swap { .. } => "KEEP YOUR EYE ON ME",
            Phase::Pick => "WHICH CUP? CLICK IT OR PRESS 1-3",
            Phase::Reveal(..) if self.results.last() == Some(&true) => "FOUND ME!",
            Phase::Reveal(..) => "NOT THERE - I WAS HERE",
        }
    }

    /// Middle x of a cup, and how far it is lifted off the table (negative: up).
    fn cup_pose(&self, cup: usize) -> (i32, i32) {
        if let Phase::Swap { i, start } = self.phase {
            let t = (start.elapsed().as_millis() as f32 / self.swap_ms as f32).min(1.0);
            let (a, b) = self.swaps[i];
            let (from, to) = if self.cup_slot[cup] == a {
                (a, b)
            } else if self.cup_slot[cup] == b {
                (b, a)
            } else {
                return (SLOT_X[self.cup_slot[cup]], 0);
            };
            let x = SLOT_X[from] as f32 + (SLOT_X[to] - SLOT_X[from]) as f32 * t;
            // The cup that goes left-to-right hops over the other one.
            let hop = if from < to { -(t * std::f32::consts::PI).sin() * 10.0 } else { 0.0 };
            return (x.round() as i32, hop.round() as i32);
        }
        (SLOT_X[self.cup_slot[cup]], 0)
    }

    fn lifted(&self, cup: usize) -> bool {
        match self.phase {
            Phase::Peek(_) => cup == self.hides,
            Phase::Reveal(slot, _) => cup == self.hides || cup == self.cup_in(slot),
            _ => false,
        }
    }

    pub fn draw(&self) -> Frame {
        let t = anim_clock(self.born);
        let mut f = backdrop();
        let mood = match (&self.phase, self.results.last()) {
            (Phase::Reveal(..), Some(true)) => Expr::Happy,
            (Phase::Reveal(..), Some(false)) => Expr::Sad,
            _ => Expr::Idle,
        };
        // A cup you have not picked yet bobs a little when it is your turn.
        let wait = if matches!(self.phase, Phase::Pick) { (t % 2) as i32 } else { 0 };
        // The pet moves with its cup, hops and bobs included, so its feet never show.
        let (hx, hhop) = self.cup_pose(self.hides);
        draw_pet(&mut f, &self.look, mood, hx, CUP_TOP + CUP_H + hhop - wait);
        for cup in 0..3 {
            let (x, hop) = self.cup_pose(cup);
            let lift = if self.lifted(cup) { LIFT } else { 0 };
            draw_cup(&mut f, x, CUP_TOP + hop - lift - wait);
        }
        draw_rounds(&mut f, &self.results, self.round);
        f
    }
}

/// An upside-down cup, `x` is its middle and `y` its top.
pub fn draw_cup(f: &mut Frame, x: i32, y: i32) {
    // Every cup looks the same, so the only way to follow the pet is to watch the cups.
    let (body, light, dark) = (Pal::Orange, Pal::Yellow, Pal::Rust);
    for r in 0..CUP_H {
        let w = CUP_TOP_W + (CUP_FOOT_W - CUP_TOP_W) * r / (CUP_H - 1);
        f.rect(x - w / 2 - 1, y + r, w + 2, 1, Pal::Ink);
        let rim = r >= CUP_H - 6;
        f.rect(x - w / 2, y + r, w, 1, if rim { dark } else { body });
        if !rim && r > 1 {
            f.rect(x - w / 2, y + r, 3, 1, light);
            f.rect(x + w / 2 - 4, y + r, 4, 1, dark);
        }
    }
    f.rect(x - CUP_TOP_W / 2 - 1, y, CUP_TOP_W + 2, 2, Pal::Ink);
    f.rect(x - CUP_FOOT_W / 2 - 1, y + CUP_H - 1, CUP_FOOT_W + 2, 1, Pal::Ink);
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
    fn picking_the_cup_the_pet_is_under_wins_the_round_and_any_other_loses_it() {
        let mut p = pet();
        let mut g = Shells::new(&mut p);
        g.key(Key::Char('1'));
        assert!(g.results.is_empty(), "keys do nothing while the cups are moving");
        let under = g.cup_slot[g.hides];
        g.phase = Phase::Pick;
        g.key(Key::Char((b'1' + under as u8) as char));
        assert_eq!((g.wins, g.results.clone()), (1, vec![true]));

        let mut g = Shells::new(&mut p);
        let under = g.cup_slot[g.hides];
        g.phase = Phase::Pick;
        g.key(Key::Char((b'1' + ((under + 1) % 3) as u8) as char));
        assert_eq!((g.wins, g.results.clone()), (0, vec![false]));
    }

    #[test]
    fn swaps_keep_the_cups_in_three_different_slots_and_the_game_ends_after_three_rounds() {
        let mut p = pet();
        let mut g = Shells::new(&mut p);
        for round in 1..=ROUNDS {
            assert_eq!(g.swaps.len(), 3 + 2 * round as usize);
            g.phase = Phase::Peek(Instant::now());
            let mut guard = 0;
            while !matches!(g.phase, Phase::Pick) {
                g.swap_ms = 0;
                g.tick(&mut p);
                guard += 1;
                assert!(guard < 100);
            }
            let mut slots = g.cup_slot;
            slots.sort();
            assert_eq!(slots, [0, 1, 2]);
            g.key(Key::Char('1'));
            g.phase = Phase::Reveal(0, Instant::now());
            let out = g.tick(&mut p);
            assert!(matches!(out, Outcome::Done { .. }) == (round == ROUNDS));
        }
    }

    #[test]
    fn the_pet_stays_under_the_same_cup_when_the_next_round_starts() {
        let mut p = pet();
        let mut g = Shells::new(&mut p);
        let cup = g.hides;
        g.phase = Phase::Reveal(0, Instant::now());
        g.tick(&mut p);
        assert_eq!((g.round, g.hides), (2, cup));
    }
}
