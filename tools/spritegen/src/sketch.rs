//! Shapes drawn with anti-aliasing off, so every pixel is exactly one palette colour.

use tiny_skia::{BlendMode, FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Rect, Stroke, Transform};

use crate::img::Img;
use crate::palette::Pal;

pub struct Sketch {
    pix: Pixmap,
    xf: Transform,
}

fn paint(p: Option<Pal>) -> Paint<'static> {
    let mut paint = Paint { anti_alias: false, ..Paint::default() };
    match p {
        Some(p) => {
            let (r, g, b) = p.rgb();
            paint.set_color_rgba8(r, g, b, 255);
        }
        None => paint.blend_mode = BlendMode::Clear,
    }
    paint
}

impl Sketch {
    pub fn new(w: u32, h: u32) -> Sketch {
        Sketch { pix: Pixmap::new(w, h).expect("non-zero size"), xf: Transform::identity() }
    }

    /// Author in `k`-times smaller units (the room is drawn in its own pixels on a 4x canvas).
    pub fn scaled(mut self, k: f32) -> Sketch {
        self.xf = self.xf.pre_scale(k, k);
        self
    }

    fn fill(&mut self, path: Option<Path>, p: Option<Pal>) {
        if let Some(path) = path {
            self.pix.fill_path(&path, &paint(p), FillRule::Winding, self.xf, None);
        }
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, p: Pal) {
        self.fill(Rect::from_xywh(cx - rx, cy - ry, rx * 2.0, ry * 2.0).and_then(PathBuilder::from_oval), Some(p));
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, p: Pal) {
        self.fill(Rect::from_xywh(x, y, w, h).map(PathBuilder::from_rect), Some(p));
    }

    pub fn poly(&mut self, pts: &[(f32, f32)], p: Pal) {
        let mut pb = PathBuilder::new();
        pb.move_to(pts[0].0, pts[0].1);
        for &(x, y) in &pts[1..] {
            pb.line_to(x, y);
        }
        pb.close();
        self.fill(pb.finish(), Some(p));
    }

    /// A filled shape built from curves.
    pub fn shape(&mut self, build: impl FnOnce(&mut PathBuilder), p: Pal) {
        let mut pb = PathBuilder::new();
        build(&mut pb);
        pb.close();
        self.fill(pb.finish(), Some(p));
    }

    /// A thick line through `pts` with round ends.
    pub fn line(&mut self, pts: &[(f32, f32)], width: f32, p: Pal) {
        let mut pb = PathBuilder::new();
        pb.move_to(pts[0].0, pts[0].1);
        for &(x, y) in &pts[1..] {
            pb.line_to(x, y);
        }
        let stroke = Stroke { width, line_cap: LineCap::Round, line_join: LineJoin::Round, ..Stroke::default() };
        if let Some(path) = pb.finish() {
            self.pix.stroke_path(&path, &paint(Some(p)), &stroke, self.xf, None);
        }
    }

    /// The drawing as palette pixels. Anything not fully see-through snaps to its nearest entry.
    pub fn to_img(&self) -> Img {
        let (w, h) = (self.pix.width() as usize, self.pix.height() as usize);
        let mut img = Img::new(w, h);
        for (i, c) in self.pix.pixels().iter().enumerate() {
            if c.alpha() >= 128 {
                let c = c.demultiply();
                img.px[i] = Some(Pal::nearest((c.red(), c.green(), c.blue())));
            }
        }
        img
    }
}
