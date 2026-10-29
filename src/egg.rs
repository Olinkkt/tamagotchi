//! The egg's 30 seconds. Everything it does is worked out from its age and a seed, so the same egg
//! moves the same way after a save, a load or a restart, and no two eggs move alike.
//!
//! The seed is the unix time the egg was laid (`clock - stage_age`), which the save already keeps.
//! The age is `stage_age` plus the part of a second the game has counted since the last tick.

use crate::pet::Pet;

/// When the egg hatches. `Pet` hatches on its own at the same 30 seconds.
pub const HATCH_MS: u64 = 30_000;
/// When each crack stage (1 to 6) appears, in milliseconds.
const CRACK_AT_MS: [u64; 6] = [20_000, 22_000, 24_000, 25_000, 26_000, 28_000];
/// The egg freezes here, holding its breath, and goes white for the last `FLASH_MS`.
const PAUSE_MS: u64 = 29_700;
const FLASH_MS: u64 = 120;
/// The one shake that is stronger than the rest.
const STRONGEST_MS: (u64, u64) = (29_000, 29_450);
/// Shell fragments fly off between these times, one every `CHIP_EVERY_MS`, each for `CHIP_LIFE_MS`.
const CHIPS_MS: (u64, u64) = (26_000, PAUSE_MS);
const CHIP_EVERY_MS: u64 = 450;
const CHIP_LIFE_MS: u64 = 420;
/// The row above which the shell lifts off when it starts to separate.
pub const SPLIT_ROW: i32 = 12;

/// How lively the egg is during one stretch of its life. Every range is a bound, not a value.
struct Phase {
    from_ms: u64,
    /// Time between one shake and the next.
    gap_ms: (u64, u64),
    /// How far a shake moves it sideways, in pixels.
    px: (i32, i32),
    /// Chance (percent) of each extra shake in a row, and the most in a row.
    more_pct: u64,
    most: u32,
    /// The most it tilts, in degrees, and the chance (percent) a shake tilts at all.
    tilt: i32,
    tilt_pct: u64,
    /// The chance (percent) of a shake being a jump, and the most pixels it rises.
    jump_pct: u64,
    jump: i32,
}

const PHASES: [Phase; 5] = [
    Phase { from_ms: 0, gap_ms: (2_000, 5_000), px: (1, 1), more_pct: 0, most: 1, tilt: 0, tilt_pct: 0, jump_pct: 0, jump: 0 },
    Phase { from_ms: 8_000, gap_ms: (1_500, 3_500), px: (1, 2), more_pct: 30, most: 2, tilt: 0, tilt_pct: 0, jump_pct: 0, jump: 0 },
    Phase { from_ms: 15_000, gap_ms: (1_000, 2_500), px: (2, 3), more_pct: 50, most: 3, tilt: 4, tilt_pct: 50, jump_pct: 0, jump: 0 },
    Phase { from_ms: 20_000, gap_ms: (700, 1_800), px: (2, 4), more_pct: 55, most: 3, tilt: 5, tilt_pct: 70, jump_pct: 25, jump: 2 },
    Phase { from_ms: 26_000, gap_ms: (200, 800), px: (3, 5), more_pct: 70, most: 4, tilt: 6, tilt_pct: 80, jump_pct: 40, jump: 3 },
];

fn phase_at(ms: u64) -> &'static Phase {
    PHASES.iter().rev().find(|p| ms >= p.from_ms).unwrap_or(&PHASES[0])
}

/// SplitMix64: a small, fixed generator, so a seed always gives the same egg on every platform.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn range(&mut self, (lo, hi): (u64, u64)) -> u64 {
        lo + self.next() % (hi - lo + 1)
    }

    fn px(&mut self, (lo, hi): (i32, i32)) -> i32 {
        self.range((lo as u64, hi as u64)) as i32
    }

    fn percent(&mut self, chance: u64) -> bool {
        self.next() % 100 < chance
    }

    fn sign(&mut self) -> i32 {
        if self.next() & 1 == 0 { 1 } else { -1 }
    }
}

/// The egg's seed: when it was laid.
pub fn seed(p: &Pet) -> u64 {
    (p.clock - p.stage_age as i64) as u64
}

/// One quick movement: out, a rebound, then back to rest.
struct Shake {
    at: u64,
    len: u64,
    dx: i32,
    tilt: i32,
    jump: i32,
}

/// Every shake that starts before `age_ms`, in order. Always the same for a seed.
fn shakes(seed: u64, age_ms: u64) -> Vec<Shake> {
    let mut rng = Rng(seed);
    let mut out = Vec::new();
    let mut t = rng.range(PHASES[0].gap_ms);
    let mut last = [1, -1];
    while t < PAUSE_MS && t <= age_ms {
        let ph = phase_at(t);
        let mut count = 1;
        while count < ph.most && rng.percent(ph.more_pct) {
            count += 1;
        }
        for _ in 0..count {
            // Left or right at random, but never three the same way in a row.
            let mut dir = rng.sign();
            if last[0] == dir && last[1] == dir {
                dir = -dir;
            }
            last = [last[1], dir];
            let tilt = if rng.percent(ph.tilt_pct) { rng.sign() * rng.px((1, ph.tilt.max(1))) } else { 0 };
            let jump = if rng.percent(ph.jump_pct) { rng.px((1, ph.jump.max(1))) } else { 0 };
            let len = rng.range((110, 170));
            out.push(Shake { at: t, len, dx: dir * rng.px(ph.px), tilt, jump });
            t += len;
        }
        t += rng.range(ph.gap_ms);
    }
    out
}

/// Where a shake has got to (`u`, 0 to 1): out for the first half, a half-size rebound, then rest.
/// A jump leaves the ground for the first third and bounces once on landing.
fn shape(s: &Shake, u: f32) -> (i32, i32, i32) {
    let (dx, tilt) = match u {
        u if u < 0.5 => (s.dx, s.tilt),
        u if u < 0.75 => (-s.dx / 2, -s.tilt / 2),
        _ => (0, 0),
    };
    let dy = match u {
        u if u < 0.35 => -s.jump,
        u if (0.5..0.65).contains(&u) && s.jump > 1 => -1,
        _ => 0,
    };
    (dx, dy, tilt)
}

/// How the egg looks at one moment.
#[derive(Debug, Default, PartialEq)]
pub struct Pose {
    pub dx: i32,
    pub dy: i32,
    /// Degrees, clockwise, around the egg's feet.
    pub tilt: i32,
    /// How many pixels the top of the shell is lifted off the bottom.
    pub lift: i32,
    /// Which cracks show: 0 (none) to 6.
    pub cracks: usize,
    /// White for an instant just before the egg breaks.
    pub flash: bool,
}

pub fn crack_stage(age_ms: u64) -> usize {
    CRACK_AT_MS.iter().filter(|&&at| age_ms >= at).count()
}

/// The strongest shake: three hard ones in a row, the way depending only on the seed.
fn strongest(seed: u64, age_ms: u64) -> (i32, i32, i32) {
    let u = (age_ms - STRONGEST_MS.0) as f32 / (STRONGEST_MS.1 - STRONGEST_MS.0) as f32;
    let dir = if Rng(seed ^ 0xA5A5).next() & 1 == 0 { 1 } else { -1 };
    let n = (u * 3.0) as i32;
    let hit = Shake { at: 0, len: 0, dx: dir * if n % 2 == 0 { 5 } else { -5 }, tilt: dir * if n % 2 == 0 { 6 } else { -6 }, jump: 3 };
    shape(&hit, u * 3.0 - n as f32)
}

pub fn pose(seed: u64, age_ms: u64) -> Pose {
    let cracks = crack_stage(age_ms);
    let resting = Pose { cracks, lift: if age_ms >= 28_000 { 1 } else { 0 }, ..Pose::default() };
    if age_ms >= PAUSE_MS {
        return Pose { flash: age_ms + FLASH_MS >= HATCH_MS, ..resting };
    }
    let (dx, dy, tilt) = if (STRONGEST_MS.0..STRONGEST_MS.1).contains(&age_ms) {
        strongest(seed, age_ms)
    } else {
        let active = shakes(seed, age_ms).into_iter().find(|s| (s.at..s.at + s.len).contains(&age_ms));
        active.map_or((0, 0, 0), |s| shape(&s, (age_ms - s.at) as f32 / s.len as f32))
    };
    let moving = dx != 0 || tilt != 0 || dy != 0;
    let lift = match age_ms {
        a if a >= 26_000 && moving => 2,
        a if a >= 24_000 && moving => 1,
        _ => resting.lift,
    };
    Pose { dx, dy, tilt, lift, ..resting }
}

/// A piece of shell in the air, as pixels from the top middle of the egg.
pub struct Chip {
    pub dx: i32,
    pub dy: i32,
}

pub fn chips(seed: u64, age_ms: u64) -> Vec<Chip> {
    let (from, to) = CHIPS_MS;
    if !(from..to).contains(&age_ms) {
        return Vec::new();
    }
    let last = (age_ms - from) / CHIP_EVERY_MS;
    (0..=last)
        .filter_map(|i| {
            let born = from + i * CHIP_EVERY_MS;
            let mut rng = Rng(seed ^ (i + 1).wrapping_mul(0xD1B5_4A32_D192_ED03));
            let (dir, speed) = (rng.sign(), rng.px((6, 12)));
            let u = age_ms.checked_sub(born).filter(|&a| a < CHIP_LIFE_MS)? as f32 / CHIP_LIFE_MS as f32;
            // Out sideways at an even speed, up and then down under gravity.
            Some(Chip { dx: dir * (u * speed as f32) as i32, dy: (28.0 * u * u - 14.0 * u) as i32 })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trace(seed: u64, from: u64, to: u64) -> Vec<Pose> {
        (from..to).step_by(10).map(|ms| pose(seed, ms)).collect()
    }

    fn moves(seed: u64, from: u64, to: u64) -> usize {
        trace(seed, from, to).windows(2).filter(|w| w[0] != w[1]).count()
    }

    #[test]
    fn the_same_seed_always_gives_the_same_egg() {
        assert_eq!(trace(77, 0, HATCH_MS), trace(77, 0, HATCH_MS));
    }

    #[test]
    fn different_seeds_give_different_eggs() {
        let a: Vec<(i32, i32, i32)> = trace(1, 0, HATCH_MS).iter().map(|p| (p.dx, p.dy, p.tilt)).collect();
        for seed in 2..12 {
            let b: Vec<(i32, i32, i32)> = trace(seed, 0, HATCH_MS).iter().map(|p| (p.dx, p.dy, p.tilt)).collect();
            assert_ne!(a, b, "seed {seed} copies seed 1");
        }
    }

    #[test]
    fn every_phase_stays_inside_its_own_bounds() {
        for seed in 0..40 {
            for (i, ph) in PHASES.iter().enumerate() {
                let end = PHASES.get(i + 1).map_or(PAUSE_MS, |n| n.from_ms);
                for p in trace(seed, ph.from_ms, end) {
                    // The strongest shake (29 s) is the one thing allowed past the phase's reach.
                    if (STRONGEST_MS.0..STRONGEST_MS.1).contains(&(ph.from_ms)) {
                        continue;
                    }
                    assert!(p.dx.abs() <= ph.px.1.max(if i == 4 { 5 } else { 0 }), "seed {seed} phase {i}: {p:?}");
                    assert!(p.tilt.abs() <= ph.tilt.max(if i == 4 { 6 } else { 0 }), "seed {seed} phase {i}: {p:?}");
                    assert!((p.dy == 0) || ph.jump > 0, "seed {seed} phase {i}: {p:?}");
                }
            }
        }
    }

    #[test]
    fn the_first_phase_barely_moves_and_the_last_never_stops() {
        for seed in 0..40 {
            let calm = trace(seed, 0, 8_000);
            assert!(calm.iter().all(|p| p.dx.abs() <= 1 && p.tilt == 0 && p.dy == 0 && p.cracks == 0), "seed {seed}");
            assert!(calm.iter().filter(|p| p.dx != 0).count() < 80, "mostly stationary, seed {seed}");
        }
    }

    #[test]
    fn it_gets_livelier_every_phase() {
        for seed in 0..40 {
            let rates: Vec<f32> = PHASES
                .iter()
                .enumerate()
                .map(|(i, ph)| {
                    let end = PHASES.get(i + 1).map_or(PAUSE_MS, |n| n.from_ms);
                    moves(seed, ph.from_ms, end) as f32 / (end - ph.from_ms) as f32
                })
                .collect();
            assert!(rates[4] > rates[3] && rates[3] > rates[0] && rates[4] > rates[2], "seed {seed}: {rates:?}");
        }
    }

    #[test]
    fn the_last_seconds_shake_hardest() {
        for seed in 0..40 {
            let reach = |from, to| trace(seed, from, to).iter().map(|p| p.dx.abs()).max().unwrap();
            assert!(reach(26_000, PAUSE_MS) >= 3, "seed {seed}");
            assert!(reach(0, 8_000) <= 1);
            assert_eq!(reach(STRONGEST_MS.0, STRONGEST_MS.1), 5, "the strongest shake");
        }
    }

    #[test]
    fn it_holds_still_for_the_last_300_ms_and_flashes_at_the_very_end() {
        for seed in 0..20 {
            let still = trace(seed, PAUSE_MS, HATCH_MS);
            assert!(still.iter().all(|p| p.dx == 0 && p.dy == 0 && p.tilt == 0), "seed {seed}");
            assert!(!pose(seed, PAUSE_MS).flash && pose(seed, HATCH_MS - 50).flash);
        }
    }

    #[test]
    fn cracks_come_on_schedule() {
        let stages: Vec<usize> = [0, 19_999, 20_000, 22_000, 24_000, 25_000, 26_000, 27_999, 28_000, 29_999].map(crack_stage).to_vec();
        assert_eq!(stages, [0, 0, 1, 2, 3, 4, 5, 5, 6, 6]);
    }

    #[test]
    fn the_shell_lifts_apart_only_once_it_is_cracking() {
        for seed in 0..40 {
            assert!(trace(seed, 0, 24_000).iter().all(|p| p.lift == 0));
            assert!(trace(seed, 24_000, 26_000).iter().all(|p| p.lift <= 1));
            assert!(trace(seed, 28_000, HATCH_MS).iter().all(|p| p.lift >= 1));
        }
    }

    #[test]
    fn fragments_fly_late_and_only_briefly() {
        assert!(chips(5, 25_999).is_empty());
        assert!((26_000..PAUSE_MS).step_by(50).any(|ms| !chips(5, ms).is_empty()));
        assert!(chips(5, HATCH_MS).is_empty(), "none left once the egg has broken");
        assert!((26_000..PAUSE_MS).step_by(50).all(|ms| chips(5, ms).len() <= 2));
    }

    #[test]
    fn a_loaded_egg_picks_up_where_it_left_off() {
        // Laid at clock 1000, 25 seconds old: same seed whether it was loaded or never saved.
        let mut egg = Pet::new(1000);
        egg.stage_age = 25;
        egg.clock = 1025;
        let again = egg.clone();
        assert_eq!(seed(&egg), 1000);
        assert_eq!(pose(seed(&egg), 25_300), pose(seed(&again), 25_300));
        assert_eq!(pose(seed(&egg), 25_300).cracks, 4);
    }
}
