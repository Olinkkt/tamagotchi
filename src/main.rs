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
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
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

const BORDER: Color = Color::Rgb(86, 90, 125);
const DIM: Color = Color::Rgb(120, 124, 150);
const TEXT: Color = Color::Rgb(220, 222, 240);
const ACCENT: Color = Color::Rgb(255, 160, 190);
const GOOD: Color = Color::Rgb(130, 210, 140);
const WARN: Color = Color::Rgb(245, 200, 90);
const BAD: Color = Color::Rgb(240, 95, 105);
const EMPTY: Color = Color::Rgb(70, 72, 92);

fn panel(title: &str) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(BORDER))
        .title(Span::styled(format!(" {title} "), Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)))
}

fn hearts(n: u8) -> Vec<Span<'static>> {
    (0..MAX_HEARTS)
        .map(|i| if i < n { Span::styled("♥ ", Style::new().fg(BAD)) } else { Span::styled("♥ ", Style::new().fg(EMPTY)) })
        .collect()
}

fn bar(pct: u8, width: usize, color: Color) -> Vec<Span<'static>> {
    let filled = (pct as usize * width).div_ceil(100).min(width);
    vec![
        Span::styled("█".repeat(filled), Style::new().fg(color)),
        Span::styled("░".repeat(width - filled), Style::new().fg(EMPTY)),
    ]
}

fn stat_row(label: &str, mut value: Vec<Span<'static>>) -> Line<'static> {
    let mut spans = vec![Span::styled(format!("{label:<11}"), Style::new().fg(DIM))];
    spans.append(&mut value);
    Line::from(spans)
}

fn key_chip(key: &str, label: &str) -> Vec<Span<'static>> {
    vec![
        Span::styled(format!(" {key} "), Style::new().fg(Color::Rgb(20, 20, 30)).bg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {label}  "), Style::new().fg(TEXT)),
    ]
}

fn draw(f: &mut Frame, app: &App) {
    let p = &app.pet;
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(12),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .split(f.area());

    // Header
    let mood_dot = if !p.alive() { ("●", DIM) } else if p.needs_attention() { ("●", BAD) } else { ("●", GOOD) };
    let header = Line::from(vec![
        Span::styled(format!("{} ", mood_dot.0), Style::new().fg(mood_dot.1)),
        Span::styled(p.species.clone(), Style::new().fg(TEXT).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  ·  {}  ·  age {}", p.stage.label(), fmt_age(p.age)), Style::new().fg(DIM)),
    ]);
    f.render_widget(Paragraph::new(header).block(panel("tamagotchi")), rows[0]);

    let cols = Layout::horizontal([Constraint::Min(34), Constraint::Length(30)]).split(rows[1]);

    // Room
    let room_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(BORDER));
    let inner = room_block.inner(cols[0]);
    let t = (app.started.elapsed().as_millis() / 350) as u32;
    let body: Vec<Line> = match &app.mode {
        Mode::Game(g) => {
            let pad = (inner.height as usize).saturating_sub(7) / 2;
            let mut v = vec![Line::from(""); pad];
            v.extend([
                Line::from(Span::styled("HIGHER  or  LOWER", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled(format!("round {}/{}   wins {}", g.round, GAME_ROUNDS, g.wins), Style::new().fg(DIM))),
                Line::from(""),
                Line::from(Span::styled(format!("  {}  ", g.current), Style::new().fg(Color::Rgb(20, 20, 30)).bg(WARN).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(Span::styled(g.note.clone(), Style::new().fg(TEXT))),
            ]);
            v
        }
        Mode::Normal => sprites::render(p, t, app.current_fx(), inner.width as usize, inner.height as usize),
    };
    f.render_widget(Paragraph::new(body).alignment(Alignment::Center).block(room_block), cols[0]);

    // Stats
    let discipline = bar(p.discipline, 10, if p.discipline >= 60 { GOOD } else { WARN });
    let mut stats = vec![
        stat_row("Hunger", hearts(p.hunger)),
        stat_row("Happy", hearts(p.happy)),
        stat_row("Discipline", discipline),
        stat_row("Weight", vec![Span::styled(format!("{}", p.weight), Style::new().fg(TEXT))]),
        stat_row(
            "Mistakes",
            vec![Span::styled(
                p.mistakes.to_string(),
                Style::new().fg(if p.mistakes == 0 { GOOD } else if p.mistakes < 4 { WARN } else { BAD }),
            )],
        ),
        Line::from(""),
    ];
    let mut chip = |text: &str, color: Color| stats.push(Line::from(Span::styled(format!("● {text}"), Style::new().fg(color))));
    if p.asleep {
        chip("sleeping", Color::Rgb(130, 150, 255));
    }
    if p.sick {
        chip("sick - give medicine", GOOD);
    }
    if p.disc_call.is_some() {
        chip("naughty - scold it", BAD);
    }
    if p.poops > 0 {
        chip("needs cleaning", WARN);
    }
    if p.hunger == 0 && !p.asleep {
        chip("starving!", BAD);
    }
    if p.happy == 0 && !p.asleep {
        chip("lonely", BAD);
    }
    stats.push(Line::from(""));
    stats.push(Line::from(Span::styled("recent", Style::new().fg(DIM).add_modifier(Modifier::ITALIC))));
    let room_left = (cols[1].height as usize).saturating_sub(2 + stats.len());
    for e in app.log.iter().rev().take(room_left).rev() {
        stats.push(Line::from(Span::styled(format!("· {e}"), Style::new().fg(DIM))));
    }
    f.render_widget(Paragraph::new(stats).wrap(Wrap { trim: true }).block(panel("status")), cols[1]);

    // Hint: what to do right now
    let need = p.top_need();
    let hint = match need {
        Some((k, text)) => Line::from(vec![
            Span::styled("→ ", Style::new().fg(WARN)),
            Span::styled(format!("press {k}: "), Style::new().fg(WARN).add_modifier(Modifier::BOLD)),
            Span::styled(text, Style::new().fg(TEXT)),
        ]),
        None if p.stage == Stage::Egg => Line::from(Span::styled("It's an egg - give it a moment to hatch.", Style::new().fg(DIM))),
        None if p.asleep => Line::from(Span::styled("Shh, it's sleeping.", Style::new().fg(DIM))),
        None => Line::from(Span::styled("All good! Play a game (p) to keep it happy.", Style::new().fg(GOOD))),
    };
    f.render_widget(Paragraph::new(hint).alignment(Alignment::Center), rows[2]);

    // Message
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("▸ ", Style::new().fg(ACCENT)),
            Span::styled(app.message.clone(), Style::new().fg(TEXT)),
        ]))
        .block(panel("message")),
        rows[3],
    );

    // Menu
    let mut spans: Vec<Span<'static>> = Vec::new();
    let items: &[(&str, &str)] = match app.mode {
        Mode::Game(_) => &[("h", "higher"), ("l", "lower"), ("esc", "quit game")],
        Mode::Normal if p.stage == Stage::Dead => &[("n", "new egg"), ("q", "quit")],
        Mode::Normal => &[("f", "feed"), ("s", "snack"), ("p", "play"), ("l", "light"), ("m", "medicine"), ("c", "clean"), ("d", "scold"), ("q", "quit")],
    };
    let useful = |k: &str| match k {
        "f" => p.hunger < MAX_HEARTS,
        "s" => p.happy < MAX_HEARTS,
        "m" => p.sick,
        "c" => p.poops > 0,
        "d" => p.disc_call.is_some(),
        _ => true,
    };
    let urgent = need.map(|n| n.0.to_string());
    let ink = Color::Rgb(20, 20, 30);
    for (k, l) in items {
        let mut chip = key_chip(k, l);
        if urgent.as_deref() == Some(*k) {
            chip[0].style = Style::new().fg(ink).bg(WARN).add_modifier(Modifier::BOLD);
            chip[1].style = Style::new().fg(WARN).add_modifier(Modifier::BOLD);
        } else if !useful(k) {
            chip[0].style = Style::new().fg(ink).bg(EMPTY);
            chip[1].style = Style::new().fg(EMPTY);
        }
        spans.extend(chip);
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center).block(
            Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::new().fg(BORDER)),
        ),
        rows[4],
    );
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
        for line in sprites::render(p, 0, None, 32, 10) {
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
