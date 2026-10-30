//! The cards that open over the dimmed room: picking a game, the shop, awards, the graveyard,
//! and the box asking to confirm a reset. Pixel parts go into the frame; text comes back as
//! labels, like the rest of the screen.

use crate::app::{App, Key, Mode};
use crate::art::Sprite;
use crate::cast;
use crate::faces::Expr;
use crate::frame::Frame;
use crate::hud::{Align, Label, Layout, ROOM_Y};
use crate::icons::{self, Grid};
use crate::palette::Pal;
use crate::pet::{self, Pet, Stage};
use crate::profile;
use crate::scene;

/// The card covers the room, above its hint line.
const CARD: (i32, i32, i32, i32) = (4, 4, 232, 120);
const TITLE_Y: f32 = 12.0;
const BODY_Y: i32 = 22;
const SHOP_ROW: i32 = 24;
const GAME_CARD: (i32, i32, i32) = (28, 34, 82);
/// The close button in a card's top-right corner.
const ESC: (i32, i32, i32, i32) = (CARD.0 + CARD.2 - 19, CARD.1 + 2, 15, 11);

fn label(text: impl Into<String>, x: f32, y: f32, colour: Pal) -> Label {
    Label { text: text.into(), x, y, colour, align: Align::Left }
}

fn centred(text: impl Into<String>, x: f32, y: f32, colour: Pal) -> Label {
    Label { text: text.into(), x, y, colour, align: Align::Centre }
}

/// Whether one of the cards is up, hiding the room and the panel.
pub fn showing(app: &App) -> bool {
    matches!(app.mode, Mode::Chooser | Mode::Shop | Mode::Awards | Mode::Graveyard(_))
}

fn card(f: &mut Frame) {
    let (x, y, w, h) = CARD;
    f.boxed(x, y, w, h, Pal::Night, Pal::Slate);
    f.rect(x + 1, y + 15, w - 2, 1, Pal::Dusk);
}

fn keycap(f: &mut Frame, x: i32, y: i32, lit: bool) {
    f.boxed(x, y, 9, 9, Pal::Ink, if lit { Pal::Yellow } else { Pal::Slate });
}

fn item_icon(item: pet::Item) -> Grid {
    match item {
        pet::Item::FancyMeal => icons::CAKE,
        pet::Item::Toy => icons::PLAY,
        pet::Item::Salad => icons::SALAD,
        pet::Item::Vitamins => icons::VITAMINS,
    }
}

fn shop_row_y(i: usize) -> i32 {
    BODY_Y + 2 + i as i32 * SHOP_ROW
}

fn draw_shop(f: &mut Frame, app: &App) {
    f.grid(icons::COIN, CARD.0 + CARD.2 - 46, 7);
    for (i, item) in pet::Item::ALL.iter().enumerate() {
        let y = shop_row_y(i);
        let can = app.profile.coins >= item.price();
        f.boxed(CARD.0 + 4, y, CARD.2 - 8, SHOP_ROW - 3, if can { Pal::Dusk } else { Pal::Ink }, Pal::Night);
        keycap(f, CARD.0 + 8, y + 6, can);
        let mut icon = Frame::new(12, 12, if can { Pal::Dusk } else { Pal::Ink });
        icon.grid(item_icon(*item), 0, 0);
        if !can {
            icon.tint(0, 0, 12, 12, |p| if p == Pal::Ink { p } else { p.dark() });
        }
        f.frame(&icon, CARD.0 + 22, y + 4);
        f.grid(icons::COIN, CARD.0 + CARD.2 - 34, y + 7);
    }
}

fn shop_labels(app: &App) -> Vec<Label> {
    let mut v = vec![
        label("SHOP", CARD.0 as f32 + 6.0, TITLE_Y, Pal::Pink),
        label(app.profile.coins.to_string(), (CARD.0 + CARD.2 - 37) as f32, TITLE_Y, Pal::Yellow),
    ];
    for (i, item) in pet::Item::ALL.iter().enumerate() {
        let y = shop_row_y(i) as f32;
        let can = app.profile.coins >= item.price();
        let (name, blurb, price) = if can { (Pal::White, Pal::Mist, Pal::Yellow) } else { (Pal::Slate, Pal::Dusk, Pal::Slate) };
        v.push(centred((i + 1).to_string(), CARD.0 as f32 + 12.5, y + 10.5, Pal::White));
        v.push(label(item.name(), CARD.0 as f32 + 40.0, y + 6.5, name));
        v.push(label(item.blurb(), CARD.0 as f32 + 40.0, y + 14.5, blurb));
        v.push(label(item.price().to_string(), (CARD.0 + CARD.2 - 25) as f32, y + 10.5, price));
    }
    v
}

/// An adult in the collection: in colour once raised, a dark silhouette until then.
fn collection_sprite(species: &str, raised: bool) -> Sprite {
    let mut p = Pet::new(1);
    p.stage = Stage::Adult;
    p.species = species.into();
    let s = scene::posed(&p, Expr::Happy);
    if raised {
        return s;
    }
    let mut dark = Sprite::empty(s.w);
    for y in 0..s.h {
        for x in 0..s.w {
            if let Some(c) = s.get(x, y) {
                dark.put(x, y, if c == Pal::Ink { Pal::Ink } else { Pal::Dusk });
            }
        }
    }
    dark
}

const AWARD_ROW: f32 = 9.0;

fn draw_awards(f: &mut Frame, app: &App) {
    for (i, a) in profile::ACHIEVEMENTS.iter().enumerate() {
        let (x, y) = (CARD.0 + 8 + (i as i32 / 5) * 110, BODY_Y + 2 + (i as i32 % 5) * AWARD_ROW as i32);
        if app.profile.has(a.id) {
            f.grid(icons::CHECK, x, y);
        } else {
            f.boxed(x + 2, y + 2, 3, 3, Pal::Dusk, Pal::Dusk);
        }
    }
    let floor = CARD.1 + CARD.3 - 4;
    f.rect(CARD.0 + 1, floor, CARD.2 - 2, 1, Pal::Dusk);
    for (i, species) in profile::ADULT_SPECIES.iter().enumerate() {
        let s = collection_sprite(species, app.profile.raised(species));
        f.sprite(&s, CARD.0 + 8 + i as i32 * 42, floor - s.bottom());
    }
}

fn awards_labels(app: &App) -> Vec<Label> {
    let p = &app.profile;
    let mut v = vec![
        label(format!("AWARDS {}/{}", p.achievements.len(), profile::ACHIEVEMENTS.len()), CARD.0 as f32 + 6.0, TITLE_Y, Pal::Pink),
        label(format!("adults raised {}/{}", p.adults.len(), profile::ADULT_SPECIES.len()), (CARD.0 + 120) as f32, TITLE_Y, Pal::Mist),
    ];
    for (i, a) in profile::ACHIEVEMENTS.iter().enumerate() {
        let (x, y) = (CARD.0 + 18 + (i as i32 / 5) * 110, BODY_Y as f32 + 5.5 + (i % 5) as f32 * AWARD_ROW);
        v.push(label(a.name, x as f32, y, if p.has(a.id) { Pal::Green } else { Pal::Mist }));
    }
    v
}

/// One graveyard line ("date | species | lived .. | n care mistakes | cause: ..") as two.
fn epitaph(entry: &str) -> (String, String) {
    let parts: Vec<&str> = entry.split(" | ").collect();
    match parts.as_slice() {
        [date, species, lived, mistakes, cause] => {
            let day = date.split(' ').next().unwrap_or(date);
            let mistakes = mistakes.replace(" care mistakes", " mistakes");
            (format!("{species} - {}", cause.trim_start_matches("cause: ")), format!("{day} · {lived} · {mistakes}"))
        }
        _ => (entry.to_string(), String::new()),
    }
}

const GRAVE_ROW: i32 = 16;
const GRAVES_SHOWN: usize = 6;

fn draw_graveyard(f: &mut Frame, entries: &[String]) {
    if entries.is_empty() {
        f.sprite(crate::art::body(&cast::GRAVE), 120 - 16, 54);
        return;
    }
    for i in 0..entries.len().min(GRAVES_SHOWN) {
        f.grid(icons::GRAVEYARD, CARD.0 + 6, BODY_Y + 2 + i as i32 * GRAVE_ROW);
    }
}

fn graveyard_labels(entries: &[String]) -> Vec<Label> {
    let mut v = vec![label(format!("GRAVEYARD  {}", entries.len()), CARD.0 as f32 + 6.0, TITLE_Y, Pal::Pink)];
    if entries.is_empty() {
        v.push(centred("No pets have passed away yet.", 120.0, 100.0, Pal::Mist));
    }
    for (i, e) in entries.iter().take(GRAVES_SHOWN).enumerate() {
        let (first, second) = epitaph(e);
        let y = (BODY_Y + 2 + i as i32 * GRAVE_ROW) as f32;
        v.push(label(first, CARD.0 as f32 + 22.0, y + 3.5, Pal::White));
        v.push(label(second, CARD.0 as f32 + 22.0, y + 10.5, Pal::Slate));
    }
    if entries.len() > GRAVES_SHOWN {
        v.push(label(format!("and {} more", entries.len() - GRAVES_SHOWN), (CARD.0 + CARD.2 - 60) as f32, TITLE_Y, Pal::Slate));
    }
    v
}

fn game_card_x(i: i32) -> i32 {
    GAME_CARD.0 + i * (GAME_CARD.2 + 20)
}

fn draw_chooser(f: &mut Frame) {
    let (_, y, w) = GAME_CARD;
    for i in 0..2 {
        let x = game_card_x(i);
        f.boxed(x, y, w, 70, Pal::Dusk, Pal::Slate);
        keycap(f, x + 4, y + 4, false);
        if i == 0 {
            // Four little pads, one of them lit.
            for (n, c) in [Pal::Red, Pal::Yellow, Pal::Green, Pal::Sky].into_iter().enumerate() {
                let fill = if n == 1 { c.highlight() } else { c.shadow() };
                f.boxed(x + 8 + n as i32 * 17, y + 26, 15, 15, fill, Pal::Ink);
            }
        } else {
            f.rect(x + 6, y + 20, w - 12, 28, Pal::DeepTeal);
            f.rect(x + w / 2, y + 22, 1, 24, Pal::Teal);
            f.boxed(x + 10, y + 26, 3, 12, Pal::Pink, Pal::Ink);
            f.boxed(x + 40, y + 30, 4, 4, Pal::Yellow, Pal::Orange);
            f.boxed(x + w - 14, y + 28, 6, 10, Pal::Mist, Pal::Ink);
        }
    }
}

fn chooser_labels() -> Vec<Label> {
    let mut v = vec![label("PICK A GAME", CARD.0 as f32 + 6.0, TITLE_Y, Pal::Pink)];
    for (i, (key, name)) in crate::games::CHOICES.iter().enumerate() {
        let x = game_card_x(i as i32) as f32;
        v.push(centred(*key, x + 8.5, GAME_CARD.1 as f32 + 8.5, Pal::White));
        v.push(centred(name.to_uppercase(), x + GAME_CARD.2 as f32 / 2.0, GAME_CARD.1 as f32 + 60.0, Pal::White));
    }
    v.push(centred("Any game burns a little weight; winning makes it happier.", 120.0, 114.0, Pal::Mist));
    v
}

/// Draw the card for the current mode, centred over the dimmed screen.
pub fn draw(f: &mut Frame, app: &App, lay: Layout) {
    if !showing(app) {
        return;
    }
    f.tint(0, 0, lay.w, lay.h, Pal::dark);
    // The card is laid out as if the screen were the smallest one, then moved to the middle.
    let mut local = Frame::new(240, 128, Pal::Ink);
    card(&mut local);
    let (x, y, w, h) = ESC;
    local.boxed(x, y, w, h, Pal::Night, Pal::Slate);
    match &app.mode {
        Mode::Chooser => draw_chooser(&mut local),
        Mode::Shop => draw_shop(&mut local, app),
        Mode::Awards => draw_awards(&mut local, app),
        Mode::Graveyard(entries) => draw_graveyard(&mut local, entries),
        _ => {}
    }
    let (ox, oy) = lay.centred_card();
    f.copy_rect(&local, CARD, ox + CARD.0, oy + CARD.1);
}

pub fn labels(app: &App, lay: Layout) -> Vec<Label> {
    let mut v = match &app.mode {
        Mode::Chooser => chooser_labels(),
        Mode::Shop => shop_labels(app),
        Mode::Awards => awards_labels(app),
        Mode::Graveyard(entries) => graveyard_labels(entries),
        _ => Vec::new(),
    };
    if showing(app) {
        v.push(centred("esc", ESC.0 as f32 + ESC.2 as f32 / 2.0, ESC.1 as f32 + ESC.3 as f32 / 2.0, Pal::White));
    }
    let (ox, oy) = lay.centred_card();
    for l in &mut v {
        l.x += ox as f32;
        l.y += oy as f32;
    }
    v
}

/// The line under a card.
pub fn hint(app: &App) -> Option<(String, Pal)> {
    Some(match &app.mode {
        Mode::Chooser => ("PRESS 1 OR 2, OR CLICK A GAME".into(), Pal::Yellow),
        Mode::Shop => ("PRESS 1-4 OR CLICK TO BUY".into(), Pal::Yellow),
        Mode::Awards => match profile::ACHIEVEMENTS.iter().find(|a| !app.profile.has(a.id)) {
            Some(next) => (format!("NEXT: {} - {}", next.name.to_uppercase(), next.how.to_uppercase()), Pal::Mist),
            None => ("EVERY AWARD UNLOCKED!".into(), Pal::Yellow),
        },
        Mode::Graveyard(_) => ("ANY KEY TO GO BACK".into(), Pal::Mist),
        _ => return None,
    })
}

/// The close button over a game: the top-right corner of the screen.
pub fn game_esc(lay: Layout) -> (i32, i32, i32, i32) {
    (lay.w - 19, 3, 15, 11)
}

fn in_rect(r: (i32, i32, i32, i32), x: i32, y: i32) -> bool {
    (r.0..r.0 + r.2).contains(&x) && (r.1..r.1 + r.3).contains(&y)
}

/// The key a click presses where there is no button bar: on a card a shop row, a game or the
/// way out; in a game a Memory pad or the way out.
pub fn click(app: &App, lay: Layout, x: f32, y: f32) -> Option<Key> {
    let (ox, oy) = lay.centred_card();
    let (x, y) = (x as i32 - ox, y as i32 - oy);
    match &app.mode {
        Mode::Game(g) => {
            return if in_rect(game_esc(lay), x + ox, y + oy) { Some(Key::Esc) } else { g.click(x, y) };
        }
        Mode::Finish(_) => return Some(Key::Esc),
        _ => {}
    }
    let inside = |r: (i32, i32, i32, i32)| (r.0..r.0 + r.2).contains(&x) && (r.1..r.1 + r.3).contains(&y);
    if !showing(app) {
        return None;
    }
    // The close button, or anywhere off the card, closes it.
    if inside(ESC) || !inside(CARD) {
        return Some(Key::Esc);
    }
    match app.mode {
        Mode::Shop => (0..4).find(|&i| (shop_row_y(i)..shop_row_y(i) + SHOP_ROW - 3).contains(&y) && (CARD.0..CARD.0 + CARD.2).contains(&x)).map(|i| Key::Char((b'1' + i as u8) as char)),
        Mode::Chooser => (0..2).find(|&i| (game_card_x(i)..game_card_x(i) + GAME_CARD.2).contains(&x) && (GAME_CARD.1..GAME_CARD.1 + 70).contains(&y)).map(|i| Key::Char((b'1' + i as u8) as char)),
        _ => None,
    }
}

/// The box on screen (scene pixels) to outline under the mouse, if it is over something clickable.
/// Clicking off a card also closes it, but that is not worth an outline.
pub fn hover_rect(app: &App, lay: Layout, x: f32, y: f32) -> Option<(i32, i32, i32, i32)> {
    let (ox, oy) = lay.centred_card();
    let (x, y) = (x as i32 - ox, y as i32 - oy);
    let moved = |r: (i32, i32, i32, i32)| (r.0 + ox, r.1 + oy, r.2, r.3);
    match &app.mode {
        Mode::Game(g) => {
            let esc = game_esc(lay);
            return if in_rect(esc, x + ox, y + oy) { Some(esc) } else { g.hover_rect(x, y).map(moved) };
        }
        _ if !showing(app) => return None,
        _ => {}
    }
    if in_rect(ESC, x, y) {
        return Some(moved(ESC));
    }
    match app.mode {
        Mode::Shop => (0..4).find(|&i| in_rect((CARD.0, shop_row_y(i), CARD.2, SHOP_ROW - 3), x, y)).map(|i| moved((CARD.0 + 2, shop_row_y(i), CARD.2 - 4, SHOP_ROW - 3))),
        Mode::Chooser => (0..2).find(|&i| in_rect((game_card_x(i), GAME_CARD.1, GAME_CARD.2, 70), x, y)).map(|i| moved((game_card_x(i), GAME_CARD.1, GAME_CARD.2, 70))),
        _ => None,
    }
}

const RESET_W: i32 = 120;
const RESET_H: i32 = 44;

/// Top-left of the reset box: the middle of the room.
fn reset_at(lay: Layout) -> (i32, i32) {
    ((lay.panel_x() - RESET_W) / 2, ROOM_Y + (lay.room_h() - RESET_H) / 2)
}

/// The box over the room after Shift+R.
pub fn draw_reset(f: &mut Frame, lay: Layout) {
    let (x, y) = reset_at(lay);
    f.boxed(x, y, RESET_W, RESET_H, Pal::Night, Pal::Red);
    f.rect(x + 1, y + 1, RESET_W - 2, 1, Pal::Maroon);
}

pub fn reset_labels(lay: Layout) -> Vec<Label> {
    let (x, y) = reset_at(lay);
    let (cx, y) = (x as f32 + RESET_W as f32 / 2.0, y as f32);
    vec![
        centred("START OVER?", cx, y + 9.0, Pal::Red),
        centred("This pet is thrown away", cx, y + 19.0, Pal::White),
        centred("and a new egg appears.", cx, y + 26.0, Pal::White),
        centred("y = yes, any other key = no", cx, y + 36.0, Pal::Mist),
    ]
}

/// Labels on cards must fit inside them at the coarsest text scale.
#[cfg(test)]
fn fits(l: &Label, ratio: f32) -> bool {
    let w = crate::font::width(&l.text) as f32 * ratio;
    let (left, right) = match l.align {
        Align::Left => (l.x, l.x + w),
        Align::Centre => (l.x - w / 2.0, l.x + w / 2.0),
    };
    left >= CARD.0 as f32 && right <= (CARD.0 + CARD.2) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::app_with;

    const COARSEST: f32 = 2.0 / 3.0;

    #[test]
    fn the_shop_lists_every_item_with_its_price_and_a_click_buys_it() {
        let mut app = app_with(Stage::Adult, |p| p.hunger = 1);
        app.profile.coins = 7;
        app.mode = Mode::Shop;
        let text: Vec<String> = labels(&app, Layout::MIN).into_iter().map(|l| l.text).collect();
        for want in ["SHOP", "7", "Fancy meal", "Vitamins", "12"] {
            assert!(text.iter().any(|t| t == want), "shop is missing {want}: {text:?}");
        }
        assert_eq!(click(&app, Layout::MIN, 100.0, shop_row_y(2) as f32 + 5.0 + Layout::MIN.centred_card().1 as f32), Some(Key::Char('3')));
        assert_eq!(click(&app, Layout::MIN, 100.0, shop_row_y(0) as f32 - 3.0 + Layout::MIN.centred_card().1 as f32), None);
    }

    #[test]
    fn the_awards_list_every_achievement_and_the_collection() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.mode = Mode::Awards;
        let text: Vec<String> = labels(&app, Layout::MIN).into_iter().map(|l| l.text).collect();
        assert!(text.iter().any(|t| t == "First steps") && text.iter().any(|t| t == "Collector"), "{text:?}");
        assert!(text.iter().any(|t| t.contains("adults raised 0/5")));
        assert!(hint(&app).unwrap().0.starts_with("NEXT:"));
    }

    #[test]
    fn a_graveyard_line_is_split_into_who_and_when() {
        let (who, when) = epitaph("2026-10-29 22:00 | Marutchi | lived 1h 2m | 3 care mistakes | cause: starvation");
        assert_eq!(who, "Marutchi - starvation");
        assert_eq!(when, "2026-10-29 · lived 1h 2m · 3 mistakes");
        assert_eq!(epitaph("odd line").0, "odd line");
    }

    #[test]
    fn every_card_fits_at_the_coarsest_text_scale() {
        let mut app = app_with(Stage::Adult, |_| {});
        let graves = vec!["2026-10-29 22:00 | Hashitamatchi | lived 1d 23h 59m | 12 care mistakes | cause: starvation".to_string(); 9];
        for mode in [Mode::Chooser, Mode::Shop, Mode::Awards, Mode::Graveyard(graves), Mode::Graveyard(Vec::new())] {
            app.mode = mode;
            for mut l in labels(&app, Layout::MIN) {
                l.x -= Layout::MIN.centred_card().0 as f32;
                assert!(fits(&l, COARSEST), "does not fit: {:?}", l.text);
            }
            let mut f = Frame::new(240, 160, Pal::Ink);
            draw(&mut f, &app, Layout::MIN);
        }
    }

    #[test]
    fn every_hint_fits_the_screen_at_the_coarsest_text_scale() {
        let mut app = app_with(Stage::Adult, |_| {});
        let mut hints = Vec::new();
        for mode in [Mode::Chooser, Mode::Shop, Mode::Graveyard(Vec::new())] {
            app.mode = mode;
            hints.push(hint(&app).unwrap().0);
        }
        app.mode = Mode::Awards;
        for a in profile::ACHIEVEMENTS.iter() {
            hints.push(hint(&app).unwrap().0);
            app.profile.achievements.push(a.id.into());
        }
        for h in hints {
            assert!(crate::font::width(&h) as f32 * COARSEST <= 236.0, "too wide: {h}");
        }
    }

    #[test]
    fn a_card_is_centred_on_a_bigger_screen_and_clicks_follow_it() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.mode = Mode::Shop;
        let lay = Layout::new(400, 220);
        let (ox, oy) = lay.centred_card();
        assert_eq!((ox, oy), (80, 38));
        let title = labels(&app, lay).into_iter().find(|l| l.text == "SHOP").unwrap();
        assert_eq!((title.x, title.y), (CARD.0 as f32 + 6.0 + 80.0, TITLE_Y + 38.0));
        assert_eq!(click(&app, lay, 180.0, shop_row_y(1) as f32 + 5.0 + 38.0), Some(Key::Char('2')));
    }

    #[test]
    fn the_esc_button_and_a_click_off_the_card_close_it() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.mode = Mode::Shop;
        let lay = Layout::MIN;
        let (ox, oy) = lay.centred_card();
        assert_eq!(click(&app, lay, (ox + ESC.0 + 3) as f32, (oy + ESC.1 + 3) as f32), Some(Key::Esc));
        assert_eq!(click(&app, lay, 1.0, 1.0), Some(Key::Esc));
        assert_eq!(click(&app, lay, 100.0, shop_row_y(0) as f32 - 3.0 + oy as f32), None);
        app.mode = Mode::Normal;
        assert_eq!(click(&app, lay, 1.0, 1.0), None);
    }

    #[test]
    fn in_a_game_a_click_on_a_pad_or_esc_acts_and_a_result_card_goes_on() {
        let mut app = app_with(Stage::Adult, |_| {});
        let lay = Layout::MIN;
        let (ox, oy) = lay.centred_card();
        app.mode = Mode::Game(crate::games::Game::new(1, &mut app.pet));
        // The second pad: 20 + 52 across, 72 down.
        assert_eq!(click(&app, lay, (ox + 20 + 52 + 5) as f32, (oy + 72 + 5) as f32), Some(Key::Char('2')));
        assert_eq!(click(&app, lay, (ox + 2) as f32, (oy + 2) as f32), None);
        let (ex, ey, _, _) = game_esc(lay);
        assert_eq!(click(&app, lay, (ex + 3) as f32, (ey + 3) as f32), Some(Key::Esc));
        app.mode = Mode::Game(crate::games::Game::new(2, &mut app.pet));
        assert_eq!(click(&app, lay, (ox + 20 + 52 + 5) as f32, (oy + 72 + 5) as f32), None);
    }

    #[test]
    fn a_click_on_a_game_card_picks_it() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.mode = Mode::Chooser;
        assert_eq!(click(&app, Layout::MIN, (Layout::MIN.centred_card().0 + game_card_x(1)) as f32 + 10.0, GAME_CARD.1 as f32 + 10.0 + Layout::MIN.centred_card().1 as f32), Some(Key::Char('2')));
    }
}
