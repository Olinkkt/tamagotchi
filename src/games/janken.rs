//! Rock Paper Scissors against the pet: pick a hand, both shake three times, then both show.
//! Win two of three rounds; a tie is played again.

use std::time::Instant;

use super::*;

pub const ROUNDS: u8 = 3;
const BTN_W: i32 = 56;
const BTN_H: i32 = 30;
const BTN_Y: i32 = 82;
const BTN_X: [i32; 3] = [32, 92, 152];
/// Where the two hands are drawn: the player's on the left, the pet's on the right.
const HAND_Y: i32 = 14;
const HAND_K: i32 = 3;
const PLAYER_X: i32 = 30;
const PET_X: i32 = 165;

const ROCK: icons::Grid = &[
    "...............",
    "...............",
    "....kkkkkkk....",
    "...kccccccck...",
    "..kcckckckcck..",
    "..kcckckckttk..",
    ".kccckckcktttk.",
    ".kccckckcktttk.",
    ".kcccccccctttk.",
    ".kckkkkccctttk.",
    ".kcccccccctttk.",
    ".kcccccccctttk.",
    "..ktttttttttk..",
    "...kkkkkkkkk...",
    "...............",
];

pub(super) const PAPER: icons::Grid = &[
    "......kkkcck...",
    ".....kcckcckkk.",
    "...kkkcckcckcck",
    "..kcckcckcckcck",
    "..kcckcckcckcck",
    "..kcckcckcckcck",
    "..kcckcckcckcck",
    ".kkccccccccccck",
    "kcccccccccccttk",
    "kcccccccccccttk",
    "kcccccccccccttk",
    "kcccccccccccttk",
    ".kkcccccccccttk",
    "..ktttttttttttk",
    "...kkkkkkkkkkk.",
];

const SCISSORS: icons::Grid = &[
    ".kccck.kccck...",
    ".kccck.kccck...",
    ".kccck.kccck...",
    ".kccck.kccck...",
    "..kccckccck....",
    "..kccckccck....",
    "..kccckccck....",
    "..kccckccckkkk.",
    ".kkccccccccccck",
    "kccccccccccttck",
    "kcccckkkkkkktck",
    "kccccccccccttk.",
    ".kkccccccccttk.",
    "..kttttttttttk.",
    "...kkkkkkkkkk..",
];

const HANDS: [icons::Grid; 3] = [ROCK, PAPER, SCISSORS];

enum Phase {
    Choose,
    /// Both hands bob, then show: your hand, the pet's, and when the shaking ends.
    Shake(usize, usize, Instant),
    /// What the result of the round was (`None` for a tie), until when it shows.
    Reveal(Option<bool>, Instant),
}

pub struct Janken {
    round: u8,
    wins: u8,
    results: Vec<bool>,
    phase: Phase,
    mine: usize,
    theirs: usize,
    look: Box<Pet>,
}

/// Whether hand `a` beats hand `b` (0 rock, 1 paper, 2 scissors).
fn beats(a: usize, b: usize) -> bool {
    (b + 1) % 3 == a
}

impl Janken {
    pub fn new(pet: &mut Pet) -> Self {
        Janken { round: 1, wins: 0, results: Vec::new(), phase: Phase::Choose, mine: 0, theirs: 0, look: Box::new(pet.clone()) }
    }

    /// Pick a hand with 1-3 or R, P, S.
    pub fn key(&mut self, code: Key, pet: &mut Pet) -> Outcome {
        let pick = match code {
            Key::Char('1' | 'r') => 0,
            Key::Char('2' | 'p') => 1,
            Key::Char('3' | 's') => 2,
            _ => return Outcome::Continue,
        };
        if matches!(self.phase, Phase::Choose) {
            self.mine = pick;
            self.theirs = pet.rand(3) as usize;
            self.phase = Phase::Shake(pick, self.theirs, after_ms(1000));
        }
        Outcome::Continue
    }

    pub fn tick(&mut self) -> Outcome {
        let now = Instant::now();
        match self.phase {
            Phase::Shake(mine, theirs, until) if now >= until => {
                let result = if mine == theirs { None } else { Some(beats(mine, theirs)) };
                if let Some(won) = result {
                    self.results.push(won);
                    if won {
                        self.wins += 1;
                    }
                }
                self.phase = Phase::Reveal(result, after_ms(1500));
            }
            Phase::Reveal(result, until) if now >= until => {
                if result.is_some() {
                    if self.round >= ROUNDS {
                        return Outcome::Done { wins: self.wins, rounds: ROUNDS };
                    }
                    self.round += 1;
                }
                self.phase = Phase::Choose;
            }
            _ => {}
        }
        Outcome::Continue
    }

    /// The hand (0 to 2) whose button covers this point of the picture.
    pub fn button_at(&self, x: i32, y: i32) -> Option<usize> {
        if !matches!(self.phase, Phase::Choose) || !(BTN_Y..BTN_Y + BTN_H).contains(&y) {
            return None;
        }
        (0..3).find(|&i| (BTN_X[i]..BTN_X[i] + BTN_W).contains(&x))
    }

    pub fn hover_rect(&self, x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {
        self.button_at(x, y).map(|i| (BTN_X[i], BTN_Y, BTN_W, BTN_H))
    }

    pub fn hint(&self) -> &'static str {
        match self.phase {
            Phase::Choose => "ROCK, PAPER OR SCISSORS? CLICK OR PRESS 1-3 (R/P/S)",
            Phase::Shake(..) => "ROCK... PAPER... SCISSORS!",
            Phase::Reveal(Some(true), _) => "YOU WIN THE ROUND!",
            Phase::Reveal(Some(false), _) => "I WIN THE ROUND!",
            Phase::Reveal(None, _) => "A TIE - AGAIN!",
        }
    }

    pub fn draw(&self) -> Frame {
        let mut f = backdrop();
        // The hands shake three times (up and down) before they show.
        let (mine, theirs, bob, mood) = match self.phase {
            Phase::Choose => (None, None, 0, Expr::Idle),
            Phase::Shake(_, _, until) => {
                let left = until.saturating_duration_since(Instant::now()).as_millis() as f32;
                let phase = (1000.0 - left).max(0.0) / 1000.0 * 3.0 * std::f32::consts::PI * 2.0;
                (None, None, (phase.sin() * 5.0).round() as i32, Expr::Idle)
            }
            Phase::Reveal(result, _) => {
                let mood = match result {
                    Some(true) => Expr::Sad,
                    Some(false) => Expr::Happy,
                    None => Expr::Idle,
                };
                (Some(self.mine), Some(self.theirs), 0, mood)
            }
        };
        let shown = |hand: Option<usize>| HANDS[hand.unwrap_or(0)];
        let tint = |p: Pal| p;
        let faded = |p: Pal| if p == Pal::Ink { p } else { Pal::Slate };
        let waiting = matches!(self.phase, Phase::Choose);
        draw_hand(&mut f, shown(mine), PLAYER_X, HAND_Y - bob.abs(), false, &tint);
        draw_hand(&mut f, shown(theirs), PET_X, HAND_Y - bob.abs(), true, &tint);
        draw_pet(&mut f, &self.look, mood, GW / 2, HAND_Y + 15 * HAND_K - 2);
        draw_rounds(&mut f, &self.results, self.round);
        // The three choices, while it is your turn.
        for i in 0..3 {
            let on = waiting;
            let x = BTN_X[i];
            f.boxed(x, BTN_Y, BTN_W, BTN_H, if on { Pal::Dusk } else { Pal::Night }, if on { Pal::Slate } else { Pal::Dusk });
            f.grid_scaled(HANDS[i], x + (BTN_W - 15) / 2 + 8, BTN_Y + 8, 1, |p| if on { p } else { faded(p) });
            let digit = ((b'1' + i as u8) as char).to_string();
            f.text(&digit, x + 6, BTN_Y + 6, 2, if on { Pal::White } else { Pal::Slate });
        }
        f
    }
}

/// A 15x15 hand blown up `HAND_K` times; the pet's hand is mirrored so it faces you.
fn draw_hand(f: &mut Frame, rows: icons::Grid, x: i32, y: i32, flip: bool, tint: &dyn Fn(Pal) -> Pal) {
    for (dx, dy, p) in crate::faces::pixels(rows) {
        let dx = if flip { 14 - dx } else { dx };
        f.rect(x + dx * HAND_K, y + dy * HAND_K, HAND_K, HAND_K, tint(p));
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
    fn rock_beats_scissors_scissors_beat_paper_and_paper_beats_rock() {
        assert!(beats(0, 2) && beats(2, 1) && beats(1, 0));
        assert!(!beats(2, 0) && !beats(1, 2) && !beats(0, 1));
        assert!(!beats(1, 1), "a tie beats nobody");
    }

    #[test]
    fn hands_are_square_and_use_palette_letters() {
        for g in HANDS {
            assert_eq!(g.len(), 15);
            assert!(g.iter().all(|r| r.len() == 15 && r.chars().all(|c| c == '.' || crate::faces::colour(c).is_some())));
        }
    }

    #[test]
    fn a_pick_shakes_then_scores_the_round_and_a_tie_is_played_again() {
        let mut p = pet();
        let mut g = Janken::new(&mut p);
        g.key(Key::Char('x'), &mut p);
        assert!(matches!(g.phase, Phase::Choose), "other keys do nothing");
        g.key(Key::Char('r'), &mut p);
        g.theirs = 2;
        g.phase = Phase::Shake(0, 2, Instant::now());
        g.tick();
        assert_eq!((g.wins, g.results.clone()), (1, vec![true]), "rock beats scissors");
        g.phase = Phase::Reveal(Some(true), Instant::now());
        g.tick();
        assert_eq!(g.round, 2);

        g.phase = Phase::Shake(1, 1, Instant::now());
        g.tick();
        assert_eq!(g.results.len(), 1, "a tie scores nothing");
        g.phase = Phase::Reveal(None, Instant::now());
        g.tick();
        assert_eq!((g.round, matches!(g.phase, Phase::Choose)), (2, true), "the same round again");
    }

    #[test]
    fn the_game_ends_after_the_third_scored_round() {
        let mut p = pet();
        let mut g = Janken::new(&mut p);
        g.round = ROUNDS;
        g.wins = 1;
        g.results = vec![true, false];
        g.phase = Phase::Reveal(Some(false), Instant::now());
        assert!(matches!(g.tick(), Outcome::Done { wins: 1, rounds: ROUNDS }));
    }

    #[test]
    fn buttons_only_answer_to_clicks_on_your_turn() {
        let mut p = pet();
        let mut g = Janken::new(&mut p);
        assert_eq!(g.button_at(BTN_X[1] + 3, BTN_Y + 3), Some(1));
        assert_eq!(g.button_at(2, 2), None);
        g.phase = Phase::Shake(0, 0, Instant::now());
        assert_eq!(g.button_at(BTN_X[1] + 3, BTN_Y + 3), None);
    }
}
