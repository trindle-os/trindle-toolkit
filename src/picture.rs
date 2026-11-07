//! Pictures from files: read, turned upright, made ready to paint and drawn at any size.

use std::path::Path;
use std::rc::Rc;

use image::{DynamicImage, GenericImageView, ImageDecoder, ImageReader};
use tiny_skia::{FilterQuality, Pixmap, PixmapMut, PixmapPaint, Transform};

use crate::paint::Painter;

pub fn file(path: &Path) -> std::io::Result<std::fs::File> {
    crate::common::files::open_file(path)
}

pub fn reader(path: &Path) -> std::io::Result<ImageReader<std::io::BufReader<std::fs::File>>> {
    let mut reader = ImageReader::new(std::io::BufReader::new(file(path)?)).with_guessed_format()?;
    if reader.format().is_none() {
        if let Ok(kind) = image::ImageFormat::from_path(path) {
            reader.set_format(kind);
        }
    }
    Ok(reader)
}

pub fn turned(img: &DynamicImage, turns: u8) -> DynamicImage {
    match turns % 4 {
        1 => img.rotate90(),
        2 => img.rotate180(),
        3 => img.rotate270(),
        _ => img.clone(),
    }
}

pub fn cropped(img: &DynamicImage, r: crate::widget::Rect) -> DynamicImage {
    let (iw, ih) = img.dimensions();
    let (x, y) = (r.x.floor().clamp(0.0, iw.saturating_sub(1) as f32) as u32, r.y.floor().clamp(0.0, ih.saturating_sub(1) as f32) as u32);
    let (right, bottom) = ((r.x + r.w).ceil().clamp(0.0, iw as f32) as u32, (r.y + r.h).ceil().clamp(0.0, ih as f32) as u32);
    img.crop_imm(x, y, right.saturating_sub(x).max(1), bottom.saturating_sub(y).max(1))
}

pub fn load(path: &Path) -> Option<DynamicImage> {
    load_within(path, image::Limits::default())
}

pub fn load_within(path: &Path, limits: image::Limits) -> Option<DynamicImage> {
    let read = || -> Result<DynamicImage, String> {
        let mut decoder = reader(path).map_err(|e| e.to_string())?.into_decoder().map_err(|e| e.to_string())?;
        decoder.set_limits(limits).map_err(|e| e.to_string())?;
        let turn = decoder.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
        let mut img = DynamicImage::from_decoder(decoder).map_err(|e| e.to_string())?;
        img.apply_orientation(turn);
        Ok(img)
    };
    read().map_err(|e| eprintln!("trindle: {}: {e}", path.display())).ok()
}

pub const SVG_MOST: u64 = 16 << 20;

fn squeezed(data: &[u8]) -> bool {
    data.starts_with(&[0x1f, 0x8b])
}

pub fn svg_bytes(path: &Path) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut data = Vec::new();
    file(path).ok()?.take(SVG_MOST + 1).read_to_end(&mut data).ok()?;
    (data.len() as u64 <= SVG_MOST).then_some(data)
}

pub type SvgTree = resvg::usvg::Tree;

pub fn svg_tree(data: &[u8]) -> Option<SvgTree> {
    use resvg::usvg::{ImageHrefResolver, Options, Tree};
    use std::io::Read;
    let mut plain = Vec::new();
    let data = if squeezed(data) {
        flate2::read::GzDecoder::new(data).take(SVG_MOST + 1).read_to_end(&mut plain).ok()?;
        &plain[..]
    } else {
        data
    };
    if data.len() as u64 > SVG_MOST || squeezed(data) {
        return None;
    }
    let carried = ImageHrefResolver::default_data_resolver();
    let follows_nothing = ImageHrefResolver {
        resolve_data: Box::new(move |kind, bytes, opt| if squeezed(&bytes) { None } else { carried(kind, bytes, opt) }),
        resolve_string: Box::new(|_, _| None),
    };
    let opt = Options { resources_dir: None, image_href_resolver: follows_nothing, ..Options::default() };
    Tree::from_data(data, &opt).ok()
}

pub fn svg(data: &[u8], most: u32) -> Option<DynamicImage> {
    let tree = svg_tree(data)?;
    let s = tree.size();
    let k = (most as f32 / s.width().max(s.height())).clamp(f32::MIN_POSITIVE, 1.0);
    let (w, h) = ((s.width() * k).round().max(1.0) as u32, (s.height() * k).round().max(1.0) as u32);
    let mut px = Pixmap::new(w, h)?;
    resvg::render(&tree, Transform::from_scale(k, k), &mut px.as_mut());
    let rgba = px.pixels().iter().flat_map(|p| {
        let c = p.demultiply();
        [c.red(), c.green(), c.blue(), c.alpha()]
    }).collect();
    image::RgbaImage::from_raw(w, h, rgba).map(DynamicImage::ImageRgba8)
}

pub fn from_rgb(w: u32, h: u32, rgb: Vec<u8>) -> Option<DynamicImage> {
    image::RgbImage::from_raw(w, h, rgb).map(DynamicImage::ImageRgb8)
}

pub fn draw(canvas: &mut PixmapMut, img: &DynamicImage, x: i32, y: i32, w: u32, h: u32) -> Option<()> {
    draw_prepared(canvas, &Prepared::new(img, w, h)?, x, y);
    Some(())
}

#[allow(dead_code)]
pub(crate) fn draw_on(p: &mut Painter, img: &DynamicImage, x: i32, y: i32, w: u32, h: u32) -> Option<()> {
    draw_prepared_on(p, &Prepared::new(img, w, h)?, x, y);
    Some(())
}

#[derive(Clone)]
pub struct Prepared {
    px: Rc<Pixmap>,
    to: (u32, u32),
    grown: bool,
    id: u64,
}

impl Prepared {
    pub fn new(img: &DynamicImage, w: u32, h: u32) -> Option<Prepared> {
        let (iw, ih) = img.dimensions();
        let to = (w.max(1), h.max(1));
        let (px, grown) = if to == (iw, ih) {
            (to_pixmap(img)?, false)
        } else if to.0 <= iw && to.1 <= ih {
            (to_pixmap(&img.thumbnail_exact(to.0, to.1))?, false)
        } else {
            (to_pixmap(img)?, true)
        };
        Some(Prepared { px: Rc::new(px), to, grown, id: crate::paint::new_id() })
    }
}

pub fn draw_prepared(canvas: &mut PixmapMut, prepared: &Prepared, x: i32, y: i32) {
    crate::paint::played(canvas, &crate::shadow::Shadows::default(), |p| draw_prepared_on(p, prepared, x, y));
}

pub(crate) fn draw_prepared_on(p: &mut Painter, prepared: &Prepared, x: i32, y: i32) {
    if !prepared.grown {
        p.image(prepared.id, prepared.px.clone(), x, y);
        return;
    }
    let Some(at) = tiny_skia::IntRect::from_xywh(x, y, prepared.to.0, prepared.to.1) else { return };
    let px = prepared.px.clone();
    p.pixels(at, move |c| {
        let smooth = PixmapPaint { quality: FilterQuality::Bicubic, ..PixmapPaint::default() };
        let (iw, ih) = (px.width() as f32, px.height() as f32);
        let grow = Transform::from_row(at.width() as f32 / iw, 0.0, 0.0, at.height() as f32 / ih, x as f32, y as f32);
        c.draw_pixmap(0, 0, px.as_ref().as_ref(), &smooth, grow, None);
    });
}

thread_local! {
    static SHARED: std::cell::RefCell<Vec<(std::rc::Weak<DynamicImage>, bool, Prepared)>> = const { std::cell::RefCell::new(Vec::new()) };
}

const SIZES_KEPT: usize = 4;

pub(crate) fn shared(img: &Rc<DynamicImage>, square: bool, w: u32, h: u32) -> Option<Prepared> {
    SHARED.with_borrow_mut(|kept| {
        let to = (w.max(1), h.max(1));
        let ours = |e: &(std::rc::Weak<DynamicImage>, bool, Prepared)| std::ptr::eq(e.0.as_ptr(), Rc::as_ptr(img)) && e.1 == square;
        if let Some(e) = kept.iter().find(|e| ours(e) && e.2.to == to) {
            return Some(e.2.clone());
        }
        let ready = if square {
            let (iw, ih) = img.dimensions();
            let side = iw.min(ih);
            Prepared::new(&img.crop_imm((iw - side) / 2, (ih - side) / 2, side, side), w, h)?
        } else {
            Prepared::new(img, w, h)?
        };
        kept.retain(|e| e.0.strong_count() > 0);
        if kept.iter().filter(|e| ours(e)).count() >= SIZES_KEPT {
            let oldest = kept.iter().position(ours)?;
            kept.remove(oldest);
        }
        kept.push((Rc::downgrade(img), square, ready.clone()));
        Some(ready)
    })
}

#[derive(Default)]
pub struct Drawn(std::cell::RefCell<Option<Prepared>>);

impl Drawn {
    pub fn draw(&self, canvas: &mut PixmapMut, img: &DynamicImage, x: i32, y: i32, w: u32, h: u32) {
        crate::paint::played(canvas, &crate::shadow::Shadows::default(), |p| self.draw_on(p, img, x, y, w, h));
    }

    pub(crate) fn draw_on(&self, p: &mut Painter, img: &DynamicImage, x: i32, y: i32, w: u32, h: u32) {
        let mut kept = self.0.borrow_mut();
        let to = (w.max(1), h.max(1));
        if kept.as_ref().is_none_or(|p| p.to != to) {
            *kept = Prepared::new(img, w, h);
        }
        if let Some(prepared) = kept.as_ref() {
            draw_prepared_on(p, prepared, x, y);
        }
    }
}

pub fn to_pixmap(img: &DynamicImage) -> Option<Pixmap> {
    let (w, h) = img.dimensions();
    let mut px = Pixmap::new(w, h)?;
    if let DynamicImage::ImageRgb8(rgb) = img {
        for (dst, src) in px.data_mut().chunks_exact_mut(4).zip(rgb.as_raw().chunks_exact(3)) {
            dst.copy_from_slice(&[src[0], src[1], src[2], 255]);
        }
        return Some(px);
    }
    let converted;
    let raw = match img {
        DynamicImage::ImageRgba8(rgba) => rgba.as_raw(),
        _ => {
            converted = img.to_rgba8();
            converted.as_raw()
        }
    };
    for (dst, src) in px.data_mut().chunks_exact_mut(4).zip(raw.chunks_exact(4)) {
        let a = src[3] as u16;
        let weigh = |c: u8| ((c as u16 * a + 127) / 255) as u8;
        dst.copy_from_slice(&[weigh(src[0]), weigh(src[1]), weigh(src[2]), src[3]]);
    }
    Some(px)
}

pub fn from_argb32(w: u32, h: u32, data: &[u8]) -> Option<Pixmap> {
    let n = (w as usize).checked_mul(h as usize)?.checked_mul(4)?;
    if data.len() < n {
        return None;
    }
    let mut px = Pixmap::new(w, h)?;
    for (dst, src) in px.data_mut().chunks_exact_mut(4).zip(data[..n].chunks_exact(4)) {
        let a = src[0] as u16;
        let weigh = |c: u8| ((c as u16 * a + 127) / 255) as u8;
        dst.copy_from_slice(&[weigh(src[1]), weigh(src[2]), weigh(src[3]), src[0]]);
    }
    Some(px)
}

pub fn from_xpm(text: &str) -> Option<Pixmap> {
    let mut lines = quoted(text).into_iter();
    let head: Vec<usize> = lines.next()?.split_whitespace().take(4).map(|n| n.parse().ok()).collect::<Option<_>>()?;
    let [w, h, count, cpp] = head[..] else { return None };
    if w == 0 || h == 0 || w > 1024 || h > 1024 || cpp == 0 || cpp > 4 {
        return None;
    }
    let mut colours = std::collections::HashMap::new();
    for line in lines.by_ref().take(count) {
        let key = line.get(..cpp)?.to_string();
        let words: Vec<&str> = line[cpp..].split_whitespace().collect();
        let value = words.iter().position(|w| *w == "c").or_else(|| words.iter().position(|w| *w == "m"))
            .map(|i| words[i + 1..].iter().take_while(|w| !matches!(**w, "m" | "s" | "g" | "g4")).copied().collect::<Vec<_>>().join(" "))
            .unwrap_or_default();
        colours.insert(key, xpm_colour(&value));
    }
    let mut px = Pixmap::new(w as u32, h as u32)?;
    for (y, row) in lines.take(h).enumerate() {
        for x in 0..w {
            let Some(key) = row.get(x * cpp..(x + 1) * cpp) else { break };
            let [r, g, b, a] = colours.get(key).copied().unwrap_or([0; 4]);
            let at = (y * w + x) * 4;
            px.data_mut()[at..at + 4].copy_from_slice(&[r, g, b, a]);
        }
    }
    Some(px)
}

fn quoted(text: &str) -> Vec<&str> {
    text.split('"').skip(1).step_by(2).collect()
}

fn xpm_colour(value: &str) -> [u8; 4] {
    if value.eq_ignore_ascii_case("none") {
        return [0; 4];
    }
    let Some(hex) = value.strip_prefix('#') else { return [0, 0, 0, 255] };
    let step = hex.len() / 3;
    if step == 0 || hex.len() % 3 != 0 || !hex.is_ascii() {
        return [0, 0, 0, 255];
    }
    let part = |i: usize| {
        let d = &hex[i * step..i * step + step.min(2)];
        u8::from_str_radix(d, 16).map(|v| if step == 1 { v * 17 } else { v }).unwrap_or(0)
    };
    [part(0), part(1), part(2), 255]
}

#[cfg(any(test, feature = "testing"))]
pub mod strangers {
    use std::path::Path;

    pub fn blue(body: &str) -> String {
        format!("<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"8\" height=\"8\">\
                 <rect width=\"8\" height=\"8\" fill=\"#0000ff\"/>{body}</svg>")
    }

    pub fn red() -> String {
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"8\" height=\"8\"><rect id=\"r\" width=\"8\" height=\"8\" fill=\"#ff0000\"/></svg>".to_string()
    }

    pub fn carried(svg: &[u8]) -> String {
        let coded: String = svg.iter().map(|&b| if b.is_ascii_alphanumeric() { (b as char).to_string() } else { format!("%{b:02X}") }).collect();
        format!("data:image/svg+xml,{coded}")
    }

    pub fn squeezed(text: &[u8]) -> Vec<u8> {
        use std::io::Write;
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        gz.write_all(text).unwrap();
        gz.finish().unwrap()
    }

    pub fn naming(other: &Path) -> Vec<(&'static str, Vec<u8>)> {
        let whole = other.to_string_lossy();
        let bare = other.file_name().unwrap().to_string_lossy();
        let image = |href: &str| format!("<image href=\"{href}\" width=\"8\" height=\"8\"/>");
        let inner = blue(&image(&whole));
        let ways: Vec<(&'static str, String)> = vec![
            ("a picture by its whole name", image(&whole)),
            ("a picture by the older word", format!("<image xlink:href=\"{whole}\" width=\"8\" height=\"8\"/>")),
            ("a picture by its address", image(&format!("file://{whole}"))),
            ("a picture by its bare name", image(&bare)),
            ("a picture in a filter", format!("<filter id=\"f\" x=\"0\" y=\"0\" width=\"1\" height=\"1\"><feImage href=\"{whole}\"/></filter>\
                                               <rect width=\"8\" height=\"8\" filter=\"url(#f)\"/>")),
            ("a picture named by one carried inside", image(&carried(inner.as_bytes()))),
            ("a shape from another file", format!("<use href=\"{whole}#r\"/><use xlink:href=\"{bare}#r\"/>")),
        ];
        let mut all: Vec<(&'static str, Vec<u8>)> = ways.into_iter().map(|(what, body)| (what, blue(&body).into_bytes())).collect();
        all.push(("a squeezed SVG naming a picture", squeezed(blue(&image(&whole)).as_bytes())));
        all
    }

    pub fn red_and_blue(img: &image::DynamicImage) -> (bool, bool) {
        let rgba = img.to_rgba8();
        (rgba.pixels().any(|p| p[0] > 200 && p[1] < 60 && p[2] < 60 && p[3] > 200), rgba.pixels().any(|p| p[2] > 200 && p[0] < 60 && p[1] < 60 && p[3] > 200))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn a_picture_turns_by_quarters_and_crops_to_a_box() {
        let mut img = RgbaImage::from_pixel(4, 2, image::Rgba([0, 0, 0, 255]));
        img.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
        let img = DynamicImage::ImageRgba8(img);
        let right = turned(&img, 1);
        assert_eq!(right.dimensions(), (2, 4));
        assert_eq!(right.to_rgba8().get_pixel(1, 0).0, [255, 0, 0, 255], "the top left goes to the top right");
        assert_eq!(turned(&img, 2).to_rgba8().get_pixel(3, 1).0, [255, 0, 0, 255]);
        assert_eq!(turned(&turned(&img, 3), 1).to_rgba8(), img.to_rgba8());
        assert_eq!(turned(&img, 4).to_rgba8(), img.to_rgba8());
        let part = cropped(&img, crate::widget::Rect { x: 0.5, y: 0.0, w: 2.0, h: 9.0 });
        assert_eq!(part.dimensions(), (3, 2), "rounded out to whole pixels and kept inside");
        assert_eq!(cropped(&img, crate::widget::Rect { x: 99.0, y: 99.0, w: 5.0, h: 5.0 }).dimensions(), (1, 1), "never empty");
    }

    #[test]
    fn a_kept_picture_draws_as_a_fresh_one_does_and_follows_a_new_size() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(8, 8, image::Rgba([200, 0, 0, 255])));
        let drawn = Drawn::default();
        for (w, h) in [(4, 4), (4, 4), (16, 16)] {
            let mut once = Pixmap::new(16, 16).unwrap();
            let mut again = Pixmap::new(16, 16).unwrap();
            drawn.draw(&mut again.as_mut(), &img, 0, 0, w, h);
            draw(&mut once.as_mut(), &img, 0, 0, w, h);
            assert_eq!(once.data(), again.data(), "{w} by {h}");
            assert_eq!(drawn.0.borrow().as_ref().map(|p| p.to), Some((w, h)));
        }
    }

    #[test]
    fn a_recorded_picture_plays_as_it_was_drawn_straight_and_keeps_its_id() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_fn(8, 6, |x, y| image::Rgba([30 * x as u8, 40 * y as u8, 90, 100 + 20 * x as u8])));
        for (w, h) in [(5, 4), (8, 6), (19, 13)] {
            let prepared = Prepared::new(&img, w, h).unwrap();
            let (mut played, mut direct) = (Pixmap::new(24, 20).unwrap(), Pixmap::new(24, 20).unwrap());
            let mut p = Painter::default();
            draw_prepared_on(&mut p, &prepared, 2, 3);
            crate::paint::cpu::play(p.ops, &mut played.as_mut(), &crate::shadow::Shadows::default());
            if prepared.grown {
                let smooth = PixmapPaint { quality: FilterQuality::Bicubic, ..PixmapPaint::default() };
                let grow = Transform::from_row(w as f32 / 8.0, 0.0, 0.0, h as f32 / 6.0, 2.0, 3.0);
                direct.draw_pixmap(0, 0, prepared.px.as_ref().as_ref(), &smooth, grow, None);
            } else {
                direct.draw_pixmap(2, 3, prepared.px.as_ref().as_ref(), &PixmapPaint::default(), Transform::identity(), None);
            }
            assert!(direct.data().iter().any(|&b| b > 0));
            assert!(played.data() == direct.data(), "{w} by {h}");
        }
        let kept = Drawn::default();
        let ids: Vec<u64> = (0..2).flat_map(|_| {
            let mut p = Painter::default();
            kept.draw_on(&mut p, &img, 0, 0, 4, 3);
            p.ops.into_iter().filter_map(|op| match op { crate::paint::Op::Image { id, .. } => Some(id), _ => None })
        }).collect();
        assert!(ids.len() == 2 && ids[0] == ids[1]);
    }

    #[test]
    fn nothing_an_svg_names_is_followed() {
        use strangers::{blue, carried, naming, red, red_and_blue, squeezed as squeeze};
        let dir = std::env::temp_dir().join(format!("trindle-picture-names-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let other = dir.join("other.svg");
        std::fs::write(&other, red()).unwrap();
        let inside = blue(&format!("<image href=\"{}\" width=\"8\" height=\"8\"/>", carried(red().as_bytes())));
        assert_eq!(red_and_blue(&svg(inside.as_bytes(), 8).unwrap()), (true, false), "a picture carried inside is drawn");
        for (what, data) in naming(&other) {
            let drawn = svg(&data, 8).unwrap_or_else(|| panic!("{what}: not drawn"));
            assert_eq!(red_and_blue(&drawn), (false, true), "{what}: what it names was followed");
        }
        let pipe = dir.join("pipe.png");
        let c = std::ffi::CString::new(pipe.to_str().unwrap()).unwrap();
        assert_eq!(crate::sys::os::mkfifo(&c, 0o600), 0);
        for name in ["/dev/zero", pipe.to_str().unwrap()] {
            let data = blue(&format!("<image href=\"{name}\" width=\"8\" height=\"8\"/>"));
            assert_eq!(red_and_blue(&svg(data.as_bytes(), 8).unwrap()), (false, true), "{name}");
        }
        let packed = blue(&format!("<image href=\"{}\" width=\"8\" height=\"8\"/>", carried(&squeeze(red().as_bytes()))));
        assert_eq!(red_and_blue(&svg(packed.as_bytes(), 8).unwrap()), (false, true), "a squeezed picture carried inside was unsqueezed");
        let long = squeeze(&vec![b' '; SVG_MOST as usize + 1]);
        assert!(long.len() < 1 << 20 && svg_tree(&long).is_none(), "an SVG squeezed small grew past its limit");
        assert!(svg_tree(&vec![b' '; SVG_MOST as usize + 1]).is_none());
        assert!(svg_tree(&squeeze(&squeeze(blue("").as_bytes()))).is_none());
        assert!(svg_bytes(&pipe).is_none() && svg_bytes(Path::new("/dev/zero")).is_none());
        assert_eq!(svg_bytes(&other), Some(red().into_bytes()));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_tray_icon_is_turned_round_and_weighted() {
        let px = from_argb32(2, 1, &[255, 10, 20, 30, 128, 255, 255, 255]).unwrap();
        assert_eq!(px.data(), &[10, 20, 30, 255, 128, 128, 128, 128]);
        assert!(from_argb32(2, 2, &[0; 8]).is_none());
        assert!(from_argb32(0, 1, &[]).is_none());
    }

    #[test]
    fn see_through_colours_are_weighted_by_how_solid_they_are() {
        let img = RgbaImage::from_raw(2, 1, vec![255, 128, 0, 255, 255, 255, 255, 128]).unwrap();
        let px = to_pixmap(&DynamicImage::ImageRgba8(img)).unwrap();
        assert_eq!(to_pixmap(&from_rgb(1, 1, vec![9, 8, 7]).unwrap()).unwrap().data(), &[9, 8, 7, 255]);
        assert_eq!(px.data(), &[255, 128, 0, 255, 128, 128, 128, 128]);
    }

    #[test]
    fn an_xpm_picture_is_read() {
        let text = "/* XPM */\nstatic char *x[] = {\n\"3 2 3 1\",\n\"  c None\",\n\". c #FF8000\",\n\
                    \"+ c #fff s bright\",\n\". +\",\n\"+..\"};";
        let px = from_xpm(text).unwrap();
        assert_eq!((px.width(), px.height()), (3, 2));
        assert_eq!(&px.data()[..12], &[255, 128, 0, 255, 0, 0, 0, 0, 255, 255, 255, 255]);
        assert_eq!(&px.data()[12..16], &[255, 255, 255, 255]);
        assert!(from_xpm("\"0 0 0 1\"").is_none());
        assert!(from_xpm("nothing").is_none());
    }

    #[test]
    fn a_picture_reads_with_its_size() {
        let dir = std::env::temp_dir().join(format!("trindle-picture-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("side.png");
        RgbaImage::from_pixel(4, 2, image::Rgba([1, 2, 3, 255])).save(&file).unwrap();
        let img = load(&file).unwrap();
        assert_eq!(img.dimensions(), (4, 2));
        assert!(load(&dir.join("missing.png")).is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_pipe_named_as_a_picture_is_refused_and_keeps_nothing_waiting() {
        use std::os::unix::fs::OpenOptionsExt;
        let dir = std::env::temp_dir().join(format!("trindle-picture-pipe-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (pipe, link, real, to_real) = (dir.join("x.jpg"), dir.join("y.jpg"), dir.join("real.png"), dir.join("z.png"));
        let c = std::ffi::CString::new(pipe.to_str().unwrap()).unwrap();
        assert_eq!(crate::sys::os::mkfifo(&c, 0o600), 0);
        std::os::unix::fs::symlink(&pipe, &link).unwrap();
        RgbaImage::from_pixel(4, 2, image::Rgba([1, 2, 3, 255])).save(&real).unwrap();
        std::os::unix::fs::symlink(&real, &to_real).unwrap();
        for named in [pipe.clone(), link] {
            let (tell, told) = std::sync::mpsc::channel();
            let asked = named.clone();
            std::thread::spawn(move || tell.send((load(&asked).is_none(), file(&asked).is_err())));
            let got = told.recv_timeout(std::time::Duration::from_secs(5));
            let _ = std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(&pipe);
            assert_eq!(got, Ok((true, true)), "{} was waited on, or read as a picture", named.display());
        }
        assert_eq!(load(&to_real).map(|i| i.dimensions()), Some((4, 2)), "a link to a real picture is read");
        assert!(file(&dir).is_err(), "a folder is no picture");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_picture_is_drawn_into_its_box_smaller_or_larger() {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(4, 2, image::Rgb([200, 100, 50])));
        for (w, h) in [(2, 1), (8, 4), (4, 2), (2, 4)] {
            let mut canvas = Pixmap::new(10, 10).unwrap();
            draw(&mut canvas.as_mut(), &img, 1, 1, w, h).unwrap();
            let at = |x: u32, y: u32| canvas.pixel(x, y).unwrap();
            assert_eq!((at(1, 1).red(), at(1, 1).alpha()), (200, 255), "{w}x{h}");
            assert_eq!(at(w, h).green(), 100, "{w}x{h}");
            assert_eq!(at(1 + w, 1).alpha(), 0, "{w}x{h}");
            assert_eq!(at(0, 0).alpha(), 0, "{w}x{h}");
        }
    }

    #[test]
    fn a_photo_taken_on_its_side_is_turned_upright() {
        use image::ImageEncoder;
        let dir = std::env::temp_dir().join(format!("trindle-picture-side-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("side.jpg");
        let mut out = Vec::new();
        let mut jpeg = image::codecs::jpeg::JpegEncoder::new(&mut out);
        let mut exif = b"MM\0\x2a\0\0\0\x08\0\x01".to_vec();
        exif.extend_from_slice(&[0x01, 0x12, 0, 3, 0, 0, 0, 1, 0, 6, 0, 0, 0, 0, 0, 0]);
        jpeg.set_exif_metadata(exif).unwrap();
        jpeg.write_image(&[128u8; 4 * 2 * 3], 4, 2, image::ExtendedColorType::Rgb8).unwrap();
        std::fs::write(&file, out).unwrap();
        assert_eq!(load(&file).unwrap().dimensions(), (2, 4));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
