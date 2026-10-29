//! The game window. The scene is drawn into a fixed 240x160 pixel frame, uploaded as one
//! texture and shown at the largest whole-number scale that fits, so every scene pixel is
//! the same size. Text is drawn on top at full window resolution with the pixel font.

use macroquad::prelude::{
    clear_background, draw_rectangle, draw_text, draw_texture_ex, get_char_pressed, is_key_pressed,
    is_mouse_button_pressed, is_quit_requested, mouse_position, next_frame, prevent_quit, screen_height, screen_width,
    vec2, Color, Conf, DrawTextureParams, FilterMode, Image, KeyCode, MouseButton, Texture2D, WHITE,
};

use crate::app::{App, Key};
use crate::font;
use crate::hud::{self, Align, Label};
use crate::palette::Pal;

const START_SCALE: i32 = 4;
const LETTERBOX: Pal = Pal::Ink;

fn conf() -> Conf {
    Conf {
        window_title: "tama".into(),
        window_width: hud::W * START_SCALE,
        window_height: hud::H * START_SCALE,
        window_resizable: true,
        ..Default::default()
    }
}

fn colour(p: Pal) -> Color {
    let (r, g, b) = p.rgb();
    Color::from_rgba(r, g, b, 255)
}

/// Where the scene goes in the window: whole-number scale, centred, letterboxed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

impl Viewport {
    pub fn fit(win_w: f32, win_h: f32) -> Viewport {
        let scale = (win_w / hud::W as f32).min(win_h / hud::H as f32).floor().max(1.0);
        let x = ((win_w - hud::W as f32 * scale) / 2.0).floor().max(0.0);
        let y = ((win_h - hud::H as f32 * scale) / 2.0).floor().max(0.0);
        Viewport { x, y, scale }
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
    for (code, key) in [(KeyCode::Escape, Key::Esc), (KeyCode::Tab, Key::Tab), (KeyCode::Up, Key::Up), (KeyCode::Down, Key::Down)] {
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

/// The developer panel: plain text over the status panel, listing the dev keys.
fn draw_dev_panel(v: &Viewport, app: &App) {
    let (x, y) = (v.x + (hud::PANEL_X as f32 + 2.0) * v.scale, v.y + 14.0 * v.scale);
    draw_rectangle(x - 4.0, y - 4.0, 76.0 * v.scale, 112.0 * v.scale, Color::from_rgba(0, 0, 0, 220));
    let p = &app.pet;
    let lines = [
        format!("DEV  x{}  {:02}:00", app.speed, p.local_hour()),
        format!("[ ] form: {}", p.species),
        format!("m/M mistakes: {}", p.mistakes),
        "p poop   s sick".into(),
        "n naughty   g grow".into(),
        "e crack the egg".into(),
        ", . clock -/+1h".into(),
        "z 22:00   Z 12:00".into(),
        "f speed   l light".into(),
        "u cpu  r ram  b batt".into(),
        "h/H a/A d/D w/W".into(),
        "k kill  o coins  t/T".into(),
        "tab close".into(),
    ];
    let size = (5.0 * v.scale).max(12.0);
    for (i, l) in lines.iter().enumerate() {
        draw_text(l, x, y + size + i as f32 * size * 1.05, size, Color::from_rgba(250, 210, 90, 255));
    }
}

pub fn run(app: App) {
    macroquad::Window::from_config(conf(), main_loop(app));
}

async fn main_loop(mut app: App) {
    prevent_quit();
    let mut image = Image::gen_image_color(hud::W as u16, hud::H as u16, WHITE);
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);
    while !app.quit && !is_quit_requested() {
        let v = Viewport::fit(screen_width(), screen_height());
        let (mx, my) = mouse_position();
        let (sx, sy) = v.to_scene(mx, my);
        for k in keys_pressed() {
            app.on_key(k);
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(k) = hud::button_at(&app, sx, sy) {
                app.press(k);
            }
        }
        app.update();

        image.bytes = hud::frame(&app, hud::button_at(&app, sx, sy)).rgba();
        texture.update(&image);
        clear_background(colour(LETTERBOX));
        let size = vec2(hud::W as f32 * v.scale, hud::H as f32 * v.scale);
        draw_texture_ex(&texture, v.x, v.y, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
        for l in hud::labels(&app) {
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
    fn the_scene_takes_the_largest_whole_scale_that_fits_and_is_centred() {
        assert_eq!(Viewport::fit(960.0, 640.0), Viewport { x: 0.0, y: 0.0, scale: 4.0 });
        assert_eq!(Viewport::fit(1000.0, 700.0), Viewport { x: 20.0, y: 30.0, scale: 4.0 });
        assert_eq!(Viewport::fit(1920.0, 1080.0), Viewport { x: 240.0, y: 60.0, scale: 6.0 });
        assert_eq!(Viewport::fit(100.0, 50.0).scale, 1.0, "never smaller than 1x");
    }

    #[test]
    fn clicks_map_back_to_scene_pixels_and_text_stays_whole_pixels() {
        let v = Viewport::fit(1000.0, 700.0);
        assert_eq!(v.to_scene(20.0, 30.0), (0.0, 0.0));
        assert_eq!(v.to_scene(60.0, 70.0), (10.0, 10.0));
        assert_eq!((Viewport::fit(720.0, 480.0).text_px(), v.text_px()), (2.0, 2.0));
    }
}
