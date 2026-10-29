//! Mini-games. Each one reports how many rounds it won; the pet's happiness
//! reward is decided by `Pet::finish_game`. They draw on a 240x128 part of the scene,
//! in the same palette as everything else.

use std::time::{Duration, Instant};

use crate::app::Key;
use crate::faces::Expr;
use crate::font;
use crate::frame::Frame;
use crate::icons;
use crate::palette::Pal;
use crate::pet::Pet;
use crate::scene;

pub enum Outcome {
    Continue,
    Done { wins: u8, rounds: u8 },
    Quit,
}

pub const CHOICES: [(&str, &str); 2] = [("1", "memory"), ("2", "pong")];

/// The size of the picture a game draws.
pub const GW: i32 = 240;
pub const GH: i32 = 128;

pub enum Game {
    Memory(Memory),
    Pong(Pong),
}

impl Game {
    pub fn new(kind: u8, pet: &mut Pet) -> Game {
        match kind {
            1 => Game::Memory(Memory::new(pet)),
            _ => Game::Pong(Pong::new(pet)),
        }
    }

    pub fn key(&mut self, code: Key) -> Outcome {
        if matches!(code, Key::Esc | Key::Char('q')) {
            return Outcome::Quit;
        }
        match self {
            Game::Memory(g) => g.key(code),
            Game::Pong(_) => Outcome::Continue,
        }
    }

    /// Which direction keys are held down right now.
    pub fn hold(&mut self, up: bool, down: bool) {
        if let Game::Pong(g) = self {
            g.dir = down as i32 - up as i32;
        }
    }

    /// Called every frame for the time-based games.
    pub fn tick(&mut self, pet: &mut Pet) -> Outcome {
        match self {
            Game::Memory(g) => g.tick(pet),
            Game::Pong(g) => g.tick(),
        }
    }

    pub fn draw(&self) -> Frame {
        match self {
            Game::Memory(g) => g.draw(),
            Game::Pong(g) => g.draw(),
        }
    }

    /// What to do, for the line under the game.
    pub fn hint(&self) -> &'static str {
        match self {
            Game::Memory(m) if matches!(m.phase, MPhase::Input(_)) => "YOUR TURN: PRESS 1-4 IN THE SAME ORDER",
            Game::Memory(_) => "WATCH THE PATTERN",
            Game::Pong(_) => "HOLD UP/DOWN (W/S) TO MOVE - RETURN IT 5 TIMES",
        }
    }

    pub fn keys(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            Game::Memory(_) => &[("1-4", "repeat the pattern"), ("esc", "quit game")],
            Game::Pong(_) => &[("↑/↓", "hold to move"), ("esc", "quit game")],
        }
    }
}

// ---------------------------------------------------------------- shared backdrop

/// The pet's wallpaper with a strip of wooden floor, for Memory and the result screen.
fn backdrop() -> Frame {
    let mut f = Frame::new(GW, GH, Pal::Dusk);
    for (row, y) in (8..GH - 16).step_by(12).enumerate() {
        for x in (6 + (row as i32 % 2) * 6..GW).step_by(12) {
            for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
                f.set(x + dx, y + dy, Pal::Night);
            }
        }
    }
    f.rect(0, GH - 14, GW, 14, Pal::Brown);
    f.rect(0, GH - 14, GW, 1, Pal::Tan);
    f.rect(0, GH - 13, GW, 1, Pal::Maroon);
    f
}

/// The pet standing with its feet on row `floor`, centred on `cx`.
fn draw_pet(f: &mut Frame, pet: &Pet, e: Expr, cx: i32, floor: i32) {
    let s = scene::posed(pet, e);
    f.sprite(&s, cx - s.w / 2, floor - s.bottom());
}

fn anim_clock(born: Instant) -> u64 {
    born.elapsed().as_millis() as u64 / 350
}

fn after_ms(ms: u64) -> Instant {
    Instant::now() + Duration::from_millis(ms)
}

// ---------------------------------------------------------------- memory

const MEMORY_ROUNDS: u8 = 3;
const PADS: [Pal; 4] = [Pal::Red, Pal::Yellow, Pal::Green, Pal::Sky];
const PAD_W: i32 = 44;
const PAD_H: i32 = 34;
const PAD_GAP: i32 = 8;
const PAD_Y: i32 = 72;

enum MPhase {
    /// `idx` is the next/current step; `lit` means that step is showing.
    Show { idx: usize, lit: bool, until: Instant },
    Input(usize),
    Pause(Instant),
}

pub struct Memory {
    round: u8,
    wins: u8,
    seq: Vec<u8>,
    phase: MPhase,
    /// Whether each finished round was repeated correctly.
    results: Vec<bool>,
    /// A pad you just pressed glows briefly.
    flash: Option<(u8, Instant)>,
    look: Box<Pet>,
    born: Instant,
}

impl Memory {
    fn new(pet: &mut Pet) -> Self {
        let mut m = Memory {
            round: 1,
            wins: 0,
            seq: Vec::new(),
            phase: MPhase::Pause(after_ms(0)),
            results: Vec::new(),
            flash: None,
            look: Box::new(pet.clone()),
            born: Instant::now(),
        };
        m.start_round(pet);
        m
    }

    fn start_round(&mut self, pet: &mut Pet) {
        let len = self.round as usize + 2;
        self.seq = (0..len).map(|_| pet.rand(4) as u8 + 1).collect();
        self.phase = MPhase::Show { idx: 0, lit: false, until: after_ms(600) };
    }

    fn key(&mut self, code: Key) -> Outcome {
        let (MPhase::Input(pos), Key::Char(c @ '1'..='4')) = (&self.phase, code) else {
            return Outcome::Continue;
        };
        let pos = *pos;
        let pressed = c as u8 - b'0';
        self.flash = Some((pressed, after_ms(220)));
        if pressed == self.seq[pos] {
            if pos + 1 == self.seq.len() {
                self.wins += 1;
                self.results.push(true);
                self.phase = MPhase::Pause(after_ms(1000));
            } else {
                self.phase = MPhase::Input(pos + 1);
            }
        } else {
            self.results.push(false);
            self.phase = MPhase::Pause(after_ms(1500));
        }
        Outcome::Continue
    }

    fn tick(&mut self, pet: &mut Pet) -> Outcome {
        let now = Instant::now();
        match self.phase {
            MPhase::Show { idx, lit, until } if now >= until => {
                self.phase = if !lit {
                    MPhase::Show { idx, lit: true, until: after_ms(500) }
                } else if idx + 1 >= self.seq.len() {
                    MPhase::Input(0)
                } else {
                    MPhase::Show { idx: idx + 1, lit: false, until: after_ms(200) }
                };
            }
            MPhase::Pause(until) if now >= until => {
                if self.round >= MEMORY_ROUNDS {
                    return Outcome::Done { wins: self.wins, rounds: MEMORY_ROUNDS };
                }
                self.round += 1;
                self.start_round(pet);
            }
            _ => {}
        }
        Outcome::Continue
    }

    fn lit(&self) -> Option<u8> {
        match self.phase {
            MPhase::Show { idx, lit: true, .. } => Some(self.seq[idx]),
            _ => self.flash.filter(|(_, until)| Instant::now() < *until).map(|f| f.0),
        }
    }

    fn draw_pad(f: &mut Frame, n: u8, on: bool) {
        let base = PADS[n as usize - 1];
        let x = 20 + (n as i32 - 1) * (PAD_W + PAD_GAP);
        if on {
            // A glow just outside the lit pad.
            f.rect(x - 3, PAD_Y - 3, PAD_W + 6, PAD_H + 6, base.highlight());
        }
        // Unlit pads are a darker version of their own colour; yellow's shadow is orange,
        // which would read as a different pad, so it dims to rust instead.
        let dim = if base == Pal::Yellow { Pal::Rust } else { base.shadow() };
        let (face, top, foot) = if on { (base, base.highlight(), base.shadow()) } else { (dim, base, dim.shadow()) };
        f.boxed(x, PAD_Y, PAD_W, PAD_H, face, Pal::Ink);
        f.rect(x + 1, PAD_Y + 1, PAD_W - 2, 3, top);
        f.rect(x + 1, PAD_Y + PAD_H - 4, PAD_W - 2, 3, foot);
        let digit = ((b'0' + n) as char).to_string();
        let w = font::width(&digit) * 2;
        f.text(&digit, x + (PAD_W - w) / 2, PAD_Y + 10, 2, if on { Pal::Ink } else { base });
    }

    fn draw(&self) -> Frame {
        let t = anim_clock(self.born);
        let mut f = backdrop();
        let pausing = matches!(self.phase, MPhase::Pause(_));
        let mood = match self.results.last() {
            Some(true) if pausing => Expr::Happy,
            Some(false) if pausing => Expr::Sad,
            _ => Expr::Idle,
        };
        // The pet watches from behind the pads and bobs while it waits for you.
        let bob = if matches!(self.phase, MPhase::Input(_)) { (t % 2) as i32 } else { 0 };
        draw_pet(&mut f, &self.look, mood, GW / 2, PAD_Y - 4 - bob);
        let lit = self.lit();
        for n in 1..=4u8 {
            Memory::draw_pad(&mut f, n, lit == Some(n));
        }
        // Rounds, top left: green for a win, red for a miss, white for the one being played.
        for r in 0..MEMORY_ROUNDS as usize {
            let col = match self.results.get(r) {
                Some(true) => Pal::Green,
                Some(false) => Pal::Red,
                None if r + 1 == self.round as usize => Pal::White,
                None => Pal::Slate,
            };
            f.boxed(6 + r as i32 * 10, 6, 8, 8, col, Pal::Ink);
        }
        // Your progress through the pattern, as pips on the floor once it is your turn.
        if let MPhase::Input(pos) = self.phase {
            let n = self.seq.len() as i32;
            let x0 = (GW - (n * 10 - 4)) / 2;
            for i in 0..n {
                f.boxed(x0 + i * 10, GH - 9, 6, 4, if i < pos as i32 { Pal::Yellow } else { Pal::Mist }, Pal::Ink);
            }
        }
        f
    }
}

// ---------------------------------------------------------------- pong

/// Return the ball this many times to win outright.
const PONG_GOAL: u8 = 5;
const BALL: f32 = 4.0;
/// The court: the ball's top edge stays between these rows.
const COURT_TOP: f32 = 9.0;
const COURT_BOTTOM: f32 = GH as f32 - 9.0 - BALL;
const PADDLE_X: f32 = 8.0;
const PADDLE_W: f32 = 4.0;
const PADDLE_H: f32 = 24.0;
/// How far past either end of the paddle the ball still counts as touching it.
const SLACK: f32 = 2.0;
/// The ball bounces off the pet keeper here.
const KEEPER_X: f32 = GW as f32 - 44.0;
const PADDLE_SPEED: f32 = 160.0;
const BALL_SPEED: f32 = 95.0;
/// Each return makes the ball this much faster, up to `MAX_SPEEDUP` times the start.
const SPEEDUP: f32 = 1.087;
const MAX_SPEEDUP: f32 = 2.0;
const STEP: f32 = 1.0 / 120.0;

pub struct Pong {
    /// Top of the paddle.
    py: f32,
    /// -1 up, 1 down, 0 still: set every frame from the keys held.
    dir: i32,
    /// Top-left corner of the ball, and its velocity in pixels per second.
    bx: f32,
    by: f32,
    vx: f32,
    vy: f32,
    trail: Vec<(f32, f32)>,
    /// Top of the pet keeper's sprite.
    ky: f32,
    keeper_h: f32,
    look: Box<Pet>,
    spark: Option<(f32, f32, Instant)>,
    /// The ball got past the paddle; it rolls off the left edge before the game ends.
    missed: bool,
    hits: u8,
    speed: f32,
    last: Instant,
    left: f32,
}

impl Pong {
    fn new(pet: &mut Pet) -> Self {
        let by = COURT_TOP + pet.rand((COURT_BOTTOM - COURT_TOP) as u64) as f32;
        let down = pet.rand(2) == 0;
        Pong {
            py: (GH as f32 - PADDLE_H) / 2.0,
            dir: 0,
            bx: KEEPER_X - 20.0,
            by,
            vx: -BALL_SPEED,
            vy: if down { BALL_SPEED } else { -BALL_SPEED },
            trail: Vec::new(),
            ky: by,
            keeper_h: scene::posed(pet, Expr::Idle).h as f32,
            look: Box::new(pet.clone()),
            spark: None,
            missed: false,
            hits: 0,
            speed: 1.0,
            last: Instant::now(),
            left: 0.0,
        }
    }

    /// Advance by real time, in fixed small steps so the result does not depend on frame rate.
    fn tick(&mut self) -> Outcome {
        self.left += self.last.elapsed().as_secs_f32().min(0.1);
        self.last = Instant::now();
        while self.left >= STEP {
            self.left -= STEP;
            if let done @ Outcome::Done { .. } = self.step(STEP) {
                return done;
            }
        }
        Outcome::Continue
    }

    fn step(&mut self, dt: f32) -> Outcome {
        let lowest = COURT_BOTTOM + BALL + 2.0 - PADDLE_H;
        self.py = (self.py + self.dir as f32 * PADDLE_SPEED * dt).clamp(COURT_TOP - 2.0, lowest);
        if self.trail.first().is_none_or(|&(x, y)| (x - self.bx).abs() + (y - self.by).abs() >= 4.0) {
            self.trail.insert(0, (self.bx, self.by));
            self.trail.truncate(3);
        }
        self.by += self.vy * dt;
        if self.by < COURT_TOP || self.by > COURT_BOTTOM {
            self.vy = -self.vy;
            self.by = self.by.clamp(COURT_TOP, COURT_BOTTOM);
        }
        let was = self.bx;
        self.bx += self.vx * dt;
        if self.missed {
            return if self.bx < -BALL - 2.0 { Outcome::Done { wins: self.hits, rounds: PONG_GOAL } } else { Outcome::Continue };
        }
        let centre = self.by + BALL / 2.0 - self.keeper_h / 2.0;
        if self.bx + BALL >= KEEPER_X && self.vx > 0.0 {
            // The pet always reaches the ball, so it is shown right behind it when it bounces.
            self.bx = KEEPER_X - BALL;
            self.vx = -self.vx;
            self.ky = centre;
            self.spark = Some((KEEPER_X, self.by, after_ms(150)));
        }
        let face = PADDLE_X + PADDLE_W;
        if was > face && self.bx <= face && self.vx < 0.0 {
            if self.by + BALL + SLACK >= self.py && self.by <= self.py + PADDLE_H + SLACK {
                self.hits += 1;
                if self.hits >= PONG_GOAL {
                    return Outcome::Done { wins: PONG_GOAL, rounds: PONG_GOAL };
                }
                self.bx = face;
                self.speed = (self.speed * SPEEDUP).min(MAX_SPEEDUP);
                self.vx = BALL_SPEED * self.speed;
                self.vy = self.vy.signum() * BALL_SPEED * self.speed;
                self.spark = Some((face, self.by, after_ms(150)));
            } else {
                self.missed = true;
            }
        }
        // The pet follows the ball between bounces.
        self.ky += (centre - self.ky).clamp(-90.0 * dt, 90.0 * dt);
        Outcome::Continue
    }

    fn draw(&self) -> Frame {
        let mut f = Frame::new(GW, GH, Pal::Night);
        // A striped green court between two rails.
        for (i, y) in (7..GH - 7).step_by(8).enumerate() {
            f.rect(0, y, GW, 8.min(GH - 7 - y), if i % 2 == 0 { Pal::DeepTeal } else { Pal::Forest });
        }
        f.rect(0, 5, GW, 2, Pal::Slate);
        f.rect(0, GH - 7, GW, 2, Pal::Slate);
        for y in (9..GH - 9).step_by(8) {
            f.rect(GW / 2 - 1, y, 2, 4, Pal::Teal);
        }
        // Returns so far, as gold pips in the top rail.
        let x0 = (GW - (PONG_GOAL as i32 * 12 - 4)) / 2;
        for i in 0..PONG_GOAL as i32 {
            f.boxed(x0 + i * 12, 0, 8, 5, if i < self.hits as i32 { Pal::Yellow } else { Pal::Dusk }, Pal::Ink);
        }
        // Player paddle, lit on its left side.
        let (px, py) = (PADDLE_X as i32, self.py.round() as i32);
        f.boxed(px - 1, py - 1, PADDLE_W as i32 + 2, PADDLE_H as i32 + 2, Pal::Pink, Pal::Ink);
        f.rect(px, py, 1, PADDLE_H as i32, Pal::Blush);
        f.rect(px + PADDLE_W as i32 - 1, py, 1, PADDLE_H as i32, Pal::Violet);
        // The pet keeps goal, cheering for a moment after each bounce.
        let cheering = self.spark.is_some_and(|(x, _, until)| x >= KEEPER_X && Instant::now() < until);
        let keeper = scene::posed(&self.look, if cheering { Expr::Happy } else { Expr::Idle });
        f.sprite(&keeper, KEEPER_X as i32 + 2, (self.ky.round() as i32).clamp(4, GH - 4 - keeper.h));
        // Ball with a fading trail.
        for (i, &(x, y)) in self.trail.iter().enumerate().rev() {
            f.rect(x.round() as i32, y.round() as i32, BALL as i32, BALL as i32, if i == 0 { Pal::Orange } else { Pal::Rust });
        }
        let (bx, by) = (self.bx.round() as i32, self.by.round() as i32);
        f.boxed(bx, by, BALL as i32, BALL as i32, Pal::Yellow, Pal::Orange);
        f.set(bx + 1, by + 1, Pal::Cream);
        if let Some((x, y, until)) = self.spark {
            if Instant::now() < until {
                f.grid(icons::SPARKLE, x as i32 - 2, y as i32 - 1);
            }
        }
        f
    }
}

// ---------------------------------------------------------------- result screen

/// How long the result stays up, and how soon a key may skip it.
const FINISH_SECS: u64 = 3;
const FINISH_SKIP_MS: u64 = 500;
const CONFETTI: [Pal; 5] = [Pal::Pink, Pal::Yellow, Pal::Green, Pal::Sky, Pal::White];

/// Shown for a few seconds after a game ends: a star per round won, the coins earned,
/// and the pet cheering (with confetti) or sulking.
pub struct Finish {
    wins: u8,
    rounds: u8,
    coins: u32,
    look: Box<Pet>,
    born: Instant,
}

impl Finish {
    pub fn new(pet: &Pet, wins: u8, rounds: u8, coins: u32) -> Self {
        Finish { wins, rounds, coins, look: Box::new(pet.clone()), born: Instant::now() }
    }

    pub fn expired(&self) -> bool {
        self.born.elapsed() >= Duration::from_secs(FINISH_SECS)
    }

    pub fn skippable(&self) -> bool {
        self.born.elapsed() >= Duration::from_millis(FINISH_SKIP_MS)
    }

    pub fn draw(&self) -> Frame {
        let won = self.coins > 0;
        let ms = self.born.elapsed().as_millis() as i32;
        let mut f = backdrop();
        if won {
            for i in 0..40i32 {
                let x = (i * 53 + 11) % GW;
                let y = (ms / 12 + i * 37) % (GH + 8) - 6;
                f.rect(x, y, 2, 2 + i % 2, CONFETTI[i as usize % CONFETTI.len()]);
            }
        }
        // Stars along the top: gold for each round won, grey for the rest.
        let n = self.rounds as i32;
        let x0 = (GW - (n * 26 - 4)) / 2;
        for i in 0..n {
            let won_round = i < self.wins as i32;
            f.grid_scaled(icons::STAR, x0 + i * 26, 8, 2, |p| if won_round || p == Pal::Ink { p } else { Pal::Slate });
        }
        // The pet hops when it won and hangs its head when it lost.
        let hop = if won { [0, -4, -7, -4, 0, 0][(ms / 110) as usize % 6] } else { 0 };
        let (mood, cx) = if won { (Expr::Happy, 84) } else { (Expr::Sad, GW / 2) };
        draw_pet(&mut f, &self.look, mood, cx, GH - 12 + hop);
        if won {
            f.grid_scaled(icons::COIN, 132, 64, 3, |p| p);
            f.text(&format!("+{}", self.coins), 156, 64, 2, Pal::White);
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
    fn escape_quits_every_game() {
        for kind in 1..=2 {
            let mut p = pet();
            let mut g = Game::new(kind, &mut p);
            assert!(matches!(g.key(Key::Esc), Outcome::Quit), "game {kind}");
        }
    }

    #[test]
    fn memory_repeating_the_pattern_wins_the_round_and_a_wrong_key_loses_it() {
        let mut p = pet();
        let mut m = Memory::new(&mut p);
        m.phase = MPhase::Input(0);
        for n in m.seq.clone() {
            m.key(Key::Char((b'0' + n) as char));
        }
        assert_eq!(m.wins, 1);
        assert_eq!(m.results, [true]);
        assert!(m.lit().is_some(), "the last pad pressed glows");

        let mut m = Memory::new(&mut p);
        m.phase = MPhase::Input(0);
        let wrong = (m.seq[0] % 4) + 1;
        m.key(Key::Char((b'0' + wrong) as char));
        assert_eq!(m.results, [false]);
        assert_eq!(m.wins, 0);
    }

    #[test]
    fn memory_ignores_keys_while_the_pattern_is_showing() {
        let mut p = pet();
        let mut m = Memory::new(&mut p);
        m.key(Key::Char('1'));
        assert_eq!(m.wins, 0);
        assert!(matches!(m.phase, MPhase::Show { .. }));
    }

    #[test]
    fn memory_patterns_are_three_four_then_five_long() {
        let mut p = pet();
        let mut m = Memory::new(&mut p);
        let mut lengths = vec![m.seq.len()];
        for _ in 0..2 {
            m.phase = MPhase::Pause(Instant::now());
            m.tick(&mut p);
            lengths.push(m.seq.len());
        }
        assert_eq!(lengths, [3, 4, 5]);
    }

    fn run(pong: &mut Pong, secs: f32) -> Option<(u8, u8)> {
        for _ in 0..(secs / STEP) as usize {
            if let Outcome::Done { wins, rounds } = pong.step(STEP) {
                return Some((wins, rounds));
            }
        }
        None
    }

    #[test]
    fn pong_always_ends_and_never_reports_more_than_the_goal() {
        let mut p = pet();
        let mut pong = Pong::new(&mut p);
        let (wins, rounds) = run(&mut pong, 120.0).expect("pong never ended");
        assert_eq!(rounds, PONG_GOAL);
        assert!(wins <= PONG_GOAL);
    }

    #[test]
    fn pong_a_ball_touching_the_paddle_is_returned_and_one_beside_it_rolls_past() {
        let mut p = pet();
        let mut g = Pong::new(&mut p);
        g.py = 50.0;
        (g.bx, g.by, g.vx, g.vy) = (PADDLE_X + PADDLE_W + 1.0, 50.0 - BALL - 1.0, -BALL_SPEED, 0.0);
        run(&mut g, 0.1);
        assert_eq!(g.hits, 1, "a ball just grazing the top of the paddle is returned");
        assert!(g.vx > 0.0);

        let mut g = Pong::new(&mut p);
        g.py = 80.0;
        (g.bx, g.by, g.vx, g.vy) = (PADDLE_X + PADDLE_W + 1.0, 40.0, -BALL_SPEED, 0.0);
        assert_eq!(run(&mut g, 0.05), None, "a miss is not over until the ball is off screen");
        assert!(g.missed);
        assert_eq!(run(&mut g, 2.0), Some((0, PONG_GOAL)));
    }

    #[test]
    fn pong_returns_get_faster_up_to_twice_the_start() {
        let mut speed = 1.0f32;
        for _ in 0..20 {
            speed = (speed * SPEEDUP).min(MAX_SPEEDUP);
        }
        assert_eq!(speed, MAX_SPEEDUP);
        assert!(SPEEDUP.powi(9) >= MAX_SPEEDUP, "like the old game, it tops out after about nine returns");
    }

    #[test]
    fn pong_paddle_moves_only_while_a_key_is_held_and_stays_in_the_court() {
        let mut p = pet();
        let mut g = Game::new(2, &mut p);
        g.hold(true, false);
        let Game::Pong(pong) = &mut g else { panic!("game 2 is pong") };
        (pong.vx, pong.vy) = (0.0, 0.0);
        let start = pong.py;
        run(pong, 0.1);
        assert!(pong.py < start, "holding up moves the paddle up");
        run(pong, 5.0);
        assert_eq!(pong.py, COURT_TOP - 2.0, "it stops at the top rail");
        g.hold(false, false);
        let Game::Pong(pong) = &mut g else { unreachable!() };
        let at = pong.py;
        run(pong, 0.5);
        assert_eq!(pong.py, at, "letting go stops it");
        g.hold(false, true);
        let Game::Pong(pong) = &mut g else { unreachable!() };
        run(pong, 5.0);
        assert_eq!(pong.py + PADDLE_H, COURT_BOTTOM + BALL + 2.0, "and at the bottom rail");
    }

    #[test]
    fn the_result_screen_stays_a_moment_then_goes() {
        let mut f = Finish::new(&pet(), 5, 5, 10);
        assert!(!f.skippable() && !f.expired());
        assert_eq!((f.draw().w, f.draw().h), (GW, GH));
        f.born = Instant::now() - Duration::from_secs(FINISH_SECS);
        assert!(f.skippable() && f.expired());
    }

    #[test]
    fn the_picker_offers_two_games_and_each_one_draws() {
        assert_eq!(CHOICES.len(), 2);
        for kind in 1..=2 {
            let mut p = pet();
            let g = Game::new(kind, &mut p);
            let f = g.draw();
            assert_eq!((f.w, f.h), (GW, GH), "game {kind}");
            assert!(!g.keys().is_empty() && !g.hint().is_empty());
        }
    }
}
