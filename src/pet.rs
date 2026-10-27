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

pub const MAX_HEARTS: u8 = 4;
const DAY: u64 = 86_400;
const AWAKE_SECS: u64 = 16 * 3600;
/// Seconds a need can go unanswered before it counts as a care mistake.
const MISTAKE_AFTER: u64 = 900;

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
        Pet {
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
        }
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
        if self.stage == Stage::Adult && self.stage_age >= 2 * DAY {
            self.die("old age");
        }
    }

    fn update_sleep(&mut self) {
        let asleep = self.age % DAY >= AWAKE_SECS;
        if asleep != self.asleep {
            self.asleep = asleep;
            self.light_on = true;
            self.light_t = 0;
            let msg = if asleep { "went to sleep." } else { "woke up!" };
            self.events.push(format!("{} {msg}", self.species));
        }
    }

    fn grow(&mut self) {
        let (limit, next): (u64, Stage) = match self.stage {
            Stage::Egg => (30, Stage::Baby),
            Stage::Baby => (600, Stage::Child),
            Stage::Child => (3600, Stage::Teen),
            Stage::Teen => (7200, Stage::Adult),
            _ => return,
        };
        if self.stage_age < limit {
            return;
        }
        self.stage = next;
        self.stage_age = 0;
        self.species = self.species_for(next).into();
        self.events.push(format!("Grew into a {} ({})!", next.label(), self.species));
    }

    fn species_for(&self, stage: Stage) -> &'static str {
        match stage {
            Stage::Baby => "Babytchi",
            Stage::Child => "Marutchi",
            Stage::Teen if self.mistakes <= 2 => "Tonmarutchi",
            Stage::Teen => "Hashitamatchi",
            Stage::Adult => match (self.mistakes, self.discipline) {
                (0..=1, 75..) => "Mametchi",
                (0..=2, _) => "Kuchipatchi",
                (3..=4, _) => "Tarakotchi",
                (5..=6, _) => "Ichigotchi",
                _ => "Nyorotchi",
            },
            _ => "Egg",
        }
    }

    fn hunger_interval(&self) -> u64 {
        match self.stage {
            Stage::Baby => 240,
            Stage::Child => 480,
            Stage::Teen => 600,
            _ => 720,
        }
    }

    fn decay(&mut self) {
        let hi = self.hunger_interval();
        self.hunger_t += 1;
        if self.hunger_t >= hi {
            self.hunger_t = 0;
            self.hunger = self.hunger.saturating_sub(1);
        }
        self.happy_t += 1;
        if self.happy_t >= hi * 5 / 4 {
            self.happy_t = 0;
            self.happy = self.happy.saturating_sub(1);
        }
        if self.hunger == 0 {
            self.hunger_zero += 1;
            if self.hunger_zero == MISTAKE_AFTER {
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
            if self.happy_zero == MISTAKE_AFTER {
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
            self.poop_next = 900 + self.rand(900);
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
            if self.rand(20_000) == 0 {
                self.get_sick();
            }
            return;
        }
        self.sick_t += 1;
        if self.sick_t == MISTAKE_AFTER {
            self.mistake("ignored an illness");
        }
        if self.sick_t >= 4 * 3600 {
            self.die("illness");
        }
    }

    fn discipline_call(&mut self) {
        match self.disc_call {
            None => {
                if self.rand(1800) == 0 {
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
        "Nom nom! Hunger +1.".into()
    }

    pub fn feed_snack(&mut self) -> String {
        if let Some(m) = self.blocked() {
            return m.into();
        }
        if self.happy >= MAX_HEARTS {
            return "Too happy to snack.".into();
        }
        self.happy += 1;
        self.weight = self.weight.saturating_add(1).min(99);
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
            self.happy = (self.happy + 1).min(MAX_HEARTS);
            if wins == rounds && self.mistakes > 0 {
                self.mistakes -= 1;
                return "Perfect game! Happy +1 and a care mistake forgiven.".into();
            }
            "You won! Happy +1.".into()
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
    fn neglect_kills_eventually() {
        let mut p = Pet::new(7);
        for _ in 0..(3 * 86_400) {
            p.tick();
        }
        assert_eq!(p.stage, Stage::Dead);
        assert!(p.mistakes > 0);
    }

    #[test]
    fn care_keeps_it_alive_to_adulthood() {
        let mut p = Pet::new(9);
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
