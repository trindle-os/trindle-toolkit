//! Letters that are pictures (emoji): bitmaps the font keeps per size (CBDT, sbix) or coloured layers (COLR).

use skrifa::bitmap::{BitmapData, BitmapStrikes, Origin};
use skrifa::color::{Brush, ColorPainter, ColorStop, CompositeMode, Extend, Transform as FontTransform};
use skrifa::instance::{Location, LocationRef, Size};
use skrifa::raw::types::{BoundingBox, Tag};
use skrifa::raw::TableProvider;
use skrifa::{FontRef, GlyphId, MetadataProvider};
use tiny_skia::{
    BlendMode, Color, FillRule, FilterQuality, GradientStop, LinearGradient, Mask, Paint, Path, PathBuilder,
    Pixmap, PixmapPaint, Point, RadialGradient, Rect, Shader, SpreadMode, SweepGradient, Transform,
};

use super::colour::Colour;
use super::text::outline;

pub struct Picture {
    pub pixmap: Pixmap,
    pub left: f32,
    pub top: f32,
}

pub fn has_pictures(font: &FontRef) -> bool {
    [b"COLR", b"CBDT", b"sbix"].into_iter().any(|t| font.table_data(Tag::new(t)).is_some())
}

pub fn picture(font: &FontRef, at: &Location, glyph: GlyphId, size: f32, ink: Colour) -> Option<Picture> {
    if font.color_glyphs().get(glyph).is_some() {
        return layers(font, at, glyph, size, ink);
    }
    bitmap(font, glyph, size)
}

fn bitmap(font: &FontRef, glyph: GlyphId, size: f32) -> Option<Picture> {
    let image = BitmapStrikes::new(font).glyph_for_size(Size::new(size.round().clamp(1.0, u16::MAX as f32)), glyph)?;
    let BitmapData::Png(data) = image.data else { return None };
    let mut source = crate::picture::to_pixmap(&image::load_from_memory_with_format(data, image::ImageFormat::Png).ok()?)?;
    let s = size / image.ppem_y.max(1.0);
    let (left, top) = (image.inner_bearing_x * s, image.inner_bearing_y * s);
    let outer = size / font.head().ok()?.units_per_em() as f32;
    let (left, top) = (image.bearing_x * outer + left, image.bearing_y * outer + top);
    let top = match image.placement_origin {
        Origin::TopLeft => -top,
        Origin::BottomLeft => -(top + source.height() as f32 * s),
    };
    let mut scale = 1.0;
    while s / scale < 0.5 {
        source = scaled(&source, 0.5)?;
        scale *= 0.5;
    }
    let pixmap = scaled(&source, s / scale)?;
    Some(Picture { pixmap, left, top })
}

fn scaled(source: &Pixmap, s: f32) -> Option<Pixmap> {
    let w = (source.width() as f32 * s).ceil().max(1.0) as u32;
    let h = (source.height() as f32 * s).ceil().max(1.0) as u32;
    if w > MOST_SIDE || h > MOST_SIDE {
        return None;
    }
    let mut out = Pixmap::new(w, h)?;
    let paint = PixmapPaint { quality: FilterQuality::Bicubic, ..PixmapPaint::default() };
    out.draw_pixmap(0, 0, source.as_ref(), &paint, Transform::from_scale(s, s), None);
    Some(out)
}

const MOST_SIDE: u32 = 2048;

fn layers(font: &FontRef, at: &Location, glyph: GlyphId, size: f32, ink: Colour) -> Option<Picture> {
    let colour_glyph = font.color_glyphs().get(glyph)?;
    let s = size / font.head().ok()?.units_per_em() as f32;
    let mut reach = Reach { font, location: at, transforms: vec![Transform::from_scale(s, -s)], bounds: None };
    colour_glyph.paint(LocationRef::from(at), &mut reach).ok()?;
    let bounds = reach.bounds?;
    let (left, top) = (bounds.left().floor(), bounds.top().floor());
    let w = (bounds.right().ceil() - left).max(1.0) as u32;
    let h = (bounds.bottom().ceil() - top).max(1.0) as u32;
    if w > MOST_SIDE || h > MOST_SIDE {
        return None;
    }
    let base = Transform::from_row(s, 0.0, 0.0, -s, -left, -top);
    let palettes = font.color_palettes();
    let mut painter = Painter {
        font,
        location: at,
        palette: palettes.get(0).map(|p| p.colors().iter().map(|c| Color::from_rgba8(c.red, c.green, c.blue, c.alpha)).collect()).unwrap_or_default(),
        ink: ink.to_skia(),
        w,
        h,
        transforms: vec![base],
        clips: vec![None],
        layers: vec![(Pixmap::new(w, h)?, CompositeMode::SrcOver)],
    };
    colour_glyph.paint(LocationRef::from(at), &mut painter).ok()?;
    let pixmap = painter.layers.pop()?.0;
    Some(Picture { pixmap, left, top })
}

fn clip_box(b: BoundingBox<f32>) -> Option<Path> {
    Rect::from_ltrb(b.x_min, b.y_min, b.x_max, b.y_max).map(PathBuilder::from_rect)
}

fn skia(t: FontTransform) -> Transform {
    Transform::from_row(t.xx, t.yx, t.xy, t.yy, t.dx, t.dy)
}

struct Reach<'f, 'a> {
    font: &'f FontRef<'a>,
    location: &'f Location,
    transforms: Vec<Transform>,
    bounds: Option<Rect>,
}

impl Reach<'_, '_> {
    fn add(&mut self, path: Option<Path>) {
        let at = *self.transforms.last().unwrap_or(&Transform::identity());
        let Some(b) = path.and_then(|p| p.transform(at)).map(|p| p.bounds()) else { return };
        self.bounds = Some(match self.bounds {
            Some(r) => Rect::from_ltrb(r.left().min(b.left()), r.top().min(b.top()),
                r.right().max(b.right()), r.bottom().max(b.bottom())).unwrap_or(r),
            None => b,
        });
    }
}

impl ColorPainter for Reach<'_, '_> {
    fn push_clip_glyph(&mut self, glyph: GlyphId) {
        self.add(outline(self.font, self.location, glyph));
    }
    fn fill(&mut self, _: Brush<'_>) {}
    fn push_clip_box(&mut self, b: BoundingBox<f32>) {
        self.add(clip_box(b));
    }
    fn pop_clip(&mut self) {}
    fn push_layer(&mut self, _: CompositeMode) {}
    fn push_transform(&mut self, t: FontTransform) {
        let top = *self.transforms.last().unwrap_or(&Transform::identity());
        self.transforms.push(top.pre_concat(skia(t)));
    }
    fn pop_transform(&mut self) {
        self.transforms.pop();
    }
}

struct Painter<'f, 'a> {
    font: &'f FontRef<'a>,
    location: &'f Location,
    palette: Vec<Color>,
    ink: Color,
    w: u32,
    h: u32,
    transforms: Vec<Transform>,
    clips: Vec<Option<Mask>>,
    layers: Vec<(Pixmap, CompositeMode)>,
}

impl Painter<'_, '_> {
    fn at(&self) -> Transform {
        *self.transforms.last().unwrap_or(&Transform::identity())
    }

    fn narrowed(&self, path: &Path, at: Transform) -> Option<Mask> {
        let mut mask = match self.clips.last() {
            Some(Some(m)) => m.clone(),
            _ => {
                let mut m = Mask::new(self.w, self.h)?;
                m.fill_path(path, FillRule::Winding, true, at);
                return Some(m);
            }
        };
        mask.intersect_path(path, FillRule::Winding, true, at);
        Some(mask)
    }

    fn colour(&self, index: u16, alpha: f32) -> Color {
        let mut c = if index == u16::MAX { self.ink } else { self.palette.get(index as usize).copied().unwrap_or(Color::BLACK) };
        c.apply_opacity(alpha);
        c
    }

    fn shader(&self, brush: Brush<'_>) -> Option<Shader<'static>> {
        let at = self.at();
        let stops = |list: &[ColorStop]| list.iter().map(|s| GradientStop::new(s.offset, self.colour(s.palette_index, s.alpha))).collect::<Vec<_>>();
        let spread = |e: Extend| match e {
            Extend::Repeat => SpreadMode::Repeat,
            Extend::Reflect => SpreadMode::Reflect,
            _ => SpreadMode::Pad,
        };
        match brush {
            Brush::Solid { palette_index, alpha } => Some(Shader::SolidColor(self.colour(palette_index, alpha))),
            Brush::LinearGradient { p0, p1, color_stops, extend } => LinearGradient::new(Point::from_xy(p0.x, p0.y), Point::from_xy(p1.x, p1.y),
                stops(color_stops), spread(extend), at),
            Brush::RadialGradient { c0, r0, c1, r1, color_stops, extend } => RadialGradient::new(Point::from_xy(c0.x, c0.y), r0,
                Point::from_xy(c1.x, c1.y), r1, stops(color_stops), spread(extend), at),
            Brush::SweepGradient { c0, start_angle, end_angle, color_stops, extend } => SweepGradient::new(Point::from_xy(c0.x, -c0.y),
                start_angle, end_angle, stops(color_stops), spread(extend), at.pre_scale(1.0, -1.0)),
        }
    }
}

fn blend(mode: CompositeMode) -> BlendMode {
    use CompositeMode as C;
    match mode {
        C::Clear => BlendMode::Clear,
        C::Src => BlendMode::Source,
        C::Dest => BlendMode::Destination,
        C::DestOver => BlendMode::DestinationOver,
        C::SrcIn => BlendMode::SourceIn,
        C::DestIn => BlendMode::DestinationIn,
        C::SrcOut => BlendMode::SourceOut,
        C::DestOut => BlendMode::DestinationOut,
        C::SrcAtop => BlendMode::SourceAtop,
        C::DestAtop => BlendMode::DestinationAtop,
        C::Xor => BlendMode::Xor,
        C::Plus => BlendMode::Plus,
        C::Screen => BlendMode::Screen,
        C::Overlay => BlendMode::Overlay,
        C::Darken => BlendMode::Darken,
        C::Lighten => BlendMode::Lighten,
        C::ColorDodge => BlendMode::ColorDodge,
        C::ColorBurn => BlendMode::ColorBurn,
        C::HardLight => BlendMode::HardLight,
        C::SoftLight => BlendMode::SoftLight,
        C::Difference => BlendMode::Difference,
        C::Exclusion => BlendMode::Exclusion,
        C::Multiply => BlendMode::Multiply,
        C::HslHue => BlendMode::Hue,
        C::HslSaturation => BlendMode::Saturation,
        C::HslColor => BlendMode::Color,
        C::HslLuminosity => BlendMode::Luminosity,
        _ => BlendMode::SourceOver,
    }
}

impl ColorPainter for Painter<'_, '_> {
    fn fill(&mut self, brush: Brush<'_>) {
        let mask = self.clips.last().cloned().flatten();
        let Some(shader) = self.shader(brush) else { return };
        let fill = Paint { shader, anti_alias: true, ..Paint::default() };
        let Some(area) = Rect::from_xywh(0.0, 0.0, self.w as f32, self.h as f32) else { return };
        if let Some((target, _)) = self.layers.last_mut() {
            target.fill_rect(area, &fill, Transform::identity(), mask.as_ref());
        }
    }

    fn push_clip_glyph(&mut self, glyph: GlyphId) {
        let at = self.at();
        let mask = outline(self.font, self.location, glyph).and_then(|p| self.narrowed(&p, at));
        self.clips.push(mask.or_else(|| Mask::new(self.w, self.h)));
    }

    fn push_clip_box(&mut self, b: BoundingBox<f32>) {
        let at = self.at();
        let mask = clip_box(b).and_then(|p| self.narrowed(&p, at));
        self.clips.push(mask.or_else(|| Mask::new(self.w, self.h)));
    }

    fn pop_clip(&mut self) {
        if self.clips.len() > 1 {
            self.clips.pop();
        }
    }

    fn push_layer(&mut self, mode: CompositeMode) {
        if let Some(p) = Pixmap::new(self.w, self.h) {
            self.layers.push((p, mode));
        }
    }

    fn pop_layer(&mut self) {
        if self.layers.len() < 2 {
            return;
        }
        let Some((layer, mode)) = self.layers.pop() else { return };
        if let Some((target, _)) = self.layers.last_mut() {
            let paint = PixmapPaint { blend_mode: blend(mode), ..PixmapPaint::default() };
            target.draw_pixmap(0, 0, layer.as_ref(), &paint, Transform::identity(), None);
        }
    }

    fn push_transform(&mut self, t: FontTransform) {
        let top = self.at();
        self.transforms.push(top.pre_concat(skia(t)));
    }

    fn pop_transform(&mut self) {
        if self.transforms.len() > 1 {
            self.transforms.pop();
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn test_font(name: &str) -> Option<Vec<u8>> {
        let path = crate::common::source::toolkit_root().join("target/emoji").join(name);
        let how = if name == NOTO {
            "install Debian's fonts-noto-color-emoji (sudo apt install fonts-noto-color-emoji)"
        } else {
            "copy the Twemoji Mozilla font file (TwemojiMozilla.ttf) to this path"
        };
        match std::fs::read(&path) {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!("SKIPPED: the emoji font {} could not be read ({e}), and this test checks nothing without it: {how}", path.display());
                None
            }
        }
    }

    pub(crate) const NOTO: &str = "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf";
    const TWEMOJI: &str = "TwemojiMozilla.ttf";

    pub(crate) fn coloured(p: &Pixmap) -> usize {
        p.pixels().iter().filter(|c| {
            let c = c.demultiply();
            let (hi, lo) = (c.red().max(c.green()).max(c.blue()), c.red().min(c.green()).min(c.blue()));
            c.alpha() > 128 && hi - lo > 60
        }).count()
    }

    fn check(data: &[u8]) {
        let font = FontRef::from_index(data, 0).unwrap();
        let at = Location::default();
        assert!(has_pictures(&font));
        let glyph = font.charmap().map('\u{1F600}').unwrap();
        for size in [16.0, 32.0, 120.0] {
            let p = picture(&font, &at, glyph, size, Colour::rgb(1.0, 1.0, 1.0)).unwrap();
            let (w, h) = (p.pixmap.width() as f32, p.pixmap.height() as f32);
            assert!(w > size * 0.8 && w < size * 1.5 && h > size * 0.8 && h < size * 1.5, "{size}: {w}x{h}");
            assert!(p.top < -size * 0.6 && p.top + h > 0.0, "{size}: top {}", p.top);
            assert!(coloured(&p.pixmap) as f32 > w * h * 0.3, "{size}");
        }
        assert!(picture(&font, &at, font.charmap().map(' ').unwrap_or_default(), 16.0, Colour::rgb(1.0, 1.0, 1.0)).is_none());
    }

    fn at(p: &Picture, x: f32, y: f32) -> (u8, u8, u8) {
        let (px, py) = ((p.pixmap.width() as f32 * x) as u32, (p.pixmap.height() as f32 * y) as u32);
        let c = p.pixmap.pixel(px, py).unwrap().demultiply();
        (c.green(), c.red(), c.blue())
    }

    #[test]
    fn layers_are_drawn_as_a_browser_draws_them() {
        let font = FontRef::from_index(include_bytes!("../tests/fonts/test_glyphs-glyf_colr_1.ttf"), 0).unwrap();
        let at_rest = Location::default();
        assert!(has_pictures(&font));
        let draw = |glyph: u32, ink: Colour| picture(&font, &at_rest, GlyphId::new(glyph), 64.0, ink);
        let white = Colour::rgb(1.0, 1.0, 1.0);
        let sweep = draw(15, white).unwrap();
        let (above, below) = (at(&sweep, 0.62, 0.3), at(&sweep, 0.62, 0.5));
        assert!(above.1 > 200 && above.0 < 60 && below.0 > 200, "{above:?} {below:?}");
        let stripes = draw(9, white).unwrap();
        let reds: Vec<bool> = (1..40).map(|i| { let (_, r, b) = at(&stripes, i as f32 / 40.0, 0.5); r > b }).collect();
        assert!(reds.windows(2).filter(|w| w[0] != w[1]).count() >= 3, "{reds:?}");
        let own = draw(148, Colour::rgb(0.0, 1.0, 0.0)).unwrap();
        assert!(at(&own, 0.5, 0.5).0 > 200, "{:?}", at(&own, 0.5, 0.5));
        assert!(draw(194, white).unwrap().pixmap.pixels().iter().all(|c| c.alpha() == 0));
        assert!(draw(178, white).is_none());
    }

    #[test]
    fn a_picture_emoji_comes_out_in_colour_at_the_size_asked() {
        if let Some(font) = test_font(NOTO) {
            check(&font);
        }
    }

    #[test]
    fn a_layered_emoji_comes_out_in_colour_at_the_size_asked() {
        if let Some(font) = test_font(TWEMOJI) {
            check(&font);
        }
    }
}
