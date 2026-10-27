mod cli;
mod config;
mod games;
mod pet;
mod save;
mod sprites;
mod sysinfo;

use std::time::{Duration, Instant};

use config::Config;
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
/// Most shell events applied at once, so a runaway script cannot stall the game.
const MAX_QUEUED_EVENTS: usize = 200;

pub fn fmt_age(secs: u64) -> String {
    let (d, h, m) = (secs / 86_400, secs / 3600 % 24, secs / 60 % 60);
    if d > 0 { format!("{d}d {h}h {m}m") } else if h > 0 { format!("{h}h {m}m") } else { format!("{m}m {}s", secs % 60) }
}

enum Mode {
    Normal,
    Chooser,
    Game(games::Game),
    Graveyard(Vec<String>),
}

struct App {
    pet: Pet,
    mode: Mode,
    message: String,
    log: Vec<String>,
    cfg: Config,
    tz: i64,
    speed: f64,
    tick_acc: f64,
    last: Instant,
    last_save: Instant,
    last_event_check: Instant,
    last_sys_check: Instant,
    dev_load_override: bool,
    was_calling: bool,
    started: Instant,
    fx: Option<(sprites::Fx, Instant)>,
    last_stage: Stage,
    confirm_reset: bool,
    dev: bool,
    dev_open: bool,
    quit: bool,
}

impl App {
    fn new() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let has = |f: &str| args.iter().any(|a| a == f);
        let mut speed = std::env::var("TAMA_SPEED").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0);
        if has("--fast") {
            speed = 60.0;
        }
        let dev = has("--dev");
        if has("--fast") || dev {
            save::use_test_slot();
        }
        if save::claim_game().is_err() {
            eprintln!("tama is already open in another window.");
            std::process::exit(1);
        }
        let (cfg, cfg_warning) = Config::load();
        let tz = save::tz_offset();
        let (pet, away, note) = match save::load() {
            save::Loaded::Pet(p, away) => (p, away, "Welcome back!".to_string()),
            save::Loaded::None => (Pet::new(save::now()), 0, "A new egg! Take good care of it.".to_string()),
            save::Loaded::Corrupt(path) => (
                Pet::new(save::now()),
                0,
                format!("Your save was unreadable, so it was moved to {} and a new egg started.", path.display()),
            ),
        };
        let mut app = App::from_parts(pet, cfg.clone(), tz, speed, dev, cfg_warning.unwrap_or(note));
        app.pet.set_env(cfg, tz);
        if app.pet.clock == 0 {
            // Save from before the clock existed.
            app.pet.clock = (save::now() - away) as i64;
        }
        let ticks = ((away as f64 * speed) as u64).min(MAX_CATCHUP_TICKS);
        app.pet.away = true;
        for _ in 0..ticks {
            app.pet.tick();
        }
        app.pet.away = false;
        if speed == 1.0 {
            app.pet.clock = save::now() as i64;
        }
        if ticks > 0 {
            app.message = format!("Welcome back! You were away {}.", fmt_age(away));
        }
        app.pet.load = sysinfo::read();
        app.apply_queued_events();
        app.drain_events();
        app.last_stage = app.pet.stage;
        save::heartbeat();
        app
    }

    fn from_parts(pet: Pet, cfg: Config, tz: i64, speed: f64, dev: bool, message: String) -> Self {
        App {
            pet,
            mode: Mode::Normal,
            message,
            log: Vec::new(),
            cfg,
            tz,
            speed,
            tick_acc: 0.0,
            last: Instant::now(),
            last_save: Instant::now(),
            last_event_check: Instant::now(),
            last_sys_check: Instant::now(),
            dev_load_override: false,
            was_calling: false,
            started: Instant::now(),
            fx: None,
            last_stage: Stage::Egg,
            confirm_reset: false,
            dev,
            dev_open: false,
            quit: false,
        }
    }

    /// Apply things that happened in the shell (git commits, failed commands).
    fn apply_queued_events(&mut self) {
        let events = save::take_events();
        if events.is_empty() {
            return;
        }
        let before = self.snapshot();
        for e in events.iter().take(MAX_QUEUED_EVENTS) {
            if let Some(m) = self.pet.apply_event(e) {
                self.message = m.clone();
                self.log.push(m);
            }
        }
        self.react(before);
    }

    fn fresh_pet(&self) -> Pet {
        let mut p = Pet::new(save::now());
        p.set_env(self.cfg.clone(), self.tz);
        p
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
        if self.last_event_check.elapsed() >= Duration::from_secs(2) {
            self.last_event_check = Instant::now();
            self.apply_queued_events();
        }
        if !self.dev_load_override && self.last_sys_check.elapsed() >= Duration::from_secs(5) {
            self.last_sys_check = Instant::now();
            self.pet.load = sysinfo::read();
        }
        self.drain_events();
        if self.pet.stage != self.last_stage {
            if self.pet.alive() {
                self.trigger(sprites::Fx::Evolve);
            }
            self.last_stage = self.pet.stage;
        }
        if matches!(self.mode, Mode::Game(_) | Mode::Chooser) && (self.pet.asleep || !self.pet.alive()) {
            self.mode = Mode::Normal;
        }
        self.tick_game();
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
            save::heartbeat();
            self.last_save = Instant::now();
        }
    }

    fn tick_game(&mut self) {
        let Mode::Game(g) = &mut self.mode else { return };
        let outcome = g.tick(&mut self.pet);
        self.handle_outcome(outcome);
    }

    fn handle_outcome(&mut self, outcome: games::Outcome) {
        match outcome {
            games::Outcome::Continue => {}
            games::Outcome::Quit => {
                self.mode = Mode::Normal;
                self.message = "Game abandoned.".into();
            }
            games::Outcome::Done { wins, rounds } => {
                let before = self.snapshot();
                self.message = self.pet.finish_game(wins, rounds);
                self.mode = Mode::Normal;
                self.react(before);
            }
        }
    }

    fn on_key(&mut self, code: KeyCode) {
        match &mut self.mode {
            Mode::Game(g) => {
                let outcome = g.key(code, &mut self.pet);
                self.handle_outcome(outcome);
                return;
            }
            Mode::Chooser => {
                match code {
                    KeyCode::Char(c @ '1'..='4') => {
                        let g = games::Game::new(c as u8 - b'0', &mut self.pet);
                        self.mode = Mode::Game(g);
                    }
                    KeyCode::Esc | KeyCode::Char('q') => self.mode = Mode::Normal,
                    _ => {}
                }
                return;
            }
            Mode::Graveyard(_) => {
                self.mode = Mode::Normal;
                return;
            }
            Mode::Normal => {}
        }
        if self.confirm_reset {
            self.confirm_reset = false;
            if code == KeyCode::Char('y') {
                self.pet = self.fresh_pet();
                self.log.clear();
                self.fx = None;
                self.last_stage = Stage::Egg;
                self.message = "Reset! A fresh egg appears.".into();
                let _ = save::save(&self.pet);
            } else {
                self.message = "Reset cancelled.".into();
            }
            return;
        }
        if self.dev && code == KeyCode::Char('`') {
            self.dev_open = !self.dev_open;
            return;
        }
        if self.dev_open && self.on_dev_key(code) {
            return;
        }
        let before = self.snapshot();
        let msg = match code {
            KeyCode::Char('R') => {
                self.confirm_reset = true;
                self.message = "Reset your pet? Press y to confirm, any other key to cancel.".into();
                return;
            }
            KeyCode::Char('f') => self.pet.feed_meal(),
            KeyCode::Char('s') => self.pet.feed_snack(),
            KeyCode::Char('p') => {
                match self.pet.can_play() {
                    Some(m) => self.message = m.into(),
                    None => self.mode = Mode::Chooser,
                }
                return;
            }
            KeyCode::Char('g') => {
                self.mode = Mode::Graveyard(save::graveyard());
                return;
            }
            KeyCode::Char('m') => self.pet.medicine(),
            KeyCode::Char('c') => self.pet.clean(),
            KeyCode::Char('d') => self.pet.scold(),
            KeyCode::Char('l') => self.pet.toggle_light(),
            KeyCode::Char('n') if !self.pet.alive() => {
                self.pet = self.fresh_pet();
                self.last_stage = Stage::Egg;
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

    /// Developer keys, active only with `--dev` while the panel is open. Returns true if handled.
    fn on_dev_key(&mut self, code: KeyCode) -> bool {
        let p = &mut self.pet;
        let step = |v: u8, up: bool, max: u8| if up { v.saturating_add(1).min(max) } else { v.saturating_sub(1) };
        let KeyCode::Char(c) = code else { return false };
        let note = match c {
            'h' | 'H' => {
                p.hunger = step(p.hunger, c == 'H', MAX_HEARTS);
                format!("hunger = {}", p.hunger)
            }
            'a' | 'A' => {
                p.happy = step(p.happy, c == 'A', MAX_HEARTS);
                format!("happy = {}", p.happy)
            }
            'd' | 'D' => {
                p.discipline = if c == 'D' { p.discipline.saturating_add(10).min(100) } else { p.discipline.saturating_sub(10) };
                format!("discipline = {}", p.discipline)
            }
            'w' | 'W' => {
                p.weight = step(p.weight, c == 'W', 99);
                format!("weight = {}", p.weight)
            }
            'm' | 'M' => {
                p.mistakes = if c == 'M' { p.mistakes + 1 } else { p.mistakes.saturating_sub(1) };
                format!("mistakes = {}", p.mistakes)
            }
            'p' => {
                p.poops = (p.poops + 1).min(3);
                format!("poops = {}", p.poops)
            }
            's' => {
                p.dev_sick();
                "made sick".into()
            }
            'n' => {
                p.dev_naughty();
                "naughty call started".into()
            }
            'g' => {
                p.dev_grow();
                "growing at next tick".into()
            }
            'k' => {
                p.dev_kill();
                "killed".into()
            }
            'u' | 'r' | 'b' => {
                match c {
                    'u' => p.load.busy_cpu = !p.load.busy_cpu,
                    'r' => p.load.low_ram = !p.load.low_ram,
                    _ => p.load.low_battery = !p.load.low_battery,
                }
                self.dev_load_override = true;
                format!("cpu {}  ram {}  battery {} (auto-detect off)", p.load.busy_cpu, p.load.low_ram, p.load.low_battery)
            }
            't' | 'T' => {
                let secs = if c == 'T' { 8 * 3600 } else { 3600 };
                for _ in 0..secs {
                    p.tick();
                }
                format!("skipped {}h", secs / 3600)
            }
            'z' => {
                p.dev_set_hour(22);
                "clock set to 22:00".into()
            }
            'Z' => {
                p.dev_set_hour(12);
                "clock set to 12:00".into()
            }
            'f' => {
                self.speed = match self.speed as u32 {
                    1 => 10.0,
                    10 => 60.0,
                    60 => 600.0,
                    _ => 1.0,
                };
                format!("speed = x{}", self.speed)
            }
            _ => return false,
        };
        self.message = format!("[dev] {note}");
        true
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
    key_chip_sized(key, label, 0)
}

/// A shorter label for narrow terminals.
fn short_label(label: &str) -> &str {
    match label {
        "medicine" => "med",
        "graveyard" => "grave",
        "new egg" => "new",
        "quit game" => "quit",
        "repeat the pattern" => "repeat",
        "move paddle" => "move",
        "higher/lower" => "hi/lo",
        other => other,
    }
}

/// `size` 0 is the full chip, 1 uses a short label with less padding, 2 is the key alone.
fn key_chip_sized(key: &str, label: &str, size: u8) -> Vec<Span<'static>> {
    let key_span = Span::styled(format!(" {key} "), Style::new().fg(Color::Rgb(20, 20, 30)).bg(ACCENT).add_modifier(Modifier::BOLD));
    let label_span = match size {
        0 => Span::styled(format!(" {label}  "), Style::new().fg(TEXT)),
        1 => Span::styled(format!("{} ", short_label(label)), Style::new().fg(TEXT)),
        _ => Span::styled(" ", Style::new().fg(TEXT)),
    };
    vec![key_span, label_span]
}

/// The smallest window the layout is designed for.
const MIN_W: u16 = 64;
const MIN_H: u16 = 22;

fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    if area.width < MIN_W || area.height < MIN_H {
        let lines = vec![
            Line::from(Span::styled("tama needs a bigger window", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled(format!("at least {MIN_W}x{MIN_H}, this one is {}x{}", area.width, area.height), Style::new().fg(TEXT))),
            Line::from(Span::styled("(q quits)", Style::new().fg(DIM))),
        ];
        let pad = (area.height as usize).saturating_sub(lines.len()) / 2;
        let mut v = vec![Line::from(""); pad];
        v.extend(lines);
        f.render_widget(Paragraph::new(v).alignment(Alignment::Center).wrap(Wrap { trim: true }), area);
        return;
    }
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
    let mut header = header;
    if p.stage != Stage::Egg && p.alive() {
        header.spans.push(Span::styled(format!("  ·  {}", p.quirk.label()), Style::new().fg(ACCENT)));
    }
    let mut header = header;
    if app.dev {
        header.spans.push(Span::styled("   DEV", Style::new().fg(Color::Rgb(20, 20, 30)).bg(WARN).add_modifier(Modifier::BOLD)));
    }
    f.render_widget(Paragraph::new(header).block(panel("tamagotchi")), rows[0]);

    let cols = Layout::horizontal([Constraint::Min(34), Constraint::Length(30)]).split(rows[1]);

    // Room
    let room_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(BORDER));
    let inner = room_block.inner(cols[0]);
    let t = (app.started.elapsed().as_millis() / 350) as u32;
    let centered = |lines: Vec<Line<'static>>| -> Vec<Line<'static>> {
        let pad = (inner.height as usize).saturating_sub(lines.len()) / 2;
        let mut v = vec![Line::from(""); pad];
        v.extend(lines);
        v
    };
    let body: Vec<Line> = match &app.mode {
        Mode::Game(g) => centered(g.lines()),
        Mode::Chooser => {
            let mut v = vec![
                Line::from(Span::styled("PICK A GAME", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))),
                Line::from(""),
            ];
            for (k, l) in games::CHOICES {
                v.push(Line::from(key_chip(k, l)));
            }
            centered(v)
        }
        Mode::Graveyard(entries) => {
            let mut v = vec![Line::from(Span::styled("GRAVEYARD", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))), Line::from("")];
            if entries.is_empty() {
                v.push(Line::from(Span::styled("No pets have passed away yet.", Style::new().fg(DIM))));
            }
            for e in entries.iter().take((inner.height as usize).saturating_sub(4)) {
                v.push(Line::from(Span::styled(e.clone(), Style::new().fg(TEXT))));
            }
            v.push(Line::from(""));
            v.push(Line::from(Span::styled("press any key to go back", Style::new().fg(DIM))));
            centered(v)
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
        stat_row(
            "Weight",
            vec![if p.overweight() {
                Span::styled(format!("{} (heavy)", p.weight), Style::new().fg(WARN))
            } else {
                Span::styled(format!("{}", p.weight), Style::new().fg(TEXT))
            }],
        ),
        stat_row(
            "Mistakes",
            vec![Span::styled(
                p.mistakes.to_string(),
                Style::new().fg(if p.mistakes == 0 { GOOD } else if p.mistakes < 4 { WARN } else { BAD }),
            )],
        ),
        stat_row("Trait", vec![Span::styled(p.quirk.label(), Style::new().fg(ACCENT))]),
        Line::from(Span::styled(format!("  {}", p.quirk.description()), Style::new().fg(DIM))),
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
    if p.overweight() {
        chip("overweight - play games", WARN);
    }
    if p.load.busy_cpu {
        chip("sweating - the CPU is busy", Color::Rgb(110, 190, 255));
    }
    if p.load.low_ram {
        chip("dizzy - memory is nearly full", Color::Rgb(190, 150, 255));
    }
    if p.load.low_battery {
        chip("sleepy - the battery is low", Color::Rgb(130, 150, 255));
    }
    if p.hunger == 0 && !p.asleep {
        chip("starving!", BAD);
    }
    if p.happy == 0 && !p.asleep {
        chip("lonely", BAD);
    }
    stats.push(Line::from(""));
    if app.dev_open {
        let help = [
            format!("dev   x{}  {:02}h", app.speed, p.local_hour()),
            "h/H hunger -/+".into(),
            "a/A happy -/+".into(),
            "d/D discipline -/+10".into(),
            "w/W weight -/+".into(),
            "m/M mistakes -/+".into(),
            "p poop   s sick".into(),
            "n naughty  g grow".into(),
            "k kill".into(),
            "u cpu  r ram  b battery".into(),
            "t/T skip 1h / 8h".into(),
            "z 22:00  Z 12:00".into(),
            "f speed  ` close".into(),
        ];
        for h in help {
            stats.push(Line::from(Span::styled(h, Style::new().fg(WARN))));
        }
    } else {
        let preview = p.evolution_preview();
        if !preview.is_empty() {
            stats.push(Line::from(Span::styled("growth", Style::new().fg(DIM).add_modifier(Modifier::ITALIC))));
            for l in preview {
                stats.push(Line::from(Span::styled(l, Style::new().fg(TEXT))));
            }
            stats.push(Line::from(""));
        }
        stats.push(Line::from(Span::styled("recent", Style::new().fg(DIM).add_modifier(Modifier::ITALIC))));
        let room_left = (cols[1].height as usize).saturating_sub(2 + stats.len());
        for e in app.log.iter().rev().take(room_left).rev() {
            stats.push(Line::from(Span::styled(format!("· {e}"), Style::new().fg(DIM))));
        }
    }
    f.render_widget(Paragraph::new(stats).wrap(Wrap { trim: true }).block(panel("status")), cols[1]);

    // Hint: what to do right now
    let need = p.top_need();
    let hint = match need {
        Some((k, text)) => Line::from(vec![
            Span::styled("→ ", Style::new().fg(WARN)),
            Span::styled(format!("PRESS {}: ", k.to_ascii_uppercase()), Style::new().fg(WARN).add_modifier(Modifier::BOLD)),
            Span::styled(text.to_uppercase(), Style::new().fg(TEXT)),
        ]),
        None if p.stage == Stage::Egg => Line::from(Span::styled("IT'S AN EGG - GIVE IT A MOMENT TO HATCH.", Style::new().fg(DIM))),
        None => Line::from(""),
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
    let items: &[(&str, &str)] = match &app.mode {
        Mode::Game(g) => g.keys(),
        Mode::Chooser => &[("1-4", "choose"), ("esc", "back")],
        Mode::Graveyard(_) => &[("any key", "back")],
        Mode::Normal if p.stage == Stage::Dead => &[("n", "new egg"), ("g", "graveyard"), ("q", "quit")],
        Mode::Normal => &[("f", "feed"), ("s", "snack"), ("p", "play"), ("l", "light"), ("m", "medicine"), ("c", "clean"), ("d", "scold"), ("g", "graveyard"), ("q", "quit")],
    };
    let useful = |k: &str| match k {
        "f" => p.hunger < MAX_HEARTS,
        "s" => p.happy < MAX_HEARTS,
        "m" => p.sick,
        "c" => p.poops > 0,
        "d" => p.disc_call.is_some(),
        "l" => p.asleep,
        _ => true,
    };
    let urgent = if matches!(app.mode, Mode::Normal) { need.map(|n| n.0.to_string()) } else { None };
    let ink = Color::Rgb(20, 20, 30);
    let room = rows[4].width.saturating_sub(2) as usize;
    for size in 0..=2u8 {
        spans.clear();
        for (k, l) in items {
            let mut chip = key_chip_sized(k, l, size);
            if urgent.as_deref() == Some(*k) {
                chip[0].style = Style::new().fg(ink).bg(WARN).add_modifier(Modifier::BOLD);
                chip[1].style = Style::new().fg(WARN).add_modifier(Modifier::BOLD);
            } else if !useful(k) {
                chip[0].style = Style::new().fg(ink).bg(EMPTY);
                chip[1].style = Style::new().fg(EMPTY);
            }
            spans.extend(chip);
        }
        if Line::from(spans.clone()).width() <= room {
            break;
        }
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
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(k) = event::read()? {
                if k.kind == KeyEventKind::Press {
                    app.on_key(k.code);
                }
            }
        }
    }
    Ok(())
}

/// `--gallery`: print every character once, following the `animate_parts` setting
/// (the ears, wings, tail etc. only appear when it is on).
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
    let (cfg, _) = Config::load();
    println!("Animated parts (ears, wings, tuft, leaves, tail): {}  (change with `tama --animate on|off`)\n", if cfg.animate_parts { "on" } else { "off" });
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
    dead.species = "Departed".into();
    pets.push(dead);
    let ansi = |spans: &[Span]| -> String {
        let mut out = String::new();
        for s in spans {
            if let Some(Color::Rgb(r, g, b)) = s.style.fg {
                out.push_str(&format!("\x1b[38;2;{r};{g};{b}m"));
            }
            if let Some(Color::Rgb(r, g, b)) = s.style.bg {
                out.push_str(&format!("\x1b[48;2;{r};{g};{b}m"));
            }
            out.push_str(&s.content);
            out.push_str("\x1b[0m");
        }
        out
    };
    for p in &mut pets {
        p.set_env(cfg.clone(), 0);
        println!("-- {} --", p.species);
        for line in sprites::render_still(p, 0, None, 32, 10) {
            println!("{}", ansi(&line.spans));
        }
    }
}

/// GitHub repo (`owner/name`) that `tama --update` installs from.
/// Not set up yet; until it is, `--update` rebuilds from the local source folder instead.
const REPO: Option<&str> = None;

/// `--update`: reinstall the newest version with cargo.
fn update() -> i32 {
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["install", "--force"]);
    match REPO {
        Some(repo) => {
            cmd.args(["--git", &format!("https://github.com/{repo}")]);
        }
        None => {
            let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            if !src.join("Cargo.toml").exists() {
                eprintln!("Source folder {} no longer exists, and no GitHub repo is configured yet.", src.display());
                return 1;
            }
            println!("Rebuilding from {}", src.display());
            cmd.arg("--path").arg(src);
        }
    }
    match cmd.status() {
        Ok(s) if s.success() => 0,
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("Could not run cargo ({e}). Is Rust installed and on your PATH?");
            1
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(code) = cli::run(&args) {
        std::process::exit(code);
    }
    if args.iter().any(|a| a == "--update") {
        std::process::exit(update());
    }
    if let Some(i) = args.iter().position(|a| a == "--animate") {
        match args.get(i + 1).map(String::as_str) {
            Some(v @ ("on" | "off")) => match Config::set_animate_parts(v == "on") {
                Ok(()) => println!("Animated parts (ears, wings, tuft, leaves, tail) are now {v}. Restart tama to see it."),
                Err(e) => {
                    eprintln!("Could not change the setting: {e}");
                    std::process::exit(1);
                }
            },
            _ => {
                let state = if Config::load().0.animate_parts { "on" } else { "off" };
                println!("Animated parts (ears, wings, tuft, leaves, tail) are {state}. Use `tama --animate on` or `tama --animate off`.");
            }
        }
        return Ok(());
    }
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("tama {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("tama - a terminal tamagotchi\n\n  tama            play\n  tama status     one line for a status bar (tmux: set -g status-right '#(tama status)'); --waybar prints JSON with a short text and a hover tooltip\n  tama feed|snack|clean|medicine|scold|light   care for the pet without opening the game\n  tama hook zsh   shell snippet: git commits feed the pet, failed commands make it sad (eval \"$(tama hook zsh)\")\n  tama watch      desktop notifications when the pet needs you (run it in the background; `tama watch --test` sends a test)\n  tama --update   reinstall the newest version\n  tama --animate on|off   show/hide the animated ears, wings, tail\n  tama --version  print the version\n  tama --dev      developer mode (separate test save)\n  tama --fast     60x speed (separate test save)\n  tama --gallery  print all character sprites");
        return Ok(());
    }
    if args.iter().any(|a| a == "--gallery") {
        gallery();
        return Ok(());
    }
    let mut app = App::new();
    let mut terminal = ratatui::init();
    let res = run(&mut terminal, &mut app);
    ratatui::restore();
    let _ = save::save(&app.pet);
    save::release();
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn app_with(stage: Stage, tweak: impl FnOnce(&mut Pet)) -> App {
        let mut pet = Pet::new(7);
        pet.stage = stage;
        pet.species = match stage {
            Stage::Egg => "Egg",
            Stage::Baby => "Babytchi",
            Stage::Child => "Marutchi",
            Stage::Teen => "Tonmarutchi",
            Stage::Adult => "Mametchi",
            Stage::Dead => "Marutchi",
        }
        .into();
        tweak(&mut pet);
        App::from_parts(pet, Config::default(), 0, 1.0, false, "hello".into())
    }

    fn render(app: &App, w: u16, h: u16) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        let buf = terminal.backend().buffer().clone();
        (0..h).map(|y| (0..w).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>()).collect()
    }

    #[test]
    fn every_stage_and_state_draws_at_any_size_without_panicking() {
        let stages = [Stage::Egg, Stage::Baby, Stage::Child, Stage::Teen, Stage::Adult, Stage::Dead];
        let sizes = [(200, 60), (120, 40), (100, 30), (80, 24), (64, 22), (63, 22), (64, 21), (40, 12), (10, 3), (1, 1)];
        for stage in stages {
            for tweak in 0..4 {
                let app = app_with(stage, |p| match tweak {
                    1 => p.sick = true,
                    2 => p.asleep = true,
                    3 => {
                        p.hunger = 0;
                        p.poops = 3;
                        p.disc_call = Some(0);
                    }
                    _ => {}
                });
                for (w, h) in sizes {
                    render(&app, w, h);
                }
            }
        }
    }

    #[test]
    fn the_menu_stays_fully_visible_on_an_80_column_terminal() {
        let app = app_with(Stage::Adult, |_| {});
        let screen = render(&app, 80, 24);
        let menu = &screen[22];
        for key in ["f", "s", "p", "l", "m", "c", "d", "g", "q"] {
            assert!(menu.contains(&format!(" {key} ")), "missing the {key} key in: {menu}");
        }
        assert!(menu.contains("feed") && menu.contains("quit"), "labels should still show at 80 columns: {menu}");
    }

    #[test]
    fn a_wide_terminal_gets_the_full_menu_labels() {
        let app = app_with(Stage::Adult, |_| {});
        let screen = render(&app, 140, 40);
        assert!(screen[38].contains("medicine") && screen[38].contains("graveyard"), "{}", screen[38]);
    }

    #[test]
    fn the_status_panel_names_what_the_machine_is_doing_to_the_pet() {
        let app = app_with(Stage::Adult, |p| p.load = sysinfo::Strain { busy_cpu: true, low_ram: true, low_battery: true });
        let screen = render(&app, 140, 40).join("\n");
        assert!(screen.contains("sweating") && screen.contains("dizzy") && screen.contains("sleepy"), "{screen}");
        let calm = render(&app_with(Stage::Adult, |_| {}), 140, 40).join("\n");
        assert!(!calm.contains("sweating") && !calm.contains("dizzy"));
    }

    #[test]
    fn a_window_that_is_too_small_says_so() {
        let app = app_with(Stage::Adult, |_| {});
        let screen = render(&app, 40, 12).join("\n");
        assert!(screen.contains("bigger window"), "{screen}");
        assert!(screen.contains("40x12"), "{screen}");
        let ok = render(&app, 64, 22).join("\n");
        assert!(!ok.contains("bigger window"));
    }
}
