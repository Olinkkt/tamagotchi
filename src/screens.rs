//! The cards that open over the dimmed room: picking a game, the shop, awards, the graveyard,
//! and the box asking to confirm a reset. Pixel parts go into the frame; text comes back as
//! labels, like the rest of the screen.

use crate::app::{App, Key, Mode};
use crate::art::Sprite;
use crate::cast;
use crate::faces::Expr;
use crate::frame::Frame;
use crate::hud::{Align, Label, Layout, ROOM_Y};
use crate::decor;
use crate::icons::{self, Grid};
use crate::palette::Pal;
use crate::pet::{self, Pet, Personality, Stage};
use crate::profile;
use crate::scene;

/// The card covers the room, above its hint line.
const CARD: (i32, i32, i32, i32) = (4, 4, 232, 120);
const TITLE_Y: f32 = 12.0;
const BODY_Y: i32 = 22;
const SHOP_ROW: i32 = 24;
const GAME_CARD: (i32, i32) = (14, 34);
const GAME_CARD_H: i32 = 70;
const GAME_CARD_GAP: i32 = 6;
/// The close button in a card's top-right corner.
const ESC: (i32, i32, i32, i32) = (CARD.0 + CARD.2 - 19, CARD.1 + 2, 15, 11);

fn label(text: impl Into<String>, x: f32, y: f32, colour: Pal) -> Label {
    Label { text: text.into(), x, y, colour, align: Align::Left }
}

fn right(text: impl Into<String>, x: f32, y: f32, colour: Pal) -> Label {
    Label { text: text.into(), x, y, colour, align: Align::Right }
}

fn centred(text: impl Into<String>, x: f32, y: f32, colour: Pal) -> Label {
    Label { text: text.into(), x, y, colour, align: Align::Centre }
}

/// Whether one of the cards is up, hiding the room and the panel.
pub fn showing(app: &App) -> bool {
    matches!(app.mode, Mode::Chooser | Mode::Shop | Mode::Awards | Mode::Keepsakes | Mode::Badges | Mode::Gifts(_) | Mode::Tree | Mode::Graveyard(_))
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
        pet::Item::Cupcake => icons::CUPCAKE,
        pet::Item::Dumbbell => icons::DUMBBELL,
        pet::Item::PottyCharm => icons::CHARM,
        pet::Item::MedKit => icons::MEDKIT,
    }
}

/// The four items on the care or treats shelf that is showing.
fn shelf(app: &App) -> &'static [pet::Item] {
    let from = (app.shop_tab * 4).min(pet::Item::ALL.len() - 4);
    &pet::Item::ALL[from..from + 4]
}

fn shop_row_y(i: usize) -> i32 {
    BODY_Y + 2 + i as i32 * SHOP_ROW
}

fn draw_shop(f: &mut Frame, app: &App) {
    f.grid(icons::COIN, ESC.0 - 10, ESC.1 + 3);
    if app.shop_tab == crate::app::SHOP_ROOM {
        for (i, d) in decor::DECOR.iter().enumerate() {
            let y = shop_row_y(i);
            let can = app.profile.coins >= d.price && !app.profile.placed(d.id);
            f.boxed(CARD.0 + 4, y, CARD.2 - 8, SHOP_ROW - 3, if can { Pal::Dusk } else { Pal::Ink }, Pal::Night);
            keycap(f, CARD.0 + 8, y + 6, can);
            let mut icon = Frame::new(16, 20, if can { Pal::Dusk } else { Pal::Ink });
            icon.grid(d.icon, 0, 0);
            if !can {
                icon.tint(0, 0, 16, 20, |p| if p == Pal::Ink { p } else { p.dark() });
            }
            f.frame(&icon, CARD.0 + 20, y);
            f.grid(icons::COIN, CARD.0 + CARD.2 - 14, y + 7);
        }
        return;
    }
    for (i, item) in shelf(app).iter().enumerate() {
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
        f.grid(icons::COIN, CARD.0 + CARD.2 - 14, y + 7);
    }
}

fn shop_labels(app: &App) -> Vec<Label> {
    let mut v = vec![
        label("SHOP", CARD.0 as f32 + 6.0, TITLE_Y, Pal::Pink),
        label("care", CARD.0 as f32 + 40.0, TITLE_Y, if app.shop_tab == 0 { Pal::White } else { Pal::Slate }),
        label("treats", CARD.0 as f32 + 66.0, TITLE_Y, if app.shop_tab == 1 { Pal::White } else { Pal::Slate }),
        label("room", CARD.0 as f32 + 100.0, TITLE_Y, if app.shop_tab == 2 { Pal::White } else { Pal::Slate }),
        right(app.profile.coins.to_string(), (ESC.0 - 12) as f32, ESC.1 as f32 + 6.5, Pal::Yellow),
    ];
    if app.shop_tab == crate::app::SHOP_ROOM {
        for (i, d) in decor::DECOR.iter().enumerate() {
            let y = shop_row_y(i) as f32;
            let placed = app.profile.placed(d.id);
            let can = app.profile.coins >= d.price && !placed;
            let (name, blurb, price) = if can { (Pal::White, Pal::Mist, Pal::Yellow) } else { (Pal::Slate, Pal::Dusk, Pal::Slate) };
            v.push(centred((i + 1).to_string(), CARD.0 as f32 + 12.5, y + 10.5, Pal::White));
            v.push(label(d.name, CARD.0 as f32 + 44.0, y + 6.5, name));
            v.push(label(if placed { "in your room" } else { d.blurb }, CARD.0 as f32 + 44.0, y + 14.5, blurb));
            v.push(right(d.price.to_string(), (CARD.0 + CARD.2 - 16) as f32, y + 10.5, price));
        }
        return v;
    }
    for (i, item) in shelf(app).iter().enumerate() {
        let y = shop_row_y(i) as f32;
        let can = app.profile.coins >= item.price();
        let (name, blurb, price) = if can { (Pal::White, Pal::Mist, Pal::Yellow) } else { (Pal::Slate, Pal::Dusk, Pal::Slate) };
        v.push(centred((i + 1).to_string(), CARD.0 as f32 + 12.5, y + 10.5, Pal::White));
        v.push(label(item.name(), CARD.0 as f32 + 40.0, y + 6.5, name));
        v.push(label(item.blurb(), CARD.0 as f32 + 40.0, y + 14.5, blurb));
        v.push(right(item.price().to_string(), (CARD.0 + CARD.2 - 16) as f32, y + 10.5, price));
    }
    v
}

/// A form in the collection: in colour once raised, a dark silhouette until then.
fn collection_sprite(species: &str, raised: bool) -> Sprite {
    form_sprite(species, Stage::Adult, raised)
}

fn form_sprite(species: &str, stage: Stage, shown: bool) -> Sprite {
    let mut p = Pet::new(1);
    p.stage = stage;
    p.species = species.into();
    let s = scene::posed(&p, Expr::Happy);
    if shown {
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
        f.sprite(&s, CARD.0 + 2 + i as i32 * 36, floor - s.bottom());
    }
}

fn awards_labels(app: &App) -> Vec<Label> {
    let p = &app.profile;
    let mut v = vec![
        label(format!("AWARDS {}/{}", p.achievements.len(), profile::ACHIEVEMENTS.len()), CARD.0 as f32 + 6.0, TITLE_Y, Pal::Pink),
        label(format!("k keepsakes {}/{}  b badges {}/{}", p.keepsakes.len(), profile::KEEPSAKES.len(), p.badges_earned(), profile::BADGES.len()), (CARD.0 + 120) as f32, BODY_Y as f32 + 5.5 + 4.0 * AWARD_ROW, Pal::Mist),
        label(format!("adults raised {}/{}", p.adults.len(), profile::ADULT_SPECIES.len()), (CARD.0 + 120) as f32, TITLE_Y, Pal::Mist),
    ];
    for (i, a) in profile::ACHIEVEMENTS.iter().enumerate() {
        let (x, y) = (CARD.0 + 18 + (i as i32 / 5) * 110, BODY_Y as f32 + 5.5 + (i % 5) as f32 * AWARD_ROW);
        v.push(label(a.name, x as f32, y, if p.has(a.id) { Pal::Green } else { Pal::Mist }));
    }
    v
}

/// Where keepsake `i` sits on its page: three across, two down.
fn keepsake_at(i: usize) -> (i32, i32) {
    (CARD.0 + 28 + (i as i32 % 3) * 70, BODY_Y + 8 + (i as i32 / 3) * 44)
}

fn draw_keepsakes(f: &mut Frame, app: &App) {
    for (i, k) in profile::KEEPSAKES.iter().enumerate() {
        let found = app.profile.found(k.id);
        let (x, y) = keepsake_at(i);
        f.grid_scaled(k.icon, x, y, 2, |c| if found || c == Pal::Ink { c } else { Pal::Dusk });
    }
}

fn keepsakes_labels(app: &App) -> Vec<Label> {
    let mut v = vec![
        label(format!("KEEPSAKES {}/{}", app.profile.keepsakes.len(), profile::KEEPSAKES.len()), CARD.0 as f32 + 6.0, TITLE_Y, Pal::Pink),
        label("left: awards  right: badges", (CARD.0 + 90) as f32, TITLE_Y, Pal::Mist),
    ];
    for (i, k) in profile::KEEPSAKES.iter().enumerate() {
        let (x, y) = keepsake_at(i);
        let found = app.profile.found(k.id);
        let on_sill = app.profile.shelf.as_deref() == Some(k.id);
        let text = if found { format!("{} {}", i + 1, k.name) } else { "???".to_string() };
        v.push(centred(text, (x + 12) as f32, (y + 31) as f32, if on_sill { Pal::Yellow } else if found { Pal::Green } else { Pal::Mist }));
    }
    v
}

const BADGE_ROW: i32 = 24;
const BADGE_KINDS: [pet::Care; 4] = [pet::Care::Meal, pet::Care::Snack, pet::Care::Game, pet::Care::Scold];

fn badge_y(row: usize) -> i32 {
    BODY_Y + 3 + row as i32 * BADGE_ROW
}

/// Where tier `t` (0 to 2) of a row's medals sits.
fn medal_x(t: usize) -> i32 {
    CARD.0 + 88 + t as i32 * 52
}

fn draw_badges(f: &mut Frame, app: &App) {
    for (row, care) in BADGE_KINDS.iter().enumerate() {
        for (t, b) in profile::BADGES.iter().filter(|b| b.care == *care).enumerate() {
            let tier = [Pal::Tan, Pal::Mist, Pal::Yellow][t];
            let earned = app.profile.has_badge(b);
            f.grid_scaled(icons::MEDAL, medal_x(t), badge_y(row), 1, |c| match c {
                Pal::Ink => Pal::Ink,
                _ if !earned => Pal::Dusk,
                Pal::Yellow => tier,
                Pal::Orange => tier.shadow(),
                Pal::Cream => tier.highlight(),
                other => other,
            });
        }
    }
}

fn badges_labels(app: &App) -> Vec<Label> {
    let mut v = vec![
        label(format!("BADGES {}/{}", app.profile.badges_earned(), profile::BADGES.len()), CARD.0 as f32 + 6.0, TITLE_Y, Pal::Pink),
        label("left: keepsakes", (CARD.0 + 120) as f32, TITLE_Y, Pal::Mist),
    ];
    for (row, care) in BADGE_KINDS.iter().enumerate() {
        let y = badge_y(row) as f32;
        v.push(label(profile::care_name(*care), CARD.0 as f32 + 10.0, y + 4.0, Pal::White));
        v.push(label(app.profile.care_count(*care).to_string(), CARD.0 as f32 + 10.0, y + 11.0, Pal::Yellow));
        for (t, b) in profile::BADGES.iter().filter(|b| b.care == *care).enumerate() {
            let earned = app.profile.has_badge(b);
            v.push(centred(if earned { b.name.to_string() } else { b.goal.to_string() }, (medal_x(t) + 6) as f32, y + 16.0, if earned { Pal::Green } else { Pal::Mist }));
        }
    }
    v
}

const GIFT_ROW: i32 = 15;

fn gift_y(i: usize) -> i32 {
    BODY_Y + 4 + i as i32 * GIFT_ROW
}

fn draw_gifts(f: &mut Frame, gifts: &[profile::Gift]) {
    for (i, g) in gifts.iter().enumerate() {
        let y = gift_y(i);
        match g {
            profile::Gift::Coins(_) => f.grid(icons::COIN, CARD.0 + 63, y + 3),
            profile::Gift::Keepsake(id) => {
                if let Some(k) = profile::KEEPSAKES.iter().find(|k| k.id == *id) {
                    f.grid(k.icon, CARD.0 + 60, y);
                }
            }
        }
    }
}

fn gifts_labels(gifts: &[profile::Gift]) -> Vec<Label> {
    let mut v = vec![label("WELCOME BACK", CARD.0 as f32 + 6.0, TITLE_Y, Pal::Pink), label("while you were away, it left you", (CARD.0 + 80) as f32, TITLE_Y, Pal::Mist)];
    for (i, g) in gifts.iter().enumerate() {
        let (text, colour) = match g {
            profile::Gift::Coins(n) => (format!("{n} coins"), Pal::Yellow),
            profile::Gift::Keepsake(id) => {
                let name = profile::KEEPSAKES.iter().find(|k| k.id == *id).map_or("keepsake", |k| k.name).to_lowercase();
                let a = if name.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
                (format!("{a} {name} (new keepsake!)"), Pal::Green)
            }
        };
        v.push(label(text, (CARD.0 + 80) as f32, gift_y(i) as f32 + 6.0, colour));
    }
    v
}

/// One form on the tree: its name, stage, middle (x) and floor (y) in the card, and what it takes.
struct Node {
    species: &'static str,
    stage: Stage,
    x: i32,
    floor: i32,
    /// The habit that earns the form.
    needs: &'static str,
}

const TREE_TOP_FLOOR: i32 = 54;
const TREE_ADULT_FLOOR: i32 = CARD.1 + CARD.3 - 4;
const TREE_NEEDS_Y: f32 = 69.5;

fn tree_nodes() -> Vec<Node> {
    let top = |species, stage, x, needs| Node { species, stage, x, floor: TREE_TOP_FLOOR, needs };
    let adult = |i: i32, species, needs| Node { species, stage: Stage::Adult, x: 30 + 36 * i, floor: TREE_ADULT_FLOOR, needs };
    vec![
        top("Babytchi", Stage::Baby, 36, ""),
        top("Marutchi", Stage::Child, 80, ""),
        top("Tonmarutchi", Stage::Teen, 140, "default"),
        top("Hashitamatchi", Stage::Teen, 196, "games"),
        adult(0, "Mametchi", "scolds"),
        adult(1, "Kuchipatchi", "treats"),
        adult(2, "Tarakotchi", "games"),
        adult(3, "Ichigotchi", "snacks"),
        adult(4, "Nyorotchi", "care"),
        adult(5, "Futsutchi", "none"),
    ]
}

/// Whether the pet is this form right now.
fn is_now(app: &App, n: &Node) -> bool {
    app.pet.alive() && app.pet.stage == n.stage && app.pet.species == n.species
}

fn is_on_track(app: &App, n: &Node) -> bool {
    app.pet.on_track() == Some(n.species)
}

fn outline(f: &mut Frame, (x, y, w, h): (i32, i32, i32, i32), colour: Pal) {
    f.rect(x, y, w, 1, colour);
    f.rect(x, y + h - 1, w, 1, colour);
    f.rect(x, y, 1, h, colour);
    f.rect(x + w - 1, y, 1, h, colour);
}

/// An arrow pointing right, `w` pixels long, with its middle at (`x`, `y`).
fn arrow(f: &mut Frame, x: i32, y: i32, w: i32) {
    f.rect(x, y, w, 1, Pal::Slate);
    f.rect(x + w - 2, y - 1, 1, 3, Pal::Slate);
    f.rect(x + w - 3, y - 2, 1, 5, Pal::Slate);
}

fn draw_tree(f: &mut Frame, app: &App) {
    arrow(f, 56, TREE_TOP_FLOOR - 14, 14);
    arrow(f, 98, TREE_TOP_FLOOR - 14, 14);
    for n in tree_nodes() {
        let shown = n.stage != Stage::Adult || app.profile.raised(n.species) || is_now(app, &n);
        let s = form_sprite(n.species, n.stage, shown);
        let (x, y) = (n.x - s.w / 2, n.floor - s.bottom());
        f.sprite(&s, x, y);
        let colour = if is_now(app, &n) { Some(Pal::Yellow) } else if is_on_track(app, &n) { Some(Pal::Sky) } else { None };
        if let Some(c) = colour {
            let (l, r) = ((0..s.w).find(|&x| (0..s.h).any(|y| s.get(x, y).is_some())).unwrap_or(0), (0..s.w).rev().find(|&x| (0..s.h).any(|y| s.get(x, y).is_some())).unwrap_or(s.w - 1));
            outline(f, (x + l - 2, y + s.top() - 2, r - l + 5, s.bottom() - s.top() + 4), c);
        }
    }
}

fn tree_labels(app: &App) -> Vec<Label> {
    let mut v = vec![label("FAMILY TREE", CARD.0 as f32 + 6.0, TITLE_Y, Pal::Pink)];
    v.push(centred("or", 168.0, (TREE_TOP_FLOOR - 14) as f32, Pal::Slate));
    for n in tree_nodes() {
        let tint = if is_now(app, &n) { Pal::Yellow } else if is_on_track(app, &n) { Pal::Sky } else { Pal::Mist };
        if n.stage == Stage::Adult {
            v.push(centred(n.needs, n.x as f32, TREE_NEEDS_Y, tint));
            v.push(centred(Personality::of(n.species).label(), n.x as f32, TREE_NEEDS_Y + 6.0, Pal::Slate));
        } else {
            v.push(centred(n.species, n.x as f32, (TREE_TOP_FLOOR + 5) as f32, tint));
            if !n.needs.is_empty() {
                let line = format!("{}, {}", n.needs, Personality::of(n.species).label());
                v.push(centred(line, n.x as f32, (TREE_TOP_FLOOR + 11) as f32, Pal::Slate));
            }
        }
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

fn game_card_w() -> i32 {
    let n = crate::games::CHOICES.len() as i32;
    (CARD.2 - 2 * (GAME_CARD.0 - CARD.0) - GAME_CARD_GAP * (n - 1)) / n
}

fn game_card_x(i: i32) -> i32 {
    GAME_CARD.0 + i * (game_card_w() + GAME_CARD_GAP)
}

fn draw_chooser(f: &mut Frame) {
    let (y, w) = (GAME_CARD.1, game_card_w());
    for i in 0..crate::games::CHOICES.len() as i32 {
        let x = game_card_x(i);
        f.boxed(x, y, w, GAME_CARD_H, Pal::Dusk, Pal::Slate);
        keycap(f, x + 4, y + 4, false);
        match i {
            0 => {
                // Four little pads, one of them lit.
                let (gap, side) = (2, 4);
                let size = (w - 2 * side - 3 * gap) / 4;
                for (n, c) in [Pal::Red, Pal::Yellow, Pal::Green, Pal::Sky].into_iter().enumerate() {
                    let fill = if n == 1 { c.highlight() } else { c.shadow() };
                    f.boxed(x + side + n as i32 * (size + gap), y + 26, size, size, fill, Pal::Ink);
                }
            }
            1 => {
                f.rect(x + 3, y + 20, w - 6, 28, Pal::DeepTeal);
                f.rect(x + w / 2, y + 22, 1, 24, Pal::Teal);
                f.boxed(x + 5, y + 26, 3, 12, Pal::Pink, Pal::Ink);
                f.boxed(x + w / 2 - 6, y + 30, 4, 4, Pal::Yellow, Pal::Orange);
                f.boxed(x + w - 9, y + 28, 4, 10, Pal::Mist, Pal::Ink);
            }
            2 => {
                // Three cups in a row, the middle one lifted.
                for n in 0..3 {
                    let cx = x + w / 6 + n * w / 3;
                    let top = if n == 1 { y + 22 } else { y + 30 };
                    f.boxed(cx - 5, top, 10, 14, Pal::Orange, Pal::Ink);
                }
            }
            3 => f.grid_scaled(crate::games::RPS_ICON, x + (w - 30) / 2, y + 20, 2, |p| p),
            4 => {
                // A track with its green zone and the marker.
                f.boxed(x + 3, y + 30, w - 6, 8, Pal::Night, Pal::Ink);
                f.rect(x + 8, y + 31, 10, 6, Pal::Green);
                f.boxed(x + 14, y + 26, 3, 16, Pal::Yellow, Pal::Ink);
            }
            5 => {
                // A crate on the floor with the hop over it.
                f.rect(x + 3, y + 44, w - 6, 2, Pal::Brown);
                f.boxed(x + w - 14, y + 34, 9, 10, Pal::Tan, Pal::Ink);
                for (dx, dy) in [(4, 30), (7, 26), (10, 25), (13, 26), (16, 30)] {
                    f.rect(x + dx, y + dy - 6, 2, 2, Pal::Yellow);
                }
            }
            _ => {}
        }
    }
}

fn chooser_labels() -> Vec<Label> {
    let mut v = vec![label("PICK A GAME", CARD.0 as f32 + 6.0, TITLE_Y, Pal::Pink)];
    for (i, (key, name)) in crate::games::CHOICES.iter().enumerate() {
        let x = game_card_x(i as i32) as f32;
        v.push(centred(*key, x + 8.5, GAME_CARD.1 as f32 + 8.5, Pal::White));
        v.push(centred(name.to_uppercase(), x + game_card_w() as f32 / 2.0, GAME_CARD.1 as f32 + 60.0, Pal::White));
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
        Mode::Keepsakes => draw_keepsakes(&mut local, app),
        Mode::Badges => draw_badges(&mut local, app),
        Mode::Gifts(gifts) => draw_gifts(&mut local, gifts),
        Mode::Tree => draw_tree(&mut local, app),
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
        Mode::Keepsakes => keepsakes_labels(app),
        Mode::Badges => badges_labels(app),
        Mode::Gifts(gifts) => gifts_labels(gifts),
        Mode::Tree => tree_labels(app),
        Mode::Graveyard(entries) => graveyard_labels(entries),
        _ => Vec::new(),
    };
    if showing(app) {
        v.push(centred("esc", ESC.0 as f32 + ESC.2 as f32 / 2.0, ESC.1 as f32 + ESC.3 as f32 / 2.0 - 0.5, Pal::White));
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
        Mode::Chooser => ("PRESS 1 TO 6, OR CLICK A GAME".into(), Pal::Yellow),
        Mode::Shop => (format!("PRESS 1-4 OR CLICK TO BUY  -  TAB: {}", ["TREATS", "FOR THE ROOM", "CARE ITEMS"][app.shop_tab]), Pal::Yellow),
        Mode::Awards => match profile::ACHIEVEMENTS.iter().find(|a| !app.profile.has(a.id)) {
            Some(next) => (format!("NEXT: {} - {}", next.name.to_uppercase(), next.how.to_uppercase()), Pal::Mist),
            None => ("EVERY AWARD UNLOCKED!".into(), Pal::Yellow),
        },
        Mode::Badges => match app.profile.next_badge() {
            Some((b, left)) => (format!("NEXT: {} - {} MORE {}", b.name.to_uppercase(), left, profile::care_name(b.care).to_uppercase()), Pal::Mist),
            None => ("EVERY BADGE EARNED!".into(), Pal::Yellow),
        },
        Mode::Keepsakes => ("PRESS 1-6 TO PUT ONE ON THE WINDOWSILL".into(), Pal::Mist),
        Mode::Gifts(_) => ("ANY KEY TO CONTINUE".into(), Pal::Yellow),
        Mode::Tree => ("YELLOW: NOW  BLUE: NEXT  UNDER EACH: HABIT, PERSONALITY".into(), Pal::Mist),
        Mode::Graveyard(entries) if entries.is_empty() => ("ANY KEY TO GO BACK".into(), Pal::Mist),
        Mode::Graveyard(entries) if app.profile.plaque_pets >= entries.len() => ("ANY KEY TO GO BACK  -  THE LAST PET HAS ITS PLAQUE".into(), Pal::Mist),
        Mode::Graveyard(_) => (format!("P: HANG A PLAQUE FOR THE LAST PET ({} COINS)  -  ANY OTHER KEY: BACK", decor::PLAQUE_PRICE), Pal::Yellow),
        _ => return None,
    })
}

/// The close button over a game: the top-right corner of the screen.
pub fn game_esc(lay: Layout) -> (i32, i32, i32, i32) {
    (lay.w - 20, 3, 16, 12)
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
        Mode::Chooser => (0..crate::games::CHOICES.len() as i32).find(|&i| (game_card_x(i)..game_card_x(i) + game_card_w()).contains(&x) && (GAME_CARD.1..GAME_CARD.1 + GAME_CARD_H).contains(&y)).map(|i| Key::Char((b'1' + i as u8) as char)),
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
        Mode::Shop => (0..4).find(|&i| in_rect((CARD.0 + 4, shop_row_y(i), CARD.2 - 8, SHOP_ROW - 3), x, y)).map(|i| moved((CARD.0 + 4, shop_row_y(i), CARD.2 - 8, SHOP_ROW - 3))),
        Mode::Chooser => (0..crate::games::CHOICES.len() as i32).find(|&i| in_rect((game_card_x(i), GAME_CARD.1, game_card_w(), GAME_CARD_H), x, y)).map(|i| moved((game_card_x(i), GAME_CARD.1, game_card_w(), GAME_CARD_H))),
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
        Align::Right => (l.x - w, l.x),
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
        for want in ["SHOP", "7", "Fancy meal", "Vitamins", "25"] {
            assert!(text.iter().any(|t| t == want), "shop is missing {want}: {text:?}");
        }
        assert_eq!(click(&app, Layout::MIN, 100.0, shop_row_y(2) as f32 + 5.0 + Layout::MIN.centred_card().1 as f32), Some(Key::Char('3')));
        assert_eq!(click(&app, Layout::MIN, 100.0, shop_row_y(0) as f32 - 3.0 + Layout::MIN.centred_card().1 as f32), None);
    }

    #[test]
    fn the_tree_names_every_stage_and_marks_where_the_pet_is() {
        let mut app = app_with(Stage::Teen, |p| {
            p.species = "Tonmarutchi".into();
            p.habits = [6, 0, 0, 0, 0];
        });
        app.mode = Mode::Tree;
        let text: Vec<String> = labels(&app, Layout::MIN).into_iter().map(|l| l.text).collect();
        for want in ["Babytchi", "Marutchi", "Tonmarutchi", "Hashitamatchi", "scolds", "none", "strict"] {
            assert!(text.iter().any(|t| t == want), "the tree is missing {want}: {text:?}");
        }
        let forms = tree_nodes();
        let now: Vec<&str> = forms.iter().filter(|n| is_now(&app, n)).map(|n| n.species).collect();
        assert_eq!(now, ["Tonmarutchi"]);
        let next: Vec<&str> = forms.iter().filter(|n| is_on_track(&app, n)).map(|n| n.species).collect();
        assert_eq!(next, ["Tarakotchi"], "six games lead to Tarakotchi");
    }

    #[test]
    fn the_welcome_back_card_lists_the_gifts_and_any_key_closes_it() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.mode = Mode::Gifts(vec![profile::Gift::Coins(4), profile::Gift::Keepsake("acorn")]);
        let text: Vec<String> = labels(&app, Layout::MIN).into_iter().map(|l| l.text).collect();
        for want in ["WELCOME BACK", "4 coins", "an acorn (new keepsake!)"] {
            assert!(text.iter().any(|t| t == want), "missing {want}: {text:?}");
        }
        assert_eq!(hint(&app).unwrap().0, "ANY KEY TO CONTINUE");
        app.on_key(Key::Char('x'));
        assert!(matches!(app.mode, Mode::Normal));
    }

    #[test]
    fn the_keepsakes_page_hides_what_is_not_found_and_sits_next_to_the_awards() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.profile.keepsakes.push("pebble".into());
        app.mode = Mode::Awards;
        app.on_key(Key::Char('k'));
        assert!(matches!(app.mode, Mode::Keepsakes));
        let text: Vec<String> = labels(&app, Layout::MIN).into_iter().map(|l| l.text).collect();
        assert!(text.iter().any(|t| t == "KEEPSAKES 1/6") && text.iter().any(|t| t == "2 Pebble"), "{text:?}");
        assert_eq!(text.iter().filter(|t| *t == "???").count(), 5);
        app.on_key(Key::Left);
        assert!(matches!(app.mode, Mode::Awards));
        app.on_key(Key::Right);
        app.on_key(Key::Char('x'));
        assert!(matches!(app.mode, Mode::Normal));
    }

    #[test]
    fn the_badges_page_shows_the_counts_and_the_next_goal() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.profile.cared = [120, 40, 0, 0];
        app.mode = Mode::Awards;
        app.on_key(Key::Char('b'));
        assert!(matches!(app.mode, Mode::Badges));
        let text: Vec<String> = labels(&app, Layout::MIN).into_iter().map(|l| l.text).collect();
        for want in ["BADGES 1/12", "meals", "120", "Caring cook", "500", "1000"] {
            assert!(text.iter().any(|t| t == want), "missing {want}: {text:?}");
        }
        assert_eq!(hint(&app).unwrap().0, "NEXT: SWEET TOOTH - 60 MORE SNACKS");
        app.on_key(Key::Left);
        assert!(matches!(app.mode, Mode::Keepsakes));
        app.on_key(Key::Right);
        app.on_key(Key::Char('a'));
        assert!(matches!(app.mode, Mode::Awards));
    }

    #[test]
    fn the_shop_has_three_shelves_and_tab_flips_between_them() {
        crate::save::with_temp_dir(|| {
            let mut app = app_with(Stage::Adult, |_| {});
            app.profile.coins = 50;
            app.mode = Mode::Shop;
            app.on_key(Key::Tab);
            let treats: Vec<String> = labels(&app, Layout::MIN).into_iter().map(|l| l.text).collect();
            for want in ["Cupcake", "Dumbbell", "Potty charm", "Medicine kit", "22"] {
                assert!(treats.iter().any(|t| t == want), "missing {want}: {treats:?}");
            }
            assert!(hint(&app).unwrap().0.contains("TAB: FOR THE ROOM"));
            app.on_key(Key::Tab);
            assert_eq!(app.shop_tab, 2);
            let text: Vec<String> = labels(&app, Layout::MIN).into_iter().map(|l| l.text).collect();
            for want in ["Star poster", "Potted fern", "Floor lamp", "Teddy bear", "60"] {
                assert!(text.iter().any(|t| t == want), "missing {want}: {text:?}");
            }
            assert!(hint(&app).unwrap().0.contains("TAB: CARE ITEMS"));
            app.on_key(Key::Char('2'));
            assert!(app.profile.placed("fern") && app.profile.coins == 10);
            assert!(matches!(app.mode, Mode::Normal), "the shop closes so you can see the room");
            app.mode = Mode::Shop;
            app.on_key(Key::Char('2'));
            assert_eq!(app.profile.coins, 10, "buying it again charges nothing");
            app.on_key(Key::Left);
            assert_eq!(app.shop_tab, 1, "left goes back a shelf");
            app.on_key(Key::Right);
            app.on_key(Key::Right);
            assert_eq!(app.shop_tab, 0, "and the shelves wrap round");
        });
    }

    #[test]
    fn the_graveyard_offers_a_plaque_for_the_last_pet_once() {
        crate::save::with_temp_dir(|| {
            let entry = "2026-11-01 | Mametchi | lived 3d 4h | 1 care mistakes | cause: old age".to_string();
            let mut app = app_with(Stage::Adult, |_| {});
            app.profile.coins = 30;
            app.mode = Mode::Graveyard(vec![entry.clone()]);
            assert!(hint(&app).unwrap().0.starts_with("P: HANG A PLAQUE"));
            app.on_key(Key::Char('p'));
            assert_eq!(app.profile.plaque.as_ref().map(|p| p.lived.as_str()), Some("3d"));
            assert_eq!(app.profile.coins, 10);
            app.mode = Mode::Graveyard(vec![entry]);
            assert!(hint(&app).unwrap().0.contains("HAS ITS PLAQUE"));
    
        });
    }

    #[test]
    fn the_awards_list_every_achievement_and_the_collection() {
        let mut app = app_with(Stage::Adult, |_| {});
        app.mode = Mode::Awards;
        let text: Vec<String> = labels(&app, Layout::MIN).into_iter().map(|l| l.text).collect();
        assert!(text.iter().any(|t| t == "First steps") && text.iter().any(|t| t == "Collector"), "{text:?}");
        assert!(text.iter().any(|t| t.contains("adults raised 0/6")));
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
        for mode in [Mode::Chooser, Mode::Shop, Mode::Awards, Mode::Tree, Mode::Graveyard(graves), Mode::Graveyard(Vec::new())] {
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
