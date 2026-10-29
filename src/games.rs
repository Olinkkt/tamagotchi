//! Mini-games. Each one reports how many rounds it won; the pet's happiness
//! reward is decided by `Pet::finish_game`.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::cli::face;
use crate::pet::Pet;
use crate::ui::{ACCENT, BAD, DIM, EMPTY, GOOD, TEXT, WARN};

pub enum Outcome {
    Continue,
    Done { wins: u8, rounds: u8 },
    Quit,
}

pub const CHOICES: [(&str, &str); 6] =
    [("1", "snake"), ("2", "reaction"), ("3", "memory"), ("4", "pong"), ("5", "catch"), ("6", "dodge")];

const INK: Color = Color::Rgb(20, 20, 30);

fn title(t: &str) -> Line<'static> {
    Line::from(Span::styled(t.to_string(), Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)))
}

/// A title and a short status on one line, for the games that need every row for the field.
fn heading(name: &str, info: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{name}  "), Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled(info, Style::new().fg(DIM)),
    ])
}

fn dim(t: String) -> Line<'static> {
    Line::from(Span::styled(t, Style::new().fg(DIM)))
}

fn text(t: String) -> Line<'static> {
    Line::from(Span::styled(t, Style::new().fg(TEXT)))
}

pub enum Game {
    Snake(Snake),
    Reaction(Reaction),
    Memory(Memory),
    Pong(Pong),
    Catch(Catch),
    Dodge(Dodge),
}

impl Game {
    pub fn new(kind: u8, pet: &mut Pet) -> Game {
        match kind {
            1 => Game::Snake(Snake::new(pet)),
            2 => Game::Reaction(Reaction::new(pet)),
            3 => Game::Memory(Memory::new(pet)),
            5 => Game::Catch(Catch::new(pet)),
            6 => Game::Dodge(Dodge::new(pet)),
            _ => Game::Pong(Pong::new(pet)),
        }
    }

    pub fn key(&mut self, code: KeyCode) -> Outcome {
        if matches!(code, KeyCode::Esc | KeyCode::Char('q')) {
            return Outcome::Quit;
        }
        match self {
            Game::Snake(g) => g.key(code),
            Game::Reaction(g) => g.key(code),
            Game::Memory(g) => g.key(code),
            Game::Pong(g) => g.key(code),
            Game::Catch(g) => g.key(code),
            Game::Dodge(g) => g.key(code),
        }
    }

    /// Called every frame for the time-based games.
    pub fn tick(&mut self, pet: &mut Pet) -> Outcome {
        match self {
            Game::Snake(g) => g.tick(pet),
            Game::Reaction(g) => g.tick(pet),
            Game::Memory(g) => g.tick(pet),
            Game::Pong(g) => g.tick(),
            Game::Catch(g) => g.tick(pet),
            Game::Dodge(g) => g.tick(pet),
        }
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        match self {
            Game::Snake(g) => g.lines(),
            Game::Reaction(g) => g.lines(),
            Game::Memory(g) => g.lines(),
            Game::Pong(g) => g.lines(),
            Game::Catch(g) => g.lines(),
            Game::Dodge(g) => g.lines(),
        }
    }

    pub fn keys(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            Game::Snake(_) => &[("↑↓←→", "steer"), ("esc", "quit game")],
            Game::Reaction(_) => &[("space", "react"), ("esc", "quit game")],
            Game::Memory(_) => &[("1-4", "repeat the pattern"), ("esc", "quit game")],
            Game::Pong(_) => &[("↑/↓", "steer"), ("space", "stop"), ("esc", "quit game")],
            Game::Catch(_) => &[("←/→", "steer"), ("space", "stop"), ("esc", "quit game")],
            Game::Dodge(_) => &[("←/→", "steer"), ("space", "stop"), ("esc", "quit game")],
        }
    }
}

// ---------------------------------------------------------------- reaction

const REACTION_ROUNDS: u8 = 3;
/// A press at or under this many milliseconds after "GO" wins the round.
const REACTION_WIN_MS: u128 = 500;

enum RPhase {
    Wait(Instant),
    Go(Instant),
    Pause(Instant),
}

pub struct Reaction {
    round: u8,
    wins: u8,
    phase: RPhase,
    note: String,
    /// Colour of the lamp after a press: green for a win, amber for slow, red for too early.
    tone: Color,
    born: Instant,
}

fn random_delay(pet: &mut Pet) -> Instant {
    Instant::now() + Duration::from_millis(1000 + pet.rand(2000))
}

impl Reaction {
    fn new(pet: &mut Pet) -> Self {
        Reaction { round: 1, wins: 0, phase: RPhase::Wait(random_delay(pet)), note: String::new(), tone: EMPTY, born: Instant::now() }
    }

    fn key(&mut self, code: KeyCode) -> Outcome {
        if !matches!(code, KeyCode::Char(' ') | KeyCode::Enter) {
            return Outcome::Continue;
        }
        let pause = Instant::now() + Duration::from_millis(1200);
        match self.phase {
            RPhase::Wait(_) => {
                self.note = "Too early!".into();
                self.tone = BAD;
                self.phase = RPhase::Pause(pause);
            }
            RPhase::Go(since) => {
                let ms = since.elapsed().as_millis();
                if ms <= REACTION_WIN_MS {
                    self.wins += 1;
                    self.note = format!("{ms} ms - fast!");
                    self.tone = GOOD;
                } else {
                    self.note = format!("{ms} ms - too slow.");
                    self.tone = WARN;
                }
                self.phase = RPhase::Pause(pause);
            }
            RPhase::Pause(_) => {}
        }
        Outcome::Continue
    }

    fn tick(&mut self, pet: &mut Pet) -> Outcome {
        let now = Instant::now();
        match self.phase {
            RPhase::Wait(at) if now >= at => self.phase = RPhase::Go(now),
            RPhase::Pause(until) if now >= until => {
                if self.round >= REACTION_ROUNDS {
                    return Outcome::Done { wins: self.wins, rounds: REACTION_ROUNDS };
                }
                self.round += 1;
                self.note.clear();
                self.phase = RPhase::Wait(random_delay(pet));
            }
            _ => {}
        }
        Outcome::Continue
    }

    fn lines(&self) -> Vec<Line<'static>> {
        const W: usize = 24;
        let lamp = |bg: Color, label: &str| -> Vec<Line<'static>> {
            let style = Style::new().fg(INK).bg(bg).add_modifier(Modifier::BOLD);
            vec![
                Line::from(Span::styled(" ".repeat(W), style)),
                Line::from(Span::styled(format!("{label:^W$}"), style)),
                Line::from(Span::styled(" ".repeat(W), style)),
            ]
        };
        let (face, mut body, bar) = match self.phase {
            RPhase::Wait(_) => {
                // The red lamp breathes so it is clear the game is alive and you should wait.
                let on = (self.born.elapsed().as_millis() / 350) % 2 == 0;
                let red = if on { Color::Rgb(215, 80, 90) } else { Color::Rgb(130, 50, 62) };
                ("o_o", lamp(red, "wait for it..."), Line::from(""))
            }
            RPhase::Go(since) => {
                let left = REACTION_WIN_MS.saturating_sub(since.elapsed().as_millis());
                let filled = (left as usize * W / REACTION_WIN_MS as usize).min(W);
                let colour = if left * 2 > REACTION_WIN_MS { GOOD } else if left * 5 > REACTION_WIN_MS { WARN } else { BAD };
                let bar = Line::from(vec![
                    Span::styled("█".repeat(filled), Style::new().fg(colour)),
                    Span::styled("░".repeat(W - filled), Style::new().fg(EMPTY)),
                ]);
                ("O_O", lamp(GOOD, "G O !   press space"), bar)
            }
            RPhase::Pause(_) => {
                let face = if self.tone == GOOD { "^_^" } else { "T_T" };
                let label = self.note.to_uppercase();
                (face, lamp(self.tone, &label), Line::from(""))
            }
        };
        let pips: String = (1..=REACTION_ROUNDS).map(|r| if r <= self.wins { "●" } else if r < self.round { "○" } else { "·" }).collect::<Vec<_>>().join(" ");
        let mut out = vec![
            heading("REACTION", format!("round {}/{}   {pips}", self.round, REACTION_ROUNDS)),
            Line::from(Span::styled(face, Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))),
        ];
        out.append(&mut body);
        out.push(bar);
        out.push(dim(format!("press space within {REACTION_WIN_MS} ms of green")));
        out
    }
}

// ---------------------------------------------------------------- memory

const MEMORY_ROUNDS: u8 = 3;
const MEMORY_COLORS: [Color; 4] =
    [Color::Rgb(240, 95, 105), Color::Rgb(245, 200, 90), Color::Rgb(130, 210, 140), Color::Rgb(110, 160, 250)];

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
    note: String,
}

fn after_ms(ms: u64) -> Instant {
    Instant::now() + Duration::from_millis(ms)
}

impl Memory {
    fn new(pet: &mut Pet) -> Self {
        let mut m = Memory { round: 1, wins: 0, seq: Vec::new(), phase: MPhase::Pause(after_ms(0)), note: String::new() };
        m.start_round(pet);
        m
    }

    fn start_round(&mut self, pet: &mut Pet) {
        let len = self.round as usize + 2;
        self.seq = (0..len).map(|_| pet.rand(4) as u8 + 1).collect();
        self.note = "Watch the pattern...".into();
        self.phase = MPhase::Show { idx: 0, lit: false, until: after_ms(600) };
    }

    fn key(&mut self, code: KeyCode) -> Outcome {
        let (MPhase::Input(pos), KeyCode::Char(c @ '1'..='4')) = (&self.phase, code) else {
            return Outcome::Continue;
        };
        let pos = *pos;
        let pressed = c as u8 - b'0';
        if pressed == self.seq[pos] {
            if pos + 1 == self.seq.len() {
                self.wins += 1;
                self.note = "Correct!".into();
                self.phase = MPhase::Pause(after_ms(1000));
            } else {
                self.phase = MPhase::Input(pos + 1);
            }
        } else {
            let answer: Vec<String> = self.seq.iter().map(|n| n.to_string()).collect();
            self.note = format!("Wrong - it was {}", answer.join(" "));
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
                    self.note = "Your turn: repeat it with 1-4".into();
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

    fn lines(&self) -> Vec<Line<'static>> {
        let lit_n = match self.phase {
            MPhase::Show { idx, lit: true, .. } => Some(self.seq[idx]),
            _ => None,
        };
        let boxes: Vec<Span<'static>> = (1..=4u8)
            .flat_map(|n| {
                let c = MEMORY_COLORS[n as usize - 1];
                let style = if lit_n == Some(n) { Style::new().fg(INK).bg(c).add_modifier(Modifier::BOLD) } else { Style::new().fg(c).bg(EMPTY) };
                [Span::styled(format!("  {n}  "), style), Span::raw("  ")]
            })
            .collect();
        let progress = match self.phase {
            MPhase::Input(pos) => format!("{pos}/{}", self.seq.len()),
            _ => String::new(),
        };
        vec![
            title("MEMORY"),
            dim(format!("round {}/{}   wins {}", self.round, MEMORY_ROUNDS, self.wins)),
            Line::from(""),
            Line::from(boxes),
            Line::from(""),
            text(self.note.clone()),
            dim(progress),
        ]
    }
}

// ---------------------------------------------------------------- pong

const PW: i32 = 28;
/// Field height: a title line, this many rows and a bottom wall must fit the smallest window's 10-row room.
const PH: i32 = 8;
const PADDLE: i32 = 3;
/// Return the ball this many times to win outright.
const PONG_GOAL: u8 = 5;

/// How often a steered paddle, basket or pet moves one cell.
const MOVE_MS: u64 = 55;

/// Press a direction once and it keeps going until you press the other way or space, so the
/// game does not depend on the terminal's key repeat.
fn steer(code: KeyCode, neg: &[KeyCode], pos: &[KeyCode], dir: &mut i32) {
    if neg.contains(&code) {
        *dir = -1;
    } else if pos.contains(&code) {
        *dir = 1;
    } else if code == KeyCode::Char(' ') {
        *dir = 0;
    }
}

const LEFT: [KeyCode; 3] = [KeyCode::Left, KeyCode::Char('a'), KeyCode::Char('h')];
const RIGHT: [KeyCode; 3] = [KeyCode::Right, KeyCode::Char('d'), KeyCode::Char('l')];
const UP: [KeyCode; 3] = [KeyCode::Up, KeyCode::Char('w'), KeyCode::Char('k')];
const DOWN: [KeyCode; 3] = [KeyCode::Down, KeyCode::Char('s'), KeyCode::Char('j')];

pub struct Pong {
    py: i32,
    dir: i32,
    moved: Instant,
    bx: i32,
    by: i32,
    dx: i32,
    dy: i32,
    hits: u8,
    last: Instant,
    step_ms: u64,
}

impl Pong {
    fn new(pet: &mut Pet) -> Self {
        Pong {
            py: PH / 2 - 1,
            dir: 0,
            moved: Instant::now(),
            bx: PW - 2,
            by: pet.rand(PH as u64) as i32,
            dx: -1,
            dy: if pet.rand(2) == 0 { 1 } else { -1 },
            hits: 0,
            last: Instant::now(),
            step_ms: 90,
        }
    }

    fn key(&mut self, code: KeyCode) -> Outcome {
        steer(code, &UP, &DOWN, &mut self.dir);
        Outcome::Continue
    }

    fn nudge(&mut self) {
        self.py = (self.py + self.dir).clamp(0, PH - PADDLE);
    }

    fn tick(&mut self) -> Outcome {
        if self.moved.elapsed() >= Duration::from_millis(MOVE_MS + 25) {
            self.moved = Instant::now();
            self.nudge();
        }
        if self.last.elapsed() < Duration::from_millis(self.step_ms) {
            return Outcome::Continue;
        }
        self.last = Instant::now();
        let mut ny = self.by + self.dy;
        if !(0..PH).contains(&ny) {
            self.dy = -self.dy;
            ny = self.by + self.dy;
        }
        let mut nx = self.bx + self.dx;
        if nx >= PW {
            self.dx = -self.dx;
            nx = self.bx + self.dx;
        }
        if nx <= 0 {
            if (self.py..self.py + PADDLE).contains(&ny) {
                self.hits += 1;
                if self.hits >= PONG_GOAL {
                    return Outcome::Done { wins: PONG_GOAL, rounds: PONG_GOAL };
                }
                self.dx = 1;
                nx = 1;
                self.step_ms = (self.step_ms * 9 / 10).max(40);
            } else {
                return Outcome::Done { wins: self.hits, rounds: PONG_GOAL };
            }
        }
        self.bx = nx;
        self.by = ny;
        Outcome::Continue
    }

    fn lines(&self) -> Vec<Line<'static>> {
        let wall = Style::new().fg(EMPTY);
        let mut out = vec![heading("PONG", format!("returns {}/{}  (a miss ends it)", self.hits, PONG_GOAL))];
        for y in 0..PH {
            let row: Vec<Span<'static>> = (0..PW)
                .map(|x| {
                    if x == 0 && (self.py..self.py + PADDLE).contains(&y) {
                        Span::styled("█", Style::new().fg(ACCENT))
                    } else if x == self.bx && y == self.by {
                        Span::styled("●", Style::new().fg(WARN))
                    } else if x == PW - 1 {
                        Span::styled("│", wall)
                    } else if x == PW / 2 && y % 2 == 0 {
                        Span::styled("┊", wall)
                    } else {
                        Span::raw(" ")
                    }
                })
                .collect();
            out.push(Line::from(row));
        }
        out.push(Line::from(Span::styled("─".repeat(PW as usize), wall)));
        out
    }
}

// ---------------------------------------------------------------- catch

/// Five pieces of food fall one after another; catch them in the basket.
const CATCH_ITEMS: u8 = 5;
const BASKET: i32 = 7;
const CATCH_FOOD: [(&str, Color); 3] = [("●", BAD), ("◆", WARN), ("♥", GOOD)];

pub struct Catch {
    face: &'static str,
    dir: i32,
    moved: Instant,
    bx: i32,
    ix: i32,
    iy: i32,
    item: u8,
    caught: u8,
    last: Instant,
    step_ms: u64,
}

impl Catch {
    fn new(pet: &mut Pet) -> Self {
        Catch { face: face(pet), dir: 0, moved: Instant::now(), bx: PW / 2 - 3, ix: pet.rand(PW as u64) as i32, iy: 0, item: 1, caught: 0, last: Instant::now(), step_ms: 190 }
    }

    fn key(&mut self, code: KeyCode) -> Outcome {
        steer(code, &LEFT, &RIGHT, &mut self.dir);
        Outcome::Continue
    }

    fn nudge(&mut self) {
        self.bx = (self.bx + self.dir).clamp(0, PW - BASKET);
    }

    fn tick(&mut self, pet: &mut Pet) -> Outcome {
        if self.moved.elapsed() >= Duration::from_millis(MOVE_MS) {
            self.moved = Instant::now();
            self.nudge();
        }
        if self.last.elapsed() < Duration::from_millis(self.step_ms) {
            return Outcome::Continue;
        }
        self.last = Instant::now();
        self.iy += 1;
        if self.iy < PH - 1 {
            return Outcome::Continue;
        }
        if (self.bx..self.bx + BASKET).contains(&self.ix) {
            self.caught += 1;
        }
        if self.item >= CATCH_ITEMS {
            return Outcome::Done { wins: self.caught, rounds: CATCH_ITEMS };
        }
        self.item += 1;
        self.ix = pet.rand(PW as u64) as i32;
        self.iy = 0;
        self.step_ms = (self.step_ms * 95 / 100).max(100);
        Outcome::Continue
    }

    fn lines(&self) -> Vec<Line<'static>> {
        let wall = Style::new().fg(EMPTY);
        let mut out = vec![heading("CATCH", format!("food {}/{}  caught {}", self.item, CATCH_ITEMS, self.caught))];
        for y in 0..PH {
            let row: Vec<Span<'static>> = (0..PW)
                .map(|x| {
                    if y == PH - 1 && (self.bx..self.bx + BASKET).contains(&x) {
                        let i = (x - self.bx) as usize;
                        if i == 0 {
                            Span::styled("╰", Style::new().fg(ACCENT))
                        } else if i == BASKET as usize - 1 {
                            Span::styled("╯", Style::new().fg(ACCENT))
                        } else {
                            // The pet's three-character face sits in the middle of the basket.
                            let off = (BASKET as usize - 3) / 2;
                            let ch = i.checked_sub(off).and_then(|k| self.face.chars().nth(k)).unwrap_or('─');
                            Span::styled(ch.to_string(), Style::new().fg(TEXT))
                        }
                    } else if x == self.ix && y == self.iy {
                        let (glyph, colour) = CATCH_FOOD[self.item as usize % CATCH_FOOD.len()];
                        Span::styled(glyph, Style::new().fg(colour))
                    } else {
                        Span::raw(" ")
                    }
                })
                .collect();
            out.push(Line::from(row));
        }
        out.push(Line::from(Span::styled("─".repeat(PW as usize), wall)));
        out
    }
}

// ---------------------------------------------------------------- snake

/// Eat this much food to win outright; hitting a wall or yourself ends the game.
const SNAKE_GOAL: u8 = 8;

pub struct Snake {
    body: VecDeque<(i32, i32)>,
    dir: (i32, i32),
    want: (i32, i32),
    food: (i32, i32),
    eaten: u8,
    last: Instant,
    step_ms: u64,
    born: Instant,
}

impl Snake {
    fn new(pet: &mut Pet) -> Self {
        let body: VecDeque<(i32, i32)> = [(5, PH / 2), (4, PH / 2), (3, PH / 2)].into();
        let mut s = Snake { body, dir: (1, 0), want: (1, 0), food: (0, 0), eaten: 0, last: Instant::now(), step_ms: 150, born: Instant::now() };
        s.place_food(pet);
        s
    }

    fn place_food(&mut self, pet: &mut Pet) {
        loop {
            let f = (pet.rand(PW as u64) as i32, pet.rand(PH as u64) as i32);
            if !self.body.contains(&f) {
                self.food = f;
                return;
            }
        }
    }

    fn key(&mut self, code: KeyCode) -> Outcome {
        let want = if UP.contains(&code) {
            (0, -1)
        } else if DOWN.contains(&code) {
            (0, 1)
        } else if LEFT.contains(&code) {
            (-1, 0)
        } else if RIGHT.contains(&code) {
            (1, 0)
        } else {
            return Outcome::Continue;
        };
        // Turning straight back would be instant death, so it is ignored.
        if (want.0 + self.dir.0, want.1 + self.dir.1) != (0, 0) {
            self.want = want;
        }
        Outcome::Continue
    }

    fn tick(&mut self, pet: &mut Pet) -> Outcome {
        // A cell is twice as tall as it is wide, so up/down steps take longer to look as fast.
        let wait = if self.want.1 != 0 { self.step_ms * 2 } else { self.step_ms };
        if self.last.elapsed() < Duration::from_millis(wait) {
            return Outcome::Continue;
        }
        self.last = Instant::now();
        self.dir = self.want;
        let head = self.body[0];
        let next = (head.0 + self.dir.0, head.1 + self.dir.1);
        // The tail moves out of the way this step, unless we grow.
        let hits_self = self.body.iter().take(self.body.len() - 1).any(|&c| c == next);
        if !(0..PW).contains(&next.0) || !(0..PH).contains(&next.1) || hits_self {
            return Outcome::Done { wins: self.eaten, rounds: SNAKE_GOAL };
        }
        self.body.push_front(next);
        if next == self.food {
            self.eaten += 1;
            if self.eaten >= SNAKE_GOAL {
                return Outcome::Done { wins: SNAKE_GOAL, rounds: SNAKE_GOAL };
            }
            self.step_ms = (self.step_ms * 94 / 100).max(80);
            self.place_food(pet);
        } else {
            self.body.pop_back();
        }
        Outcome::Continue
    }

    fn lines(&self) -> Vec<Line<'static>> {
        let hearts: String = (0..SNAKE_GOAL).map(|i| if i < self.eaten { "♥" } else { "♡" }).collect::<Vec<_>>().join("");
        let mut out = vec![heading("SNAKE", format!("{hearts}  (a crash ends it)"))];
        // The food pulses between two colours.
        let pulse = if (self.born.elapsed().as_millis() / 250) % 2 == 0 { BAD } else { ACCENT };
        let last = self.body.len().saturating_sub(1).max(1);
        for y in 0..PH {
            let row: Vec<Span<'static>> = (0..PW)
                .map(|x| {
                    // A faint checkerboard lawn, so the snake moves over something.
                    let lawn = if (x + y) % 2 == 0 { Color::Rgb(26, 38, 34) } else { Color::Rgb(31, 45, 40) };
                    let base = Style::new().bg(lawn);
                    if let Some(i) = self.body.iter().position(|&c| c == (x, y)) {
                        if i == 0 {
                            return Span::styled("◉", base.fg(ACCENT).add_modifier(Modifier::BOLD));
                        }
                        // Bright near the head, darker towards the tail.
                        let t = i * 100 / last;
                        let green = Color::Rgb((130 - t as i32 * 70 / 100) as u8, (210 - t as i32 * 90 / 100) as u8, (140 - t as i32 * 70 / 100) as u8);
                        Span::styled(if i == last { "•" } else { "●" }, base.fg(green))
                    } else if self.food == (x, y) {
                        Span::styled("♥", base.fg(pulse).add_modifier(Modifier::BOLD))
                    } else {
                        Span::styled(" ", base)
                    }
                })
                .collect();
            out.push(Line::from(row));
        }
        out.push(Line::from(Span::styled("─".repeat(PW as usize), Style::new().fg(EMPTY))));
        out
    }
}

// ---------------------------------------------------------------- dodge

/// Rocks fall in five waves of six; every rock that lands without hitting the pet counts.
const DODGE_ROCKS: u8 = 30;
const DODGE_ROUNDS: u8 = 5;
const DODGE_PLAYER: i32 = 3;

pub struct Dodge {
    face: &'static str,
    px: i32,
    dir: i32,
    rocks: Vec<(i32, i32)>,
    spawned: u8,
    passed: u8,
    since_spawn: u8,
    moved: Instant,
    last: Instant,
    step_ms: u64,
}

impl Dodge {
    fn new(pet: &mut Pet) -> Self {
        Dodge {
            face: face(pet),
            px: PW / 2 - 1,
            dir: 0,
            rocks: Vec::new(),
            spawned: 0,
            passed: 0,
            since_spawn: 0,
            moved: Instant::now(),
            last: Instant::now(),
            step_ms: 150,
        }
    }

    fn key(&mut self, code: KeyCode) -> Outcome {
        steer(code, &LEFT, &RIGHT, &mut self.dir);
        Outcome::Continue
    }

    fn nudge(&mut self) {
        self.px = (self.px + self.dir).clamp(0, PW - DODGE_PLAYER);
    }

    fn hit(&self) -> bool {
        self.rocks.iter().any(|&(x, y)| y == PH - 1 && (self.px..self.px + DODGE_PLAYER).contains(&x))
    }

    fn over(&self) -> Outcome {
        Outcome::Done { wins: self.passed / (DODGE_ROCKS / DODGE_ROUNDS), rounds: DODGE_ROUNDS }
    }

    fn tick(&mut self, pet: &mut Pet) -> Outcome {
        if self.moved.elapsed() >= Duration::from_millis(MOVE_MS) {
            self.moved = Instant::now();
            self.nudge();
            if self.hit() {
                return self.over();
            }
        }
        if self.last.elapsed() < Duration::from_millis(self.step_ms) {
            return Outcome::Continue;
        }
        self.last = Instant::now();
        for r in &mut self.rocks {
            r.1 += 1;
        }
        if self.hit() {
            return self.over();
        }
        let before = self.rocks.len();
        self.rocks.retain(|r| r.1 < PH);
        self.passed += (before - self.rocks.len()) as u8;
        if self.passed >= DODGE_ROCKS {
            return Outcome::Done { wins: DODGE_ROUNDS, rounds: DODGE_ROUNDS };
        }
        self.since_spawn += 1;
        if self.since_spawn >= 3 && self.spawned < DODGE_ROCKS {
            self.since_spawn = 0;
            self.spawned += 1;
            self.rocks.push((pet.rand(PW as u64) as i32, 0));
            self.step_ms = (self.step_ms * 98 / 100).max(90);
        }
        Outcome::Continue
    }

    fn lines(&self) -> Vec<Line<'static>> {
        let wave = (self.passed / (DODGE_ROCKS / DODGE_ROUNDS) + 1).min(DODGE_ROUNDS);
        let mut out = vec![heading("DODGE", format!("wave {wave}/{DODGE_ROUNDS}  (a hit ends it)"))];
        for y in 0..PH {
            let row: Vec<Span<'static>> = (0..PW)
                .map(|x| {
                    let i = (x - self.px) as usize;
                    if y == PH - 1 && (self.px..self.px + DODGE_PLAYER).contains(&x) {
                        Span::styled(self.face.chars().nth(i).unwrap_or(' ').to_string(), Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))
                    } else if self.rocks.contains(&(x, y)) {
                        Span::styled("▼", Style::new().fg(WARN))
                    } else {
                        Span::raw(" ")
                    }
                })
                .collect();
            out.push(Line::from(row));
        }
        out.push(Line::from(Span::styled("─".repeat(PW as usize), Style::new().fg(EMPTY))));
        out
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
        for kind in 1..=6 {
            let mut p = pet();
            let mut g = Game::new(kind, &mut p);
            assert!(matches!(g.key(KeyCode::Esc), Outcome::Quit), "game {kind}");
        }
    }

    #[test]
    fn reaction_pressing_too_early_loses_the_round() {
        let mut p = pet();
        let mut r = Reaction::new(&mut p);
        r.key(KeyCode::Char(' '));
        assert_eq!(r.note, "Too early!");
        assert_eq!(r.wins, 0);
    }

    #[test]
    fn reaction_a_quick_press_after_go_wins() {
        let mut p = pet();
        let mut r = Reaction::new(&mut p);
        r.phase = RPhase::Go(Instant::now());
        r.key(KeyCode::Char(' '));
        assert_eq!(r.wins, 1);
    }

    #[test]
    fn reaction_ends_after_three_rounds() {
        let mut p = pet();
        let mut r = Reaction::new(&mut p);
        for round in 1..=REACTION_ROUNDS {
            r.key(KeyCode::Char(' '));
            r.phase = RPhase::Pause(Instant::now() - Duration::from_secs(1));
            let out = r.tick(&mut p);
            if round < REACTION_ROUNDS {
                assert!(matches!(out, Outcome::Continue));
            } else {
                assert!(matches!(out, Outcome::Done { wins: 0, rounds: 3 }));
            }
        }
    }

    #[test]
    fn memory_repeating_the_pattern_wins_the_round_and_a_wrong_key_loses_it() {
        let mut p = pet();
        let mut m = Memory::new(&mut p);
        m.phase = MPhase::Input(0);
        for n in m.seq.clone() {
            m.key(KeyCode::Char((b'0' + n) as char));
        }
        assert_eq!(m.wins, 1);
        assert_eq!(m.note, "Correct!");

        let mut m = Memory::new(&mut p);
        m.phase = MPhase::Input(0);
        let wrong = (m.seq[0] % 4) + 1;
        m.key(KeyCode::Char((b'0' + wrong) as char));
        assert!(m.note.starts_with("Wrong"));
        assert_eq!(m.wins, 0);
    }

    #[test]
    fn memory_ignores_keys_while_the_pattern_is_showing() {
        let mut p = pet();
        let mut m = Memory::new(&mut p);
        m.key(KeyCode::Char('1'));
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
    fn pong_paddle_keeps_moving_until_stopped_and_stays_in_the_court() {
        let mut p = pet();
        let mut pong = Pong::new(&mut p);
        pong.key(KeyCode::Up);
        for _ in 0..30 {
            pong.nudge();
        }
        assert_eq!(pong.py, 0);
        pong.key(KeyCode::Down);
        for _ in 0..30 {
            pong.nudge();
        }
        assert_eq!(pong.py, PH - PADDLE);
        pong.key(KeyCode::Up);
        pong.nudge();
        pong.key(KeyCode::Char(' '));
        let at = pong.py;
        pong.nudge();
        assert_eq!(pong.py, at, "space stops the paddle");
    }

    #[test]
    fn the_basket_keeps_moving_and_stays_inside_the_field() {
        let mut p = pet();
        let mut c = Catch::new(&mut p);
        c.key(KeyCode::Left);
        for _ in 0..30 {
            c.nudge();
        }
        assert_eq!(c.bx, 0);
        c.key(KeyCode::Right);
        for _ in 0..30 {
            c.nudge();
        }
        assert_eq!(c.bx, PW - BASKET);
    }

    #[test]
    fn snake_eats_grows_and_cannot_reverse_into_itself() {
        let mut p = pet();
        let mut s = Snake::new(&mut p);
        s.food = (s.body[0].0 + 1, s.body[0].1);
        s.key(KeyCode::Left);
        assert_eq!(s.want, (1, 0), "reversing is ignored");
        s.last = Instant::now() - Duration::from_millis(1000);
        assert!(matches!(s.tick(&mut p), Outcome::Continue));
        assert_eq!((s.eaten, s.body.len()), (1, 4));
    }

    #[test]
    fn snake_dies_on_the_wall_and_reports_what_it_ate() {
        let mut p = pet();
        let mut s = Snake::new(&mut p);
        s.eaten = 3;
        for _ in 0..PW {
            s.last = Instant::now() - Duration::from_millis(1000);
            if let Outcome::Done { wins, rounds } = s.tick(&mut p) {
                assert_eq!((wins, rounds), (3, SNAKE_GOAL));
                return;
            }
        }
        panic!("snake never hit the wall");
    }

    #[test]
    fn dodge_a_rock_on_the_pet_ends_it_and_a_rock_beside_it_does_not() {
        let mut p = pet();
        let mut d = Dodge::new(&mut p);
        d.rocks.push((0, PH - 2));
        d.px = PW - DODGE_PLAYER;
        d.last = Instant::now() - Duration::from_millis(500);
        assert!(matches!(d.tick(&mut p), Outcome::Continue));
        d.rocks.push((d.px + 1, PH - 2));
        d.last = Instant::now() - Duration::from_millis(500);
        assert!(matches!(d.tick(&mut p), Outcome::Done { rounds: DODGE_ROUNDS, .. }));
    }

    #[test]
    fn dodge_always_ends_and_never_reports_more_than_the_rounds() {
        let mut p = pet();
        let mut d = Dodge::new(&mut p);
        for _ in 0..20_000 {
            d.last = Instant::now() - Duration::from_millis(500);
            if let Outcome::Done { wins, rounds } = d.tick(&mut p) {
                assert_eq!(rounds, DODGE_ROUNDS);
                assert!(wins <= DODGE_ROUNDS);
                return;
            }
        }
        panic!("dodge never ended");
    }

    #[test]
    fn catch_a_fruit_over_the_basket_is_caught_and_one_beside_it_is_missed() {
        let mut p = pet();
        let mut c = Catch::new(&mut p);
        c.ix = c.bx + 1;
        c.iy = PH - 2;
        c.last = Instant::now() - Duration::from_millis(500);
        c.tick(&mut p);
        assert_eq!(c.caught, 1);
        assert_eq!(c.item, 2, "the next piece of food starts falling");

        c.ix = if c.bx > 10 { 0 } else { PW - 1 };
        c.iy = PH - 2;
        c.last = Instant::now() - Duration::from_millis(500);
        c.tick(&mut p);
        assert_eq!(c.caught, 1, "missed");
    }

    #[test]
    fn catch_always_ends_after_five_pieces_of_food() {
        let mut p = pet();
        let mut c = Catch::new(&mut p);
        for _ in 0..5000 {
            c.last = Instant::now() - Duration::from_millis(500);
            if let Outcome::Done { wins, rounds } = c.tick(&mut p) {
                assert_eq!(rounds, CATCH_ITEMS);
                assert!(wins <= CATCH_ITEMS);
                return;
            }
        }
        panic!("catch never ended");
    }

    #[test]
    fn every_game_fits_the_smallest_room() {
        // The room is 10 rows tall in the smallest supported window (64x22).
        for kind in 1..=6 {
            let mut p = pet();
            let g = Game::new(kind, &mut p);
            assert!(g.lines().len() <= 10, "game {kind} needs {} rows", g.lines().len());
        }
    }

    #[test]
    fn the_picker_offers_six_games_and_each_one_starts() {
        assert_eq!(CHOICES.len(), 6);
        for kind in 1..=6 {
            let mut p = pet();
            let g = Game::new(kind, &mut p);
            assert!(!g.lines().is_empty() && !g.keys().is_empty(), "game {kind}");
        }
    }
}
