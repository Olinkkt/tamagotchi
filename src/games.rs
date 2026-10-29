//! Mini-games. Each one reports how many rounds it won; the pet's happiness
//! reward is decided by `Pet::finish_game`.

use std::time::{Duration, Instant};

use crate::app::Key;
use ratatui::text::Line;

use crate::pet::Pet;
use crate::sprites::{body_colour, darken, mini_pet, mix, to_lines, Canvas, Mood, C, INK, WHITE};

pub enum Outcome {
    Continue,
    Done { wins: u8, rounds: u8 },
    Quit,
}

pub const CHOICES: [(&str, &str); 2] = [("1", "memory"), ("2", "pong")];

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
            Game::Pong(g) => g.key(code),
        }
    }

    /// Called every frame for the time-based games.
    pub fn tick(&mut self, pet: &mut Pet) -> Outcome {
        match self {
            Game::Memory(g) => g.tick(pet),
            Game::Pong(g) => g.tick(),
        }
    }

    /// The game drawn into a `w` x `h` cell area.
    pub fn lines(&self, w: usize, h: usize) -> Vec<Line<'static>> {
        match self {
            Game::Memory(g) => g.lines(w, h),
            Game::Pong(g) => g.lines(w, h),
        }
    }

    pub fn keys(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            Game::Memory(_) => &[("1-4", "repeat the pattern"), ("esc", "quit game")],
            Game::Pong(_) => &[("↑/↓", "steer"), ("space", "stop"), ("esc", "quit game")],
        }
    }
}

// ---------------------------------------------------------------- shared pixel stage

/// Every game draws onto a canvas of this many pixels (two per terminal row), which fits the
/// smallest room and is scaled up in bigger windows. Scores are drawn in pixels too.
const FW: usize = 32;
const FH: usize = 20;
const OUTSIDE: C = (20, 22, 34);

/// The room's wall gradient with a strip of wooden floor, like the pet's own room.
fn room_backdrop() -> Canvas {
    let mut c = Canvas::sized(FW, FH);
    for y in 0..FH as i32 {
        let col = if y >= FH as i32 - 1 { (122, 88, 66) } else { mix((34, 40, 74), (74, 58, 92), y as f32 / FH as f32) };
        c.rect(0, y, FW as i32 - 1, y, col);
    }
    c
}

const GOLD: C = (250, 205, 80);
const PIP_OFF: C = (70, 72, 92);

/// A 3x5 pixel digit.
fn digit(c: &mut Canvas, n: u8, x: i32, y: i32, col: C) {
    let rows: [&str; 5] = match n {
        0 => ["###", "#.#", "#.#", "#.#", "###"],
        1 => [".#.", "##.", ".#.", ".#.", "###"],
        2 => ["##.", "..#", ".#.", "#..", "###"],
        3 => ["##.", "..#", ".#.", "..#", "##."],
        4 => ["#.#", "#.#", "###", "..#", "..#"],
        5 => ["###", "#..", "##.", "..#", "##."],
        6 => [".##", "#..", "###", "#.#", "###"],
        7 => ["###", "..#", ".#.", ".#.", ".#."],
        8 => ["###", "#.#", "###", "#.#", "###"],
        _ => ["###", "#.#", "###", "..#", "##."],
    };
    sprite(c, &rows, x, y, col);
}

fn sprite(c: &mut Canvas, rows: &[&str], x: i32, y: i32, col: C) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, ch) in row.chars().enumerate() {
            if ch == '#' {
                c.set(x + dx as i32, y + dy as i32, col);
            }
        }
    }
}

fn anim_clock(born: Instant) -> u32 {
    (born.elapsed().as_millis() / 350) as u32
}

// ---------------------------------------------------------------- memory

const MEMORY_ROUNDS: u8 = 3;
const MEMORY_COLORS: [C; 4] = [(240, 95, 105), (245, 200, 90), (130, 210, 140), (110, 160, 250)];
/// Left edge of each 7-pixel pad.
const PAD_X: [i32; 4] = [0, 8, 16, 24];
const PAD_Y: i32 = 11;

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

fn after_ms(ms: u64) -> Instant {
    Instant::now() + Duration::from_millis(ms)
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

    fn lines(&self, w: usize, h: usize) -> Vec<Line<'static>> {
        let t = anim_clock(self.born);
        let mut c = room_backdrop();
        let pausing = matches!(self.phase, MPhase::Pause(_));
        let mood = match self.results.last() {
            Some(true) if pausing => Mood::Happy,
            Some(false) if pausing => Mood::Sad,
            _ => Mood::Neutral,
        };
        // The pet watches from behind the pads and bobs while it waits for you.
        let bob = if matches!(self.phase, MPhase::Input(_)) { (t % 2) as i32 } else { 0 };
        c.blit(&mini_pet(&self.look, t, mood), 8, bob + 1, None);
        let lit = self.lit();
        for n in 1..=4u8 {
            let base = MEMORY_COLORS[n as usize - 1];
            let x = PAD_X[n as usize - 1];
            let on = lit == Some(n);
            let face = if on { base } else { darken(base, 0.55) };
            if on {
                // A soft glow around the lit pad.
                c.rect(x - 1, PAD_Y - 1, x + 7, PAD_Y + 7, mix(base, (34, 40, 74), 0.55));
            }
            c.rect(x, PAD_Y, x + 6, PAD_Y + 6, face);
            c.rect(x, PAD_Y, x + 6, PAD_Y, if on { mix(base, WHITE, 0.55) } else { darken(base, 0.35) });
            c.rect(x, PAD_Y + 6, x + 6, PAD_Y + 6, darken(base, if on { 0.25 } else { 0.7 }));
            digit(&mut c, n, x + 2, PAD_Y + 1, if on { INK } else { darken(base, 0.15) });
        }
        // Rounds, top left: green for a win, red for a miss, white for the one being played.
        for r in 0..MEMORY_ROUNDS as usize {
            let col = match self.results.get(r) {
                Some(true) => (130, 210, 140),
                Some(false) => (240, 95, 105),
                None if r + 1 == self.round as usize => WHITE,
                None => PIP_OFF,
            };
            c.rect(1 + r as i32 * 3, 1, 2 + r as i32 * 3, 2, col);
        }
        // Your progress through the pattern, as dots on the floor once it is your turn.
        if let MPhase::Input(pos) = self.phase {
            let n = self.seq.len() as i32;
            let x0 = (FW as i32 - (n * 3 - 1)) / 2;
            for i in 0..n {
                c.rect(x0 + i * 3, FH as i32 - 1, x0 + i * 3 + 1, FH as i32 - 1, if i < pos as i32 { GOLD } else { (200, 200, 215) });
            }
        }
        to_lines(&c, w, h, OUTSIDE)
    }
}

// ---------------------------------------------------------------- pong

const PADDLE: i32 = 5;
const PY_MIN: i32 = 1;
const PY_MAX: i32 = FH as i32 - 1 - PADDLE;
/// The pet guards the right side, so the ball always comes back.
const KEEPER: i32 = 7;
const BALL_MAX_Y: i32 = FH as i32 - 3;
/// Return the ball this many times to win outright.
const PONG_GOAL: u8 = 5;
const PADDLE_MOVE_MS: u64 = 40;

/// Press a direction once and it keeps going until you press the other way or space, so the
/// game does not depend on the terminal's key repeat.
fn steer(code: Key, neg: &[Key], pos: &[Key], dir: &mut i32) {
    if neg.contains(&code) {
        *dir = -1;
    } else if pos.contains(&code) {
        *dir = 1;
    } else if code == Key::Char(' ') {
        *dir = 0;
    }
}

const UP: [Key; 3] = [Key::Up, Key::Char('w'), Key::Char('k')];
const DOWN: [Key; 3] = [Key::Down, Key::Char('s'), Key::Char('j')];

pub struct Pong {
    py: i32,
    dir: i32,
    moved: Instant,
    /// Top-left pixel of the 2x2 ball.
    bx: i32,
    by: i32,
    dx: i32,
    dy: i32,
    trail: Vec<(i32, i32)>,
    /// Top of the pet keeper.
    ky: i32,
    keeper: C,
    spark: Option<(i32, i32, Instant)>,
    /// The ball got past the paddle; it rolls off the left edge before the game ends.
    missed: bool,
    hits: u8,
    last: Instant,
    step_ms: u64,
}

impl Pong {
    fn new(pet: &mut Pet) -> Self {
        let by = 2 + pet.rand((BALL_MAX_Y - 2) as u64) as i32;
        Pong {
            py: (PY_MIN + PY_MAX) / 2,
            dir: 0,
            moved: Instant::now(),
            bx: 24,
            by,
            dx: -1,
            dy: if pet.rand(2) == 0 { 1 } else { -1 },
            trail: Vec::new(),
            ky: (by - 2).clamp(1, FH as i32 - 1 - KEEPER),
            keeper: body_colour(pet),
            spark: None,
            missed: false,
            hits: 0,
            last: Instant::now(),
            step_ms: 80,
        }
    }

    fn key(&mut self, code: Key) -> Outcome {
        steer(code, &UP, &DOWN, &mut self.dir);
        Outcome::Continue
    }

    fn nudge(&mut self) {
        self.py = (self.py + self.dir).clamp(PY_MIN, PY_MAX);
    }

    fn tick(&mut self) -> Outcome {
        if self.moved.elapsed() >= Duration::from_millis(PADDLE_MOVE_MS) {
            self.moved = Instant::now();
            self.nudge();
        }
        if self.last.elapsed() < Duration::from_millis(self.step_ms) {
            return Outcome::Continue;
        }
        self.last = Instant::now();
        self.trail.insert(0, (self.bx, self.by));
        self.trail.truncate(3);
        let mut ny = self.by + self.dy;
        if !(1..=BALL_MAX_Y).contains(&ny) {
            self.dy = -self.dy;
            ny = self.by + self.dy;
        }
        let mut nx = self.bx + self.dx;
        if self.missed {
            self.bx = nx;
            self.by = ny;
            return if nx < -2 { Outcome::Done { wins: self.hits, rounds: PONG_GOAL } } else { Outcome::Continue };
        }
        let keeper_top = (ny - 2).clamp(1, FH as i32 - 1 - KEEPER);
        if nx >= 26 {
            self.dx = -1;
            nx = 25;
            // The pet always reaches the ball, so it is shown right behind it when it bounces.
            self.ky = keeper_top;
            self.spark = Some((27, ny, after_ms(150)));
        }
        if nx <= 2 && self.dx < 0 {
            // The 2-pixel ball counts as a hit when it touches the paddle, plus one pixel of slack.
            if ny + 2 >= self.py && ny <= self.py + PADDLE {
                self.hits += 1;
                if self.hits >= PONG_GOAL {
                    return Outcome::Done { wins: PONG_GOAL, rounds: PONG_GOAL };
                }
                self.dx = 1;
                nx = 3;
                self.spark = Some((3, ny, after_ms(150)));
                self.step_ms = (self.step_ms * 92 / 100).max(40);
            } else {
                self.missed = true;
            }
        }
        // The pet follows the ball, one pixel a step.
        self.ky += (keeper_top - self.ky).signum();
        self.bx = nx;
        self.by = ny;
        Outcome::Continue
    }

    fn lines(&self, w: usize, h: usize) -> Vec<Line<'static>> {
        let (fw, fh) = (FW as i32, FH as i32);
        let mut c = Canvas::sized(FW, FH);
        for y in 0..fh {
            c.rect(0, y, fw - 1, y, mix((26, 62, 56), (16, 40, 42), y as f32 / fh as f32));
        }
        let rail = (92, 100, 136);
        c.rect(0, 0, fw - 1, 0, rail);
        c.rect(0, fh - 1, fw - 1, fh - 1, rail);
        for y in (1..fh - 1).filter(|y| y % 4 < 2) {
            c.set(fw / 2, y, (60, 104, 94));
        }
        // Player paddle, with a highlight down one side.
        let pink = (255, 160, 190);
        c.rect(1, self.py, 2, self.py + PADDLE - 1, pink);
        c.rect(1, self.py, 1, self.py + PADDLE - 1, mix(pink, WHITE, 0.45));
        // The pet keeper: a little body in the pet's colour with eyes that watch the ball.
        let k = self.keeper;
        c.rect(27, self.ky + 1, 30, self.ky + KEEPER - 2, k);
        c.rect(28, self.ky, 29, self.ky + KEEPER - 1, k);
        let look = if self.by < self.ky + 2 { 0 } else if self.by > self.ky + 3 { 2 } else { 1 };
        c.set(28, self.ky + 1 + look, INK);
        c.set(30, self.ky + 1 + look, INK);
        c.set(29, self.ky + KEEPER - 2, darken(k, 0.5));
        // Ball with a fading trail.
        for (i, &(x, y)) in self.trail.iter().enumerate().rev() {
            let col = mix((245, 200, 90), (16, 40, 42), 0.45 + i as f32 * 0.18);
            c.rect(x, y, x + 1, y + 1, col);
        }
        c.rect(self.bx, self.by, self.bx + 1, self.by + 1, (245, 200, 90));
        c.set(self.bx, self.by, (255, 240, 190));
        if let Some((x, y, until)) = self.spark {
            if Instant::now() < until {
                c.pixels(&[(x, y - 1), (x, y + 2), (x - 1, y), (x + 1, y + 1)], (255, 245, 170));
            }
        }
        // Returns so far, as gold pips in the top rail.
        for i in 0..PONG_GOAL as i32 {
            c.rect(3 + i * 3, 0, 4 + i * 3, 0, if i < self.hits as i32 { GOLD } else { PIP_OFF });
        }
        to_lines(&c, w, h, OUTSIDE)
    }
}

// ---------------------------------------------------------------- result screen

const STAR: [&str; 5] = ["..#..", ".###.", "#####", ".###.", ".#.#."];
/// How long the result stays up, and how soon a key may skip it.
const FINISH_SECS: u64 = 3;
const FINISH_SKIP_MS: u64 = 500;

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

    pub fn lines(&self, w: usize, h: usize) -> Vec<Line<'static>> {
        let won = self.coins > 0;
        let ms = self.born.elapsed().as_millis() as i32;
        let mut c = room_backdrop();
        if won {
            let colours = [(255, 110, 150), GOLD, (130, 210, 140), (110, 160, 250), WHITE];
            for i in 0..16i32 {
                let x = (i * 13 + 5) % FW as i32;
                let y = (ms / 90 + i * 7) % (FH as i32 + 4) - 3;
                c.rect(x, y, x, y + (i % 2), colours[i as usize % colours.len()]);
            }
        }
        // Stars, centred along the top.
        let n = self.rounds as i32;
        let x0 = (FW as i32 - (n * 6 - 1)) / 2;
        for i in 0..n {
            sprite(&mut c, &STAR, x0 + i * 6, 1, if i < self.wins as i32 { GOLD } else { PIP_OFF });
        }
        // The pet hops when it won and hangs its head when it lost.
        let hop = if won { [0, -2, -3, -2, 0, 0][(ms / 110) as usize % 6] } else { 1 };
        let mood = if won { Mood::Happy } else { Mood::Sad };
        let px = if won { 2 } else { 8 };
        c.blit(&mini_pet(&self.look, (ms / 350) as u32, mood), px, 8 + hop, None);
        if won {
            c.ellipse(18.5, 13.5, 2.6, 2.6, GOLD);
            c.ellipse(18.5, 13.5, 1.4, 1.4, darken(GOLD, 0.25));
            let text = self.coins.to_string();
            for (k, ch) in text.chars().enumerate() {
                digit(&mut c, ch as u8 - b'0', 23 + k as i32 * 4, 11, WHITE);
            }
        }
        to_lines(&c, w, h, OUTSIDE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pet() -> Pet {
        Pet::new(12345)
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
    fn pong_always_ends_and_never_reports_more_than_the_goal() {
        let mut p = pet();
        let mut pong = Pong::new(&mut p);
        for _ in 0..5000 {
            pong.last = Instant::now() - Duration::from_millis(500);
            if let Outcome::Done { wins, rounds } = pong.tick() {
                assert_eq!(rounds, PONG_GOAL);
                assert!(wins <= PONG_GOAL);
                return;
            }
        }
        panic!("pong never ended");
    }

    #[test]
    fn pong_a_ball_touching_the_paddle_is_returned_and_one_beside_it_rolls_past() {
        let mut p = pet();
        let mut g = Pong::new(&mut p);
        g.py = 6;
        (g.bx, g.by, g.dx, g.dy) = (3, 4, -1, 1);
        g.last = Instant::now() - Duration::from_millis(500);
        g.tick();
        assert_eq!((g.hits, g.dx), (1, 1), "ball rows 5-6 touch the paddle at 6-10");

        let mut g = Pong::new(&mut p);
        g.py = 10;
        (g.bx, g.by, g.dx, g.dy) = (3, 4, -1, -1);
        g.last = Instant::now() - Duration::from_millis(500);
        assert!(matches!(g.tick(), Outcome::Continue), "a miss is not over until the ball is off screen");
        assert!(g.missed);
        for _ in 0..10 {
            g.last = Instant::now() - Duration::from_millis(500);
            if let Outcome::Done { wins: 0, .. } = g.tick() {
                return;
            }
        }
        panic!("the missed ball never left the court");
    }

    #[test]
    fn pong_paddle_keeps_moving_until_stopped_and_stays_in_the_court() {
        let mut p = pet();
        let mut pong = Pong::new(&mut p);
        pong.key(Key::Up);
        for _ in 0..30 {
            pong.nudge();
        }
        assert_eq!(pong.py, PY_MIN);
        pong.key(Key::Down);
        for _ in 0..30 {
            pong.nudge();
        }
        assert_eq!(pong.py, PY_MAX);
        pong.key(Key::Up);
        pong.nudge();
        pong.key(Key::Char(' '));
        let at = pong.py;
        pong.nudge();
        assert_eq!(pong.py, at, "space stops the paddle");
    }

    #[test]
    fn the_result_screen_stays_a_moment_then_goes() {
        let mut f = Finish::new(&pet(), 5, 5, 10);
        assert!(!f.skippable() && !f.expired());
        assert_eq!(f.lines(32, 10).len(), 10);
        f.born = Instant::now() - Duration::from_secs(FINISH_SECS);
        assert!(f.skippable() && f.expired());
    }

    #[test]
    fn every_game_fits_the_smallest_room() {
        // The room is 10 rows tall in the smallest supported window (64x22).
        for kind in 1..=2 {
            let mut p = pet();
            let g = Game::new(kind, &mut p);
            let lines = g.lines(32, 10);
            assert_eq!(lines.len(), 10, "game {kind}");
            assert!(lines.iter().all(|l| l.width() == 32), "game {kind} spills sideways");
        }
    }

    #[test]
    fn the_picker_offers_two_games_and_each_one_starts() {
        assert_eq!(CHOICES.len(), 2);
        for kind in 1..=2 {
            let mut p = pet();
            let g = Game::new(kind, &mut p);
            assert!(!g.lines(32, 10).is_empty() && !g.keys().is_empty(), "game {kind}");
        }
    }
}
