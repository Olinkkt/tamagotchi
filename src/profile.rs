//! Things that outlive a single pet: coins, achievements, and the adults you have raised.

use serde::{Deserialize, Serialize};

use crate::pet::{Pet, Stage};

pub const ADULT_SPECIES: [&str; 5] = ["Mametchi", "Kuchipatchi", "Tarakotchi", "Ichigotchi", "Nyorotchi"];

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

pub const ACHIEVEMENTS: [Achievement; 10] = [
    Achievement { id: "hatched", name: "First steps", how: "hatch an egg" },
    Achievement { id: "grown_up", name: "Grown up", how: "raise a pet to adulthood" },
    Achievement { id: "mametchi", name: "Teacher's pet", how: "raise a Mametchi" },
    Achievement { id: "spotless", name: "Spotless", how: "reach adulthood with no care mistakes" },
    Achievement { id: "survivor", name: "Survivor", how: "keep a pet alive for a whole day" },
    Achievement { id: "old_age", name: "A long life", how: "a pet dies of old age" },
    Achievement { id: "commit_fed", name: "Fed by git", how: "a git commit feeds your pet" },
    Achievement { id: "gamer", name: "Gamer", how: "win 10 mini-games" },
    Achievement { id: "rich", name: "Piggy bank", how: "earn 100 coins in total" },
    Achievement { id: "collector", name: "Collector", how: "raise all five adult species" },
];

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
            "commit_fed" => pet.last_commit_meal.is_some(),
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
        old.last_commit_meal = Some(1);
        profile.check(&old);
        for id in ["gamer", "rich", "old_age", "survivor", "commit_fed"] {
            assert!(profile.has(id), "{id}");
        }
    }

    #[test]
    fn an_egg_unlocks_nothing() {
        let mut profile = Profile::default();
        assert!(profile.check(&Pet::new(1)).is_empty());
    }
}
