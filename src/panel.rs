//! The status panel to the right of the room: hearts, discipline, the numbers, what is going
//! on with the pet right now, what it will grow into, and the latest events.

use crate::app::App;
use crate::font;
use crate::frame::Frame;
use crate::hud::{Align, Label, Layout};
use crate::icons;
use crate::palette::Pal;
use crate::pet::{Pet, Stage, MAX_HEARTS};

/// The panel is laid out for the standard screen, where it starts at this column; on a wider
/// screen everything moves right with it.
const BASE_X: i32 = 160;
const LEFT: f32 = BASE_X as f32 + 4.0;
/// Where the values in the fixed rows start.
const VALUE_X: i32 = BASE_X + 38;
const WIDTH: f32 = 72.0;
/// Fixed rows, by the middle of their line.
const HUNGER_Y: i32 = 18;
const HAPPY_Y: i32 = 27;
const DISCIPLINE_Y: i32 = 36;
const WEIGHT_Y: f32 = 44.0;
const MISTAKES_Y: f32 = 51.0;
const COINS_Y: i32 = 58;
const TRAIT_Y: f32 = 65.0;
/// Free-flowing lines start here and stop this far above the hint line.
const FLOW_TOP: f32 = 73.0;
const FLOW_MARGIN: i32 = 3;
const PITCH: f32 = 7.0;

/// The pixel parts: hearts, the discipline bar and the coin.
pub fn draw(f: &mut Frame, app: &App, lay: Layout) {
    let p = &app.pet;
    if !has_stats(p) {
        return;
    }
    let dx = lay.panel_x() - BASE_X;
    for (y, n) in [(HUNGER_Y, p.hunger), (HAPPY_Y, p.happy)] {
        for i in 0..MAX_HEARTS {
            let icon = if i < n { icons::HEART_FULL } else { icons::HEART_EMPTY };
            f.grid(icon, VALUE_X + dx + i as i32 * 9, y - 3);
        }
    }
    // The word "discipline" is long, so its bar starts further right than the other values.
    let (x, w, fill) = (BASE_X + dx + 46, 30, if p.discipline >= 60 { Pal::Green } else { Pal::Yellow });
    f.boxed(x, DISCIPLINE_Y - 3, w, 6, Pal::Ink, Pal::Slate);
    f.rect(x + 1, DISCIPLINE_Y - 2, ((w - 2) * p.discipline as i32 + 99) / 100, 4, fill);
    f.grid(icons::COIN, VALUE_X + dx, COINS_Y - 3);
}

fn has_stats(p: &Pet) -> bool {
    p.alive() && p.stage != Stage::Egg
}

/// What is going on right now, each with its colour.
fn chips(p: &Pet) -> Vec<(&'static str, Pal)> {
    let flags = [
        (p.asleep, "sleeping", Pal::Sky),
        (p.sick, "sick - give medicine", Pal::Lime),
        (p.disc_call.is_some(), "naughty - scold it", Pal::Red),
        (p.urge.is_some(), "needs the toilet - press T", Pal::Yellow),
        (p.overweight(), "overweight - play games", Pal::Yellow),
        (p.load.busy_cpu, "sweating - the CPU is busy", Pal::Ice),
        (p.load.low_ram, "dizzy - memory is nearly full", Pal::Violet),
        (p.load.low_battery, "sleepy - the battery is low", Pal::Sky),
        (p.hunger == 0 && !p.asleep, "starving!", Pal::Red),
        (p.happy == 0 && !p.asleep, "lonely", Pal::Red),
    ];
    flags.into_iter().filter(|f| f.0 && p.alive()).map(|(_, text, c)| (text, c)).collect()
}

/// The text of the panel. `ratio` is how many scene pixels one font pixel covers, so lines
/// can be wrapped to what actually fits at the window's current scale.
pub fn labels(app: &App, ratio: f32, lay: Layout) -> Vec<Label> {
    let p = &app.pet;
    let dx = (lay.panel_x() - BASE_X) as f32;
    let (col, value_x) = (LEFT + dx, VALUE_X as f32 + dx);
    let flow_bottom = (lay.hint_y() - FLOW_MARGIN) as f32;
    let max = (WIDTH / ratio) as i32;
    let text = |t: &str, x: f32, y: f32, c: Pal| Label { text: t.into(), x, y, colour: c, align: Align::Left };
    let mut out = Vec::new();
    if has_stats(p) {
        out.push(text("hunger", col, HUNGER_Y as f32, Pal::Mist));
        out.push(text("happy", col, HAPPY_Y as f32, Pal::Mist));
        out.push(text("discipline", col, DISCIPLINE_Y as f32, Pal::Mist));
        out.push(text("weight", col, WEIGHT_Y, Pal::Mist));
        let weight = if p.overweight() { format!("{} heavy", p.weight) } else { p.weight.to_string() };
        out.push(text(&weight, value_x, WEIGHT_Y, if p.overweight() { Pal::Yellow } else { Pal::White }));
        out.push(text("mistakes", col, MISTAKES_Y, Pal::Mist));
        let colour = match p.mistakes {
            0 => Pal::Green,
            1..=3 => Pal::Yellow,
            _ => Pal::Red,
        };
        out.push(text(&p.mistakes.to_string(), value_x, MISTAKES_Y, colour));
        out.push(text("trait", col, TRAIT_Y, Pal::Mist));
        out.push(text(p.quirk.label(), value_x, TRAIT_Y, Pal::Pink));
    }
    out.push(text("coins", col, COINS_Y as f32, Pal::Mist));
    out.push(text(&app.profile.coins.to_string(), value_x + 8.0, COINS_Y as f32, Pal::Yellow));

    // The rest flows down the panel, most important first, until it is full.
    let mut flow: Vec<(String, Pal)> = Vec::new();
    let mut add = |t: &str, c: Pal, indent: bool| {
        for (i, line) in font::wrap(t, max - if indent { 8 } else { 0 }).into_iter().enumerate() {
            flow.push((if i > 0 && indent { format!("  {line}") } else { line }, c));
        }
    };
    for (t, c) in chips(p) {
        add(&format!("• {t}"), c, true);
    }
    let growth = p.evolution_preview();
    if !growth.is_empty() {
        add("growth", Pal::Pink, false);
        for line in &growth {
            add(line, Pal::White, false);
        }
    }
    if has_stats(p) {
        add(p.quirk.description(), Pal::Slate, false);
    }
    let room = ((flow_bottom - FLOW_TOP) / PITCH) as usize + 1;
    let left = room.saturating_sub(flow.len());
    if left >= 2 && !app.log.is_empty() {
        flow.push(("recent".into(), Pal::Pink));
        let recent: Vec<String> = app.log.iter().rev().take(4).flat_map(|e| font::wrap(e, max)).collect();
        flow.extend(recent.into_iter().take(left - 1).map(|l| (l, Pal::Slate)));
    }
    for (i, (t, c)) in flow.into_iter().take(room).enumerate() {
        out.push(text(&t, col, FLOW_TOP + i as f32 * PITCH, c));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::app_with;
    use crate::sysinfo::Strain;

    fn panel_text(app: &App, ratio: f32) -> String {
        labels(app, ratio, Layout::MIN).into_iter().map(|l| l.text).collect::<Vec<_>>().join("\n")
    }

    #[test]
    fn the_panel_names_what_the_machine_is_doing_to_the_pet() {
        let app = app_with(Stage::Adult, |p| p.load = Strain { busy_cpu: true, low_ram: true, low_battery: true });
        let text = panel_text(&app, 0.5);
        assert!(text.contains("sweating") && text.contains("dizzy") && text.contains("sleepy"), "{text}");
        let calm = panel_text(&app_with(Stage::Adult, |_| {}), 0.5);
        assert!(!calm.contains("sweating") && !calm.contains("dizzy"));
    }

    #[test]
    fn the_growth_section_says_what_it_is_on_track_for() {
        let app = app_with(Stage::Teen, |p| {
            p.mistakes = 0;
            p.discipline = 80;
        });
        let text = panel_text(&app, 0.5);
        assert!(text.contains("growth") && text.contains("Mametchi"), "{text}");
    }

    #[test]
    fn every_line_fits_the_panel_at_the_coarsest_text_scale() {
        // At 3x the window, a font pixel covers two thirds of a scene pixel: the widest case.
        let ratio = 2.0 / 3.0;
        let mut app = app_with(Stage::Teen, |p| {
            p.sick = true;
            p.urge = Some(0);
            p.disc_call = Some(0);
            p.weight = 99;
            p.load = Strain { busy_cpu: true, low_ram: true, low_battery: true };
        });
        app.log = (0..20).map(|i| format!("Something long happened to the pet, number {i}")).collect();
        for l in labels(&app, ratio, Layout::MIN) {
            assert!(l.x + font::width(&l.text) as f32 * ratio <= 240.0 - 1.0, "too wide: {}", l.text);
            assert!(l.y <= 125.0, "below the panel: {}", l.text);
        }
    }

    #[test]
    fn an_egg_shows_no_hearts() {
        let app = app_with(Stage::Egg, |_| {});
        assert!(!panel_text(&app, 0.5).contains("hunger"));
        let mut f = Frame::new(240, 160, Pal::Ink);
        draw(&mut f, &app, Layout::MIN);
        assert_eq!(f, Frame::new(240, 160, Pal::Ink));
    }
}
