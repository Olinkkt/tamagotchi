use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::pet::{self, Pet, Stage, MAX_HEARTS};
use crate::profile::{self, Profile};
use crate::{fmt_age, games, sprites, App, Mode};

pub const BORDER: Color = Color::Rgb(86, 90, 125);
pub const DIM: Color = Color::Rgb(120, 124, 150);
pub const TEXT: Color = Color::Rgb(220, 222, 240);
pub const ACCENT: Color = Color::Rgb(255, 160, 190);
pub const GOOD: Color = Color::Rgb(130, 210, 140);
pub const WARN: Color = Color::Rgb(245, 200, 90);
pub const BAD: Color = Color::Rgb(240, 95, 105);
pub const EMPTY: Color = Color::Rgb(70, 72, 92);

fn panel(title: &str) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(BORDER))
        .title(Span::styled(format!(" {title} "), Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)))
}

fn hearts(n: u8) -> Vec<Span<'static>> {
    (0..MAX_HEARTS)
        .map(|i| if i < n { Span::styled("♥ ", Style::new().fg(BAD)) } else { Span::styled("♥ ", Style::new().fg(EMPTY)) })
        .collect()
}

fn bar(pct: u8, width: usize, color: Color) -> Vec<Span<'static>> {
    let filled = (pct as usize * width).div_ceil(100).min(width);
    vec![
        Span::styled("█".repeat(filled), Style::new().fg(color)),
        Span::styled("░".repeat(width - filled), Style::new().fg(EMPTY)),
    ]
}

fn stat_row(label: &str, mut value: Vec<Span<'static>>) -> Line<'static> {
    let mut spans = vec![Span::styled(format!("{label:<11}"), Style::new().fg(DIM))];
    spans.append(&mut value);
    Line::from(spans)
}

/// A shorter label for narrow terminals.
fn short_label(label: &str) -> &str {
    match label {
        "medicine" => "med",
        "graveyard" => "grave",
        "new egg" => "new",
        "quit game" => "quit",
        "repeat the pattern" => "repeat",
        "move paddle" => "move",
        other => other,
    }
}

/// `size` 0 is the full chip, 1 uses a short label with less padding, 2 is the key alone.
fn key_chip_sized(key: &str, label: &str, size: u8) -> Vec<Span<'static>> {
    let key_span = Span::styled(format!(" {key} "), Style::new().fg(Color::Rgb(20, 20, 30)).bg(ACCENT).add_modifier(Modifier::BOLD));
    let label_span = match size {
        0 => Span::styled(format!(" {label}  "), Style::new().fg(TEXT)),
        1 => Span::styled(format!("{} ", short_label(label)), Style::new().fg(TEXT)),
        _ => Span::styled(" ", Style::new().fg(TEXT)),
    };
    vec![key_span, label_span]
}

fn shop_lines(profile: &Profile, pet: &Pet) -> Vec<Line<'static>> {
    let mut v = vec![
        Line::from(Span::styled("SHOP", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("you have ◎ {}", profile.coins), Style::new().fg(WARN))),
        Line::from(""),
    ];
    for (i, item) in pet::Item::ALL.iter().enumerate() {
        let affordable = profile.coins >= item.price();
        let colour = if affordable { TEXT } else { EMPTY };
        // Every row is the same width, so the centred block lines up.
        let spans = vec![
            Span::styled(format!(" {} ", i + 1), Style::new().fg(Color::Rgb(20, 20, 30)).bg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {:<11}", item.name()), Style::new().fg(colour).add_modifier(Modifier::BOLD)),
            Span::styled(format!("◎{:<4}", item.price()), Style::new().fg(if affordable { WARN } else { EMPTY })),
            Span::styled(format!("{:<24}", item.blurb()), Style::new().fg(DIM)),
        ];
        v.push(Line::from(spans));
    }
    v.push(Line::from(""));
    v.push(Line::from(Span::styled(
        if pet.alive() { "win mini-games to earn coins" } else { "your pet has passed away" },
        Style::new().fg(DIM),
    )));
    v
}

fn awards_lines(profile: &Profile) -> Vec<Line<'static>> {
    let unlocked = profile.achievements.len();
    let mut v = vec![Line::from(vec![
        Span::styled("AWARDS ", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{unlocked}/{}   ◎ {}", profile::ACHIEVEMENTS.len(), profile.coins), Style::new().fg(DIM)),
    ])];
    for pair in profile::ACHIEVEMENTS.chunks(2) {
        let mut spans = Vec::new();
        for a in pair {
            let done = profile.has(a.id);
            spans.push(Span::styled(
                format!("{} {:<16}", if done { "✓" } else { "·" }, a.name),
                Style::new().fg(if done { GOOD } else { DIM }),
            ));
        }
        v.push(Line::from(spans));
    }
    v.push(Line::from(""));
    v.push(Line::from(Span::styled(
        format!("Adults raised {}/{}", profile.adults.len(), profile::ADULT_SPECIES.len()),
        Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
    )));
    for names in profile::ADULT_SPECIES.chunks(3) {
        let mut spans: Vec<Span<'static>> = names
            .iter()
            .map(|s| {
                let done = profile.raised(s);
                Span::styled(format!("{} {:<12}", if done { "✓" } else { "·" }, s), Style::new().fg(if done { GOOD } else { DIM }))
            })
            .collect();
        // Pad a short last row so the block stays aligned.
        spans.resize(3, Span::raw(" ".repeat(14)));
        v.push(Line::from(spans));
    }
    if let Some(next) = profile::ACHIEVEMENTS.iter().find(|a| !profile.has(a.id)) {
        v.push(Line::from(""));
        v.push(Line::from(Span::styled(format!("next: {} - {}", next.name, next.how), Style::new().fg(DIM))));
    }
    v
}

/// The smallest window the layout is designed for.
const MIN_W: u16 = 64;
const MIN_H: u16 = 22;

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    if area.width < MIN_W || area.height < MIN_H {
        let lines = vec![
            Line::from(Span::styled("tama needs a bigger window", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled(format!("at least {MIN_W}x{MIN_H}, this one is {}x{}", area.width, area.height), Style::new().fg(TEXT))),
            Line::from(Span::styled("(q quits)", Style::new().fg(DIM))),
        ];
        let pad = (area.height as usize).saturating_sub(lines.len()) / 2;
        let mut v = vec![Line::from(""); pad];
        v.extend(lines);
        f.render_widget(Paragraph::new(v).alignment(Alignment::Center).wrap(Wrap { trim: true }), area);
        return;
    }
    let p = &app.pet;
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(12),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .split(f.area());

    // Header
    let mood_dot = if !p.alive() { ("●", DIM) } else if p.needs_attention() { ("●", BAD) } else { ("●", GOOD) };
    let mut header = Line::from(vec![
        Span::styled(format!("{} ", mood_dot.0), Style::new().fg(mood_dot.1)),
        Span::styled(p.species.clone(), Style::new().fg(TEXT).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  ·  {}  ·  age {}", p.stage.label(), fmt_age(p.age)), Style::new().fg(DIM)),
    ]);
    if p.stage != Stage::Egg && p.alive() {
        header.spans.push(Span::styled(format!("  ·  {}", p.quirk.label()), Style::new().fg(ACCENT)));
    }
    if app.dev {
        header.spans.push(Span::styled("   DEV", Style::new().fg(Color::Rgb(20, 20, 30)).bg(WARN).add_modifier(Modifier::BOLD)));
    }
    f.render_widget(Paragraph::new(header).block(panel("tamagotchi")), rows[0]);

    // A game gets the whole width; the status panel comes back when it ends.
    let in_game = matches!(app.mode, Mode::Game(_) | Mode::Finish(_));
    let cols = Layout::horizontal([Constraint::Min(34), Constraint::Length(if in_game { 0 } else { 30 })]).split(rows[1]);

    // Room
    let room_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(BORDER));
    let inner = room_block.inner(cols[0]);
    let t = (app.started.elapsed().as_millis() / 350) as u32;
    let centered = |lines: Vec<Line<'static>>| -> Vec<Line<'static>> {
        let pad = (inner.height as usize).saturating_sub(lines.len()) / 2;
        let mut v = vec![Line::from(""); pad];
        v.extend(lines);
        v
    };
    let body: Vec<Line> = match &app.mode {
        Mode::Game(g) => g.lines(inner.width as usize, inner.height as usize),
        Mode::Finish(f) => f.lines(inner.width as usize, inner.height as usize),
        Mode::Chooser => {
            let mut v = vec![
                Line::from(Span::styled("PICK A GAME", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))),
                Line::from(""),
            ];
            for (k, l) in games::CHOICES {
                // Every row is the same width, so the centred block lines up.
                v.push(Line::from(vec![
                    Span::styled(format!(" {k} "), Style::new().fg(Color::Rgb(20, 20, 30)).bg(ACCENT).add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" {l:<20}"), Style::new().fg(TEXT)),
                ]));
            }
            centered(v)
        }
        Mode::Shop => centered(shop_lines(&app.profile, p)),
        Mode::Awards => centered(awards_lines(&app.profile)),
        Mode::Graveyard(entries) => {
            let mut v = vec![Line::from(Span::styled("GRAVEYARD", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))), Line::from("")];
            if entries.is_empty() {
                v.push(Line::from(Span::styled("No pets have passed away yet.", Style::new().fg(DIM))));
            }
            for e in entries.iter().take((inner.height as usize).saturating_sub(4)) {
                v.push(Line::from(Span::styled(e.clone(), Style::new().fg(TEXT))));
            }
            v.push(Line::from(""));
            v.push(Line::from(Span::styled("press any key to go back", Style::new().fg(DIM))));
            centered(v)
        }
        Mode::Normal => sprites::render(p, t, app.current_fx(), inner.width as usize, inner.height as usize),
    };
    f.render_widget(Paragraph::new(body).alignment(Alignment::Center).block(room_block), cols[0]);

    // Stats
    let discipline = bar(p.discipline, 10, if p.discipline >= 60 { GOOD } else { WARN });
    let mut stats = vec![
        stat_row("Hunger", hearts(p.hunger)),
        stat_row("Happy", hearts(p.happy)),
        stat_row("Discipline", discipline),
        stat_row(
            "Weight",
            vec![if p.overweight() {
                Span::styled(format!("{} (heavy)", p.weight), Style::new().fg(WARN))
            } else {
                Span::styled(format!("{}", p.weight), Style::new().fg(TEXT))
            }],
        ),
        stat_row(
            "Mistakes",
            vec![Span::styled(
                p.mistakes.to_string(),
                Style::new().fg(if p.mistakes == 0 { GOOD } else if p.mistakes < 4 { WARN } else { BAD }),
            )],
        ),
        stat_row("Coins", vec![Span::styled(format!("◎ {}", app.profile.coins), Style::new().fg(WARN))]),
        stat_row("Trait", vec![Span::styled(p.quirk.label(), Style::new().fg(ACCENT))]),
        Line::from(Span::styled(format!("  {}", p.quirk.description()), Style::new().fg(DIM))),
        Line::from(""),
    ];
    let mut chip = |text: &str, color: Color| stats.push(Line::from(Span::styled(format!("● {text}"), Style::new().fg(color))));
    if p.asleep {
        chip("sleeping", Color::Rgb(130, 150, 255));
    }
    if p.sick {
        chip("sick - give medicine", GOOD);
    }
    if p.disc_call.is_some() {
        chip("naughty - scold it", BAD);
    }
    if p.poops > 0 {
        chip("needs cleaning", WARN);
    }
    if p.overweight() {
        chip("overweight - play games", WARN);
    }
    if p.load.busy_cpu {
        chip("sweating - the CPU is busy", Color::Rgb(110, 190, 255));
    }
    if p.load.low_ram {
        chip("dizzy - memory is nearly full", Color::Rgb(190, 150, 255));
    }
    if p.load.low_battery {
        chip("sleepy - the battery is low", Color::Rgb(130, 150, 255));
    }
    if p.hunger == 0 && !p.asleep {
        chip("starving!", BAD);
    }
    if p.happy == 0 && !p.asleep {
        chip("lonely", BAD);
    }
    stats.push(Line::from(""));
    if app.dev_open {
        let help = [
            format!("dev   x{}  {:02}h", app.speed, p.local_hour()),
            "h/H hunger -/+".into(),
            "a/A happy -/+".into(),
            "d/D discipline -/+10".into(),
            "w/W weight -/+".into(),
            "m/M mistakes -/+".into(),
            "p poop   s sick".into(),
            "n naughty  g grow".into(),
            "k kill   o +10 coins".into(),
            "u cpu  r ram  b battery".into(),
            "t/T skip 1h / 8h".into(),
            "z 22:00  Z 12:00".into(),
            "f speed  tab close".into(),
        ];
        for h in help {
            stats.push(Line::from(Span::styled(h, Style::new().fg(WARN))));
        }
    } else {
        let preview = p.evolution_preview();
        if !preview.is_empty() {
            stats.push(Line::from(Span::styled("growth", Style::new().fg(DIM).add_modifier(Modifier::ITALIC))));
            for l in preview {
                stats.push(Line::from(Span::styled(l, Style::new().fg(TEXT))));
            }
            stats.push(Line::from(""));
        }
        stats.push(Line::from(Span::styled("recent", Style::new().fg(DIM).add_modifier(Modifier::ITALIC))));
        let room_left = (cols[1].height as usize).saturating_sub(2 + stats.len());
        for e in app.log.iter().rev().take(room_left).rev() {
            stats.push(Line::from(Span::styled(format!("· {e}"), Style::new().fg(DIM))));
        }
    }
    if !in_game {
        f.render_widget(Paragraph::new(stats).wrap(Wrap { trim: true }).block(panel("status")), cols[1]);
    }

    // Hint: what to do right now
    let need = p.top_need();
    let hint = match need {
        Some((k, text)) => Line::from(vec![
            Span::styled("→ ", Style::new().fg(WARN)),
            Span::styled(format!("PRESS {}: ", k.to_ascii_uppercase()), Style::new().fg(WARN).add_modifier(Modifier::BOLD)),
            Span::styled(text.to_uppercase(), Style::new().fg(TEXT)),
        ]),
        None if p.stage == Stage::Egg => Line::from(Span::styled("IT'S AN EGG - GIVE IT A MOMENT TO HATCH.", Style::new().fg(DIM))),
        None => Line::from(""),
    };
    f.render_widget(Paragraph::new(hint).alignment(Alignment::Center), rows[2]);

    // Message
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("▸ ", Style::new().fg(ACCENT)),
            Span::styled(app.message.clone(), Style::new().fg(TEXT)),
        ]))
        .block(panel("message")),
        rows[3],
    );

    // Menu
    let mut spans: Vec<Span<'static>> = Vec::new();
    let items: &[(&str, &str)] = match &app.mode {
        Mode::Game(g) => g.keys(),
        Mode::Chooser => &[("1-2", "choose"), ("esc", "back")],
        Mode::Graveyard(_) | Mode::Awards | Mode::Finish(_) => &[("any key", "back")],
        Mode::Shop => &[("1-4", "buy"), ("esc", "back")],
        Mode::Normal if p.stage == Stage::Dead => &[("n", "new egg"), ("a", "awards"), ("g", "graveyard"), ("q", "quit")],
        Mode::Normal => &[("f", "feed"), ("s", "snack"), ("p", "play"), ("l", "light"), ("m", "medicine"), ("c", "clean"), ("d", "scold"), ("b", "shop"), ("a", "awards"), ("g", "graveyard"), ("q", "quit")],
    };
    let useful = |k: &str| match k {
        "f" => p.hunger < MAX_HEARTS,
        "s" => p.happy < MAX_HEARTS,
        "m" => p.sick,
        "c" => p.poops > 0,
        "d" => p.disc_call.is_some(),
        "l" => p.asleep,
        _ => true,
    };
    let urgent = if matches!(app.mode, Mode::Normal) { need.map(|n| n.0.to_string()) } else { None };
    let ink = Color::Rgb(20, 20, 30);
    let room = rows[4].width.saturating_sub(2) as usize;
    // Largest style that fits: full labels, short labels, short labels only on the chips you can
    // use right now (the rest keys only), or keys only.
    for size in 0..=3u8 {
        spans.clear();
        for (k, l) in items {
            let chip_size = if size == 2 && !(urgent.as_deref() == Some(*k) || useful(k)) { 3 } else { size.min(3) };
            let chip_size = if size == 2 && chip_size != 3 { 1 } else { chip_size };
            let mut chip = key_chip_sized(k, l, chip_size);
            if urgent.as_deref() == Some(*k) {
                chip[0].style = Style::new().fg(ink).bg(WARN).add_modifier(Modifier::BOLD);
                chip[1].style = Style::new().fg(WARN).add_modifier(Modifier::BOLD);
            } else if !useful(k) {
                chip[0].style = Style::new().fg(ink).bg(EMPTY);
                chip[1].style = Style::new().fg(EMPTY);
            }
            spans.extend(chip);
        }
        if Line::from(spans.clone()).width() <= room {
            break;
        }
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center).block(
            Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::new().fg(BORDER)),
        ),
        rows[4],
    );
}
