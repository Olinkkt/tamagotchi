//! The game window. The scene is drawn into a fixed 240x160 pixel frame, uploaded as one
//! texture and shown at the largest whole-number scale that fits, so every scene pixel is
//! the same size. Text is drawn on top at full window resolution with the pixel font.

use macroquad::prelude::{
    clear_background, draw_rectangle, draw_text, measure_text, set_fullscreen, draw_texture_ex, get_char_pressed, is_key_down, is_key_pressed,
    is_mouse_button_down, is_mouse_button_pressed, is_quit_requested, mouse_position, next_frame, prevent_quit, screen_height, screen_width,
    vec2, Color, Conf, DrawTextureParams, FilterMode, Image, KeyCode, MouseButton, Texture2D, WHITE,
};

use crate::app::{App, Key};
use crate::font;
use crate::hud::{self, Align, Label, Layout};
use crate::palette::Pal;
use crate::screens;

const START_SCALE: i32 = 4;
const LETTERBOX: Pal = Pal::Ink;

fn conf() -> Conf {
    Conf {
        window_title: "tama".into(),
        window_width: Layout::MIN.w * START_SCALE,
        window_height: Layout::MIN.h * START_SCALE,
        window_resizable: true,
        ..Default::default()
    }
}

fn colour(p: Pal) -> Color {
    let (r, g, b) = p.rgb();
    Color::from_rgba(r, g, b, 255)
}

/// Where the screen goes in the window. The scale is a whole number, so every scene pixel is the
/// same size; the screen then grows to use all of the window at that scale, and only a sliver
/// (less than one scene pixel, or more on a window bigger than the art can fill) is left over.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub layout: Layout,
}

impl Viewport {
    pub fn fit(win_w: f32, win_h: f32) -> Viewport {
        let min = Layout::MIN;
        let scale = (win_w / min.w as f32).min(win_h / min.h as f32).floor().max(1.0);
        let layout = Layout::new((win_w / scale) as i32, (win_h / scale) as i32);
        let x = ((win_w - layout.w as f32 * scale) / 2.0).floor();
        let y = ((win_h - layout.h as f32 * scale) / 2.0).floor();
        Viewport { x, y, scale, layout }
    }

    /// A window position in scene pixels.
    pub fn to_scene(&self, x: f32, y: f32) -> (f32, f32) {
        ((x - self.x) / self.scale, (y - self.y) / self.scale)
    }

    /// Screen pixels per font pixel: half a scene pixel, rounded up, so text is finer than the
    /// art but never blurry.
    pub fn text_px(&self) -> f32 {
        (self.scale / 2.0).ceil()
    }
}

/// The keys pressed since the last frame, in the game's own terms.
fn keys_pressed() -> Vec<Key> {
    let mut keys = Vec::new();
    for (code, key) in [(KeyCode::Escape, Key::Esc), (KeyCode::Tab, Key::Tab), (KeyCode::Up, Key::Up), (KeyCode::Down, Key::Down), (KeyCode::Left, Key::Left), (KeyCode::Right, Key::Right)] {
        if is_key_pressed(code) {
            keys.push(key);
        }
    }
    while let Some(c) = get_char_pressed() {
        if !c.is_control() {
            keys.push(Key::Char(c));
        }
    }
    keys
}

fn draw_label(v: &Viewport, l: &Label) {
    let px = v.text_px();
    let w = font::width(&l.text) as f32 * px;
    let x = v.x + l.x * v.scale - if l.align == Align::Centre { (w / 2.0).round() } else { 0.0 };
    // `y` is the middle of the capitals, which are 7 font pixels tall.
    let y = (v.y + l.y * v.scale - 3.5 * px).round();
    let c = colour(l.colour);
    for (dx, dy) in font::pixels(&l.text) {
        draw_rectangle(x.round() + dx as f32 * px, y + dy as f32 * px, px, px, c);
    }
}

/// The developer panel: plain text over the status panel, every dev key with what it changes.
fn draw_dev_panel(v: &Viewport, app: &App) {
    let (x, y) = (v.x + (v.layout.panel_x() as f32 + 2.0) * v.scale, v.y + 13.0 * v.scale);
    let (w, h) = (76.0 * v.scale, (v.layout.hint_y() - 15) as f32 * v.scale);
    draw_rectangle(x - 2.0, y - 2.0, w, h, Color::from_rgba(0, 0, 0, 225));
    let p = &app.pet;
    let on = |b: bool| if b { "ON" } else { "off" };
    let lines = [
        format!("DEV  speed x{}  {:02}:00", app.speed, p.local_hour()),
        format!("<- ->  form: {}", p.species),
        format!("h/H  hunger {}/4", p.hunger),
        format!("a/A  happy {}/4", p.happy),
        format!("d/D  discipline {}", p.discipline),
        format!("w/W  weight {}", p.weight),
        format!("m/M  mistakes {}", p.mistakes),
        format!("o    +10 coins ({})", app.profile.coins),
        format!("p    poop ({}/3)", p.poops),
        "s sick  n naughty  g grow".into(),
        "k    kill the pet".into(),
        "e    crack the egg".into(),
        "t/T  skip 1h / 8h".into(),
        ", .  clock -/+ 1h".into(),
        "z/Z  clock 22:00 / 12:00".into(),
        format!("u cpu {}  r ram {}", on(p.load.busy_cpu), on(p.load.low_ram)),
        format!("b battery {}", on(p.load.low_battery)),
        "f    speed 1/2/5/10/60/600".into(),
        "1    back to speed x1".into(),
        "tab  close the panel".into(),
    ];
    // Shrink the text until the longest line and all the rows fit the panel.
    let mut size = (h / lines.len() as f32).floor();
    while size > 8.0 && lines.iter().any(|l| measure_text(l, None, size as u16, 1.0).width > w - 6.0) {
        size -= 1.0;
    }
    let colour = Color::from_rgba(250, 210, 90, 255);
    for (i, l) in lines.iter().enumerate() {
        draw_text(l, x + 2.0, y + size * (i as f32 + 0.85), size, colour);
    }
}

pub fn run(app: App) {
    macroquad::Window::from_config(conf(), main_loop(app));
}

async fn main_loop(mut app: App) {
    prevent_quit();
    let mut fullscreen = false;
    let mut size = (0, 0);
    let mut image = Image::gen_image_color(1, 1, WHITE);
    let mut texture = Texture2D::from_image(&image);
    while !app.quit && !is_quit_requested() {
        if is_key_pressed(KeyCode::F11) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }
        let v = Viewport::fit(screen_width(), screen_height());
        let lay = v.layout;
        if size != (lay.w, lay.h) {
            // The window changed shape, so the screen did too.
            size = (lay.w, lay.h);
            image = Image::gen_image_color(lay.w as u16, lay.h as u16, WHITE);
            texture = Texture2D::from_image(&image);
            texture.set_filter(FilterMode::Nearest);
        }
        let (mx, my) = mouse_position();
        let (sx, sy) = v.to_scene(mx, my);
        for k in keys_pressed() {
            app.on_key(k);
        }
        let under_mouse = hud::button_at(&app, lay, sx, sy).or_else(|| screens::click(&app, lay, sx, sy));
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(k) = under_mouse {
                app.press(k);
            }
        }
        // Real key-down state, so the Pong paddle moves exactly while a key or button is held.
        let held = |keys: [KeyCode; 3], button: Key| {
            keys.iter().any(|&k| is_key_down(k)) || (is_mouse_button_down(MouseButton::Left) && under_mouse == Some(button))
        };
        app.hold(held([KeyCode::Up, KeyCode::W, KeyCode::K], Key::Up), held([KeyCode::Down, KeyCode::S, KeyCode::J], Key::Down));
        app.update();

        image.bytes = hud::frame(&app, lay, hud::button_at(&app, lay, sx, sy)).rgba();
        texture.update(&image);
        clear_background(colour(LETTERBOX));
        let dest = vec2(lay.w as f32 * v.scale, lay.h as f32 * v.scale);
        draw_texture_ex(&texture, v.x, v.y, WHITE, DrawTextureParams { dest_size: Some(dest), ..Default::default() });
        for l in hud::labels(&app, lay, v.text_px() / v.scale) {
            draw_label(&v, &l);
        }
        if app.dev_open {
            draw_dev_panel(&v, &app);
        }
        next_frame().await;
    }
    app.shutdown();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scale_is_a_whole_number_and_the_screen_grows_to_fill_the_window() {
        let at = |w: f32, h: f32| {
            let v = Viewport::fit(w, h);
            (v.scale, v.layout.w, v.layout.h, v.x, v.y)
        };
        assert_eq!(at(960.0, 640.0), (4.0, 240, 160, 0.0, 0.0));
        assert_eq!(at(1000.0, 700.0), (4.0, 250, 175, 0.0, 0.0), "the extra room goes to the screen, not to bars");
        assert_eq!(at(1920.0, 1080.0), (6.0, 320, 180, 0.0, 0.0));
        assert_eq!(at(1893.0, 993.0), (6.0, 315, 165, 1.0, 1.0), "less than a scene pixel is left over");
        assert_eq!(at(100.0, 50.0).0, 1.0, "never smaller than 1x");
        assert_eq!(at(100.0, 50.0).1, 240, "and never a smaller screen than the layout needs");
    }

    #[test]
    fn a_huge_window_gets_bars_instead_of_a_bigger_room_than_the_art() {
        let v = Viewport::fit(10_000.0, 1500.0);
        assert_eq!((v.layout.w, v.layout.h), (Layout::MAX.w, 1500 / 9));
        assert!(v.x > 0.0, "centred, with bars at the sides");
    }

    #[test]
    fn clicks_map_back_to_scene_pixels_and_text_stays_whole_pixels() {
        let v = Viewport::fit(1003.0, 705.0);
        assert_eq!((v.x, v.y), (1.0, 0.0));
        assert_eq!(v.to_scene(1.0, 0.0), (0.0, 0.0));
        assert_eq!(v.to_scene(41.0, 40.0), (10.0, 10.0));
        assert_eq!((Viewport::fit(720.0, 480.0).text_px(), v.text_px()), (2.0, 2.0));
    }
}
