//! The whole screen around the room: header strip, status panel, hint line and the clickable
//! icon bar. The screen is at least 288x160 scene pixels and grows with the window; the room
//! takes the extra space, and the panel and bar stay glued to the right and bottom edges. Pixel parts are drawn into the frame; text is handed back as `Label`s
//! for the window to draw sharp, at full resolution.

use crate::app::{App, Key, Mode};
use crate::fmt_age;
use crate::frame::Frame;
use crate::icons::{self, Grid};
use crate::palette::Pal;
use crate::pet::{Stage, MAX_HEARTS};
use crate::panel;
use crate::scene;
use crate::screens;

/// Top of the room, under the header strip.
pub const ROOM_Y: i32 = 12;
pub const PANEL_W: i32 = 80;
/// Height of the hint line plus the button bar under the room.
const FOOT: i32 = 32;
/// Height of the hint line under a card.
const CARD_HINT_H: i32 = 16;
const BUTTON: i32 = 16;
const GAP: i32 = 3;
const KEYCAP: i32 = 7;

/// The size of the screen in scene pixels, and where its parts go.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Layout {
    pub w: i32,
    pub h: i32,
}

impl Layout {
    /// The smallest screen the layout is designed for.
    pub const MIN: Layout = Layout { w: 288, h: 160 };
    /// The biggest: the room art stops growing here, and a bigger window gets bars.
    pub const MAX: Layout = Layout { w: PANEL_W + crate::cast::ROOM_MASTER.0 as i32, h: ROOM_Y + FOOT + crate::cast::ROOM_MASTER.1 as i32 };

    pub fn new(w: i32, h: i32) -> Layout {
        Layout { w: w.clamp(Layout::MIN.w, Layout::MAX.w), h: h.clamp(Layout::MIN.h, Layout::MAX.h) }
    }

    pub fn panel_x(self) -> i32 {
        self.w - PANEL_W
    }

    /// Where the line telling you what to do sits; the button bar is under it.
    pub fn hint_y(self) -> i32 {
        self.h - FOOT
    }

    pub fn bar_y(self) -> i32 {
        self.h - 20
    }

    pub fn room_h(self) -> i32 {
        self.hint_y() - ROOM_Y
    }

    /// Where the one-line hint under a card sits; a card has no button bar.
    pub fn card_hint_y(self) -> i32 {
        self.h - CARD_HINT_H
    }

    /// Where a 240x128 card goes so it is centred above its hint line.
    pub fn centred_card(self) -> (i32, i32) {
        ((self.w - 240) / 2, (self.card_hint_y() - 128) / 2)
    }
}

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
    Right,
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

const ACTIONS: [char; 11] = ['f', 's', 'p', 'l', 'm', 't', 'd', 'b', 'a', 'g', 'q'];
const DEAD_ACTIONS: [char; 4] = ['n', 'a', 'g', 'q'];

/// Whether an action would do anything right now (the rest are drawn dimmed).
fn useful(app: &App, key: char) -> bool {
    let p = &app.pet;
    match key {
        'f' => p.hunger < MAX_HEARTS,
        's' => p.happy < MAX_HEARTS,
        'm' => p.sick,
        't' => p.needs_toilet(),
        'd' => p.disc_call.is_some(),
        'l' => p.asleep,
        _ => true,
    }
}

fn row(items: Vec<(Key, Option<Grid>, &'static str, Look)>, lay: Layout, width: i32) -> Vec<Button> {
    let n = items.len() as i32;
    let x0 = (width - (n * BUTTON + (n - 1) * GAP)) / 2;
    items
        .into_iter()
        .enumerate()
        .map(|(i, (key, icon, label, look))| Button { key, icon, label, x: x0 + i as i32 * (BUTTON + GAP), y: lay.bar_y() + 2, look })
        .collect()
}

fn key_name(c: char) -> &'static str {
    match c {
        'f' => "f",
        's' => "s",
        'p' => "p",
        'l' => "l",
        'm' => "m",
        't' => "t",
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

/// Whether the screen has the status panel: not during a game or over a card (shop, awards...).
fn has_panel(app: &App) -> bool {
    matches!(app.mode, Mode::Normal | Mode::Toilet(_))
}

/// How wide the hint line and button bar are: the whole screen when there is no status panel,
/// otherwise just the space under the room, since the panel runs all the way down.
fn bar_w(app: &App, lay: Layout) -> i32 {
    if has_panel(app) { lay.panel_x() } else { lay.w }
}

/// The buttons along the bottom for the current mode.
pub fn buttons(app: &App, lay: Layout) -> Vec<Button> {
    let urgent = app.pet.top_need().map(|n| n.0);
    let action = |c: char| {
        let look = if urgent == Some(c) { Look::Urgent } else if useful(app, c) { Look::Normal } else { Look::Idle };
        (Key::Char(c), icons::for_key(c), key_name(c), look)
    };
    row(match &app.mode {
        Mode::Normal if app.pet.stage == Stage::Dead => DEAD_ACTIONS.iter().map(|&c| action(c)).collect(),
        Mode::Normal => ACTIONS.iter().map(|&c| action(c)).collect(),
        _ => Vec::new(),
    }, lay, bar_w(app, lay))
}

/// The key a click at scene position (`x`, `y`) presses, if it lands on a button.
pub fn button_at(app: &App, lay: Layout, x: f32, y: f32) -> Option<Key> {
    buttons(app, lay).into_iter().find(|b| (b.x..b.x + BUTTON).contains(&(x as i32)) && (b.y..b.y + BUTTON).contains(&(y as i32)) && x >= 0.0 && y >= 0.0).map(|b| b.key)
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

/// The full picture, with `hover` the key under the mouse.
pub fn frame(app: &App, lay: Layout, hover: Option<Key>) -> Frame {
    let mut f = Frame::new(lay.w, lay.h, Pal::Ink);
    let panel_x = lay.panel_x();
    // A game or a card takes the whole width above the hint line, over the dimmed room; the
    // status panel comes back afterwards.
    let game = match &app.mode {
        Mode::Game(g) => Some(g.draw()),
        Mode::Finish(fin) => Some(fin.draw()),
        _ => None,
    };
    if !has_panel(app) {
        let y = lay.card_hint_y();
        f.frame(&scene::area(app, lay.w, y), 0, 0);
        if let Some(g) = game {
            let (x, y) = lay.centred_card();
            if (x, y) != (0, 0) {
                f.boxed(x - 1, y - 1, g.w + 2, g.h + 2, Pal::Ink, Pal::Slate);
            }
            f.frame(&g, x, y);
            let (ex, ey, ew, eh) = screens::game_esc(lay);
            f.boxed(ex, ey, ew, eh, Pal::Night, Pal::Slate);
        } else {
            screens::draw(&mut f, app, lay);
        }
        f.rect(0, y, lay.w, CARD_HINT_H, Pal::Ink);
        f.rect(0, y, lay.w, 1, Pal::Dusk);
        return f;
    }
    // Header strip over the room, status panel to its right.
    f.rect(0, 0, panel_x, ROOM_Y, Pal::Night);
    f.rect(0, ROOM_Y - 1, panel_x, 1, Pal::Dusk);
    let mut room = scene::area(app, panel_x, lay.room_h());
    if let Some(wipe) = app.wipe() {
        wipe.cover(&mut room);
    }
    f.frame(&room, 0, ROOM_Y);
    f.boxed(panel_x, 0, PANEL_W, lay.h, Pal::Night, Pal::Dusk);
    f.rect(panel_x + 1, 11, PANEL_W - 2, 1, Pal::Dusk);
    panel::draw(&mut f, app, lay);
    if app.confirming_reset() {
        screens::draw_reset(&mut f, lay);
    }
    draw_bar(&mut f, app, lay, hover);
    f
}

/// The hint line and the button bar under it.
fn draw_bar(f: &mut Frame, app: &App, lay: Layout, hover: Option<Key>) {
    let w = bar_w(app, lay);
    f.rect(0, lay.hint_y(), w, lay.bar_y() - lay.hint_y(), Pal::Ink);
    f.rect(0, lay.bar_y(), w, lay.h - lay.bar_y(), Pal::Night);
    f.rect(0, lay.bar_y(), w, 1, Pal::Dusk);
    let bounce = ((app.started.elapsed().as_millis() / 350) % 2) as i32;
    for b in buttons(app, lay) {
        draw_button(f, &b, hover == Some(b.key), bounce);
    }
}

/// The most urgent thing to do, for the line under the room.
fn hint(app: &App) -> Option<(String, Pal)> {
    let p = &app.pet;
    match &app.mode {
        Mode::Game(g) => return Some((g.hint().into(), Pal::Yellow)),
        Mode::Finish(_) => return Some(("ANY KEY TO GO ON".into(), Pal::Mist)),
        Mode::Toilet(_) => return Some(("THE DOOR IS SHUT...".into(), Pal::Mist)),
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
pub fn labels(app: &App, lay: Layout, ratio: f32) -> Vec<Label> {
    let p = &app.pet;
    let mut out = Vec::new();
    let (panel_x, hint_y) = (lay.panel_x(), lay.hint_y());
    // Games and cards cover the header and the panel.
    let in_game = !has_panel(app);
    let mut head = format!("{} · {} · {}", p.species, p.stage.label(), fmt_age(p.age));
    if app.dev {
        head.push_str(" · DEV");
    }
    if !in_game {
        out.push(label(head, 4.0, 6.0, Pal::White, Align::Left));
        out.push(label("STATUS", panel_x as f32 + 40.0, 6.0, Pal::Pink, Align::Centre));
    }
    if let Some((text, colour)) = hint(app) {
        let (x, y) = if has_panel(app) {
            (panel_x as f32 / 2.0, hint_y as f32 + 6.0)
        } else {
            (lay.w as f32 / 2.0, lay.card_hint_y() as f32 + CARD_HINT_H as f32 / 2.0)
        };
        out.push(label(text, x, y, colour, Align::Centre));
    }
    if !in_game {
        out.extend(panel::labels(app, ratio, lay));
    }
    out.extend(screens::labels(app, lay));
    if matches!(app.mode, Mode::Game(_) | Mode::Finish(_)) {
        let (x, y, w, h) = screens::game_esc(lay);
        out.push(label("esc", x as f32 + w as f32 / 2.0, y as f32 + h as f32 / 2.0 - 0.5, Pal::White, Align::Centre));
    }
    if app.confirming_reset() {
        out.extend(screens::reset_labels(lay));
    }
    for b in buttons(app, lay) {
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
    use crate::font;
    use crate::games::Game;
    use crate::app::tests::app_with;

    #[test]
    fn the_bar_has_every_action_and_a_click_presses_it() {
        let app = app_with(Stage::Adult, |_| {});
        let lay = Layout::new(320, 160);
        let bar = buttons(&app, lay);
        let keys: String = bar.iter().filter_map(|b| if let Key::Char(c) = b.key { Some(c) } else { None }).collect();
        assert_eq!(keys, "fsplmtdbagq");
        assert!(bar.iter().all(|b| b.x >= 0 && b.x + BUTTON <= lay.panel_x() && b.y + BUTTON <= 160), "buttons stay on screen");
        let feed = bar[0];
        assert_eq!(button_at(&app, lay, feed.x as f32 + 8.0, feed.y as f32 + 8.0), Some(Key::Char('f')));
        assert_eq!(button_at(&app, lay, 1.0, 1.0), None);
    }

    #[test]
    fn the_most_urgent_action_stands_out_and_useless_ones_are_dimmed() {
        let app = app_with(Stage::Adult, |p| {
            p.hunger = 0;
            p.sick = false;
        });
        let look = |c: char| buttons(&app, Layout::MIN).into_iter().find(|b| b.key == Key::Char(c)).unwrap().look;
        assert_eq!(look('f'), Look::Urgent);
        assert_eq!(look('m'), Look::Idle, "it is not sick");
        assert_eq!(look('b'), Look::Normal);
    }

    #[test]
    fn a_dead_pet_offers_a_new_egg() {
        let app = app_with(Stage::Dead, |_| {});
        let keys: Vec<Key> = buttons(&app, Layout::MIN).iter().map(|b| b.key).collect();
        assert_eq!(keys, [Key::Char('n'), Key::Char('a'), Key::Char('g'), Key::Char('q')]);
    }

    #[test]
    fn the_header_is_on_screen() {
        let app = app_with(Stage::Baby, |_| {});
        let text: Vec<String> = labels(&app, Layout::MIN, 0.5).into_iter().map(|l| l.text).collect();
        assert!(text.iter().any(|t| t.starts_with("Babytchi · Baby")), "{text:?}");
    }

    #[test]
    fn cards_and_games_have_no_panel_and_a_full_width_bar() {
        let lay = Layout::MIN;
        let mut app = app_with(Stage::Adult, |_| {});
        let normal = bar_w(&app, lay);
        assert_eq!(normal, lay.panel_x());
        for mode in [Mode::Chooser, Mode::Shop, Mode::Awards, Mode::Graveyard(Vec::new())] {
            app.mode = mode;
            assert_eq!(bar_w(&app, lay), lay.w);
            let text: Vec<String> = labels(&app, lay, 0.5).into_iter().map(|l| l.text).collect();
            assert!(!text.iter().any(|t| t == "STATUS"), "{text:?}");
        }
    }

    #[test]
    fn the_hint_line_says_what_to_press() {
        let app = app_with(Stage::Baby, |p| p.hunger = 0);
        let text: Vec<String> = labels(&app, Layout::MIN, 0.5).into_iter().map(|l| l.text).collect();
        assert!(text.iter().any(|t| t.starts_with("→ PRESS F: STARVING")), "{text:?}");
        let egg = app_with(Stage::Egg, |_| {});
        assert!(labels(&egg, Layout::MIN, 0.5).iter().any(|l| l.text.starts_with("IT'S AN EGG")));
    }

    #[test]
    fn a_game_fills_the_screen_and_says_what_to_do() {
        let mut app = app_with(Stage::Baby, |_| {});
        app.mode = Mode::Game(Game::new(2, &mut app.pet));
        let text: Vec<String> = labels(&app, Layout::MIN, 0.5).into_iter().map(|l| l.text).collect();
        assert!(text.iter().any(|t| t.starts_with("HOLD W/S")), "{text:?}");
        assert!(text.iter().any(|t| t == "esc"), "a game has a way out: {text:?}");
        assert!(!text.iter().any(|t| t == "STATUS"), "the panel makes way for the game");
        assert!(buttons(&app, Layout::MIN).is_empty(), "no button bar in a game");
    }

    #[test]
    fn the_need_hints_fit_the_screen_at_the_coarsest_text_scale() {
        let states: [fn(&mut crate::pet::Pet); 6] = [
            |p| p.asleep = true,
            |p| p.sick = true,
            |p| p.hunger = 0,
            |p| p.disc_call = Some(0),
            |p| p.urge = Some(0),
            |p| p.weight = 99,
        ];
        for tweak in states {
            let app = app_with(Stage::Adult, tweak);
            let (text, _) = hint(&app).unwrap();
            assert!(font::width(&text) as f32 * 2.0 / 3.0 <= 240.0 - 4.0, "too wide: {text}");
        }
    }

    #[test]
    fn a_bigger_screen_moves_the_panel_and_bar_to_its_edges() {
        let app = app_with(Stage::Adult, |_| {});
        let lay = Layout::new(400, 240);
        assert_eq!((lay.panel_x(), lay.hint_y(), lay.bar_y()), (320, 208, 220));
        let f = frame(&app, lay, None);
        assert_eq!((f.w, f.h), (400, 240));
        let bar = buttons(&app, lay);
        assert!(bar.iter().all(|b| b.y >= lay.bar_y() && b.y + BUTTON <= 240 && b.x >= 0 && b.x + BUTTON <= 400));
        let status = labels(&app, lay, 0.5).into_iter().find(|l| l.text == "STATUS").unwrap();
        assert_eq!(status.x, 360.0);
        let click = &bar[0];
        assert_eq!(button_at(&app, lay, click.x as f32 + 8.0, click.y as f32 + 8.0), Some(Key::Char('f')));
    }

    #[test]
    fn every_mode_draws() {
        let mut app = app_with(Stage::Adult, |_| {});
        for mode in [Mode::Chooser, Mode::Shop, Mode::Awards, Mode::Graveyard(vec!["x".into()])] {
            app.mode = mode;
            assert_eq!(frame(&app, Layout::MIN, None).w, 288);
            assert!(!labels(&app, Layout::MIN, 0.5).is_empty());
        }
    }
}
