//! Mini-games. Each one reports how many rounds it won; the pet's happiness
//! reward is decided by `Pet::finish_game`.

use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::pet::Pet;
use crate::ui::{ACCENT, DIM, EMPTY, TEXT, WARN};

pub enum Outcome {
    Continue,
    Done { wins: u8, rounds: u8 },
    Quit,
}

pub const CHOICES: [(&str, &str); 2] = [("1", "memory"), ("2", "pong")];

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

    pub fn key(&mut self, code: KeyCode) -> Outcome {
        if matches!(code, KeyCode::Esc | KeyCode::Char('q')) {
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

    pub fn lines(&self) -> Vec<Line<'static>> {
        match self {
            Game::Memory(g) => g.lines(),
            Game::Pong(g) => g.lines(),
        }
    }

    pub fn keys(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            Game::Memory(_) => &[("1-4", "repeat the pattern"), ("esc", "quit game")],
            Game::Pong(_) => &[("↑/↓", "steer"), ("space", "stop"), ("esc", "quit game")],
        }
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
            assert!(matches!(g.key(KeyCode::Esc), Outcome::Quit), "game {kind}");
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
    fn every_game_fits_the_smallest_room() {
        // The room is 10 rows tall in the smallest supported window (64x22).
        for kind in 1..=2 {
            let mut p = pet();
            let g = Game::new(kind, &mut p);
            assert!(g.lines().len() <= 10, "game {kind} needs {} rows", g.lines().len());
        }
    }

    #[test]
    fn the_picker_offers_two_games_and_each_one_starts() {
        assert_eq!(CHOICES.len(), 2);
        for kind in 1..=2 {
            let mut p = pet();
            let g = Game::new(kind, &mut p);
            assert!(!g.lines().is_empty() && !g.keys().is_empty(), "game {kind}");
        }
    }
}
