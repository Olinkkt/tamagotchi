//! The whole 240x160 screen around the room: header strip, status panel, hint line and the
//! clickable icon bar. Pixel parts are drawn into the frame; text is handed back as `Label`s
//! for the window to draw sharp, at full resolution.

use crate::app::{App, Key, Mode};
use crate::fmt_age;
use crate::font;
use crate::frame::Frame;
use crate::games::Game;
use crate::icons::{self, Grid};
use crate::palette::Pal;
use crate::pet::{Stage, MAX_HEARTS};
use crate::panel;
use crate::scene;
use crate::screens;

pub const W: i32 = 240;
pub const H: i32 = 160;
/// Top-left corner of the room.
pub const ROOM: (i32, i32) = (0, 12);
pub const PANEL_X: i32 = 160;
const HINT_Y: i32 = 128;
/// The message about the last thing that happened sits on a dimmed strip along the room's foot.
const MESSAGE_Y: i32 = 117;
const BAR_Y: i32 = 140;
const BUTTON: i32 = 16;
const GAP: i32 = 3;
const KEYCAP: i32 = 7;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Look {
    Normal,
    /// Would do nothing right now.
    Idle,
    /// The thing the pet needs most.
    Urgent,
}

#[derive(Clone, Copy, Debug)]
pub struct Button {
    pub key: Key,
    pub icon: Option<Grid>,
    /// Shown on the button when there is no icon, and as the key hint when there is.
    pub label: &'static str,
    pub x: i32,
    pub y: i32,
    pub look: Look,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    Left,
    Centre,
}

/// Text to draw on top of the scene. Positions are in scene pixels; `y` is the middle of the line.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub colour: Pal,
    pub align: Align,
}

fn label(text: impl Into<String>, x: f32, y: f32, colour: Pal, align: Align) -> Label {
    Label { text: text.into(), x, y, colour, align }
}

const ACTIONS: [char; 11] = ['f', 's', 'p', 'l', 'm', 'c', 'd', 'b', 'a', 'g', 'q'];
const DEAD_ACTIONS: [char; 4] = ['n', 'a', 'g', 'q'];

/// Whether an action would do anything right now (the rest are drawn dimmed).
fn useful(app: &App, key: char) -> bool {
    let p = &app.pet;
    match key {
        'f' => p.hunger < MAX_HEARTS,
        's' => p.happy < MAX_HEARTS,
        'm' => p.sick,
        'c' => p.poops > 0,
        'd' => p.disc_call.is_some(),
        'l' => p.asleep,
        _ => true,
    }
}

fn row(items: Vec<(Key, Option<Grid>, &'static str, Look)>) -> Vec<Button> {
    let n = items.len() as i32;
    let x0 = (W - (n * BUTTON + (n - 1) * GAP)) / 2;
    items
        .into_iter()
        .enumerate()
        .map(|(i, (key, icon, label, look))| Button { key, icon, label, x: x0 + i as i32 * (BUTTON + GAP), y: BAR_Y + 2, look })
        .collect()
}

fn key_name(c: char) -> &'static str {
    match c {
        'f' => "f",
        's' => "s",
        'p' => "p",
        'l' => "l",
        'm' => "m",
        'c' => "c",
        'd' => "d",
        'b' => "b",
        'a' => "a",
        'g' => "g",
        'q' => "q",
        'n' => "n",
        '1' => "1",
        '2' => "2",
        '3' => "3",
        _ => "4",
    }
}

fn text_button(c: char) -> (Key, Option<Grid>, &'static str, Look) {
    (Key::Char(c), None, key_name(c), Look::Normal)
}

const BACK: (Key, Option<Grid>, &str, Look) = (Key::Esc, None, "esc", Look::Normal);

/// The buttons along the bottom for the current mode.
pub fn buttons(app: &App) -> Vec<Button> {
    let urgent = app.pet.top_need().map(|n| n.0);
    let action = |c: char| {
        let look = if urgent == Some(c) { Look::Urgent } else if useful(app, c) { Look::Normal } else { Look::Idle };
        (Key::Char(c), icons::for_key(c), key_name(c), look)
    };
    row(match &app.mode {
        Mode::Normal if app.pet.stage == Stage::Dead => DEAD_ACTIONS.iter().map(|&c| action(c)).collect(),
        Mode::Normal => ACTIONS.iter().map(|&c| action(c)).collect(),
        Mode::Chooser => vec![text_button('1'), text_button('2'), BACK],
        Mode::Shop => vec![text_button('1'), text_button('2'), text_button('3'), text_button('4'), BACK],
        Mode::Game(Game::Memory(_)) => vec![text_button('1'), text_button('2'), text_button('3'), text_button('4'), BACK],
        Mode::Game(Game::Pong(_)) => vec![(Key::Up, Some(icons::UP), "w", Look::Normal), (Key::Down, Some(icons::DOWN), "s", Look::Normal), BACK],
        Mode::Finish(_) | Mode::Awards | Mode::Graveyard(_) => vec![BACK],
    })
}

/// The key a click at scene position (`x`, `y`) presses, if it lands on a button.
pub fn button_at(app: &App, x: f32, y: f32) -> Option<Key> {
    buttons(app).into_iter().find(|b| (b.x..b.x + BUTTON).contains(&(x as i32)) && (b.y..b.y + BUTTON).contains(&(y as i32)) && x >= 0.0 && y >= 0.0).map(|b| b.key)
}

fn draw_button(f: &mut Frame, b: &Button, hover: bool, bounce: i32) {
    let y = b.y - if b.look == Look::Urgent { bounce } else { 0 };
    let (fill, border) = match (b.look, hover) {
        (_, true) => (Pal::Dusk, Pal::White),
        (Look::Urgent, _) => (Pal::Dusk, Pal::Yellow),
        (Look::Idle, _) => (Pal::Ink, Pal::Dusk),
        (Look::Normal, _) => (Pal::Night, Pal::Slate),
    };
    f.boxed(b.x, y, BUTTON, BUTTON, fill, border);
    if let Some(icon) = b.icon {
        let mut glyph = Frame::new(12, 12, fill);
        glyph.grid(icon, 0, 0);
        if b.look == Look::Idle {
            glyph.tint(0, 0, 12, 12, |p| if p == fill { p } else { p.dark() });
        }
        f.frame(&glyph, b.x + 2, y + 2);
        // A little keycap in the corner for the key's letter.
        if matches!(b.key, Key::Char(_)) {
            f.boxed(b.x - 2, y - 2, KEYCAP, KEYCAP, Pal::Ink, border);
        }
    }
}

/// The full 240x160 picture, with `hover` the key under the mouse.
pub fn frame(app: &App, hover: Option<Key>) -> Frame {
    let mut f = Frame::new(W, H, Pal::Ink);
    // A game takes the whole width above the hint line; the status panel comes back afterwards.
    let game = match &app.mode {
        Mode::Game(g) => Some(g.draw()),
        Mode::Finish(fin) => Some(fin.draw()),
        _ => None,
    };
    if let Some(g) = game {
        f.frame(&g, 0, 0);
        draw_bar(&mut f, app, hover);
        return f;
    }
    // Header strip over the room, status panel to its right.
    f.rect(0, 0, PANEL_X, ROOM.1, Pal::Night);
    f.rect(0, ROOM.1 - 1, PANEL_X, 1, Pal::Dusk);
    f.frame(&scene::area(app), ROOM.0, ROOM.1);
    if !app.message.is_empty() && matches!(app.mode, Mode::Normal) {
        f.tint(0, MESSAGE_Y, PANEL_X, HINT_Y - MESSAGE_Y, Pal::dark);
    }
    f.boxed(PANEL_X, 0, W - PANEL_X, HINT_Y, Pal::Night, Pal::Dusk);
    f.rect(PANEL_X + 1, 11, W - PANEL_X - 2, 1, Pal::Dusk);
    panel::draw(&mut f, app);
    screens::draw(&mut f, app);
    if app.confirming_reset() {
        screens::draw_reset(&mut f);
    }
    draw_bar(&mut f, app, hover);
    f
}

/// The hint line and the button bar under it.
fn draw_bar(f: &mut Frame, app: &App, hover: Option<Key>) {
    f.rect(0, HINT_Y, W, BAR_Y - HINT_Y, Pal::Ink);
    f.rect(0, BAR_Y, W, H - BAR_Y, Pal::Night);
    f.rect(0, BAR_Y, W, 1, Pal::Dusk);
    let bounce = ((app.started.elapsed().as_millis() / 350) % 2) as i32;
    for b in buttons(app) {
        draw_button(f, &b, hover == Some(b.key), bounce);
    }
}

/// The most urgent thing to do, for the line under the room.
fn hint(app: &App) -> Option<(String, Pal)> {
    let p = &app.pet;
    match &app.mode {
        Mode::Game(g) => return Some((g.hint().into(), Pal::Yellow)),
        Mode::Finish(_) => return Some(("ANY KEY TO GO ON".into(), Pal::Mist)),
        Mode::Normal if app.confirming_reset() => return Some(("PRESS Y TO START OVER".into(), Pal::Red)),
        Mode::Normal => {}
        _ => return screens::hint(app),
    }
    match p.top_need() {
        Some((k, text)) => Some((format!("→ PRESS {}: {}", k.to_ascii_uppercase(), text.to_uppercase()), Pal::Yellow)),
        None if p.stage == Stage::Egg => Some(("IT'S AN EGG - GIVE IT A MOMENT TO HATCH.".into(), Pal::Mist)),
        None => None,
    }
}

/// Every piece of text on screen. `ratio` is scene pixels per font pixel at the current scale.
pub fn labels(app: &App, ratio: f32) -> Vec<Label> {
    let p = &app.pet;
    let mut out = Vec::new();
    // Games and cards cover the header and the panel.
    let in_game = !matches!(app.mode, Mode::Normal);
    let mut head = format!("{} · {} · {}", p.species, p.stage.label(), fmt_age(p.age));
    if app.dev {
        head.push_str(" · DEV");
    }
    if !in_game {
        out.push(label(head, 4.0, 6.0, Pal::White, Align::Left));
        out.push(label("STATUS", PANEL_X as f32 + 40.0, 6.0, Pal::Pink, Align::Centre));
    }
    if let Some((text, colour)) = hint(app) {
        out.push(label(text, W as f32 / 2.0, HINT_Y as f32 + 6.0, colour, Align::Centre));
    }
    if !app.message.is_empty() && matches!(app.mode, Mode::Normal) {
        // One line only: long messages lose their tail rather than spill over the panel.
        let max = ((PANEL_X - 8) as f32 / ratio) as i32;
        let mut text = app.message.clone();
        while font::width(&text) > max && text.pop().is_some() {}
        out.push(label(text, PANEL_X as f32 / 2.0, (MESSAGE_Y + HINT_Y) as f32 / 2.0, Pal::White, Align::Centre));
    }
    if !in_game {
        out.extend(panel::labels(app, ratio));
    }
    out.extend(screens::labels(app));
    if app.confirming_reset() {
        out.extend(screens::reset_labels());
    }
    for b in buttons(app) {
        if b.icon.is_some() && matches!(b.key, Key::Char(_)) {
            let bounce = if b.look == Look::Urgent { ((app.started.elapsed().as_millis() / 350) % 2) as f32 } else { 0.0 };
            let cap = KEYCAP as f32 / 2.0;
            out.push(label(b.label, b.x as f32 - 2.0 + cap, b.y as f32 - 2.0 + cap - bounce, Pal::White, Align::Centre));
        } else if b.icon.is_none() {
            out.push(label(b.label, b.x as f32 + BUTTON as f32 / 2.0, b.y as f32 + BUTTON as f32 / 2.0, Pal::White, Align::Centre));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::app_with;

    #[test]
    fn the_bar_has_every_action_and_a_click_presses_it() {
        let app = app_with(Stage::Adult, |_| {});
        let bar = buttons(&app);
        let keys: String = bar.iter().filter_map(|b| if let Key::Char(c) = b.key { Some(c) } else { None }).collect();
        assert_eq!(keys, "fsplmcdbagq");
        assert!(bar.iter().all(|b| b.x >= 0 && b.x + BUTTON <= W && b.y + BUTTON <= H), "buttons stay on screen");
        let feed = bar[0];
        assert_eq!(button_at(&app, feed.x as f32 + 8.0, feed.y as f32 + 8.0), Some(Key::Char('f')));
        assert_eq!(button_at(&app, 1.0, 1.0), None);
    }

    #[test]
    fn the_most_urgent_action_stands_out_and_useless_ones_are_dimmed() {
        let app = app_with(Stage::Adult, |p| {
            p.hunger = 0;
            p.sick = false;
        });
        let look = |c: char| buttons(&app).into_iter().find(|b| b.key == Key::Char(c)).unwrap().look;
        assert_eq!(look('f'), Look::Urgent);
        assert_eq!(look('m'), Look::Idle, "it is not sick");
        assert_eq!(look('b'), Look::Normal);
    }

    #[test]
    fn a_dead_pet_offers_a_new_egg() {
        let app = app_with(Stage::Dead, |_| {});
        let keys: Vec<Key> = buttons(&app).iter().map(|b| b.key).collect();
        assert_eq!(keys, [Key::Char('n'), Key::Char('a'), Key::Char('g'), Key::Char('q')]);
    }

    #[test]
    fn the_header_and_message_are_on_screen() {
        let app = app_with(Stage::Baby, |_| {});
        let text: Vec<String> = labels(&app, 0.5).into_iter().map(|l| l.text).collect();
        assert!(text.iter().any(|t| t.starts_with("Babytchi · Baby")), "{text:?}");
        assert!(text.contains(&"hello".to_string()));
    }

    #[test]
    fn the_hint_line_says_what_to_press() {
        let app = app_with(Stage::Baby, |p| p.hunger = 0);
        let text: Vec<String> = labels(&app, 0.5).into_iter().map(|l| l.text).collect();
        assert!(text.iter().any(|t| t.starts_with("→ PRESS F: STARVING")), "{text:?}");
        let egg = app_with(Stage::Egg, |_| {});
        assert!(labels(&egg, 0.5).iter().any(|l| l.text.starts_with("IT'S AN EGG")));
    }

    #[test]
    fn a_game_fills_the_screen_and_says_what_to_do() {
        let mut app = app_with(Stage::Baby, |_| {});
        app.mode = Mode::Game(Game::new(2, &mut app.pet));
        let text: Vec<String> = labels(&app, 0.5).into_iter().map(|l| l.text).collect();
        assert!(text.iter().any(|t| t.starts_with("HOLD UP/DOWN")), "{text:?}");
        assert!(!text.iter().any(|t| t == "STATUS"), "the panel makes way for the game");
        let keys: Vec<Key> = buttons(&app).iter().map(|b| b.key).collect();
        assert_eq!(keys, [Key::Up, Key::Down, Key::Esc]);
    }

    #[test]
    fn the_need_hints_fit_the_screen_at_the_coarsest_text_scale() {
        let states: [fn(&mut crate::pet::Pet); 6] = [
            |p| p.asleep = true,
            |p| p.sick = true,
            |p| p.hunger = 0,
            |p| p.disc_call = Some(0),
            |p| p.poops = 1,
            |p| p.weight = 99,
        ];
        for tweak in states {
            let app = app_with(Stage::Adult, tweak);
            let (text, _) = hint(&app).unwrap();
            assert!(font::width(&text) as f32 * 2.0 / 3.0 <= W as f32 - 4.0, "too wide: {text}");
        }
    }

    #[test]
    fn every_mode_draws() {
        let mut app = app_with(Stage::Adult, |_| {});
        for mode in [Mode::Chooser, Mode::Shop, Mode::Awards, Mode::Graveyard(vec!["x".into()])] {
            app.mode = mode;
            assert_eq!(frame(&app, None).w, W);
            assert!(!labels(&app, 0.5).is_empty());
        }
    }
}
