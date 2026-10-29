mod app;
mod cli;
mod config;
mod games;
mod pet;
mod profile;
mod save;
mod sprites;
mod sysinfo;
mod tui;
mod window;

const MAX_CATCHUP_TICKS: u64 = 86_400;
/// Most shell events applied at once, so a runaway script cannot stall the game.
const MAX_QUEUED_EVENTS: usize = 200;

pub fn fmt_age(secs: u64) -> String {
    let (d, h, m) = (secs / 86_400, secs / 3600 % 24, secs / 60 % 60);
    if d > 0 { format!("{d}d {h}h {m}m") } else if h > 0 { format!("{h}h {m}m") } else { format!("{m}m {}s", secs % 60) }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    // Subcommands and flags stay headless: they return before any window code runs.
    if let Some(code) = cli::run(&args).or_else(|| cli::flags(&args)) {
        std::process::exit(code);
    }
    let mut app = app::App::new();
    if args.iter().any(|a| a == "--tui") {
        let res = tui::run(&mut app);
        app.shutdown();
        return res;
    }
    window::run(app);
    Ok(())
}
