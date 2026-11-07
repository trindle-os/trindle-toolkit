//! Printing: widgets drawn by the window's own painting onto pages of a PDF file, one picture to a page.

use crate::t;
use std::io::Write;
use std::path::Path;

use flate2::write::ZlibEncoder;
use flate2::Compression;

use super::theme::Theme;
use super::ui::Ui;
use super::widget::Node;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Paper {
    pub width: f32,
    pub height: f32,
    pub margin: f32,
}

impl Paper {
    pub const A4: Paper = Paper { width: 595.28, height: 841.89, margin: 36.0 };
    pub const LETTER: Paper = Paper { width: 612.0, height: 792.0, margin: 36.0 };
}

const SCALE: f32 = 2.0;
const MOST_PAGES: usize = 200;

pub fn pdf<M: Clone>(tree: Node<M>, theme: Theme, paper: Paper) -> std::io::Result<Vec<u8>> {
    use std::io::{Error, ErrorKind};
    let (wide, tall) = (paper.width - 2.0 * paper.margin, paper.height - 2.0 * paper.margin);
    let mut ui = Ui::new(tree, theme);
    let height = ui.natural_height(wide).max(1.0);
    let pages = ((height / tall).ceil() as usize).max(1);
    if pages > MOST_PAGES {
        return Err(Error::new(ErrorKind::InvalidInput, t!("kit-print-too-long")));
    }
    let (w, page_h) = ((wide * SCALE).round() as u32, (tall * SCALE).round() as u32);
    let mut pictures = Vec::with_capacity(pages);
    let Some(mut all) = tiny_skia::Pixmap::new(w.max(1), (page_h * pages as u32).max(1)) else { return Err(Error::new(ErrorKind::OutOfMemory, t!("kit-print-no-room"))) };
    all.fill(tiny_skia::Color::WHITE);
    ui.paint(&mut all.as_mut(), SCALE);
    for p in 0..pages {
        let rows = &all.data()[(p * page_h as usize * w as usize * 4)..((p + 1) * page_h as usize * w as usize * 4)];
        let rgb: Vec<u8> = rows.chunks_exact(4).flat_map(|px| {
            let clear = 255 - px[3] as u16;
            [0, 1, 2].map(|i| (px[i] as u16 + clear).min(255) as u8)
        }).collect();
        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        z.write_all(&rgb)?;
        pictures.push(z.finish()?);
    }
    Ok(write_pdf(&pictures, (w, page_h), paper))
}

pub fn print_to<M: Clone>(path: &Path, tree: Node<M>, theme: Theme, paper: Paper) -> std::io::Result<()> {
    super::common::files::write_whole(path, &pdf(tree, theme, paper)?)
}

pub fn print_on_paper<M: Clone>(bus: &crate::bus::Bus, events: &crate::event::Loop, title: &str, tree: Node<M>, theme: Theme, paper: Paper,
                                done: impl FnOnce(crate::portal::Answer<()>) + 'static) -> std::io::Result<Option<crate::bus::Listen>> {
    Ok(crate::portal::print_pdf(bus, events, title, &pdf(tree, theme, paper)?, done))
}

fn write_pdf(pictures: &[Vec<u8>], (w, h): (u32, u32), paper: Paper) -> Vec<u8> {
    let mut out = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n".to_vec();
    let mut places = Vec::new();
    let mut object = |out: &mut Vec<u8>, body: &[u8]| {
        places.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", places.len()).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    };
    let n = pictures.len();
    let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 3 + 3 * i)).collect();
    object(&mut out, b"<< /Type /Catalog /Pages 2 0 R >>");
    object(&mut out, format!("<< /Type /Pages /Kids [{}] /Count {n} >>", kids.join(" ")).as_bytes());
    let (x, y, pw, ph) = (paper.margin, paper.margin, paper.width - 2.0 * paper.margin, paper.height - 2.0 * paper.margin);
    for (i, packed) in pictures.iter().enumerate() {
        let me = 3 + 3 * i;
        object(&mut out, format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /XObject << /Im0 {} 0 R >> >> /Contents {} 0 R >>",
                                 paper.width, paper.height, me + 2, me + 1).as_bytes());
        let draw = format!("q {pw:.2} 0 0 {ph:.2} {x:.2} {y:.2} cm /Im0 Do Q");
        object(&mut out, format!("<< /Length {} >>\nstream\n{draw}\nendstream", draw.len()).as_bytes());
        let mut body = format!("<< /Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n",
                               packed.len()).into_bytes();
        body.extend_from_slice(packed);
        body.extend_from_slice(b"\nendstream");
        object(&mut out, &body);
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", places.len() + 1).as_bytes());
    for at in &places {
        out.extend_from_slice(format!("{at:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", places.len() + 1).as_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::widget::{column, label};
    use std::io::Read;

    fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
        hay.windows(needle.len()).position(|w| w == needle)
    }

    fn text(pdf: &[u8]) -> String {
        String::from_utf8_lossy(pdf).into_owned()
    }

    #[test]
    fn a_short_page_is_one_page_of_a_sound_pdf() {
        let pdf = pdf::<()>(column(vec![label("Shopping list"), label("Bread and milk")]), Theme::new(true, "#fbc711"), Paper::A4).unwrap();
        let s = text(&pdf);
        assert!(pdf.starts_with(b"%PDF-1.4") && s.ends_with("%%EOF\n"));
        assert!(s.contains("/Count 1 >>") && s.contains("/MediaBox [0 0 595.28 841.89]"));
        let at = find(&pdf, b"startxref\n").unwrap();
        let xref: usize = std::str::from_utf8(&pdf[at + 10..]).unwrap().lines().next().unwrap().parse().unwrap();
        assert!(pdf[xref..].starts_with(b"xref"));
        for (i, line) in std::str::from_utf8(&pdf[xref..]).unwrap().lines().skip(3).take(5).enumerate() {
            let place: usize = line[..10].parse().unwrap();
            assert!(pdf[place..].starts_with(format!("{} 0 obj", i + 1).as_bytes()), "object {}", i + 1);
        }
    }

    #[test]
    fn the_picture_holds_the_words_inside_the_margins() {
        let pdf = pdf::<()>(column(vec![label("Shopping list")]), Theme::new(true, "#fbc711"), Paper::LETTER).unwrap();
        let start = find(&pdf, b"/FlateDecode").unwrap();
        let stream = find(&pdf[start..], b"stream\n").unwrap() + start + 7;
        let end = find(&pdf[stream..], b"\nendstream").unwrap() + stream;
        let mut rgb = Vec::new();
        flate2::read::ZlibDecoder::new(&pdf[stream..end]).read_to_end(&mut rgb).unwrap();
        let (w, h) = (((612.0 - 72.0) * SCALE).round() as usize, ((792.0 - 72.0) * SCALE).round() as usize);
        assert_eq!(rgb.len(), w * h * 3);
        assert!(rgb.chunks(3).any(|p| p.iter().all(|&v| v < 128)), "dark words on the light page");
        assert!(text(&pdf).contains("q 540.00 0 0 720.00 36.00 36.00 cm /Im0 Do Q"));
    }

    #[test]
    fn a_document_past_the_page_cap_is_an_error_not_a_cut_file() {
        let rows = (0..20_000).map(|i| label(format!("Line {i}"))).collect();
        assert!(pdf::<()>(column(rows), Theme::new(true, "#fbc711"), Paper::A4).is_err());
    }

    #[test]
    fn a_long_page_runs_on_to_more_pages() {
        let rows = (0..200).map(|i| label(format!("Line {i}"))).collect();
        let s = text(&pdf::<()>(column(rows), Theme::new(true, "#fbc711"), Paper::A4).unwrap());
        let pages: usize = s.split("/Count ").nth(1).unwrap().split(' ').next().unwrap().parse().unwrap();
        assert!(pages >= 3, "{pages}");
        assert_eq!(s.matches("/Type /Page ").count(), pages);
    }
}
