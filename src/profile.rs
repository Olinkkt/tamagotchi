//! Things that outlive a single pet: coins, achievements, and the adults you have raised.

use serde::{Deserialize, Serialize};

use crate::decor;
use crate::icons::{self, Grid};
use crate::pet::{Care, Pet, Stage};

pub const ADULT_SPECIES: [&str; 6] = ["Mametchi", "Kuchipatchi", "Tarakotchi", "Ichigotchi", "Nyorotchi", "Futsutchi"];

/// Coins for finishing a mini-game: nothing for a loss, a few for a win, more for a perfect game.
pub const COINS_WIN: u32 = 4;
pub const COINS_PERFECT: u32 = 10;
/// A win pays in full only if the last full payout was at least this long ago; otherwise half.
pub const FULL_PAY_GAP_SECS: u64 = 600;

pub fn game_reward(wins: u8, rounds: u8) -> u32 {
    if wins * 2 <= rounds {
        0
    } else if wins == rounds {
        COINS_PERFECT
    } else {
        COINS_WIN
    }
}

pub struct Achievement {
    pub id: &'static str,
    pub name: &'static str,
    pub how: &'static str,
}

pub const ACHIEVEMENTS: [Achievement; 9] = [
    Achievement { id: "hatched", name: "First steps", how: "hatch an egg" },
    Achievement { id: "grown_up", name: "Grown up", how: "raise a pet to adulthood" },
    Achievement { id: "mametchi", name: "Teacher's pet", how: "raise a Mametchi" },
    Achievement { id: "spotless", name: "Spotless", how: "reach adulthood with no care mistakes" },
    Achievement { id: "survivor", name: "Survivor", how: "keep a pet alive for a whole day" },
    Achievement { id: "old_age", name: "A long life", how: "a pet dies of old age" },
    Achievement { id: "gamer", name: "Gamer", how: "win 10 mini-games" },
    Achievement { id: "rich", name: "Piggy bank", how: "earn 100 coins in total" },
    Achievement { id: "collector", name: "Collector", how: "raise all six adult species" },
];

/// Something the pet can bring home while you are away. Shown as a silhouette until found.
pub struct Keepsake {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: Grid,
}

pub const KEEPSAKES: [Keepsake; 6] = [
    Keepsake { id: "acorn", name: "Acorn", icon: icons::ACORN },
    Keepsake { id: "pebble", name: "Pebble", icon: icons::PEBBLE },
    Keepsake { id: "feather", name: "Feather", icon: icons::FEATHER },
    Keepsake { id: "button", name: "Button", icon: icons::BUTTON },
    Keepsake { id: "seashell", name: "Seashell", icon: icons::SEASHELL },
    Keepsake { id: "clover", name: "Clover", icon: icons::CLOVER },
];

/// A care badge: do something this many times, over all your pets, to earn it.
pub struct Badge {
    pub care: Care,
    pub goal: u32,
    pub name: &'static str,
}

/// Three tiers (bronze, silver, gold) for each kind of care.
pub const BADGES: [Badge; 12] = [
    Badge { care: Care::Meal, goal: 100, name: "Caring cook" },
    Badge { care: Care::Meal, goal: 500, name: "Head chef" },
    Badge { care: Care::Meal, goal: 1000, name: "Master chef" },
    Badge { care: Care::Snack, goal: 100, name: "Sweet tooth" },
    Badge { care: Care::Snack, goal: 500, name: "Candy fan" },
    Badge { care: Care::Snack, goal: 1000, name: "Candy king" },
    Badge { care: Care::Game, goal: 100, name: "Playmate" },
    Badge { care: Care::Game, goal: 500, name: "Game night" },
    Badge { care: Care::Game, goal: 1000, name: "Arcade star" },
    Badge { care: Care::Scold, goal: 100, name: "Firm friend" },
    Badge { care: Care::Scold, goal: 500, name: "Strict teacher" },
    Badge { care: Care::Scold, goal: 1000, name: "Headmaster" },
];

pub fn care_name(c: Care) -> &'static str {
    match c {
        Care::Meal => "meals",
        Care::Snack => "snacks",
        Care::Game => "games",
        Care::Scold => "scolds",
    }
}

/// One gift for every this many seconds away, up to `MAX_GIFTS` when you come back.
pub const GIFT_EVERY_SECS: u64 = 2 * 3600;
pub const MAX_GIFTS: u64 = 5;

/// What the pet left while you were away.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gift {
    Coins(u32),
    Keepsake(&'static str),
}

/// The memorial plaque on the wall: the last pet you lost, as the graveyard has it.
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct Plaque {
    pub species: String,
    /// How long it lived, short ("3d", "5h").
    pub lived: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(default)]
pub struct Profile {
    pub coins: u32,
    pub coins_earned: u32,
    pub games_won: u32,
    pub achievements: Vec<String>,
    /// Adult species you have raised.
    pub adults: Vec<String>,
    /// Unix time of the last game that paid in full.
    pub last_full_pay: u64,
    /// Keepsakes found (ids from `KEEPSAKES`).
    pub keepsakes: Vec<String>,
    /// Room decorations bought and standing in the room (ids from `decor::DECOR`).
    pub decor: Vec<String>,
    /// The keepsake standing on the windowsill.
    pub shelf: Option<String>,
    pub plaque: Option<Plaque>,
    /// How many graveyard entries there were when the plaque was bought: one plaque per pet.
    pub plaque_pets: usize,
    /// How much care you have given in all, indexed by `Care as usize`.
    pub cared: [u32; 4],
    /// Unix time the game last ran; what gifts are counted from. 0 until the first run.
    pub last_seen: u64,
}

impl Profile {
    pub fn add_coins(&mut self, n: u32) {
        self.coins += n;
        self.coins_earned += n;
    }

    /// What a game's reward really is at `now`: full once per `FULL_PAY_GAP_SECS`, half in between.
    pub fn pay_for(&mut self, reward: u32, now: u64) -> u32 {
        if reward == 0 {
            0
        } else if now.saturating_sub(self.last_full_pay) >= FULL_PAY_GAP_SECS {
            self.last_full_pay = now;
            reward
        } else {
            reward / 2
        }
    }

    /// Hand out the gifts for the time since the game last ran, and note that it runs now.
    /// A pet that is not there to leave them (an egg, or gone) leaves none. Never takes anything
    /// away, and more than `MAX_GIFTS` worth of time away just stops adding up.
    pub fn collect_gifts(&mut self, now: u64, pet_is_there: bool, mut rand: impl FnMut(u64) -> u64) -> Vec<Gift> {
        let away = if self.last_seen == 0 { 0 } else { now.saturating_sub(self.last_seen) };
        self.last_seen = now;
        let mut gifts = Vec::new();
        if !pet_is_there {
            return gifts;
        }
        for _ in 0..(away / GIFT_EVERY_SECS).min(MAX_GIFTS) {
            let unfound: Vec<&Keepsake> = KEEPSAKES.iter().filter(|k| !self.found(k.id)).collect();
            if !unfound.is_empty() && rand(3) == 0 {
                let k = unfound[rand(unfound.len() as u64) as usize];
                self.keepsakes.push(k.id.to_string());
                gifts.push(Gift::Keepsake(k.id));
            } else {
                let coins = 2 + rand(4) as u32;
                self.add_coins(coins);
                gifts.push(Gift::Coins(coins));
            }
        }
        gifts
    }

    /// Add one piece of care to the totals. Returns the badge it earned, if it just did.
    pub fn record(&mut self, care: Care) -> Option<&'static Badge> {
        let n = &mut self.cared[care as usize];
        *n = n.saturating_add(1);
        let n = *n;
        BADGES.iter().find(|b| b.care == care && b.goal == n)
    }

    pub fn care_count(&self, care: Care) -> u32 {
        self.cared[care as usize]
    }

    pub fn has_badge(&self, b: &Badge) -> bool {
        self.care_count(b.care) >= b.goal
    }

    pub fn badges_earned(&self) -> usize {
        BADGES.iter().filter(|b| self.has_badge(b)).count()
    }

    /// The badge closest to being earned, and how many more it takes.
    pub fn next_badge(&self) -> Option<(&'static Badge, u32)> {
        BADGES.iter().filter(|b| !self.has_badge(b)).map(|b| (b, b.goal - self.care_count(b.care))).min_by_key(|&(_, left)| left)
    }

    pub fn placed(&self, id: &str) -> bool {
        self.decor.iter().any(|d| d == id)
    }

    /// Buy a decoration and put it in its spot, taking out whatever was there.
    pub fn buy_decor(&mut self, d: &decor::Decor) -> Result<(), &'static str> {
        if self.placed(d.id) {
            return Err("It is already in the room.");
        }
        if self.coins < d.price {
            return Err("Not enough coins.");
        }
        self.coins -= d.price;
        self.decor.retain(|id| decor::by_id(id).is_some_and(|o| o.spot != d.spot));
        self.decor.push(d.id.to_string());
        Ok(())
    }

    /// Stand a found keepsake on the windowsill, or take it down if it is already there.
    pub fn place_keepsake(&mut self, id: &str) -> bool {
        if !self.found(id) {
            return false;
        }
        self.shelf = if self.shelf.as_deref() == Some(id) { None } else { Some(id.to_string()) };
        true
    }

    /// Hang a plaque for the newest graveyard entry (`entries` are newest first, as the
    /// graveyard lists them). Once per pet.
    pub fn buy_plaque(&mut self, entries: &[String]) -> Result<(), &'static str> {
        let Some(last) = entries.first() else { return Err("No pet has passed away yet.") };
        if self.plaque_pets >= entries.len() {
            return Err("The last pet already has its plaque.");
        }
        if self.coins < decor::PLAQUE_PRICE {
            return Err("Not enough coins.");
        }
        let parts: Vec<&str> = last.split(" | ").collect();
        let species = parts.get(1).copied().unwrap_or("").to_string();
        let lived = parts.get(2).and_then(|l| l.strip_prefix("lived ")).and_then(|l| l.split_whitespace().next()).unwrap_or("?").to_string();
        self.coins -= decor::PLAQUE_PRICE;
        self.plaque = Some(Plaque { species, lived });
        self.plaque_pets = entries.len();
        Ok(())
    }

    pub fn found(&self, id: &str) -> bool {
        self.keepsakes.iter().any(|k| k == id)
    }

    pub fn has(&self, id: &str) -> bool {
        self.achievements.iter().any(|a| a == id)
    }

    pub fn raised(&self, species: &str) -> bool {
        self.adults.iter().any(|s| s == species)
    }

    fn earned(&self, id: &str, pet: &Pet) -> bool {
        let adult = pet.stage == Stage::Adult;
        match id {
            "hatched" => pet.stage != Stage::Egg,
            "grown_up" => adult,
            "mametchi" => adult && pet.species == "Mametchi",
            "spotless" => adult && pet.mistakes == 0,
            "survivor" => pet.age >= 86_400,
            "old_age" => pet.cause_of_death.as_deref() == Some("old age"),
            "gamer" => self.games_won >= 10,
            "rich" => self.coins_earned >= 100,
            "collector" => self.adults.len() >= ADULT_SPECIES.len(),
            _ => false,
        }
    }

    /// Note the adult species the pet has become and unlock whatever it now qualifies for.
    /// Returns only the achievements that were newly unlocked.
    pub fn check(&mut self, pet: &Pet) -> Vec<&'static Achievement> {
        if pet.stage == Stage::Adult && ADULT_SPECIES.contains(&pet.species.as_str()) && !self.raised(&pet.species) {
            self.adults.push(pet.species.clone());
        }
        let mut fresh = Vec::new();
        for a in ACHIEVEMENTS.iter() {
            if !self.has(a.id) && self.earned(a.id, pet) {
                self.achievements.push(a.id.to_string());
                fresh.push(a);
            }
        }
        fresh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adult(species: &str) -> Pet {
        let mut p = Pet::new(5);
        p.stage = Stage::Adult;
        p.species = species.into();
        p
    }

    #[test]
    fn one_gift_per_two_hours_away_up_to_five() {
        let mut roll = 1u64;
        let mut rand = |n: u64| {
            roll = roll.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (roll >> 33) % n
        };
        let mut p = Profile { last_seen: 1_000, ..Profile::default() };
        assert!(p.collect_gifts(1_000 + 2 * 3600 - 1, true, &mut rand).is_empty(), "under two hours is nothing");
        p.last_seen = 1_000;
        assert_eq!(p.collect_gifts(1_000 + 4 * 3600 + 5, true, &mut rand).len(), 2);
        p.last_seen = 1_000;
        assert_eq!(p.collect_gifts(1_000 + 30 * 24 * 3600, true, &mut rand).len() as u64, MAX_GIFTS, "the pile stops at the cap");
        assert_eq!(p.last_seen, 1_000 + 30 * 24 * 3600);
    }

    #[test]
    fn gifts_are_a_few_coins_or_a_new_keepsake_and_never_a_repeat() {
        let mut p = Profile { last_seen: 1, ..Profile::default() };
        let mut n = 0u64;
        for round in 0..40 {
            p.last_seen = 1;
            let gifts = p.collect_gifts(1 + 10 * 3600, true, |m| {
                n += 1;
                n % m
            });
            for g in gifts {
                match g {
                    Gift::Coins(c) => assert!((2..=5).contains(&c), "round {round}: {c}"),
                    Gift::Keepsake(id) => assert!(p.found(id)),
                }
            }
        }
        let mut ids = p.keepsakes.clone();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), p.keepsakes.len(), "no keepsake is given twice");
        assert!(p.coins > 0 && p.coins == p.coins_earned);
    }

    #[test]
    fn no_pet_to_leave_gifts_or_a_first_run_means_none() {
        let mut p = Profile::default();
        assert!(p.collect_gifts(99_999, true, |_| 0).is_empty(), "a first run has nothing to count from");
        assert_eq!(p.last_seen, 99_999);
        assert!(p.collect_gifts(99_999 + 10 * 3600, false, |_| 0).is_empty(), "an egg or a gone pet leaves none");
        assert_eq!(p.coins, 0);
    }

    #[test]
    fn a_decoration_costs_coins_and_swaps_what_was_on_its_spot() {
        let mut p = Profile { coins: 200, ..Profile::default() };
        let (lamp, teddy, fern) = (decor::by_id("lamp").unwrap(), decor::by_id("teddy").unwrap(), decor::by_id("fern").unwrap());
        assert!(p.buy_decor(lamp).is_ok());
        assert_eq!(p.coins, 200 - lamp.price);
        assert!(p.buy_decor(lamp).is_err(), "it is already there");
        assert!(p.buy_decor(fern).is_ok());
        assert!(p.buy_decor(teddy).is_ok());
        assert!(p.placed("teddy") && !p.placed("lamp") && p.placed("fern"), "the teddy took the lamp's spot only");
        let mut poor = Profile { coins: 5, ..Profile::default() };
        assert!(poor.buy_decor(fern).is_err() && poor.decor.is_empty() && poor.coins == 5);
    }

    #[test]
    fn the_plaque_records_the_last_pet_once_per_pet() {
        let grave = |species: &str, lived: &str| format!("2026-11-01 | {species} | lived {lived} | 3 care mistakes | cause: old age");
        let mut p = Profile { coins: 100, ..Profile::default() };
        assert!(p.buy_plaque(&[]).is_err());
        let one = [grave("Mametchi", "3d 4h")];
        assert!(p.buy_plaque(&one).is_ok());
        assert_eq!(p.plaque, Some(Plaque { species: "Mametchi".into(), lived: "3d".into() }));
        assert_eq!(p.coins, 100 - decor::PLAQUE_PRICE);
        assert!(p.buy_plaque(&one).is_err(), "once per pet");
        let two = [grave("Ichigotchi", "5h 2m"), grave("Mametchi", "3d 4h")];
        assert!(p.buy_plaque(&two).is_ok());
        assert_eq!(p.plaque.as_ref().map(|q| q.lived.as_str()), Some("5h"), "the new plaque replaces the old one");
    }

    #[test]
    fn a_keepsake_goes_on_the_sill_only_once_found_and_comes_down_again() {
        let mut p = Profile::default();
        assert!(!p.place_keepsake("acorn"));
        p.keepsakes.push("acorn".into());
        assert!(p.place_keepsake("acorn") && p.shelf.as_deref() == Some("acorn"));
        assert!(p.place_keepsake("acorn") && p.shelf.is_none());
    }

    #[test]
    fn badges_come_at_100_500_and_1000_of_each_kind_of_care() {
        let mut p = Profile::default();
        let mut earned = Vec::new();
        for _ in 0..1000 {
            if let Some(b) = p.record(Care::Snack) {
                earned.push(b.name);
            }
        }
        assert_eq!(earned, ["Sweet tooth", "Candy fan", "Candy king"]);
        assert_eq!(p.badges_earned(), 3);
        assert_eq!(p.care_count(Care::Meal), 0, "kinds of care are counted apart");
        let (next, left) = p.next_badge().unwrap();
        assert_eq!((next.care, left), (Care::Meal, 100));
        p.cared[Care::Game as usize] = 90;
        assert_eq!(p.next_badge().map(|(b, l)| (b.name, l)), Some(("Playmate", 10)));
    }

    #[test]
    fn every_keepsake_icon_is_a_12_by_12_grid() {
        for k in KEEPSAKES.iter() {
            assert_eq!(k.icon.len(), 12, "{}", k.name);
            assert!(k.icon.iter().all(|r| r.chars().count() == 12), "{}", k.name);
        }
    }

    #[test]
    fn a_loss_pays_nothing_a_win_a_little_and_a_perfect_game_more() {
        assert_eq!(game_reward(2, 5), 0);
        assert_eq!(game_reward(3, 5), COINS_WIN);
        assert_eq!(game_reward(4, 5), COINS_WIN);
        assert_eq!(game_reward(5, 5), COINS_PERFECT);
        assert_eq!(game_reward(2, 3), COINS_WIN);
        assert_eq!(game_reward(1, 1), COINS_PERFECT);
    }

    #[test]
    fn games_pay_in_full_once_per_gap_and_half_in_between() {
        let mut p = Profile::default();
        assert_eq!(p.pay_for(10, 1000), 10);
        assert_eq!(p.pay_for(10, 1100), 5);
        assert_eq!(p.pay_for(4, 1200), 2);
        assert_eq!(p.pay_for(0, 1300), 0);
        assert_eq!(p.pay_for(10, 1000 + FULL_PAY_GAP_SECS), 10);
    }

    #[test]
    fn earning_coins_also_counts_toward_the_total_earned() {
        let mut p = Profile::default();
        p.add_coins(7);
        p.coins -= 5;
        p.add_coins(3);
        assert_eq!((p.coins, p.coins_earned), (5, 10));
    }

    #[test]
    fn an_adult_unlocks_the_matching_achievements_exactly_once() {
        let mut profile = Profile::default();
        let names: Vec<&str> = profile.check(&adult("Mametchi")).iter().map(|a| a.id).collect();
        for id in ["hatched", "grown_up", "mametchi", "spotless"] {
            assert!(names.contains(&id), "{id} missing from {names:?}");
        }
        assert!(profile.raised("Mametchi"));
        assert!(profile.check(&adult("Mametchi")).is_empty(), "nothing new the second time");
    }

    #[test]
    fn mistakes_stop_an_adult_from_being_spotless() {
        let mut p = adult("Kuchipatchi");
        p.mistakes = 1;
        let mut profile = Profile::default();
        profile.check(&p);
        assert!(profile.has("grown_up") && !profile.has("spotless") && !profile.has("mametchi"));
    }

    #[test]
    fn the_collector_award_needs_all_five_adults() {
        let mut profile = Profile::default();
        for (i, s) in ADULT_SPECIES.iter().enumerate() {
            profile.check(&adult(s));
            assert_eq!(profile.has("collector"), i == ADULT_SPECIES.len() - 1, "after {} adults", i + 1);
        }
    }

    #[test]
    fn counters_and_a_peaceful_death_unlock_the_rest() {
        let mut profile = Profile { games_won: 10, coins_earned: 100, ..Profile::default() };
        let mut old = adult("Nyorotchi");
        old.stage = Stage::Dead;
        old.cause_of_death = Some("old age".into());
        old.age = 200_000;
        profile.check(&old);
        for id in ["gamer", "rich", "old_age", "survivor"] {
            assert!(profile.has(id), "{id}");
        }
    }

    #[test]
    fn an_egg_unlocks_nothing() {
        let mut profile = Profile::default();
        assert!(profile.check(&Pet::new(1)).is_empty());
    }
}
