mod app;
mod art;
// palette, faces and cast are shared with tools/spritegen, which uses parts the game does not.
#[allow(dead_code)]
mod cast;
mod cli;
mod config;
mod egg;
#[allow(dead_code)]
mod faces;
mod font;
mod frame;
mod gallery;
mod games;
mod hud;
mod icons;
#[allow(dead_code)]
mod palette;
mod panel;
mod pet;
mod profile;
mod save;
mod scene;
mod screens;
mod sysinfo;
mod toilet;
mod window;

const MAX_CATCHUP_TICKS: u64 = 86_400;
/// Most shell events applied at once, so a runaway script cannot stall the game.
const MAX_QUEUED_EVENTS: usize = 200;

pub fn fmt_age(secs: u64) -> String {
    let (d, h, m) = (secs / 86_400, secs / 3600 % 24, secs / 60 % 60);
    if d > 0 { format!("{d}d {h}h {m}m") } else if h > 0 { format!("{h}h {m}m") } else { format!("{m}m {}s", secs % 60) }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Subcommands and flags stay headless: they return before any window code runs.
    if let Some(code) = cli::run(&args).or_else(|| cli::flags(&args)) {
        std::process::exit(code);
    }
    window::run(app::App::new());
}
