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
    started: Instant,
    fx: Option<(sprites::Fx, Instant)>,
    last_stage: Stage,
    quit: bool,
}

impl App {
    fn new() -> Self {
        let mut speed = std::env::var("TAMA_SPEED").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0);
        if std::env::args().any(|a| a == "--fast") {
            speed = 60.0;
            save::use_test_slot();
        }
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
            started: Instant::now(),
            fx: None,
            last_stage: Stage::Egg,
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
        app.last_stage = app.pet.stage;
        app
    }

    fn trigger(&mut self, fx: sprites::Fx) {
        self.fx = Some((fx, Instant::now()));
    }

    fn snapshot(&self) -> (u8, u8, u8, bool, u8, bool) {
        let p = &self.pet;
        (p.hunger, p.happy, p.doses, p.sick, p.poops, p.disc_call.is_some())
    }

    /// Pick an animation by comparing pet state before and after an action.
    fn react(&mut self, before: (u8, u8, u8, bool, u8, bool)) {
        let after = self.snapshot();
        let fx = if after.0 > before.0 {
            Some(sprites::Fx::Eat)
        } else if after.1 > before.1 {
            Some(sprites::Fx::Hearts)
        } else if after.2 < before.2 || (before.3 && !after.3) {
            Some(sprites::Fx::Heal)
        } else if after.4 < before.4 {
            Some(sprites::Fx::Clean)
        } else if (before.5 && !after.5) || after.1 < before.1 {
            Some(sprites::Fx::Scold)
        } else {
            None
        };
        if let Some(fx) = fx {
            self.trigger(fx);
        }
    }

    fn current_fx(&self) -> Option<(sprites::Fx, f32)> {
        let (fx, at) = self.fx?;
        let p = at.elapsed().as_secs_f32() / fx.duration_secs();
        (p < 1.0).then_some((fx, p))
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
        if self.pet.stage != self.last_stage {
            if self.pet.alive() {
                self.trigger(sprites::Fx::Evolve);
            }
            self.last_stage = self.pet.stage;
        }
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
        let before = self.snapshot();
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
            self.react(before);
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
        let before = self.snapshot();
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
        self.react(before);
    }
}

fn hearts(n: u8) -> String {
    (0..MAX_HEARTS).map(|i| if i < n { '♥' } else { '♡' }).collect()
}

fn draw(f: &mut Frame, app: &App) {
    let p = &app.pet;
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(12),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .split(f.area());

    let title = format!(" {} - {} - age {} ", p.species, p.stage.label(), fmt_age(p.age));
    f.render_widget(
        Paragraph::new(Line::from(format!("mistakes: {}   weight: {}", p.mistakes, p.weight)))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(title)),
        rows[0],
    );

    let cols = Layout::horizontal([Constraint::Min(34), Constraint::Length(28)]).split(rows[1]);

    let t = (app.started.elapsed().as_millis() / 350) as u32;
    let body: Vec<Line> = match &app.mode {
        Mode::Game(g) => vec![
            Line::from(""),
            Line::from(format!("Higher or Lower!  round {}/{}  wins {}", g.round, GAME_ROUNDS, g.wins)),
            Line::from(""),
            Line::from(Span::styled(format!("[ {} ]", g.current), Style::new().add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(g.note.clone()),
        ],
        Mode::Normal => sprites::render(p, t, app.current_fx()),
    };
    f.render_widget(
        Paragraph::new(body).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL)),
        cols[0],
    );

    let mut stats = vec![
        Line::from(format!("Hunger     {}", hearts(p.hunger))),
        Line::from(format!("Happy      {}", hearts(p.happy))),
        Line::from(format!("Discipline {}%", p.discipline)),
        Line::from(""),
    ];
    let mut flag = |text: &str, color: Color| stats.push(Line::from(Span::styled(text.to_string(), Style::new().fg(color))));
    if p.asleep {
        flag("zzz sleeping", Color::Blue);
    }
    if p.sick {
        flag("sick - needs medicine", Color::Green);
    }
    if p.disc_call.is_some() {
        flag("naughty - scold it", Color::Red);
    }
    if p.poops > 0 {
        flag("needs cleaning", Color::Yellow);
    }
    if p.needs_attention() {
        flag("(!) needs attention", Color::Red);
    }
    f.render_widget(Paragraph::new(stats).block(Block::default().borders(Borders::ALL).title(" Stats ")), cols[1]);

    f.render_widget(Paragraph::new(app.message.as_str()).block(Block::default().borders(Borders::ALL).title(" Message ")), rows[2]);

    let menu = match app.mode {
        Mode::Game(_) => "[h/Up] higher  [l/Down] lower  [Esc] quit game",
        Mode::Normal if p.stage == Stage::Dead => "[n] new egg  [q] quit",
        Mode::Normal => "[f]eed [s]nack [p]lay [l]ight [m]edicine [c]lean [d]iscipline [q]uit",
    };
    f.render_widget(Paragraph::new(menu).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL)), rows[3]);
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

/// `--gallery`: print every character, plain, to stdout (handy for art tweaks).
fn gallery() {
    let cases: [(Stage, &str); 10] = [
        (Stage::Egg, "Egg"),
        (Stage::Baby, "Babytchi"),
        (Stage::Child, "Marutchi"),
        (Stage::Teen, "Tonmarutchi"),
        (Stage::Teen, "Hashitamatchi"),
        (Stage::Adult, "Mametchi"),
        (Stage::Adult, "Kuchipatchi"),
        (Stage::Adult, "Tarakotchi"),
        (Stage::Adult, "Ichigotchi"),
        (Stage::Adult, "Nyorotchi"),
    ];
    let mut pets: Vec<Pet> = cases
        .iter()
        .map(|(stage, name)| {
            let mut p = Pet::new(1);
            p.stage = *stage;
            p.species = name.to_string();
            p.happy = 4;
            p.hunger = 4;
            p
        })
        .collect();
    let mut dead = Pet::new(1);
    dead.stage = Stage::Dead;
    pets.push(dead);
    for p in &pets {
        println!("-- {} --", p.species);
        for line in sprites::render(p, 0, None) {
            let mut out = String::new();
            for s in line.spans {
                let (mut fg, mut bg) = (None, None);
                if let Some(Color::Rgb(r, g, b)) = s.style.fg { fg = Some((r, g, b)); }
                if let Some(Color::Rgb(r, g, b)) = s.style.bg { bg = Some((r, g, b)); }
                if let Some((r, g, b)) = fg { out.push_str(&format!("\x1b[38;2;{r};{g};{b}m")); }
                if let Some((r, g, b)) = bg { out.push_str(&format!("\x1b[48;2;{r};{g};{b}m")); }
                out.push_str(&s.content);
                out.push_str("\x1b[0m");
            }
            println!("{out}");
        }
    }
}

fn main() -> std::io::Result<()> {
    if std::env::args().any(|a| a == "--gallery") {
        gallery();
        return Ok(());
    }
    let mut app = App::new();
    let mut terminal = ratatui::init();
    let res = run(&mut terminal, &mut app);
    ratatui::restore();
    let _ = save::save(&app.pet);
    res
}
