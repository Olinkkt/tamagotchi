use crate::config::Config;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Stage {
    Egg,
    Baby,
    Child,
    Teen,
    Adult,
    Dead,
}

impl Stage {
    pub fn label(self) -> &'static str {
        match self {
            Stage::Egg => "Egg",
            Stage::Baby => "Baby",
            Stage::Child => "Child",
            Stage::Teen => "Teen",
            Stage::Adult => "Adult",
            Stage::Dead => "Departed",
        }
    }
}

/// A per-pet personality, picked at random when the egg is laid.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Quirk {
    #[default]
    Easygoing,
    Greedy,
    Playful,
    Hardy,
    Mischievous,
}

impl Quirk {
    fn pick(n: u64) -> Quirk {
        match n % 5 {
            0 => Quirk::Easygoing,
            1 => Quirk::Greedy,
            2 => Quirk::Playful,
            3 => Quirk::Hardy,
            _ => Quirk::Mischievous,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Quirk::Easygoing => "easygoing",
            Quirk::Greedy => "greedy",
            Quirk::Playful => "playful",
            Quirk::Hardy => "hardy",
            Quirk::Mischievous => "mischievous",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Quirk::Easygoing => "no strong habits",
            Quirk::Greedy => "gets hungry faster",
            Quirk::Playful => "gets bored faster, loves games",
            Quirk::Hardy => "rarely gets sick",
            Quirk::Mischievous => "acts up more often",
        }
    }
}

/// Heaviest weight that is still healthy at each stage.
fn healthy_max_weight(stage: Stage) -> u8 {
    match stage {
        Stage::Egg | Stage::Baby => 10,
        Stage::Child => 18,
        Stage::Teen => 28,
        _ => 38,
    }
}

pub const MAX_HEARTS: u8 = 4;

#[derive(Serialize, Deserialize, Clone)]
pub struct Pet {
    pub species: String,
    pub stage: Stage,
    pub age: u64,
    pub stage_age: u64,
    pub hunger: u8,
    pub happy: u8,
    pub discipline: u8,
    pub weight: u8,
    pub poops: u8,
    pub sick: bool,
    pub doses: u8,
    pub asleep: bool,
    pub light_on: bool,
    pub mistakes: u32,
    pub disc_call: Option<u64>,
    pub cause_of_death: Option<String>,
    pub buried: bool,
    #[serde(default)]
    pub quirk: Quirk,
    /// Pet's wall-clock time (unix seconds); advances one second per tick.
    #[serde(default)]
    pub clock: i64,
    #[serde(skip)]
    pub cfg: Config,
    #[serde(skip)]
    pub tz_offset: i64,
    hunger_t: u64,
    happy_t: u64,
    poop_t: u64,
    poop_next: u64,
    dirty_t: u64,
    hunger_zero: u64,
    happy_zero: u64,
    sick_t: u64,
    light_t: u64,
    rng: u64,
    #[serde(skip)]
    pub events: Vec<String>,
}

impl Pet {
    pub fn new(seed: u64) -> Self {
        let mut pet = Pet {
            species: "Egg".into(),
            stage: Stage::Egg,
            age: 0,
            stage_age: 0,
            hunger: 3,
            happy: 3,
            discipline: 50,
            weight: 5,
            poops: 0,
            sick: false,
            doses: 0,
            asleep: false,
            light_on: true,
            mistakes: 0,
            disc_call: None,
            cause_of_death: None,
            buried: false,
            quirk: Quirk::Easygoing,
            clock: seed as i64,
            cfg: Config::default(),
            tz_offset: 0,
            hunger_t: 0,
            happy_t: 0,
            poop_t: 0,
            poop_next: 1200,
            dirty_t: 0,
            hunger_zero: 0,
            happy_zero: 0,
            sick_t: 0,
            light_t: 0,
            rng: seed | 1,
            events: Vec::new(),
        };
        let n = pet.rand(5);
        pet.quirk = Quirk::pick(n);
        pet
    }

    /// xorshift64; returns a value in 0..n.
    pub fn rand(&mut self, n: u64) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x % n
    }

    /// Give the pet its config, the local UTC offset, and (optionally) re-sync its clock.
    pub fn set_env(&mut self, cfg: Config, tz_offset: i64) {
        self.cfg = cfg;
        self.tz_offset = tz_offset;
    }

    // ---- developer helpers (used by --dev mode) ----

    pub fn dev_kill(&mut self) {
        if self.alive() {
            self.die("dev command");
        }
    }

    pub fn dev_sick(&mut self) {
        self.get_sick();
    }

    pub fn dev_naughty(&mut self) {
        self.disc_call = Some(0);
    }

    /// Make the next tick advance the pet to its next life stage.
    pub fn dev_grow(&mut self) {
        if self.alive() && self.stage != Stage::Adult {
            self.stage_age = 1_000_000_000;
        }
    }

    pub fn dev_set_hour(&mut self, hour: i64) {
        let local = self.clock + self.tz_offset;
        let day_start = local - local.rem_euclid(86_400);
        self.clock = day_start + hour * 3600 - self.tz_offset;
    }

    pub fn alive(&self) -> bool {
        self.stage != Stage::Dead
    }

    pub fn needs_attention(&self) -> bool {
        self.alive()
            && self.stage != Stage::Egg
            && !self.asleep
            && (self.hunger == 0 || self.happy == 0 || self.sick || self.disc_call.is_some())
    }

    /// The single most urgent thing the player should do: (key, hint text).
    pub fn top_need(&self) -> Option<(char, &'static str)> {
        match self.stage {
            Stage::Dead => return Some(('n', "It has passed away - hatch a new egg")),
            Stage::Egg => return None,
            _ => {}
        }
        if self.asleep {
            return self.light_on.then_some(('l', "It's asleep with the light on - turn it off"));
        }
        if self.sick {
            Some(('m', "Sick! Give it medicine"))
        } else if self.hunger == 0 {
            Some(('f', "Starving! Feed it a meal"))
        } else if self.disc_call.is_some() {
            Some(('d', "Naughty - scold it"))
        } else if self.poops > 0 {
            Some(('c', "Clean up the poop"))
        } else if self.happy == 0 {
            Some(('p', "Lonely - play a game with it"))
        } else if self.hunger <= 1 {
            Some(('f', "Getting hungry - feed it"))
        } else if self.happy <= 1 {
            Some(('p', "Getting sad - play with it"))
        } else if self.overweight() {
            Some(('p', "Overweight - play a game to burn it off"))
        } else {
            None
        }
    }

    fn mistake(&mut self, why: &str) {
        self.mistakes += 1;
        self.events.push(format!("Care mistake: {why}"));
    }

    fn die(&mut self, cause: &str) {
        self.stage = Stage::Dead;
        self.cause_of_death = Some(cause.into());
        self.events.push(format!("{} has passed away ({cause}).", self.species));
    }

    fn get_sick(&mut self) {
        if !self.sick {
            self.sick = true;
            self.doses = 2;
            self.sick_t = 0;
            self.events.push(format!("{} got sick!", self.species));
        }
    }

    /// Advance the simulation by one second.
    pub fn tick(&mut self) {
        if !self.alive() {
            return;
        }
        self.age += 1;
        self.stage_age += 1;
        self.clock += 1;
        self.grow();
        if self.stage == Stage::Egg || !self.alive() {
            return;
        }
        self.update_sleep();
        if self.asleep {
            if self.light_on {
                self.light_t += 1;
                if self.light_t == 600 {
                    self.mistake("left the light on while sleeping");
                }
            }
            return;
        }
        self.decay();
        self.poop();
        self.sickness();
        self.discipline_call();
        if self.stage == Stage::Adult && self.stage_age >= self.cfg.adult_lifespan_secs {
            self.die("old age");
        }
    }

    fn update_sleep(&mut self) {
        let asleep = self.in_sleep_hours();
        if asleep != self.asleep {
            self.asleep = asleep;
            self.light_on = true;
            self.light_t = 0;
            let msg = if asleep { "went to sleep." } else { "woke up!" };
            self.events.push(format!("{} {msg}", self.species));
        }
    }

    pub fn local_hour(&self) -> i64 {
        (self.clock + self.tz_offset).rem_euclid(86_400) / 3600
    }

    fn in_sleep_hours(&self) -> bool {
        let (h, start, end) = (self.local_hour(), self.cfg.sleep_start_hour, self.cfg.sleep_end_hour);
        if start > end { h >= start || h < end } else { h >= start && h < end }
    }

    fn grow(&mut self) {
        let Some((limit, next)) = self.next_stage() else { return };
        if self.stage_age < limit {
            return;
        }
        self.stage = next;
        self.stage_age = 0;
        self.weight = self.weight.saturating_add(5).min(99);
        self.species = self.species_for(next).into();
        self.events.push(format!("Grew into a {} ({})!", next.label(), self.species));
    }

    fn species_for(&self, stage: Stage) -> &'static str {
        Self::species_of(stage, self.mistakes, self.discipline)
    }

    fn species_of(stage: Stage, mistakes: u32, discipline: u8) -> &'static str {
        match stage {
            Stage::Baby => "Babytchi",
            Stage::Child => "Marutchi",
            Stage::Teen if mistakes <= 2 => "Tonmarutchi",
            Stage::Teen => "Hashitamatchi",
            Stage::Adult => match (mistakes, discipline) {
                (0..=1, 75..) => "Mametchi",
                (0..=2, _) => "Kuchipatchi",
                (3..=4, _) => "Tarakotchi",
                (5..=6, _) => "Ichigotchi",
                _ => "Nyorotchi",
            },
            _ => "Egg",
        }
    }

    fn next_stage(&self) -> Option<(u64, Stage)> {
        match self.stage {
            Stage::Egg => Some((30, Stage::Baby)),
            Stage::Baby => Some((self.cfg.baby_secs, Stage::Child)),
            Stage::Child => Some((self.cfg.child_secs, Stage::Teen)),
            Stage::Teen => Some((self.cfg.teen_secs, Stage::Adult)),
            _ => None,
        }
    }

    pub fn overweight(&self) -> bool {
        self.alive() && self.stage != Stage::Egg && self.weight > healthy_max_weight(self.stage)
    }

    /// Short lines describing what growth is coming and how care changes the outcome.
    pub fn evolution_preview(&self) -> Vec<String> {
        if !self.alive() || self.stage == Stage::Egg {
            return Vec::new();
        }
        let Some((limit, next)) = self.next_stage() else {
            let left = self.cfg.adult_lifespan_secs.saturating_sub(self.stage_age);
            return vec![format!("Fully grown. About {} left.", crate::fmt_age(left))];
        };
        let mut out = vec![format!("Next: {} in {}", next.label(), crate::fmt_age(limit.saturating_sub(self.stage_age)))];
        if matches!(next, Stage::Teen | Stage::Adult) {
            let now = Self::species_of(next, self.mistakes, self.discipline);
            out.push(format!("On track: {now}"));
            if let Some((k, worse)) = (1..=3u32)
                .map(|k| (k, Self::species_of(next, self.mistakes + k, self.discipline)))
                .find(|(_, s)| *s != now)
            {
                let s = if k == 1 { "" } else { "s" };
                out.push(format!("{k} more mistake{s} -> {worse}"));
            }
            let better = Self::species_of(next, self.mistakes, 100);
            if better != now {
                out.push(format!("75%+ discipline -> {better}"));
            }
        }
        out
    }

    fn hunger_interval(&self) -> u64 {
        let base = match self.stage {
            Stage::Baby => self.cfg.hunger_secs_baby,
            Stage::Child => self.cfg.hunger_secs_child,
            Stage::Teen => self.cfg.hunger_secs_teen,
            _ => self.cfg.hunger_secs_adult,
        };
        if self.quirk == Quirk::Greedy { base * 3 / 4 } else { base }
    }

    fn decay(&mut self) {
        let hi = self.hunger_interval();
        self.hunger_t += 1;
        if self.hunger_t >= hi {
            self.hunger_t = 0;
            self.hunger = self.hunger.saturating_sub(1);
        }
        self.happy_t += 1;
        let happy_secs = (hi as f64 * self.cfg.happy_decay_factor) as u64;
        let happy_secs = if self.quirk == Quirk::Playful { happy_secs * 3 / 4 } else { happy_secs };
        if self.happy_t >= happy_secs {
            self.happy_t = 0;
            self.happy = self.happy.saturating_sub(1);
        }
        if self.hunger == 0 {
            self.hunger_zero += 1;
            if self.hunger_zero == self.cfg.mistake_after_secs {
                self.mistake("ignored a hunger call");
            }
            if self.hunger_zero >= 12 * 3600 {
                self.die("starvation");
            }
        } else {
            self.hunger_zero = 0;
        }
        if self.happy == 0 {
            self.happy_zero += 1;
            if self.happy_zero == self.cfg.mistake_after_secs {
                self.mistake("ignored an unhappy call");
            }
        } else {
            self.happy_zero = 0;
        }
    }

    fn poop(&mut self) {
        self.poop_t += 1;
        if self.poop_t >= self.poop_next {
            self.poop_t = 0;
            self.poop_next = self.cfg.poop_min_secs + self.rand(self.cfg.poop_extra_secs.max(1));
            if self.poops < 3 {
                self.poops += 1;
                self.events.push(format!("{} pooped.", self.species));
            }
        }
        if self.poops > 0 {
            self.dirty_t += 1;
            if self.dirty_t >= 1200 {
                self.dirty_t = 0;
                if self.rand(3) == 0 {
                    self.get_sick();
                }
            }
        } else {
            self.dirty_t = 0;
        }
    }

    fn sickness(&mut self) {
        if !self.sick {
            let mut odds = self.cfg.sick_odds.max(1);
            if self.quirk == Quirk::Hardy {
                odds *= 2;
            }
            if self.overweight() {
                odds = (odds / self.cfg.overweight_sick_factor.max(1)).max(1);
            }
            if self.rand(odds) == 0 {
                self.get_sick();
            }
            return;
        }
        self.sick_t += 1;
        if self.sick_t == self.cfg.mistake_after_secs {
            self.mistake("ignored an illness");
        }
        if self.sick_t >= 4 * 3600 {
            self.die("illness");
        }
    }

    fn discipline_call(&mut self) {
        match self.disc_call {
            None => {
                let odds = if self.quirk == Quirk::Mischievous { self.cfg.naughty_odds / 2 } else { self.cfg.naughty_odds };
                if self.rand(odds.max(1)) == 0 {
                    self.disc_call = Some(0);
                    self.events.push(format!("{} is being naughty!", self.species));
                }
            }
            Some(a) => {
                self.disc_call = Some(a + 1);
                if a + 1 == 600 {
                    self.disc_call = None;
                    self.mistake("ignored a discipline call");
                }
            }
        }
    }

    // ---- player actions: each returns a message for the UI ----

    fn blocked(&self) -> Option<&'static str> {
        match self.stage {
            Stage::Egg => Some("It's still an egg..."),
            Stage::Dead => Some("Your pet has passed away. Press N for a new egg."),
            _ if self.asleep => Some("Shh, it's sleeping."),
            _ => None,
        }
    }

    pub fn feed_meal(&mut self) -> String {
        if let Some(m) = self.blocked() {
            return m.into();
        }
        if self.hunger >= MAX_HEARTS {
            return "Not hungry.".into();
        }
        self.hunger += 1;
        self.weight = self.weight.saturating_add(1).min(99);
        if self.quirk == Quirk::Greedy { "Gobbles it up! Hunger +1.".into() } else { "Nom nom! Hunger +1.".into() }
    }

    pub fn feed_snack(&mut self) -> String {
        if let Some(m) = self.blocked() {
            return m.into();
        }
        if self.happy >= MAX_HEARTS {
            return "Too happy to snack.".into();
        }
        self.happy += 1;
        self.weight = self.weight.saturating_add(2).min(99);
        if self.rand(10) == 0 {
            self.get_sick();
        }
        "Yummy snack! Happy +1.".into()
    }

    pub fn can_play(&self) -> Option<&'static str> {
        self.blocked()
    }

    pub fn finish_game(&mut self, wins: u8, rounds: u8) -> String {
        self.weight = self.weight.saturating_sub(1).max(1);
        if wins * 2 > rounds {
            let bonus = if self.quirk == Quirk::Playful { 2 } else { 1 };
            self.happy = (self.happy + bonus).min(MAX_HEARTS);
            if wins == rounds && self.mistakes > 0 {
                self.mistakes -= 1;
                return "Perfect game! Happy +1 and a care mistake forgiven.".into();
            }
            if self.quirk == Quirk::Playful { "You won! It loves games: Happy +2.".into() } else { "You won! Happy +1.".into() }
        } else {
            "You lost... no happiness gained.".into()
        }
    }

    pub fn medicine(&mut self) -> String {
        if let Some(m) = self.blocked() {
            return m.into();
        }
        if !self.sick {
            return "Not sick.".into();
        }
        self.doses = self.doses.saturating_sub(1);
        if self.doses == 0 {
            self.sick = false;
            self.sick_t = 0;
            "Cured!".into()
        } else {
            "Needs another dose.".into()
        }
    }

    pub fn clean(&mut self) -> String {
        if self.stage == Stage::Egg || !self.alive() {
            return self.blocked().unwrap_or("").into();
        }
        if self.poops == 0 {
            return "Nothing to clean.".into();
        }
        self.poops = 0;
        self.dirty_t = 0;
        "Squeaky clean.".into()
    }

    pub fn scold(&mut self) -> String {
        if let Some(m) = self.blocked() {
            return m.into();
        }
        if self.disc_call.take().is_some() {
            self.discipline = self.discipline.saturating_add(25).min(100);
            "Lesson learned. Discipline up.".into()
        } else {
            self.happy = self.happy.saturating_sub(1);
            "It didn't deserve that! Happy -1.".into()
        }
    }

    pub fn toggle_light(&mut self) -> String {
        if !self.asleep {
            return "The pet is awake; lights don't matter.".into();
        }
        self.light_on = !self.light_on;
        if self.light_on { "Light on.".into() } else { "Light off. Sweet dreams.".into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_through_stages() {
        let mut p = Pet::new(42);
        for _ in 0..30 {
            p.tick();
        }
        assert_eq!(p.stage, Stage::Baby);
    }

    #[test]
    fn overweight_pet_is_flagged_and_hinted() {
        let mut p = Pet::new(3);
        p.dev_set_hour(12);
        for _ in 0..40 {
            p.tick();
        }
        assert_eq!(p.stage, Stage::Baby);
        p.weight = 11;
        assert!(p.overweight());
        p.hunger = 4;
        p.happy = 4;
        p.sick = false;
        p.disc_call = None;
        p.poops = 0;
        assert_eq!(p.top_need().map(|n| n.0), Some('p'));
        p.weight = 5;
        assert!(!p.overweight());
    }

    #[test]
    fn preview_warns_about_mistakes_and_discipline() {
        let mut p = Pet::new(4);
        p.stage = Stage::Teen;
        p.mistakes = 1;
        p.discipline = 50;
        let text = p.evolution_preview().join(" | ");
        assert!(text.contains("On track: Kuchipatchi"), "{text}");
        assert!(text.contains("1 more mistake -> Kuchipatchi") || text.contains("more mistake"), "{text}");
        assert!(text.contains("75%+ discipline -> Mametchi"), "{text}");
    }

    #[test]
    fn greedy_gets_hungry_sooner() {
        let mut a = Pet::new(8);
        a.stage = Stage::Adult;
        a.quirk = Quirk::Easygoing;
        let mut b = a.clone();
        b.quirk = Quirk::Greedy;
        assert!(b.hunger_interval() < a.hunger_interval());
    }

    #[test]
    fn sleeps_at_night_by_clock() {
        let mut p = Pet::new(5);
        for _ in 0..30 {
            p.tick();
        }
        p.dev_set_hour(21);
        p.clock += 3598;
        p.tick();
        assert!(!p.asleep);
        p.tick();
        assert!(p.asleep, "22:00 should be asleep");
        p.dev_set_hour(8);
        p.tick();
        assert!(!p.asleep);
    }

    #[test]
    fn neglect_kills_eventually() {
        let mut p = Pet::new(7);
        p.dev_set_hour(12);
        for _ in 0..(3 * 86_400) {
            p.tick();
        }
        assert_eq!(p.stage, Stage::Dead);
        assert!(p.mistakes > 0);
    }

    #[test]
    fn care_keeps_it_alive_to_adulthood() {
        let mut p = Pet::new(9);
        p.dev_set_hour(12);
        for t in 0..(4 * 3600 + 200) {
            p.tick();
            if t % 60 == 0 {
                if p.hunger < 3 { p.feed_meal(); }
                if p.happy < 3 { p.feed_snack(); }
                p.clean();
                p.medicine();
                p.medicine();
                p.scold();
                if p.asleep && p.light_on { p.toggle_light(); }
            }
        }
        assert!(p.alive());
        assert!(p.stage >= Stage::Teen || p.stage == Stage::Teen);
    }
}
