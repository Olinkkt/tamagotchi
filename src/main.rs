mod cli;
mod config;
mod games;
mod pet;
mod profile;
mod save;
mod sprites;
mod sysinfo;
mod ui;

use std::time::{Duration, Instant};

use config::Config;
use profile::Profile;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use pet::{Pet, Stage, MAX_HEARTS};
use ratatui::DefaultTerminal;

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
    Shop,
    Awards,
}

struct App {
    pet: Pet,
    mode: Mode,
    message: String,
    log: Vec<String>,
    cfg: Config,
    profile: Profile,
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
        app.profile = save::load_profile();
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
            profile: Profile::default(),
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

    /// Buy something from the shop and use it on the pet. Nothing is charged if it would do nothing.
    fn buy(&mut self, item: pet::Item) {
        if self.profile.coins < item.price() {
            self.message = format!("Not enough coins: {} costs {}.", item.name(), item.price());
            return;
        }
        let before = self.snapshot();
        match self.pet.use_item(item) {
            Ok(msg) => {
                self.profile.coins -= item.price();
                self.message = format!("{msg} (-{} coins)", item.price());
                self.react(before);
            }
            Err(msg) => self.message = msg,
        }
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
        for a in self.profile.check(&self.pet) {
            let note = format!("Achievement unlocked: {} - {}", a.name, a.how);
            self.message = note.clone();
            self.log.push(note);
            let _ = save::save_profile(&self.profile);
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
            let _ = save::save_profile(&self.profile);
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
                let coins = profile::game_reward(wins, rounds);
                if coins > 0 {
                    self.profile.add_coins(coins);
                    self.profile.games_won += 1;
                    self.message.push_str(&format!(" +{coins} coins."));
                }
                self.mode = Mode::Normal;
                self.react(before);
            }
        }
    }

    fn on_key(&mut self, code: KeyCode) {
        match &mut self.mode {
            Mode::Game(g) => {
                let outcome = g.key(code);
                self.handle_outcome(outcome);
                return;
            }
            Mode::Chooser => {
                match code {
                    KeyCode::Char(c @ '1'..='6') => {
                        let g = games::Game::new(c as u8 - b'0', &mut self.pet);
                        self.mode = Mode::Game(g);
                    }
                    KeyCode::Esc | KeyCode::Char('q') => self.mode = Mode::Normal,
                    _ => {}
                }
                return;
            }
            Mode::Shop => {
                match code {
                    KeyCode::Char(c @ '1'..='4') => self.buy(pet::Item::ALL[(c as u8 - b'1') as usize]),
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('b') => self.mode = Mode::Normal,
                    _ => {}
                }
                return;
            }
            Mode::Awards | Mode::Graveyard(_) => {
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
            KeyCode::Char('b') => {
                self.mode = Mode::Shop;
                return;
            }
            KeyCode::Char('a') => {
                self.mode = Mode::Awards;
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
            'o' => {
                self.profile.add_coins(10);
                format!("coins = {}", self.profile.coins)
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

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    while !app.quit {
        app.update();
        terminal.draw(|f| ui::draw(f, app))?;
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

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(code) = cli::run(&args).or_else(|| cli::flags(&args)) {
        std::process::exit(code);
    }
    let mut app = App::new();
    let mut terminal = ratatui::init();
    let res = run(&mut terminal, &mut app);
    ratatui::restore();
    let _ = save::save(&app.pet);
    let _ = save::save_profile(&app.profile);
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
        terminal.draw(|f| ui::draw(f, app)).unwrap();
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
        for key in ["f", "s", "p", "l", "m", "c", "d", "b", "a", "g", "q"] {
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
    fn buying_charges_the_coins_and_uses_the_item() {
        let mut app = app_with(Stage::Adult, |p| p.hunger = 1);
        app.profile.coins = 10;
        app.buy(pet::Item::FancyMeal);
        assert_eq!((app.profile.coins, app.pet.hunger), (2, 3));
        app.buy(pet::Item::Vitamins);
        assert_eq!(app.profile.coins, 2, "12 coins is more than it has");
        assert!(app.message.contains("Not enough coins"), "{}", app.message);
    }

    #[test]
    fn nothing_is_charged_for_an_item_that_would_do_nothing() {
        let mut app = app_with(Stage::Adult, |p| p.hunger = 4);
        app.profile.coins = 20;
        app.buy(pet::Item::FancyMeal);
        assert_eq!((app.profile.coins, app.pet.hunger), (20, 4));
        assert!(app.message.contains("Not hungry"), "{}", app.message);
    }

    #[test]
    fn winning_a_mini_game_pays_coins_and_losing_pays_nothing() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.handle_outcome(games::Outcome::Done { wins: 5, rounds: 5 });
        assert_eq!((app.profile.coins, app.profile.games_won), (profile::COINS_PERFECT, 1));
        assert!(app.message.contains("coins"), "{}", app.message);
        app.handle_outcome(games::Outcome::Done { wins: 1, rounds: 5 });
        assert_eq!((app.profile.coins, app.profile.games_won), (profile::COINS_PERFECT, 1));
    }

    #[test]
    fn the_shop_and_awards_screens_show_their_contents() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.profile.coins = 7;
        app.mode = Mode::Shop;
        let shop = render(&app, 100, 30).join("\n");
        for want in ["SHOP", "Fancy meal", "Vitamins", "◎ 7"] {
            assert!(shop.contains(want), "shop is missing {want}:\n{shop}");
        }
        app.mode = Mode::Awards;
        let awards = render(&app, 100, 30).join("\n");
        for want in ["AWARDS", "First steps", "Adults raised 0/5", "Mametchi"] {
            assert!(awards.contains(want), "awards is missing {want}:\n{awards}");
        }
    }

    #[test]
    fn the_awards_screen_fits_a_standard_80x24_terminal() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.mode = Mode::Awards;
        let screen = render(&app, 80, 24).join("\n");
        assert!(screen.contains("Collector") && screen.contains("Nyorotchi"), "{screen}");
    }

    #[test]
    fn the_keys_open_and_close_the_shop_and_awards_and_buy() {
        save::with_temp_dir(|| {
            let mut app = app_with(Stage::Adult, |p| p.hunger = 1);
            app.on_key(KeyCode::Char('b'));
            assert!(matches!(app.mode, Mode::Shop));
            app.profile.coins = 20;
            app.on_key(KeyCode::Char('1'));
            assert_eq!((app.profile.coins, app.pet.hunger), (12, 3));
            assert!(matches!(app.mode, Mode::Shop), "stays open for another purchase");
            app.on_key(KeyCode::Esc);
            assert!(matches!(app.mode, Mode::Normal));
            app.on_key(KeyCode::Char('a'));
            assert!(matches!(app.mode, Mode::Awards));
            app.on_key(KeyCode::Char('x'));
            assert!(matches!(app.mode, Mode::Normal));
        });
    }

    #[test]
    fn the_catch_basket_and_the_pong_court_are_fully_visible_at_the_smallest_size() {
        for (kind, mark) in [(5u8, "╰"), (4u8, "█")] {
            let mut app = app_with(Stage::Adult, |_| {});
            let g = games::Game::new(kind, &mut app.pet);
            app.mode = Mode::Game(g);
            let screen = render(&app, 64, 22).join("\n");
            assert!(screen.contains(mark), "game {kind} lost its {mark} at 64x22:\n{screen}");
            let bottom_wall = "─".repeat(28);
            assert!(screen.contains(&bottom_wall), "game {kind} lost its bottom edge at 64x22:\n{screen}");
        }
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
