//! What the pet says: a speech bubble above its head that dithers in, types its words out one
//! letter at a time, waits, and dithers out. Everything is a function of how long ago the pet
//! started speaking, so the window needs no state of its own.

use crate::font;
use crate::frame::Frame;
use crate::palette::Pal;

/// The bubble dithers in over this long, and the letters start a little after it begins.
const FADE_IN_MS: u64 = 160;
/// One letter every this long.
const TYPE_MS: u64 = 35;
/// How long the finished line stays up.
const HOLD_MS: u64 = 1600;
const FADE_OUT_MS: u64 = 260;
/// Steps of the dither: 0 is gone, `STEPS` is solid.
const STEPS: u64 = 4;
const PAD_X: i32 = 4;
const PAD_Y: i32 = 3;
/// Rows from the top of one line of text to the top of the next.
const LINE: i32 = 10;
/// Widest line of text, in font pixels.
const MAX_TEXT: i32 = 96;

/// How the bubble looks `age` milliseconds after the pet started speaking.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Look {
    /// 0 to `STEPS`: how much of the bubble the dither lets through.
    pub level: u64,
    /// How many letters have been typed so far.
    pub typed: usize,
}

fn letters(text: &str) -> usize {
    text.chars().count()
}

/// How long the whole thing lasts, from the first dither to the last.
pub fn total_ms(text: &str) -> u64 {
    FADE_IN_MS + letters(text) as u64 * TYPE_MS + HOLD_MS + FADE_OUT_MS
}

pub fn look(text: &str, age: u64) -> Option<Look> {
    let total = total_ms(text);
    if age >= total {
        return None;
    }
    let level = if age < FADE_IN_MS {
        1 + age * (STEPS - 1) / FADE_IN_MS
    } else if age >= total - FADE_OUT_MS {
        let left = total - age;
        (left * STEPS / FADE_OUT_MS).clamp(1, STEPS - 1)
    } else {
        STEPS
    };
    let typed = (age.saturating_sub(FADE_IN_MS / 2) / TYPE_MS) as usize;
    Some(Look { level, typed: typed.min(letters(text)) })
}

/// Keep only what the pet itself would say out loud: drop the sentences that are bookkeeping
/// ("Hunger +1.", "+3 coins.") and anything long enough to be an explanation.
pub fn voice(msg: &str) -> Option<String> {
    let mut out: Vec<&str> = Vec::new();
    let mut start = 0;
    let ends = msg.char_indices().filter(|&(i, c)| {
        matches!(c, '.' | '!' | '?') && msg[i + c.len_utf8()..].chars().next().is_none_or(char::is_whitespace)
    });
    let mut cuts: Vec<usize> = ends.map(|(i, c)| i + c.len_utf8()).collect();
    if cuts.last() != Some(&msg.len()) {
        cuts.push(msg.len());
    }
    for end in cuts {
        let s = msg[start..end].trim();
        start = end;
        let stats = ["Hunger", "Happy", "Discipline", "coin", "hour", "mistake"];
        if s.is_empty() || s.chars().any(|c| c.is_ascii_digit()) || stats.iter().any(|w| s.contains(w)) {
            continue;
        }
        out.push(s);
    }
    let text = out.join(" ");
    (!text.is_empty() && text.len() <= 40).then_some(text)
}

/// 4x4 ordered dither: where in the bubble a pixel switches on as `level` rises.
fn shows(x: i32, y: i32, level: u64) -> bool {
    const BAYER: [[u64; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    BAYER[(y & 3) as usize][(x & 3) as usize] < level * 16 / STEPS
}

/// Draw the bubble up and to the right of the pet, like the one for a need: its left edge at `left`
/// (pulled in if it would leave the room) and its tail pointing down at `head`. It is sized for
/// the whole line so it does not jump as the letters arrive.
pub fn draw(f: &mut Frame, text: &str, age: u64, left: i32, head: i32) {
    let Some(look) = look(text, age) else { return };
    let lines = font::wrap(text, MAX_TEXT);
    let widest = lines.iter().map(|l| font::width(l)).max().unwrap_or(0);
    let (w, h) = (widest + PAD_X * 2 + 2, lines.len() as i32 * LINE - 1 + PAD_Y * 2 + 2);
    let x = left.min(f.w - w - 1).max(1);
    let y = (head - h - 2).max(1);
    let tx = (left + 2).clamp(x + 3, x + w - 6);
    let (bw, bh) = (w, h + 2);
    let behind: Vec<Pal> = (0..bw * bh).map(|i| f.get(x + i % bw, y + i / bw).unwrap_or(Pal::Ink)).collect();
    f.boxed(x, y, w, h, Pal::White, Pal::Ink);
    for (cx, cy) in [(x, y), (x + w - 1, y), (x, y + h - 1), (x + w - 1, y + h - 1)] {
        f.set(cx, cy, behind[((cy - y) * bw + (cx - x)) as usize]);
    }
    f.rect(tx, y + h, 2, 1, Pal::White);
    f.set(tx - 1, y + h, Pal::Ink);
    f.set(tx + 2, y + h, Pal::Ink);
    f.set(tx, y + h + 1, Pal::Ink);
    f.set(tx + 1, y + h + 1, Pal::Ink);
    let mut left = look.typed;
    for (i, line) in lines.iter().enumerate() {
        let shown: String = line.chars().take(left).collect();
        left = left.saturating_sub(line.chars().count() + 1);
        f.text(&shown, x + 1 + PAD_X, y + 1 + PAD_Y + i as i32 * LINE, 1, Pal::Ink);
    }
    for dy in 0..bh {
        for dx in 0..bw {
            if !shows(x + dx, y + dy, look.level) {
                f.set(x + dx, y + dy, behind[(dy * bw + dx) as usize]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_fades_in_types_holds_and_fades_out() {
        let text = "Yum.";
        let at = |age| look(text, age);
        assert!(at(0).unwrap().level < STEPS, "starts faint");
        assert_eq!(at(0).unwrap().typed, 0, "and empty");
        assert_eq!(at(FADE_IN_MS + 200).unwrap(), Look { level: STEPS, typed: 4 });
        let end = total_ms(text);
        assert_eq!(at(end - FADE_OUT_MS - 1).unwrap().typed, 4, "all typed before it fades");
        assert!(at(end - 5).unwrap().level < STEPS, "faint again at the end");
        assert!(at(end).is_none());
    }

    #[test]
    fn letters_arrive_one_by_one_never_backwards() {
        let text = "Ahh, much better.";
        let counts: Vec<usize> = (0..total_ms(text)).step_by(5).filter_map(|a| look(text, a)).map(|l| l.typed).collect();
        assert!(counts.windows(2).all(|w| w[1] >= w[0] && w[1] - w[0] <= 1));
        assert_eq!(*counts.last().unwrap(), text.len());
    }

    #[test]
    fn only_the_pets_own_words_are_spoken() {
        assert_eq!(voice("Nom nom! Hunger +1.").as_deref(), Some("Nom nom!"));
        assert_eq!(voice("Lesson learned. Discipline up.").as_deref(), Some("Lesson learned."));
        assert_eq!(voice("So many failed commands... Happy -1.").as_deref(), Some("So many failed commands..."));
        assert_eq!(voice("Perfect game! Happy +1 and a care mistake forgiven."), Some("Perfect game!".into()));
        assert_eq!(voice("Happy +1."), None);
        assert_eq!(voice("Your pet has passed away. Press N for a new egg."), None, "too long to be speech");
    }

    #[test]
    fn the_bubble_draws_inside_the_room_at_every_moment() {
        for text in ["Yum.", "Ahh, much better.", "Thanks for the commit!", "Need toilet!"] {
            for age in (0..total_ms(text)).step_by(25) {
                for left in [0, 40, 100, 159] {
                    let mut f = Frame::new(160, 120, Pal::Night);
                    draw(&mut f, text, age, left, 40);
                }
            }
        }
    }

    #[test]
    fn a_solid_bubble_is_white_with_ink_text() {
        let mut f = Frame::new(160, 120, Pal::Night);
        draw(&mut f, "Yum.", FADE_IN_MS + 400, 80, 60);
        let count = |p| (0..120).flat_map(|y| (0..160).map(move |x| (x, y))).filter(|&(x, y)| f.get(x, y) == Some(p)).count();
        assert!(count(Pal::White) > 100 && count(Pal::Ink) > 40);
    }
}
