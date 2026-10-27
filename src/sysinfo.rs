//! How busy the machine is, so the pet can react (sweat, dizzy eyes, sleepy eyes).
//! Reads /proc and /sys, so it only does anything on Linux; elsewhere everything reads as calm.

use std::fs;

/// The 1-minute load average counts as busy at this share of the CPU cores.
const CPU_BUSY_SHARE: f64 = 0.8;
/// Memory counts as nearly full above this share in use.
const RAM_HIGH_PERCENT: f64 = 85.0;
/// A discharging battery at or below this percentage counts as low.
const BATTERY_LOW_PERCENT: u8 = 20;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Strain {
    pub busy_cpu: bool,
    pub low_ram: bool,
    pub low_battery: bool,
}

pub fn parse_load1(loadavg: &str) -> Option<f64> {
    loadavg.split_whitespace().next()?.parse().ok()
}

pub fn cpu_busy(load1: f64, cores: usize) -> bool {
    load1 >= cores.max(1) as f64 * CPU_BUSY_SHARE
}

/// Percentage of memory in use, from the text of /proc/meminfo (uses MemAvailable).
pub fn ram_used_percent(meminfo: &str) -> Option<f64> {
    let kb = |key: &str| -> Option<f64> {
        meminfo.lines().find(|l| l.starts_with(key))?.split_whitespace().nth(1)?.parse().ok()
    };
    let (total, available) = (kb("MemTotal:")?, kb("MemAvailable:")?);
    (total > 0.0).then(|| 100.0 * (1.0 - available / total))
}

pub fn battery_low(capacity: u8, status: &str) -> bool {
    capacity <= BATTERY_LOW_PERCENT && status.trim() == "Discharging"
}

fn read_battery_low() -> bool {
    let Ok(dir) = fs::read_dir("/sys/class/power_supply") else { return false };
    dir.flatten().any(|entry| {
        let p = entry.path();
        let is_battery = fs::read_to_string(p.join("type")).map(|t| t.trim() == "Battery").unwrap_or(false);
        let capacity = fs::read_to_string(p.join("capacity")).ok().and_then(|s| s.trim().parse::<u8>().ok());
        let status = fs::read_to_string(p.join("status")).unwrap_or_default();
        is_battery && capacity.map_or(false, |c| battery_low(c, &status))
    })
}

pub fn read() -> Strain {
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let busy_cpu = fs::read_to_string("/proc/loadavg").ok().and_then(|s| parse_load1(&s)).map_or(false, |l| cpu_busy(l, cores));
    let low_ram = fs::read_to_string("/proc/meminfo").ok().and_then(|s| ram_used_percent(&s)).map_or(false, |u| u >= RAM_HIGH_PERCENT);
    Strain { busy_cpu, low_ram, low_battery: read_battery_low() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_load_average_is_read_from_the_first_field() {
        assert_eq!(parse_load1("0.52 0.40 0.30 1/200 123\n"), Some(0.52));
        assert_eq!(parse_load1(""), None);
        assert_eq!(parse_load1("busy"), None);
    }

    #[test]
    fn the_cpu_is_busy_at_eighty_percent_of_the_cores() {
        assert!(!cpu_busy(3.0, 4));
        assert!(cpu_busy(3.2, 4));
        assert!(cpu_busy(0.9, 1));
        assert!(cpu_busy(0.8, 0), "zero cores is treated as one");
    }

    #[test]
    fn memory_use_is_worked_out_from_available_memory() {
        let calm = "MemTotal:       16000000 kB\nMemFree:         1000000 kB\nMemAvailable:   12000000 kB\n";
        let tight = "MemTotal:       16000000 kB\nMemFree:          100000 kB\nMemAvailable:    1600000 kB\n";
        assert_eq!(ram_used_percent(calm), Some(25.0));
        assert_eq!(ram_used_percent(tight), Some(90.0));
        assert!(ram_used_percent(tight).unwrap() >= RAM_HIGH_PERCENT);
        assert_eq!(ram_used_percent("nonsense"), None);
    }

    #[test]
    fn the_battery_is_low_only_while_it_is_discharging() {
        assert!(battery_low(15, "Discharging\n"));
        assert!(!battery_low(15, "Charging\n"));
        assert!(!battery_low(15, "Full"));
        assert!(!battery_low(60, "Discharging"));
        assert!(battery_low(20, "Discharging"));
        assert!(!battery_low(21, "Discharging"));
    }

    #[test]
    fn reading_the_real_machine_never_panics() {
        let _ = read();
    }
}
