//! Mini-games. Each one reports how many rounds it won; the pet's happiness
//! reward is decided by `Pet::finish_game`.

use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::pet::Pet;
use crate::{ACCENT, BAD, DIM, EMPTY, GOOD, TEXT, WARN};

pub enum Outcome {
    Continue,
    Done { wins: u8, rounds: u8 },
    Quit,
}

pub const CHOICES: [(&str, &str); 4] = [("1", "higher/lower"), ("2", "reaction"), ("3", "memory"), ("4", "pong")];

const INK: Color = Color::Rgb(20, 20, 30);

fn title(t: &str) -> Line<'static> {
    Line::from(Span::styled(t.to_string(), Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)))
}

fn dim(t: String) -> Line<'static> {
    Line::from(Span::styled(t, Style::new().fg(DIM)))
}

fn text(t: String) -> Line<'static> {
    Line::from(Span::styled(t, Style::new().fg(TEXT)))
}

pub enum Game {
    HigherLower(HigherLower),
    Reaction(Reaction),
    Memory(Memory),
    Pong(Pong),
}

impl Game {
    pub fn new(kind: u8, pet: &mut Pet) -> Game {
        match kind {
            1 => Game::HigherLower(HigherLower::new(pet)),
            2 => Game::Reaction(Reaction::new(pet)),
            3 => Game::Memory(Memory::new(pet)),
            _ => Game::Pong(Pong::new(pet)),
        }
    }

    pub fn key(&mut self, code: KeyCode, pet: &mut Pet) -> Outcome {
        if matches!(code, KeyCode::Esc | KeyCode::Char('q')) {
            return Outcome::Quit;
        }
        match self {
            Game::HigherLower(g) => g.key(code, pet),
            Game::Reaction(g) => g.key(code),
            Game::Memory(g) => g.key(code),
            Game::Pong(g) => g.key(code),
        }
    }

    /// Called every frame for the time-based games.
    pub fn tick(&mut self, pet: &mut Pet) -> Outcome {
        match self {
            Game::HigherLower(_) => Outcome::Continue,
            Game::Reaction(g) => g.tick(pet),
            Game::Memory(g) => g.tick(pet),
            Game::Pong(g) => g.tick(),
        }
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        match self {
            Game::HigherLower(g) => g.lines(),
            Game::Reaction(g) => g.lines(),
            Game::Memory(g) => g.lines(),
            Game::Pong(g) => g.lines(),
        }
    }

    pub fn keys(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            Game::HigherLower(_) => &[("h", "higher"), ("l", "lower"), ("esc", "quit game")],
            Game::Reaction(_) => &[("space", "react"), ("esc", "quit game")],
            Game::Memory(_) => &[("1-4", "repeat the pattern"), ("esc", "quit game")],
            Game::Pong(_) => &[("↑/↓", "move paddle"), ("esc", "quit game")],
        }
    }
}

// ---------------------------------------------------------------- higher / lower

pub struct HigherLower {
    current: u8,
    round: u8,
    wins: u8,
    note: String,
}

const HL_ROUNDS: u8 = 5;

impl HigherLower {
    fn new(pet: &mut Pet) -> Self {
        HigherLower { current: pet.rand(9) as u8 + 1, round: 1, wins: 0, note: "Will the next number be higher or lower?".into() }
    }

    fn key(&mut self, code: KeyCode, pet: &mut Pet) -> Outcome {
        let higher = match code {
            KeyCode::Char('h') | KeyCode::Up => true,
            KeyCode::Char('l') | KeyCode::Down => false,
            _ => return Outcome::Continue,
        };
        let mut next = pet.rand(9) as u8 + 1;
        while next == self.current {
            next = pet.rand(9) as u8 + 1;
        }
        let correct = (next > self.current) == higher;
        if correct {
            self.wins += 1;
        }
        self.note = format!("{} -> {next}: {}", self.current, if correct { "right!" } else { "wrong." });
        self.current = next;
        if self.round >= HL_ROUNDS {
            return Outcome::Done { wins: self.wins, rounds: HL_ROUNDS };
        }
        self.round += 1;
        Outcome::Continue
    }

    fn lines(&self) -> Vec<Line<'static>> {
        vec![
            title("HIGHER  or  LOWER"),
            dim(format!("round {}/{}   wins {}", self.round, HL_ROUNDS, self.wins)),
            Line::from(""),
            Line::from(Span::styled(format!("  {}  ", self.current), Style::new().fg(INK).bg(WARN).add_modifier(Modifier::BOLD))),
            Line::from(""),
            text(self.note.clone()),
        ]
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
}

fn random_delay(pet: &mut Pet) -> Instant {
    Instant::now() + Duration::from_millis(1000 + pet.rand(2000))
}

impl Reaction {
    fn new(pet: &mut Pet) -> Self {
        Reaction { round: 1, wins: 0, phase: RPhase::Wait(random_delay(pet)), note: String::new() }
    }

    fn key(&mut self, code: KeyCode) -> Outcome {
        if !matches!(code, KeyCode::Char(' ') | KeyCode::Enter) {
            return Outcome::Continue;
        }
        let pause = Instant::now() + Duration::from_millis(1200);
        match self.phase {
            RPhase::Wait(_) => {
                self.note = "Too early!".into();
                self.phase = RPhase::Pause(pause);
            }
            RPhase::Go(since) => {
                let ms = since.elapsed().as_millis();
                if ms <= REACTION_WIN_MS {
                    self.wins += 1;
                    self.note = format!("{ms} ms - fast!");
                } else {
                    self.note = format!("{ms} ms - too slow.");
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
        let prompt = match self.phase {
            RPhase::Wait(_) => Line::from(Span::styled("  wait for it...  ", Style::new().fg(TEXT).bg(EMPTY))),
            RPhase::Go(_) => Line::from(Span::styled("   GO!  press space   ", Style::new().fg(INK).bg(GOOD).add_modifier(Modifier::BOLD))),
            RPhase::Pause(_) => Line::from(""),
        };
        vec![
            title("REACTION"),
            dim(format!("round {}/{}   wins {}", self.round, REACTION_ROUNDS, self.wins)),
            dim(format!("press space within {REACTION_WIN_MS} ms of GO")),
            Line::from(""),
            prompt,
            Line::from(""),
            text(self.note.clone()),
        ]
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
const PH: i32 = 10;
const PADDLE: i32 = 3;
/// Return the ball this many times to win outright.
const PONG_GOAL: u8 = 5;

pub struct Pong {
    py: i32,
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
        match code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => self.py = (self.py - 1).max(0),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => self.py = (self.py + 1).min(PH - PADDLE),
            _ => {}
        }
        Outcome::Continue
    }

    fn tick(&mut self) -> Outcome {
        if self.last.elapsed() < Duration::from_millis(self.step_ms) {
            return Outcome::Continue;
        }
        self.last = Instant::now();
        let mut ny = self.by + self.dy;
        if ny < 0 || ny >= PH {
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
        let mut out = vec![
            title("PONG"),
            dim(format!("returns {}/{}   (miss = game over)", self.hits, PONG_GOAL)),
            Line::from(Span::styled("─".repeat(PW as usize), wall)),
        ];
        for y in 0..PH {
            let row: Vec<Span<'static>> = (0..PW)
                .map(|x| {
                    if x == 0 && (self.py..self.py + PADDLE).contains(&y) {
                        Span::styled("█", Style::new().fg(ACCENT))
                    } else if x == self.bx && y == self.by {
                        Span::styled("●", Style::new().fg(WARN))
                    } else if x == PW - 1 {
                        Span::styled("│", wall)
                    } else {
                        Span::raw(" ")
                    }
                })
                .collect();
            out.push(Line::from(row));
        }
        out.push(Line::from(Span::styled("─".repeat(PW as usize), wall)));
        let _ = BAD;
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
    fn higher_lower_finishes_on_the_fifth_guess() {
        let mut p = pet();
        let mut g = Game::new(1, &mut p);
        for guess in 1..=5 {
            let out = g.key(KeyCode::Char('h'), &mut p);
            if guess < 5 {
                assert!(matches!(out, Outcome::Continue), "guess {guess} should not end the game");
            } else {
                match out {
                    Outcome::Done { wins, rounds } => {
                        assert_eq!(rounds, 5);
                        assert!(wins <= 5);
                    }
                    _ => panic!("the fifth guess should end the game"),
                }
            }
        }
    }

    #[test]
    fn escape_quits_every_game() {
        for kind in 1..=4 {
            let mut p = pet();
            let mut g = Game::new(kind, &mut p);
            assert!(matches!(g.key(KeyCode::Esc, &mut p), Outcome::Quit), "game {kind}");
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
    fn pong_paddle_stays_inside_the_court() {
        let mut p = pet();
        let mut pong = Pong::new(&mut p);
        for _ in 0..30 {
            pong.key(KeyCode::Up);
        }
        assert_eq!(pong.py, 0);
        for _ in 0..30 {
            pong.key(KeyCode::Down);
        }
        assert_eq!(pong.py, PH - PADDLE);
    }
}
