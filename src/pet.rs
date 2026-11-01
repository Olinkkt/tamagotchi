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
}

/// What the player chooses to do with a pet. Counted per life stage, hidden from the player, and
/// the biggest one decides which form the pet grows into.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Habit {
    Games,
    Snacks,
    ShopFood,
    ShopCare,
    Scolds,
}

impl Habit {
    pub const ALL: [Habit; 5] = [Habit::Games, Habit::Snacks, Habit::ShopFood, Habit::ShopCare, Habit::Scolds];

    /// How many times in one stage earns the form (first guesses; about an hour of play).
    pub fn target(self) -> u32 {
        match self {
            Habit::Games | Habit::Snacks => 6,
            Habit::ShopFood | Habit::ShopCare => 3,
            Habit::Scolds => 2,
        }
    }

    /// The adult form this habit leads to.
    pub fn adult(self) -> &'static str {
        match self {
            Habit::Games => "Tarakotchi",
            Habit::Snacks => "Ichigotchi",
            Habit::ShopFood => "Kuchipatchi",
            Habit::ShopCare => "Nyorotchi",
            Habit::Scolds => "Mametchi",
        }
    }
}

/// How often each habit happened in the current stage (indexed by `Habit as usize`).
pub type Habits = [u32; 5];

/// A form's personality. It follows from the species, so it replaces the previous form's and
/// old saves get theirs without any stored field.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Personality {
    None,
    Calm,
    Spirited,
    Sporty,
    Cuddly,
    Disciplined,
    Gourmet,
    Sturdy,
    Easygoing,
}

impl Personality {
    pub fn of(species: &str) -> Personality {
        match species {
            "Tonmarutchi" => Personality::Calm,
            "Hashitamatchi" => Personality::Spirited,
            "Tarakotchi" => Personality::Sporty,
            "Ichigotchi" => Personality::Cuddly,
            "Mametchi" => Personality::Disciplined,
            "Kuchipatchi" => Personality::Gourmet,
            "Nyorotchi" => Personality::Sturdy,
            "Futsutchi" => Personality::Easygoing,
            _ => Personality::None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Personality::None => "",
            Personality::Calm => "calm",
            Personality::Spirited => "spirited",
            Personality::Sporty => "sporty",
            Personality::Cuddly => "cuddly",
            Personality::Disciplined => "strict",
            Personality::Gourmet => "gourmet",
            Personality::Sturdy => "sturdy",
            Personality::Easygoing => "easygoing",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Personality::None => "",
            Personality::Calm => "happiness falls 25% slower",
            Personality::Spirited => "+1 coin per game win",
            Personality::Sporty => "+1 extra happy per win",
            Personality::Cuddly => "happiness falls 40% slower",
            Personality::Disciplined => "acts up half as often",
            Personality::Gourmet => "hunger falls 25% slower",
            Personality::Sturdy => "gets sick half as often",
            Personality::Easygoing => "no perk, no downside",
        }
    }
}

/// Something that can be bought in the shop.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Item {
    FancyMeal,
    Toy,
    Salad,
    Vitamins,
}

/// How long vitamins keep the pet from falling sick.
const VITAMIN_SECS: u64 = 3 * 3600;

impl Item {
    pub const ALL: [Item; 4] = [Item::FancyMeal, Item::Toy, Item::Salad, Item::Vitamins];

    pub fn name(self) -> &'static str {
        match self {
            Item::FancyMeal => "Fancy meal",
            Item::Toy => "Toy",
            Item::Salad => "Salad",
            Item::Vitamins => "Vitamins",
        }
    }

    pub fn price(self) -> u32 {
        match self {
            Item::FancyMeal => 15,
            Item::Toy => 10,
            Item::Salad => 8,
            Item::Vitamins => 25,
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Item::FancyMeal => "+2 hunger, a bit heavy",
            Item::Toy => "+2 happy, burns weight",
            Item::Salad => "+1 hunger, lighter",
            Item::Vitamins => "no sickness for 3 hours",
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
    /// Seconds the pet has needed the toilet; `None` while it doesn't.
    #[serde(default)]
    pub urge: Option<u64>,
    /// Old saves counted piles on the floor; `set_env` folds them into `urge`.
    #[serde(default, skip_serializing)]
    poops: u8,
    pub sick: bool,
    pub doses: u8,
    pub asleep: bool,
    pub mistakes: u32,
    /// Helpful choices made in the current stage; they pick the next form.
    #[serde(default)]
    pub habits: Habits,
    pub disc_call: Option<u64>,
    pub cause_of_death: Option<String>,
    pub buried: bool,
    #[serde(default)]
    pub quirk: Quirk,
    /// The pet cannot fall sick until its `age` reaches this (set by vitamins).
    #[serde(default)]
    pub immune_until: u64,
    /// Pet's wall-clock time (unix seconds); advances one second per tick.
    #[serde(default)]
    pub clock: i64,
    #[serde(skip)]
    pub cfg: Config,
    #[serde(skip)]
    pub tz_offset: i64,
    /// True while catching up on time the game was closed (see `Config::away_safe`).
    #[serde(skip)]
    pub away: bool,
    /// How busy the machine is right now (set by the game; never saved).
    #[serde(skip)]
    hunger_t: u64,
    happy_t: u64,
    /// Seconds until the meal just eaten makes the pet need the toilet.
    #[serde(default)]
    meal_urge: Option<u64>,
    hunger_zero: u64,
    happy_zero: u64,
    sick_t: u64,
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
            urge: None,
            poops: 0,
            sick: false,
            doses: 0,
            asleep: false,
            mistakes: 0,
            habits: [0; 5],
            disc_call: None,
            cause_of_death: None,
            buried: false,
            quirk: Quirk::Easygoing,
            immune_until: 0,
            clock: seed as i64,
            cfg: Config::default(),
            tz_offset: 0,
            away: false,
            hunger_t: 0,
            happy_t: 0,
            meal_urge: None,
            hunger_zero: 0,
            happy_zero: 0,
            sick_t: 0,
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
        if std::mem::take(&mut self.poops) > 0 {
            self.urge.get_or_insert(0);
        }
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
            return None;
        }
        if self.sick {
            Some(('m', "Sick! Give it medicine"))
        } else if self.hunger == 0 {
            Some(('f', "Starving! Feed it a meal"))
        } else if self.disc_call.is_some() {
            Some(('d', "Naughty - scold it"))
        } else if self.urge.is_some() {
            Some(('t', "Needs the toilet"))
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

    fn note(&mut self, habit: Habit) {
        let n = &mut self.habits[habit as usize];
        *n = n.saturating_add(1);
    }

    pub fn personality(&self) -> Personality {
        Personality::of(&self.species)
    }

    /// Extra coins a won game pays (Spirited).
    pub fn win_coin_bonus(&self) -> u32 {
        u32::from(self.personality() == Personality::Spirited)
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
            return;
        }
        self.decay();
        self.toilet_urge();
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
            let msg = if asleep { "went to sleep." } else { "woke up!" };
            self.events.push(format!("{} {msg}", self.species));
        }
    }

    /// Being away with the protection switched on.
    fn safe_away(&self) -> bool {
        self.away && self.cfg.away_safe
    }

    /// While away, the slow timers (decay, toilet) only advance on every `away_slowdown`-th tick.
    fn skip_slow_tick(&self) -> bool {
        self.away && self.age % self.cfg.away_slowdown.max(1) != 0
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
        self.habits = [0; 5];
        self.events.push(format!("Grew into a {} ({})!", next.label(), self.species));
    }

    fn species_for(&self, stage: Stage) -> &'static str {
        Self::species_of(stage, &self.habits)
    }

    /// The form a stage grows into, from what the player did during the stage before it.
    /// The habit most over its target wins; no target reached means the Plain form
    /// (teens have no Plain form and stay Tonmarutchi).
    fn species_of(stage: Stage, habits: &Habits) -> &'static str {
        match stage {
            Stage::Baby => "Babytchi",
            Stage::Child => "Marutchi",
            Stage::Teen => {
                let games = habits[Habit::Games as usize];
                if games >= Habit::Games.target() && games >= habits[Habit::Snacks as usize] {
                    "Hashitamatchi"
                } else {
                    "Tonmarutchi"
                }
            }
            Stage::Adult => Habit::ALL
                .into_iter()
                .filter(|h| habits[*h as usize] >= h.target())
                // Most over its target, comparing count / target by cross-multiplying;
                // the first habit in the list wins a tie.
                .reduce(|best, h| {
                    if habits[h as usize] * best.target() > habits[best as usize] * h.target() { h } else { best }
                })
                .map_or("Futsutchi", Habit::adult),
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

    /// The form it will grow into next at the care it has had so far (only teens and adults vary).
    pub fn on_track(&self) -> Option<&'static str> {
        if !self.alive() || self.stage == Stage::Egg {
            return None;
        }
        let (_, next) = self.next_stage()?;
        matches!(next, Stage::Teen | Stage::Adult).then(|| Self::species_of(next, &self.habits))
    }

    /// Short lines describing what growth is coming and how care changes the outcome.
    pub fn evolution_preview(&self) -> Vec<String> {
        if !self.alive() || self.stage == Stage::Egg {
            return Vec::new();
        }
        let Some((limit, next)) = self.next_stage() else {
            let left = self.cfg.adult_lifespan_secs.saturating_sub(self.stage_age);
            return vec![format!("Adult - {} left", crate::fmt_age(left))];
        };
        let mut out = vec![format!("Next: {} in {}", next.label(), crate::fmt_age(limit.saturating_sub(self.stage_age)))];
        if matches!(next, Stage::Teen | Stage::Adult) {
            out.push(format!("On track: {}", Self::species_of(next, &self.habits)));
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
        if self.safe_away() {
            self.hunger = self.hunger.max(1);
            self.happy = self.happy.max(1);
        }
        if self.skip_slow_tick() {
            return;
        }
        let mut hi = self.hunger_interval();
        if self.personality() == Personality::Gourmet {
            hi = hi * 4 / 3;
        }
        self.hunger_t += 1;
        if self.hunger_t >= hi {
            self.hunger_t = 0;
            self.hunger = self.hunger.saturating_sub(1);
        }
        self.happy_t += 1;
        let happy_secs = (hi as f64 * self.cfg.happy_decay_factor) as u64;
        let happy_secs = if self.quirk == Quirk::Playful { happy_secs * 3 / 4 } else { happy_secs };
        let happy_secs = match self.personality() {
            Personality::Calm => happy_secs * 4 / 3,
            Personality::Cuddly => happy_secs * 5 / 3,
            _ => happy_secs,
        };
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

    /// The urge comes a little after a meal, or rarely out of nowhere. Left too long, the pet
    /// has an accident (never while away with the protection on).
    fn toilet_urge(&mut self) {
        if self.skip_slow_tick() {
            return;
        }
        if let Some(left) = self.meal_urge {
            self.meal_urge = (left > 1).then_some(left - 1);
            if self.meal_urge.is_none() {
                self.feel_urge();
            }
        } else if self.urge.is_none() && self.rand(self.cfg.urge_odds.max(1)) == 0 {
            self.feel_urge();
        }
        let Some(waited) = self.urge else { return };
        if self.safe_away() {
            return;
        }
        if waited + 1 < self.cfg.urge_patience_secs {
            self.urge = Some(waited + 1);
            return;
        }
        self.urge = None;
        self.happy = self.happy.saturating_sub(1);
        self.events.push(format!("{} couldn't hold it. Happy -1.", self.species));
        self.mistake("missed a toilet call");
        if self.age >= self.immune_until && self.rand(3) == 0 {
            self.get_sick();
        }
    }

    fn feel_urge(&mut self) {
        if self.urge.is_none() {
            self.urge = Some(0);
            self.events.push(format!("{} needs the toilet!", self.species));
        }
    }

    /// A meal was eaten: the toilet call follows in a minute or two.
    fn ate_meal(&mut self) {
        if self.meal_urge.is_none() && self.urge.is_none() {
            let wait = self.cfg.urge_after_meal_secs + self.rand(self.cfg.urge_after_meal_extra_secs.max(1));
            self.meal_urge = Some(wait.max(1));
        }
    }

    fn sickness(&mut self) {
        if self.safe_away() {
            return;
        }
        if !self.sick {
            if self.age < self.immune_until {
                return;
            }
            let mut odds = self.cfg.sick_odds.max(1);
            if self.quirk == Quirk::Hardy {
                odds *= 2;
            }
            if self.personality() == Personality::Sturdy {
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
        if self.safe_away() {
            return;
        }
        match self.disc_call {
            None => {
                let mut odds = if self.quirk == Quirk::Mischievous { self.cfg.naughty_odds / 2 } else { self.cfg.naughty_odds };
                if self.personality() == Personality::Disciplined {
                    odds *= 2;
                }
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
        self.ate_meal();
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
        self.note(Habit::Snacks);
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
            let mut bonus = if self.quirk == Quirk::Playful { 2 } else { 1 };
            if self.personality() == Personality::Sporty {
                bonus += 1;
            }
            // Only a win that filled missing hearts counts; endless play earns coins, not a form.
            if self.happy < MAX_HEARTS {
                self.note(Habit::Games);
            }
            self.happy = (self.happy + bonus).min(MAX_HEARTS);
            if wins == rounds && self.mistakes > 0 {
                self.mistakes -= 1;
                return format!("Perfect game! Happy +{bonus} and a care mistake forgiven.");
            }
            format!("You won! Happy +{bonus}.")
        } else {
            "You lost... no happiness gained.".into()
        }
    }

    /// Use something from the shop. Fails (without using it up) if it would do nothing right now.
    pub fn use_item(&mut self, item: Item) -> Result<String, String> {
        if let Some(m) = self.blocked() {
            return Err(m.into());
        }
        match item {
            Item::FancyMeal | Item::Salad if self.hunger >= MAX_HEARTS => Err("Not hungry.".into()),
            Item::Toy if self.happy >= MAX_HEARTS => Err("Too happy to play.".into()),
            Item::Vitamins if self.age < self.immune_until => Err("Already protected.".into()),
            Item::FancyMeal => {
                self.hunger = (self.hunger + 2).min(MAX_HEARTS);
                self.weight = self.weight.saturating_add(1).min(99);
                self.note(Habit::ShopFood);
                self.ate_meal();
                Ok("A fancy meal! Hunger +2.".into())
            }
            Item::Toy => {
                self.happy = (self.happy + 2).min(MAX_HEARTS);
                self.weight = self.weight.saturating_sub(1).max(1);
                self.note(Habit::ShopCare);
                Ok("It loves the toy! Happy +2.".into())
            }
            Item::Salad => {
                self.hunger += 1;
                self.weight = self.weight.saturating_sub(2).max(1);
                self.note(Habit::ShopFood);
                self.ate_meal();
                Ok("A light salad. Hunger +1, a little lighter.".into())
            }
            Item::Vitamins => {
                self.immune_until = self.age + VITAMIN_SECS;
                self.note(Habit::ShopCare);
                Ok("Vitamins! No sickness for 3 hours.".into())
            }
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

    /// Awake, hatched and in need of the toilet: the game shows the washroom for it.
    pub fn needs_toilet(&self) -> bool {
        self.alive() && self.stage != Stage::Egg && !self.asleep && self.urge.is_some()
    }

    pub fn toilet(&mut self) -> String {
        if let Some(m) = self.blocked() {
            return m.into();
        }
        if self.urge.take().is_none() {
            return "Doesn't need the toilet.".into();
        }
        "Ahh, much better.".into()
    }

    /// Dev panel: the pet needs the toilet right now.
    pub fn dev_urge(&mut self) {
        self.feel_urge();
    }

    pub fn scold(&mut self) -> String {
        if let Some(m) = self.blocked() {
            return m.into();
        }
        if self.disc_call.take().is_some() {
            self.discipline = self.discipline.saturating_add(25).min(100);
            self.note(Habit::Scolds);
            "Lesson learned. Discipline up.".into()
        } else {
            self.happy = self.happy.saturating_sub(1);
            "It didn't deserve that! Happy -1.".into()
        }
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

    fn habits(games: u32, snacks: u32, food: u32, care: u32, scolds: u32) -> Habits {
        [games, snacks, food, care, scolds]
    }

    #[test]
    fn teens_follow_games_against_snacks() {
        assert_eq!(Pet::species_of(Stage::Teen, &habits(0, 0, 0, 0, 0)), "Tonmarutchi");
        assert_eq!(Pet::species_of(Stage::Teen, &habits(5, 0, 0, 0, 0)), "Tonmarutchi", "below the target");
        assert_eq!(Pet::species_of(Stage::Teen, &habits(6, 0, 0, 0, 0)), "Hashitamatchi");
        assert_eq!(Pet::species_of(Stage::Teen, &habits(6, 9, 0, 0, 0)), "Tonmarutchi", "snacks win the tie-break");
    }

    #[test]
    fn adults_follow_the_habit_most_over_its_target() {
        let adult = |h| Pet::species_of(Stage::Adult, &h);
        assert_eq!(adult(habits(6, 0, 0, 0, 0)), "Tarakotchi");
        assert_eq!(adult(habits(0, 6, 0, 0, 0)), "Ichigotchi");
        assert_eq!(adult(habits(0, 0, 3, 0, 0)), "Kuchipatchi");
        assert_eq!(adult(habits(0, 0, 0, 3, 0)), "Nyorotchi");
        assert_eq!(adult(habits(0, 0, 0, 0, 2)), "Mametchi");
        assert_eq!(adult(habits(6, 5, 2, 2, 1)), "Tarakotchi", "only reached targets count");
        assert_eq!(adult(habits(7, 0, 0, 0, 4)), "Mametchi", "4/2 is further over than 7/6");
        assert_eq!(adult(habits(5, 5, 2, 2, 1)), "Futsutchi", "no target reached is Plain");
    }

    #[test]
    fn only_helpful_actions_count() {
        let mut p = awake_adult();
        p.happy = MAX_HEARTS;
        p.feed_snack();
        p.finish_game(3, 3);
        p.scold();
        assert_eq!(p.habits, [0; 5], "nothing helped, nothing counted");
        p.happy = 1;
        p.feed_snack();
        p.happy = 1;
        p.finish_game(3, 3);
        p.disc_call = Some(0);
        p.scold();
        p.hunger = 1;
        p.use_item(Item::Salad).unwrap();
        p.use_item(Item::Vitamins).unwrap();
        assert_eq!(p.habits, habits(1, 1, 1, 1, 1));
        p.feed_meal();
        assert_eq!(p.habits, habits(1, 1, 1, 1, 1), "meals show no preference");
    }

    #[test]
    fn growing_up_uses_the_stage_just_finished_and_resets_the_counters() {
        let mut p = Pet::new(1);
        p.stage = Stage::Child;
        p.habits = habits(6, 0, 0, 0, 0);
        p.stage_age = p.cfg.child_secs;
        p.grow();
        assert_eq!(p.species, "Hashitamatchi");
        assert_eq!(p.habits, [0; 5]);
    }

    #[test]
    fn each_form_has_its_personality_and_they_change_the_rates() {
        assert_eq!(Personality::of("Marutchi"), Personality::None);
        assert_eq!(Personality::of("Futsutchi"), Personality::Easygoing);
        let hunger_after = |species: &str| {
            let mut p = awake_adult();
            p.species = species.into();
            p.hunger = 4;
            for _ in 0..p.hunger_interval() * 4 / 3 - 1 {
                p.decay();
            }
            p.hunger
        };
        assert_eq!(hunger_after("Futsutchi"), 3);
        assert_eq!(hunger_after("Kuchipatchi"), 4, "gourmets get hungry 25% slower");
        let mut p = awake_adult();
        p.species = "Tarakotchi".into();
        p.happy = 1;
        p.finish_game(3, 3);
        assert_eq!(p.happy, 3, "sporty: +1 extra on a win");
        p.species = "Hashitamatchi".into();
        assert_eq!(p.win_coin_bonus(), 1);
    }

    #[test]
    fn grow_waits_for_the_stage_time_then_resets_the_clock() {
        let mut p = Pet::new(1);
        p.stage = Stage::Baby;
        p.stage_age = p.cfg.baby_secs - 1;
        p.grow();
        assert_eq!(p.stage, Stage::Baby);
        p.stage_age = p.cfg.baby_secs;
        p.grow();
        assert_eq!(p.stage, Stage::Child);
        assert_eq!(p.species, "Marutchi");
        assert_eq!(p.stage_age, 0);
    }

    #[test]
    fn grow_picks_the_adult_from_the_care_record() {
        let mut p = Pet::new(1);
        p.stage = Stage::Teen;
        p.habits = habits(0, 6, 0, 0, 0);
        p.stage_age = p.cfg.teen_secs;
        p.grow();
        assert_eq!(p.stage, Stage::Adult);
        assert_eq!(p.species, "Ichigotchi");
    }

    #[test]
    fn on_track_names_the_next_form_from_the_care_so_far() {
        let mut p = Pet::new(1);
        assert_eq!(p.on_track(), None, "an egg has no track");
        p.stage = Stage::Teen;
        assert_eq!(p.on_track(), Some("Futsutchi"));
        p.habits = habits(6, 0, 0, 0, 0);
        assert_eq!(p.on_track(), Some("Tarakotchi"));
        p.stage = Stage::Adult;
        assert_eq!(p.on_track(), None, "an adult has nothing left to grow into");
    }

    #[test]
    fn the_preview_names_the_form_it_is_heading_for() {
        let mut p = Pet::new(1);
        p.stage = Stage::Teen;
        p.species = "Tonmarutchi".into();
        p.habits = habits(0, 0, 3, 0, 0);
        let text = p.evolution_preview().join("\n");
        assert!(text.contains("On track: Kuchipatchi"), "{text}");
    }

    fn awake_adult() -> Pet {
        let mut p = Pet::new(11);
        p.dev_set_hour(7);
        p.stage = Stage::Adult;
        p.species = "Mametchi".into();
        p.hunger = 4;
        p.happy = 4;
        p
    }

    #[test]
    fn a_pet_left_alone_for_a_day_while_away_gets_by_unharmed() {
        let mut p = awake_adult();
        p.away = true;
        for _ in 0..86_400 {
            p.tick();
        }
        assert!(p.alive());
        assert_eq!(p.mistakes, 0);
        assert!(!p.sick);
        assert!(p.hunger >= 1 && p.happy >= 1);
    }

    #[test]
    fn the_same_day_unattended_with_the_game_open_is_not_harmless() {
        let mut p = awake_adult();
        for _ in 0..86_400 {
            p.tick();
        }
        assert!(p.mistakes > 0 || !p.alive());
    }

    #[test]
    fn being_away_slows_hunger_down() {
        let mut home = awake_adult();
        let mut away = awake_adult();
        away.away = true;
        for _ in 0..1800 {
            home.tick();
            away.tick();
        }
        assert!(away.hunger > home.hunger, "away {} vs home {}", away.hunger, home.hunger);
    }

    #[test]
    fn with_the_protection_off_being_away_still_has_consequences() {
        let mut p = awake_adult();
        p.cfg.away_safe = false;
        p.away = true;
        for _ in 0..(8 * 3600) {
            p.tick();
        }
        assert!(p.mistakes > 0);
    }

    #[test]
    fn a_pet_still_grows_up_while_away() {
        let mut p = Pet::new(13);
        p.dev_set_hour(7);
        p.away = true;
        for _ in 0..(4 * 3600) {
            p.tick();
        }
        assert_eq!(p.stage, Stage::Adult);
    }

    #[test]
    fn shop_items_change_hunger_happiness_and_weight() {
        let mut p = awake_adult();
        p.hunger = 1;
        p.happy = 1;
        p.weight = 10;
        p.use_item(Item::FancyMeal).unwrap();
        assert_eq!((p.hunger, p.weight), (3, 11));
        p.use_item(Item::Toy).unwrap();
        assert_eq!((p.happy, p.weight), (3, 10));
        p.use_item(Item::Salad).unwrap();
        assert_eq!((p.hunger, p.weight), (4, 8));
    }

    #[test]
    fn an_item_that_would_do_nothing_is_refused_so_it_is_not_wasted() {
        let mut p = awake_adult();
        assert!(p.use_item(Item::FancyMeal).is_err(), "already full");
        assert!(p.use_item(Item::Salad).is_err());
        assert!(p.use_item(Item::Toy).is_err(), "already happy");
        p.use_item(Item::Vitamins).unwrap();
        assert!(p.use_item(Item::Vitamins).is_err(), "still protected");
        let mut asleep = awake_adult();
        asleep.hunger = 1;
        asleep.asleep = true;
        assert!(asleep.use_item(Item::FancyMeal).is_err(), "it is sleeping");
        assert_eq!(asleep.hunger, 1);
    }

    #[test]
    fn vitamins_stop_sickness_until_they_wear_off() {
        let mut p = awake_adult();
        p.cfg.sick_odds = 1;
        p.cfg.away_safe = false;
        p.use_item(Item::Vitamins).unwrap();
        for _ in 0..600 {
            p.tick();
            p.hunger = 4;
            p.happy = 4;
        }
        assert!(!p.sick, "protected, so it must not get sick");
        p.immune_until = 0;
        p.tick();
        assert!(p.sick, "with odds of 1 in 1 it falls sick the moment the protection ends");
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
        p.urge = None;
        assert_eq!(p.top_need().map(|n| n.0), Some('p'));
        p.weight = 5;
        assert!(!p.overweight());
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

    fn grown(seed: u64) -> Pet {
        let mut p = Pet::new(seed);
        p.stage = Stage::Child;
        p.species = "Marutchi".into();
        p.dev_set_hour(12);
        p.hunger = 1;
        (p.cfg.urge_odds, p.cfg.naughty_odds, p.cfg.sick_odds) = (u64::MAX, u64::MAX, u64::MAX);
        p
    }

    #[test]
    fn a_meal_brings_the_urge_a_little_later_and_the_toilet_clears_it() {
        let mut p = grown(5);
        p.feed_meal();
        assert!(p.urge.is_none());
        for _ in 0..p.cfg.urge_after_meal_secs + p.cfg.urge_after_meal_extra_secs {
            p.tick();
        }
        assert!(p.urge.is_some() && p.needs_toilet());
        assert_eq!(p.top_need().map(|n| n.0), Some('t'));
        assert_eq!(p.toilet(), "Ahh, much better.");
        assert_eq!(p.toilet(), "Doesn't need the toilet.");
    }

    #[test]
    fn a_snack_does_not_bring_the_urge() {
        let mut p = grown(5);
        p.happy = 1;
        p.feed_snack();
        for _ in 0..1000 {
            p.tick();
        }
        assert!(p.urge.is_none());
    }

    #[test]
    fn an_ignored_urge_ends_in_an_accident_but_never_while_away() {
        let mut p = grown(6);
        p.hunger = 4;
        p.happy = 3;
        p.urge = Some(0);
        p.away = true;
        for _ in 0..p.cfg.urge_patience_secs * 2 {
            p.tick();
        }
        assert!(p.urge.is_some() && p.mistakes == 0, "safe while away");
        p.away = false;
        for _ in 0..p.cfg.urge_patience_secs {
            p.tick();
        }
        assert!(p.urge.is_none());
        assert_eq!(p.mistakes, 1);
        assert!(p.happy < 3);
    }

    #[test]
    fn a_pile_from_an_old_save_becomes_a_pending_urge() {
        let mut p = Pet::new(2);
        let mut json: serde_json::Value = serde_json::to_value(&p).unwrap();
        json["poops"] = 2.into();
        let mut old: Pet = serde_json::from_value(json).unwrap();
        old.set_env(Config::default(), 0);
        assert_eq!(old.urge, Some(0));
        assert!(serde_json::to_value(&old).unwrap().get("poops").is_none());
        p.set_env(Config::default(), 0);
        assert_eq!(p.urge, None);
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
                p.toilet();
                p.medicine();
                p.medicine();
                p.scold();
            }
        }
        assert!(p.alive());
        assert!(p.stage >= Stage::Teen || p.stage == Stage::Teen);
    }
}
