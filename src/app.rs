//! The game state and everything the player can do, independent of how it is drawn.
//! The window calls `update` every frame, feeds it keys, and draws whatever it finds here.

use std::time::{Duration, Instant};

use crate::cast;
use crate::config::Config;
use crate::pet::{self, Pet, Stage, MAX_HEARTS};
use crate::profile::{self, Profile};
use crate::wipe::{Next, Wipe};
use crate::{fmt_age, games, save, scene, sysinfo, toilet, MAX_CATCHUP_TICKS, MAX_QUEUED_EVENTS};

/// A key press, in the terms the game cares about. Each frontend translates its own events.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Esc,
    Tab,
}

pub enum Mode {
    Normal,
    Chooser,
    Game(games::Game),
    /// The result screen after a game.
    Finish(games::Finish),
    Graveyard(Vec<String>),
    Shop,
    Awards,
    /// The pet in a washroom booth; nothing can be done until it comes out.
    Toilet(toilet::Visit),
}

/// How long the line of news at the foot of the room stays up.
const MESSAGE_SECS: Duration = Duration::from_secs(5);

pub struct App {
    pub pet: Pet,
    pub mode: Mode,
    pub message: String,
    /// What `message` said when it was last seen, and since when, so old news can fade.
    message_seen: (String, Instant),
    pub log: Vec<String>,
    pub cfg: Config,
    pub profile: Profile,
    pub tz: i64,
    pub speed: f64,
    tick_acc: f64,
    last: Instant,
    last_save: Instant,
    last_event_check: Instant,
    last_sys_check: Instant,
    dev_load_override: bool,
    was_calling: bool,
    pub started: Instant,
    fx: Option<(scene::Fx, Instant)>,
    /// The iris wipe in progress between two scenes; keys do nothing until it is over.
    pub(crate) wipe: Option<Wipe>,
    /// When the egg hatched, so its shell can lie on the floor for a while.
    shell_at: Option<Instant>,
    last_stage: Stage,
    confirm_reset: bool,
    pub dev: bool,
    pub dev_open: bool,
    pub quit: bool,
}

type Snapshot = (u8, u8, u8, bool, bool);

/// Every form the pet can take, in the order the dev panel steps through them.
pub const FORMS: [(Stage, &str); 11] = [
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
    (Stage::Dead, "Departed"),
];

/// The game speeds the dev panel's `f` key steps through, then back to the start.
const SPEEDS: [f64; 6] = [1.0, 2.0, 5.0, 10.0, 60.0, 600.0];

fn next_speed(now: f64) -> f64 {
    let i = SPEEDS.iter().position(|&s| s == now);
    SPEEDS[i.map_or(0, |i| (i + 1) % SPEEDS.len())]
}

/// Turn the pet into `FORMS[i]` straight away (dev panel only).
fn set_form(p: &mut Pet, i: usize) {
    let (stage, species) = FORMS[i];
    if stage == Stage::Dead {
        p.dev_kill();
        return;
    }
    if !p.alive() {
        p.cause_of_death = None;
        p.buried = false;
    }
    p.stage = stage;
    p.species = species.into();
    p.stage_age = 0;
}

impl App {
    pub fn new() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let has = |f: &str| args.iter().any(|a| a == f);
        let speed = std::env::var("TAMA_SPEED").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0);
        let dev = has("--dev");
        if dev {
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

    pub fn from_parts(pet: Pet, cfg: Config, tz: i64, speed: f64, dev: bool, message: String) -> Self {
        App {
            pet,
            mode: Mode::Normal,
            message_seen: (String::new(), Instant::now()),
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
            wipe: None,
            shell_at: None,
            last_stage: Stage::Egg,
            confirm_reset: false,
            dev,
            dev_open: false,
            quit: false,
        }
    }

    /// Save everything and give up the lock. Called once when the game closes.
    pub fn shutdown(&self) {
        let _ = save::save(&self.pet);
        let _ = save::save_profile(&self.profile);
        save::release();
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
    pub fn buy(&mut self, item: pet::Item) {
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

    fn trigger(&mut self, fx: scene::Fx) {
        self.fx = Some((fx, Instant::now()));
    }

    fn snapshot(&self) -> Snapshot {
        let p = &self.pet;
        (p.hunger, p.happy, p.doses, p.sick, p.disc_call.is_some())
    }

    /// Pick an animation by comparing pet state before and after an action.
    fn react(&mut self, before: Snapshot) {
        let after = self.snapshot();
        let fx = if after.0 > before.0 {
            Some(scene::Fx::Eat)
        } else if after.1 > before.1 {
            Some(scene::Fx::Hearts)
        } else if after.2 < before.2 || (before.3 && !after.3) {
            Some(scene::Fx::Heal)
        } else if (before.4 && !after.4) || after.1 < before.1 {
            Some(scene::Fx::Scold)
        } else {
            None
        };
        if let Some(fx) = fx {
            self.trigger(fx);
        }
    }

    /// Waiting for `y` after Shift+R.
    pub fn confirming_reset(&self) -> bool {
        self.confirm_reset
    }

    /// Seconds since the egg hatched, while its shell is still on the floor.
    pub fn shell_age(&self) -> Option<f32> {
        let age = self.shell_at?.elapsed().as_secs_f32();
        (age < scene::SHELL_SECS).then_some(age)
    }

    /// Milliseconds of the pet's current second that have gone by, so the egg moves between ticks.
    pub fn sub_ms(&self) -> u64 {
        (self.tick_acc.clamp(0.0, 0.999) * 1000.0) as u64
    }

    pub fn current_fx(&self) -> Option<(scene::Fx, f32)> {
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

    /// The message line clears itself a few seconds after it last changed (a pending reset prompt stays).
    fn fade_message(&mut self) {
        if self.message != self.message_seen.0 {
            self.message_seen = (self.message.clone(), Instant::now());
        } else if !self.confirm_reset && self.message_seen.1.elapsed() >= MESSAGE_SECS {
            self.message.clear();
            self.message_seen.0.clear();
        }
    }

    pub fn update(&mut self) {
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
        self.fade_message();
        if self.pet.stage != self.last_stage {
            if self.pet.alive() {
                self.trigger(scene::Fx::Evolve);
            }
            if self.last_stage == Stage::Egg && self.pet.stage == Stage::Baby {
                self.shell_at = Some(Instant::now());
            }
            self.last_stage = self.pet.stage;
        }
        if matches!(self.mode, Mode::Game(_) | Mode::Chooser) && (self.pet.asleep || !self.pet.alive()) {
            self.mode = Mode::Normal;
        }
        if self.wipe.is_none() && matches!(&self.mode, Mode::Toilet(v) if v.done()) {
            self.wipe = Some(Wipe::new(Next::Room));
        }
        self.run_wipe();
        self.tick_game();
        if matches!(&self.mode, Mode::Finish(f) if f.expired()) {
            self.mode = Mode::Normal;
        }
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

    pub fn wiping(&self) -> bool {
        self.wipe.is_some()
    }

    pub fn wipe(&self) -> Option<&Wipe> {
        self.wipe.as_ref()
    }

    /// Change scenes at the moment the wipe is shut, and drop it once it has opened again.
    fn run_wipe(&mut self) {
        let Some(wipe) = &mut self.wipe else { return };
        if wipe.swap_now() {
            match wipe.next {
                Next::Toilet(booth) => self.mode = Mode::Toilet(toilet::Visit::new(booth)),
                Next::Room => {
                    self.mode = Mode::Normal;
                    self.trigger(scene::Fx::Hearts);
                }
            }
        }
        if self.wipe.as_ref().is_some_and(Wipe::done) {
            self.wipe = None;
        }
    }

    fn tick_game(&mut self) {
        let Mode::Game(g) = &mut self.mode else { return };
        let outcome = g.tick(&mut self.pet);
        self.handle_outcome(outcome);
    }

    pub fn handle_outcome(&mut self, outcome: games::Outcome) {
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
                self.mode = Mode::Finish(games::Finish::new(&self.pet, wins, rounds, coins));
                self.react(before);
            }
        }
    }

    /// A key from the keyboard. While the dev panel is open, dev keys come first.
    pub fn on_key(&mut self, code: Key) {
        self.handle(code, true);
    }

    /// The direction keys (or buttons) held down this frame; only Pong cares.
    pub fn hold(&mut self, up: bool, down: bool) {
        if let Mode::Game(g) = &mut self.mode {
            g.hold(up, down);
        }
    }

    /// A button clicked with the mouse: always the action itself, never a dev key.
    pub fn press(&mut self, code: Key) {
        self.handle(code, false);
    }

    fn handle(&mut self, code: Key, dev_keys: bool) {
        if self.wipe.is_some() {
            return;
        }
        match &mut self.mode {
            Mode::Game(g) => {
                let outcome = g.key(code);
                self.handle_outcome(outcome);
                return;
            }
            Mode::Chooser => {
                match code {
                    Key::Char(c @ '1'..='2') => {
                        let g = games::Game::new(c as u8 - b'0', &mut self.pet);
                        self.mode = Mode::Game(g);
                    }
                    Key::Esc | Key::Char('q') => self.mode = Mode::Normal,
                    _ => {}
                }
                return;
            }
            Mode::Shop => {
                match code {
                    Key::Char(c @ '1'..='4') => self.buy(pet::Item::ALL[(c as u8 - b'1') as usize]),
                    Key::Esc | Key::Char('q') | Key::Char('b') => self.mode = Mode::Normal,
                    _ => {}
                }
                return;
            }
            Mode::Finish(f) => {
                if f.skippable() {
                    self.mode = Mode::Normal;
                }
                return;
            }
            Mode::Awards | Mode::Graveyard(_) => {
                self.mode = Mode::Normal;
                return;
            }
            Mode::Toilet(_) => return,
            Mode::Normal => {}
        }
        if self.confirm_reset {
            self.confirm_reset = false;
            if code == Key::Char('y') {
                self.pet = self.fresh_pet();
                self.log.clear();
                self.fx = None;
                self.shell_at = None;
                self.last_stage = Stage::Egg;
                self.message = "Reset! A fresh egg appears.".into();
                let _ = save::save(&self.pet);
            } else {
                self.message = "Reset cancelled.".into();
            }
            return;
        }
        if self.dev && code == Key::Tab {
            self.dev_open = !self.dev_open;
            return;
        }
        if dev_keys && self.dev_open && self.on_dev_key(code) {
            return;
        }
        let before = self.snapshot();
        let msg = match code {
            Key::Char('R') => {
                self.confirm_reset = true;
                self.message = "Reset your pet? Press y to confirm, any other key to cancel.".into();
                return;
            }
            Key::Char('f') => self.pet.feed_meal(),
            Key::Char('s') => self.pet.feed_snack(),
            Key::Char('p') => {
                match self.pet.can_play() {
                    Some(m) => self.message = m.into(),
                    None => self.mode = Mode::Chooser,
                }
                return;
            }
            Key::Char('g') => {
                self.mode = Mode::Graveyard(save::graveyard());
                return;
            }
            Key::Char('b') => {
                self.mode = Mode::Shop;
                return;
            }
            Key::Char('a') => {
                self.mode = Mode::Awards;
                return;
            }
            Key::Char('m') => self.pet.medicine(),
            Key::Char('t') => {
                let goes = self.pet.needs_toilet();
                let msg = self.pet.toilet();
                if goes {
                    self.wipe = Some(Wipe::new(Next::Toilet(self.pet.rand(cast::BOOTHS as u64) as usize)));
                }
                msg
            }
            Key::Char('d') => self.pet.scold(),
            Key::Char('l') => self.pet.toggle_light(),
            Key::Char('n') if !self.pet.alive() => {
                self.pet = self.fresh_pet();
                self.shell_at = None;
                self.last_stage = Stage::Egg;
                "A new egg appears!".into()
            }
            Key::Char('q') | Key::Esc => {
                self.quit = true;
                return;
            }
            _ => return,
        };
        self.message = msg;
        self.react(before);
        if code == Key::Char('s') && self.pet.happy > before.1 {
            self.trigger(scene::Fx::Snack);
        }
    }

    /// Developer keys, active only with `--dev` while the panel is open. Returns true if handled.
    fn on_dev_key(&mut self, code: Key) -> bool {
        let p = &mut self.pet;
        let step = |v: u8, up: bool, max: u8| if up { v.saturating_add(1).min(max) } else { v.saturating_sub(1) };
        // ← and → step through the forms, like [ and ].
        let c = match code {
            Key::Char(c) => c,
            Key::Left => '[',
            Key::Right => ']',
            _ => return false,
        };
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
                p.dev_urge();
                "needs the toilet".into()
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
            '[' | ']' => {
                let now = FORMS.iter().position(|&(s, n)| s == p.stage && (s == Stage::Dead || n == p.species)).unwrap_or(0);
                let next = if c == ']' { (now + 1) % FORMS.len() } else { (now + FORMS.len() - 1) % FORMS.len() };
                let was_egg = p.stage == Stage::Egg;
                set_form(p, next);
                self.last_stage = p.stage;
                if was_egg && p.stage == Stage::Baby {
                    self.shell_at = Some(Instant::now());
                }
                format!("form = {} ({})", p.species, p.stage.label())
            }
            'e' => {
                if p.stage == Stage::Egg {
                    p.stage_age = (p.stage_age + 8).min(29);
                }
                format!("egg age = {}s of 30", p.stage_age)
            }
            ',' | '.' => {
                let hour = (p.local_hour() + if c == '.' { 1 } else { 23 }) % 24;
                p.dev_set_hour(hour);
                format!("clock set to {hour:02}:00")
            }
            '1' => {
                self.speed = 1.0;
                "speed = x1".to_string()
            }
            'f' => {
                self.speed = next_speed(self.speed);
                format!("speed = x{}", self.speed)
            }
            _ => return false,
        };
        self.message = format!("[dev] {note}");
        true
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn app_with(stage: Stage, tweak: impl FnOnce(&mut Pet)) -> App {
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
        assert!(matches!(app.mode, Mode::Finish(_)), "a finished game shows the result screen");
        assert!(app.message.contains("coins"), "{}", app.message);
        app.handle_outcome(games::Outcome::Done { wins: 1, rounds: 5 });
        assert_eq!((app.profile.coins, app.profile.games_won), (profile::COINS_PERFECT, 1));
    }

    #[test]
    fn the_dev_panel_steps_through_every_form_and_back() {
        save::with_temp_dir(|| {
            let mut app = app_with(Stage::Egg, |p| p.species = "Egg".into());
            app.dev = true;
            app.on_key(Key::Tab);
            let mut seen = Vec::new();
            for _ in 0..FORMS.len() {
                app.on_key(Key::Right);
                seen.push((app.pet.stage, app.pet.species.clone()));
            }
            assert_eq!(seen.last().map(|s| s.0), Some(Stage::Egg), "wraps round to the egg");
            assert!(seen.iter().any(|s| s.0 == Stage::Dead));
            assert!(app.pet.alive(), "stepping past death brings a living pet back");
            app.on_key(Key::Left);
            assert_eq!(app.pet.stage, Stage::Dead);
            app.on_key(Key::Char(']'));
            assert_eq!(app.pet.stage, Stage::Egg, "[ and ] still work too");
        });
    }

    #[test]
    fn a_click_does_the_action_even_while_the_dev_panel_is_open() {
        let mut app = app_with(Stage::Adult, |p| p.hunger = 1);
        app.dev = true;
        app.dev_open = true;
        app.press(Key::Char('f'));
        assert_eq!(app.pet.hunger, 2, "f on the panel is the speed key, but the click feeds");
        app.on_key(Key::Char('f'));
        assert_eq!((app.pet.hunger, app.speed), (2, 2.0));
    }

    #[test]
    fn the_dev_speed_key_steps_through_1_2_5_10_60_600_and_round() {
        let mut speeds = vec![1.0];
        for _ in 0..6 {
            speeds.push(next_speed(*speeds.last().unwrap()));
        }
        assert_eq!(speeds, [1.0, 2.0, 5.0, 10.0, 60.0, 600.0, 1.0]);
        assert_eq!(next_speed(7.0), 1.0, "an odd speed (from TAMA_SPEED) restarts the cycle");

        save::with_temp_dir(|| {
            let mut app = app_with(Stage::Adult, |_| {});
            app.dev = true;
            app.on_key(Key::Tab);
            app.speed = 600.0;
            app.on_key(Key::Char('1'));
            assert_eq!(app.speed, 1.0, "1 goes straight back to normal speed");
            app.on_key(Key::Char('f'));
            assert_eq!(app.speed, 2.0);
        });
    }

    #[test]
    fn hatching_leaves_a_shell_that_goes_away_again() {
        save::with_temp_dir(|| {
            let mut app = app_with(Stage::Egg, |p| p.species = "Egg".into());
            assert_eq!(app.shell_age(), None);
            app.pet.dev_grow();
            app.pet.tick();
            app.update();
            assert_eq!(app.pet.stage, Stage::Baby);
            assert!(app.shell_age().is_some_and(|t| t < 1.0), "the shell is on the floor");
            app.shell_at = Some(Instant::now() - Duration::from_secs_f32(scene::SHELL_SECS + 0.1));
            assert_eq!(app.shell_age(), None, "and is gone after a few seconds");
            app.on_key(Key::Char('R'));
            app.on_key(Key::Char('y'));
            assert_eq!(app.shell_at, None, "a new egg starts without one");
        });
    }

    #[test]
    fn shift_r_starts_over_only_after_y() {
        save::with_temp_dir(|| {
            let mut app = app_with(Stage::Adult, |_| {});
            app.on_key(Key::Char('R'));
            assert!(app.confirming_reset());
            app.on_key(Key::Char('x'));
            assert!(!app.confirming_reset());
            assert_eq!(app.pet.stage, Stage::Adult, "any other key cancels");
            app.on_key(Key::Char('R'));
            app.on_key(Key::Char('y'));
            assert_eq!(app.pet.stage, Stage::Egg, "y throws the pet away for a new egg");
        });
    }

    #[test]
    fn the_keys_open_and_close_the_shop_and_awards_and_buy() {
        save::with_temp_dir(|| {
            let mut app = app_with(Stage::Adult, |p| p.hunger = 1);
            app.on_key(Key::Char('b'));
            assert!(matches!(app.mode, Mode::Shop));
            app.profile.coins = 20;
            app.on_key(Key::Char('1'));
            assert_eq!((app.profile.coins, app.pet.hunger), (12, 3));
            assert!(matches!(app.mode, Mode::Shop), "stays open for another purchase");
            app.on_key(Key::Esc);
            assert!(matches!(app.mode, Mode::Normal));
            app.on_key(Key::Char('a'));
            assert!(matches!(app.mode, Mode::Awards));
            app.on_key(Key::Char('x'));
            assert!(matches!(app.mode, Mode::Normal));
        });
    }

    #[test]
    fn the_toilet_key_wipes_into_the_washroom_and_back_out() {
        save::with_temp_dir(|| {
            let mut app = app_with(Stage::Adult, |p| p.urge = Some(0));
            app.on_key(Key::Char('t'));
            assert!(app.wiping() && matches!(app.mode, Mode::Normal), "the wipe starts in the room");
            app.on_key(Key::Char('f'));
            assert_eq!(app.pet.hunger, 3, "keys do nothing during the wipe");
            app.wipe.as_mut().unwrap().started -= Duration::from_secs_f32(0.9);
            app.update();
            assert!(matches!(app.mode, Mode::Toilet(_)), "scenes change while the picture is shut");
            app.wipe.as_mut().unwrap().started -= Duration::from_secs_f32(0.9);
            app.update();
            assert!(!app.wiping() && app.pet.urge.is_none());
        });
    }
}
