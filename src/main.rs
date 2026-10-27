mod pet;
mod save;
mod sprites;

use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use pet::{Pet, Stage, MAX_HEARTS};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};

const MAX_CATCHUP_TICKS: u64 = 86_400;
const GAME_ROUNDS: u8 = 5;

pub fn fmt_age(secs: u64) -> String {
    let (d, h, m) = (secs / 86_400, secs / 3600 % 24, secs / 60 % 60);
    if d > 0 { format!("{d}d {h}h {m}m") } else if h > 0 { format!("{h}h {m}m") } else { format!("{m}m {}s", secs % 60) }
}

struct HigherLower {
    current: u8,
    round: u8,
    wins: u8,
    note: String,
}

enum Mode {
    Normal,
    Game(HigherLower),
}

struct App {
    pet: Pet,
    mode: Mode,
    message: String,
    log: Vec<String>,
    speed: f64,
    tick_acc: f64,
    last: Instant,
    last_save: Instant,
    was_calling: bool,
    quit: bool,
}

impl App {
    fn new() -> Self {
        let speed = std::env::var("TAMA_SPEED").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0);
        let (pet, away) = match save::load() {
            Some(x) => x,
            None => (Pet::new(save::now()), 0),
        };
        let mut app = App {
            pet,
            mode: Mode::Normal,
            message: "Welcome back!".into(),
            log: Vec::new(),
            speed,
            tick_acc: 0.0,
            last: Instant::now(),
            last_save: Instant::now(),
            was_calling: false,
            quit: false,
        };
        let ticks = ((away as f64 * speed) as u64).min(MAX_CATCHUP_TICKS);
        for _ in 0..ticks {
            app.pet.tick();
        }
        if ticks > 0 {
            app.message = format!("Away for {} - catching up...", fmt_age(away));
        }
        app.drain_events();
        app
    }

    fn drain_events(&mut self) {
        for e in self.pet.events.drain(..) {
            self.message = e.clone();
            self.log.push(e);
        }
        if self.log.len() > 50 {
            self.log.drain(..self.log.len() - 50);
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        self.tick_acc += now.duration_since(self.last).as_secs_f64() * self.speed;
        self.last = now;
        while self.tick_acc >= 1.0 {
            self.tick_acc -= 1.0;
            self.pet.tick();
        }
        self.drain_events();
        let calling = self.pet.needs_attention();
        if calling && !self.was_calling {
            print!("\x07");
        }
        self.was_calling = calling;
        if !self.pet.alive() && !self.pet.buried {
            self.pet.buried = true;
            let _ = save::bury(&self.pet);
        }
        if self.last_save.elapsed() > Duration::from_secs(10) {
            let _ = save::save(&self.pet);
            self.last_save = Instant::now();
        }
    }

    fn start_game(&mut self) {
        if let Some(m) = self.pet.can_play() {
            self.message = m.into();
            return;
        }
        let current = self.pet.rand(9) as u8 + 1;
        self.mode = Mode::Game(HigherLower { current, round: 1, wins: 0, note: "Will the next number be higher or lower?".into() });
    }

    fn guess(&mut self, higher: bool) {
        let Mode::Game(g) = &mut self.mode else { return };
        let mut next = self.pet.rand(9) as u8 + 1;
        while next == g.current {
            next = self.pet.rand(9) as u8 + 1;
        }
        let correct = (next > g.current) == higher;
        if correct {
            g.wins += 1;
        }
        g.note = format!("{} -> {next}: {}", g.current, if correct { "right!" } else { "wrong." });
        g.current = next;
        if g.round >= GAME_ROUNDS {
            let wins = g.wins;
            self.message = self.pet.finish_game(wins, GAME_ROUNDS);
            self.mode = Mode::Normal;
        } else {
            g.round += 1;
        }
    }

    fn on_key(&mut self, code: KeyCode) {
        if let Mode::Game(_) = self.mode {
            match code {
                KeyCode::Char('h') | KeyCode::Up => self.guess(true),
                KeyCode::Char('l') | KeyCode::Down => self.guess(false),
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.mode = Mode::Normal;
                    self.message = "Game abandoned.".into();
                }
                _ => {}
            }
            return;
        }
        let msg = match code {
            KeyCode::Char('f') => self.pet.feed_meal(),
            KeyCode::Char('s') => self.pet.feed_snack(),
            KeyCode::Char('p') => {
                self.start_game();
                return;
            }
            KeyCode::Char('m') => self.pet.medicine(),
            KeyCode::Char('c') => self.pet.clean(),
            KeyCode::Char('d') => self.pet.scold(),
            KeyCode::Char('l') => self.pet.toggle_light(),
            KeyCode::Char('n') if !self.pet.alive() => {
                self.pet = Pet::new(save::now());
                "A new egg appears!".into()
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.quit = true;
                return;
            }
            _ => return,
        };
        self.message = msg;
    }
}

fn hearts(n: u8) -> String {
    (0..MAX_HEARTS).map(|i| if i < n { '♥' } else { '♡' }).collect()
}

fn draw(f: &mut Frame, app: &App) {
    let p = &app.pet;
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(9),
        Constraint::Length(5),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .split(f.area());

    let title = format!(" {} - {} - age {} ", p.species, p.stage.label(), fmt_age(p.age));
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(format!("mistakes: {}  ", p.mistakes)),
            Span::raw(format!("weight: {}  ", p.weight)),
            Span::raw(format!("discipline: {}%", p.discipline)),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(title)),
        rows[0],
    );

    let anim = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() / 600).unwrap_or(0)) % 2 == 0;
    let mut body: Vec<Line> = Vec::new();
    let mut top = String::new();
    if p.asleep {
        top.push_str("z Z z  ");
    }
    if p.sick {
        top.push_str("[sick]  ");
    }
    body.push(Line::from(Span::styled(top, Style::new().fg(Color::Magenta))));
    match &app.mode {
        Mode::Game(g) => {
            body.push(Line::from(format!("Higher or Lower!  round {}/{}  wins {}", g.round, GAME_ROUNDS, g.wins)));
            body.push(Line::from(""));
            body.push(Line::from(Span::styled(format!("   [ {} ]", g.current), Style::new().add_modifier(Modifier::BOLD))));
            body.push(Line::from(""));
            body.push(Line::from(g.note.clone()));
        }
        Mode::Normal => {
            for l in sprites::sprite(p, anim) {
                body.push(Line::from(Span::styled(l, Style::new().fg(if p.alive() { Color::Green } else { Color::DarkGray }))));
            }
            if p.poops > 0 {
                body.push(Line::from(Span::styled("💩 ".repeat(p.poops as usize), Style::new().fg(Color::Yellow))));
            }
        }
    }
    if p.needs_attention() {
        body.push(Line::from(Span::styled("(!) needs attention", Style::new().fg(Color::Red).add_modifier(Modifier::BOLD))));
    }
    f.render_widget(
        Paragraph::new(body).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL)),
        rows[1],
    );

    let stats = vec![
        Line::from(format!("Hunger     {}", hearts(p.hunger))),
        Line::from(format!("Happy      {}", hearts(p.happy))),
        Line::from(format!("Discipline {}", if p.disc_call.is_some() { "calling!" } else { "ok" })),
    ];
    f.render_widget(Paragraph::new(stats).block(Block::default().borders(Borders::ALL).title(" Stats ")), rows[2]);

    f.render_widget(Paragraph::new(app.message.as_str()).block(Block::default().borders(Borders::ALL).title(" Message ")), rows[3]);

    let menu = match app.mode {
        Mode::Game(_) => "[h/Up] higher  [l/Down] lower  [Esc] quit game",
        Mode::Normal if p.stage == Stage::Dead => "[n] new egg  [q] quit",
        Mode::Normal => "[f]eed [s]nack [p]lay [l]ight [m]edicine [c]lean [d]iscipline [q]uit",
    };
    f.render_widget(Paragraph::new(menu).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL)), rows[4]);
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    while !app.quit {
        app.update();
        terminal.draw(|f| draw(f, app))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(k) = event::read()? {
                if k.kind == KeyEventKind::Press {
                    app.on_key(k.code);
                }
            }
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    let mut terminal = ratatui::init();
    let res = run(&mut terminal, &mut app);
    ratatui::restore();
    let _ = save::save(&app.pet);
    res
}
