//! The old terminal frontend, kept behind `--tui` until the window port is finished.

mod draw;

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::{App, Key};

fn key(code: KeyCode) -> Option<Key> {
    Some(match code {
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Esc => Key::Esc,
        KeyCode::Tab => Key::Tab,
        _ => return None,
    })
}

/// Terminals only report key presses, so a direction key keeps the paddle going until the
/// other direction or space is pressed.
fn steer(held: &mut (bool, bool), k: Key) {
    match k {
        Key::Up | Key::Char('w') | Key::Char('k') => *held = (true, false),
        Key::Down | Key::Char('s') | Key::Char('j') => *held = (false, true),
        Key::Char(' ') => *held = (false, false),
        _ => {}
    }
}

pub fn run(app: &mut App) -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut held = (false, false);
    let res = (|| {
        while !app.quit {
            app.hold(held.0, held.1);
            app.update();
            terminal.draw(|f| draw::draw(f, app))?;
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(k) = event::read()? {
                    if let (KeyEventKind::Press, Some(k)) = (k.kind, key(k.code)) {
                        steer(&mut held, k);
                        app.on_key(k);
                    }
                }
            }
        }
        Ok(())
    })();
    ratatui::restore();
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::app_with;
    use crate::app::Mode;
    use crate::games;
    use crate::pet::Stage;
    use ratatui::{backend::TestBackend, Terminal};

    fn render(app: &App, w: u16, h: u16) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(|f| draw::draw(f, app)).unwrap();
        let buf = terminal.backend().buffer().clone();
        (0..h).map(|y| (0..w).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>()).collect()
    }

    #[test]
    fn every_stage_and_state_draws_at_any_size_without_panicking() {
        let stages = [Stage::Egg, Stage::Baby, Stage::Child, Stage::Teen, Stage::Adult, Stage::Dead];
        let sizes = [(200, 60), (120, 40), (100, 30), (80, 24), (64, 22), (63, 22), (64, 21), (40, 12), (10, 3), (1, 1)];
        for stage in stages {
            for tweak in 0..4 {
                let app = app_with(stage, |p| match tweak {
                    1 => p.sick = true,
                    2 => p.asleep = true,
                    3 => {
                        p.hunger = 0;
                        p.poops = 3;
                        p.disc_call = Some(0);
                    }
                    _ => {}
                });
                for (w, h) in sizes {
                    render(&app, w, h);
                }
            }
        }
    }

    #[test]
    fn the_menu_stays_fully_visible_on_an_80_column_terminal() {
        let app = app_with(Stage::Adult, |_| {});
        let screen = render(&app, 80, 24);
        let menu = &screen[22];
        for key in ["f", "s", "p", "l", "m", "c", "d", "b", "a", "g", "q"] {
            assert!(menu.contains(&format!(" {key} ")), "missing the {key} key in: {menu}");
        }
        assert!(menu.contains("feed") && menu.contains("quit"), "labels should still show at 80 columns: {menu}");
    }

    #[test]
    fn a_wide_terminal_gets_the_full_menu_labels() {
        let app = app_with(Stage::Adult, |_| {});
        let screen = render(&app, 140, 40);
        assert!(screen[38].contains("medicine") && screen[38].contains("graveyard"), "{}", screen[38]);
    }

    #[test]
    fn the_awards_screen_fits_a_standard_80x24_terminal() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.mode = Mode::Awards;
        let screen = render(&app, 80, 24).join("\n");
        assert!(screen.contains("Collector") && screen.contains("Nyorotchi"), "{screen}");
    }

    #[test]
    fn the_status_panel_names_what_the_machine_is_doing_to_the_pet() {
        let app = app_with(Stage::Adult, |p| p.load = crate::sysinfo::Strain { busy_cpu: true, low_ram: true, low_battery: true });
        let screen = render(&app, 140, 40).join("\n");
        assert!(screen.contains("sweating") && screen.contains("dizzy") && screen.contains("sleepy"), "{screen}");
        let calm = render(&app_with(Stage::Adult, |_| {}), 140, 40).join("\n");
        assert!(!calm.contains("sweating") && !calm.contains("dizzy"));
    }

    #[test]
    fn the_shop_and_awards_screens_show_their_contents() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.profile.coins = 7;
        app.mode = Mode::Shop;
        let shop = render(&app, 100, 30).join("\n");
        for want in ["SHOP", "Fancy meal", "Vitamins", "◎ 7"] {
            assert!(shop.contains(want), "shop is missing {want}:\n{shop}");
        }
        app.mode = Mode::Awards;
        let awards = render(&app, 100, 30).join("\n");
        for want in ["AWARDS", "First steps", "Adults raised 0/5", "Mametchi"] {
            assert!(awards.contains(want), "awards is missing {want}:\n{awards}");
        }
    }

    #[test]
    fn a_game_draws_at_the_smallest_size() {
        let mut app = app_with(Stage::Adult, |_| {});
        let g = games::Game::new(2, &mut app.pet);
        app.mode = Mode::Game(g);
        let screen = render(&app, 64, 22).join("\n");
        assert!(screen.matches('▀').count() >= 32 * 10, "the game is not drawn at 64x22:\n{screen}");
    }

    #[test]
    fn a_window_that_is_too_small_says_so() {
        let app = app_with(Stage::Adult, |_| {});
        let screen = render(&app, 40, 12).join("\n");
        assert!(screen.contains("bigger window") && screen.contains("40x12"), "{screen}");
        assert!(!render(&app, 64, 22).join("\n").contains("bigger window"));
    }
}
