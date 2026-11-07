//! What is really drawn, noted in test builds so the tests holding the table of pairs (`pairs`) see each ink and the ground under it.

use std::cell::RefCell;
use std::collections::HashMap;

use tiny_skia::PixmapMut;

use super::{contrast, over, Colour, EDGES, WORDS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Words,
    Icon,
    Dot { named: bool },
}

impl Kind {
    pub fn floor(self) -> f32 {
        match self {
            Kind::Words => WORDS,
            Kind::Icon => EDGES,
            Kind::Dot { .. } => 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Drawn {
    pub kind: Kind,
    pub ink: Colour,
    pub shown: Colour,
    pub ground: Option<Colour>,
    pub share: f32,
    pub at: (i32, i32, u32, u32),
    pub page: String,
}

impl Drawn {
    pub fn stands(&self) -> Option<f32> {
        self.ground.map(|g| contrast(self.shown, g))
    }
}

struct Layer {
    under: Vec<u8>,
    w: u32,
    h: u32,
    opacity: f32,
    origin: (i32, i32),
}

#[derive(Default)]
struct Noted {
    on: bool,
    all: Vec<Drawn>,
    vouched: Vec<(Colour, Vec<Colour>)>,
    layers: Vec<Layer>,
    page: String,
    pages: Vec<String>,
}

#[derive(Default)]
pub struct Noting {
    pub drawn: Vec<Drawn>,
    pub vouched: Vec<(Colour, Vec<Colour>)>,
    pub pages: Vec<String>,
}

thread_local! {
    static NOTED: RefCell<Noted> = RefCell::new(Noted::default());
}

pub fn begin() {
    NOTED.with(|n| *n.borrow_mut() = Noted { on: true, ..Noted::default() });
}

pub fn end() -> Noting {
    let noted = NOTED.with(|n| std::mem::take(&mut *n.borrow_mut()));
    Noting { drawn: noted.all, vouched: noted.vouched, pages: noted.pages }
}

pub fn asked_page(page: &str) {
    NOTED.with(|n| {
        let mut n = n.borrow_mut();
        if n.on && !n.pages.iter().any(|p| p == page) {
            n.pages.push(page.to_string());
        }
    });
}

pub fn vouch(ink: Colour, under: &[Colour]) {
    NOTED.with(|n| {
        let mut n = n.borrow_mut();
        if n.on && !n.vouched.iter().any(|(i, u)| *i == ink && u == under) {
            n.vouched.push((ink, under.to_vec()));
        }
    });
}

pub fn page(name: &str) {
    NOTED.with(|n| n.borrow_mut().page = name.to_string());
}

pub fn noting() -> bool {
    NOTED.with(|n| n.borrow().on)
}

fn pixel(data: &[u8], w: u32, x: u32, y: u32) -> Colour {
    let i = (y as usize * w as usize + x as usize) * 4;
    let a = f32::from(data[i + 3]) / 255.0;
    if a <= 0.0 {
        return Colour { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    }
    let c = |v: u8| (f32::from(v) / 255.0 / a).min(1.0);
    Colour { r: c(data[i]), g: c(data[i + 1]), b: c(data[i + 2]), a }
}

pub struct Laid(bool);

impl Drop for Laid {
    fn drop(&mut self) {
        if self.0 {
            NOTED.with(|n| n.borrow_mut().layers.pop());
        }
    }
}

pub fn laid_over(under: &PixmapMut, at: (i32, i32, u32, u32), opacity: f32) -> Laid {
    if !noting() {
        return Laid(false);
    }
    let (x, y, w, h) = at;
    let mut bytes: Vec<u8> = UNSEEN.repeat(w as usize * h as usize);
    let (uw, uh) = (under.width() as i32, under.height() as i32);
    let under = under.as_ref();
    let data = under.data();
    for row in 0..h as i32 {
        let sy = y + row;
        if sy < 0 || sy >= uh {
            continue;
        }
        let (from, to) = (x.max(0), (x + w as i32).min(uw));
        if from >= to {
            continue;
        }
        let src = (sy as usize * uw as usize + from as usize) * 4;
        let dst = (row as usize * w as usize + (from - x) as usize) * 4;
        let len = (to - from) as usize * 4;
        bytes[dst..dst + len].copy_from_slice(&data[src..src + len]);
    }
    NOTED.with(|n| {
        let mut n = n.borrow_mut();
        let depth = n.layers.len();
        if depth > 0 {
            for row in 0..h {
                for col in 0..w {
                    let (px, py) = (x + col as i32, y + row as i32);
                    let i = (row as usize * w as usize + col as usize) * 4;
                    if px < 0 || py < 0 || bytes[i..i + 4] == UNSEEN {
                        continue;
                    }
                    let c = pixel(&bytes, w, col, row);
                    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    match through(&n.layers, depth, c, px as u32, py as u32) {
                        Under::Solid(g) => bytes[i..i + 4].copy_from_slice(&[b(g.r), b(g.g), b(g.b), 255]),
                        Under::Unseen => bytes[i..i + 4].copy_from_slice(&UNSEEN),
                        Under::Unknown => {}
                    }
                }
            }
        }
        let from = n.layers.last().map_or((0, 0), |l| l.origin);
        n.layers.push(Layer { under: bytes, w, h, opacity, origin: (from.0 + x, from.1 + y) });
    });
    Laid(true)
}

const UNSEEN: [u8; 4] = [1, 0, 0, 0];

enum Under {
    Solid(Colour),
    Unknown,
    Unseen,
}

fn through(layers: &[Layer], depth: usize, c: Colour, x: u32, y: u32) -> Under {
    let Some(layer) = layers[..depth].last() else {
        return if c.a >= 1.0 { Under::Solid(c) } else { Under::Unknown };
    };
    if x >= layer.w || y >= layer.h {
        return Under::Unseen;
    }
    let i = (y as usize * layer.w as usize + x as usize) * 4;
    if layer.under[i..i + 4] == UNSEEN {
        return Under::Unseen;
    }
    let under = pixel(&layer.under, layer.w, x, y);
    if c.a >= 1.0 && layers[..depth].iter().all(|l| l.opacity >= 1.0) {
        return Under::Solid(c);
    }
    if under.a < 1.0 {
        return Under::Unknown;
    }
    Under::Solid(over(Colour { a: c.a * layer.opacity, ..c }, under))
}

pub fn dot(canvas: &PixmapMut, colour: Colour, named: bool) {
    ink(canvas, Kind::Dot { named }, colour, (0.0, 0.0, canvas.width() as f32, canvas.height() as f32));
}

pub fn ink(canvas: &PixmapMut, kind: Kind, colour: Colour, at: (f32, f32, f32, f32)) {
    if !noting() {
        return;
    }
    let (cw, ch) = (canvas.width() as i32, canvas.height() as i32);
    let (x0, y0) = ((at.0.floor() as i32).max(0), (at.1.floor() as i32).max(0));
    let (x1, y1) = (((at.0 + at.2).ceil() as i32).min(cw), ((at.1 + at.3).ceil() as i32).min(ch));
    if x1 <= x0 || y1 <= y0 || colour.a <= 0.0 {
        return;
    }
    let canvas = canvas.as_ref();
    let data = canvas.data();
    NOTED.with(|n| {
        let mut n = n.borrow_mut();
        let depth = n.layers.len();
        let fade: f32 = n.layers.iter().map(|l| l.opacity).product();
        let step = (((x1 - x0) as usize * (y1 - y0) as usize) / 4000).max(1);
        let mut seen: HashMap<[u8; 3], usize> = HashMap::new();
        let (mut clear, mut all) = (0usize, 0usize);
        let mut i = 0usize;
        for y in y0..y1 {
            for x in x0..x1 {
                i += 1;
                if i % step != 0 {
                    continue;
                }
                let c = pixel(data, cw as u32, x as u32, y as u32);
                match through(&n.layers, depth, c, x as u32, y as u32) {
                    Under::Solid(g) => {
                        let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                        *seen.entry([b(g.r), b(g.g), b(g.b)]).or_default() += 1;
                        all += 1;
                    }
                    Under::Unknown => {
                        clear += 1;
                        all += 1;
                    }
                    Under::Unseen => {}
                }
            }
        }
        if all == 0 {
            return;
        }
        let most = seen.iter().max_by_key(|(k, v)| (**v, **k)).map(|(k, v)| (*k, *v));
        let ground = most.filter(|(_, count)| *count >= clear)
            .map(|(k, _)| Colour::rgb(f32::from(k[0]) / 255.0, f32::from(k[1]) / 255.0, f32::from(k[2]) / 255.0));
        let share = most.map_or(0.0, |(_, count)| count as f32 / all.max(1) as f32);
        let shown = ground.map_or(Colour { a: 1.0, ..colour }, |g| over(Colour { a: colour.a * fade, ..colour }, g));
        let page = n.page.clone();
        let from = n.layers.last().map_or((0, 0), |l| l.origin);
        n.all.push(Drawn { kind, ink: colour, shown, ground, share, at: (from.0 + x0, from.1 + y0, (x1 - x0) as u32, (y1 - y0) as u32), page });
    });
}

const BESIDE: i32 = 32;

fn close(a: Colour, b: Colour) -> bool {
    let near = |x: f32, y: f32| (x - y).abs() <= (super::LAID + 0.5) / 255.0;
    near(a.r, b.r) && near(a.g, b.g) && near(a.b, b.b)
}

pub fn faults(what: &str, noted: &Noting, light: bool, accent: &str) -> Vec<String> {
    faults_with(what, noted, &crate::common::style::layers(), light, accent)
}

fn faults_with(what: &str, noted: &Noting, layers: &crate::common::style::Layers, light: bool, accent: &str) -> Vec<String> {
    let all = &noted.drawn;
    let mut table = super::painted(layers, light, accent);
    let own = crate::common::style::Layers::new(crate::common::style::CSS, "");
    table.extend(super::painted(&own, light, accent));
    table.extend(super::painted(&own, false, crate::common::style::DEFAULT_ACCENT));
    for page in &noted.pages {
        table.extend(super::painted_in(&super::Theme::on_page(layers, light, accent, page), layers, light));
    }
    let icons = [super::Theme::from(layers, light, accent).icon, super::Theme::from(&own, light, accent).icon];
    let mut out = Vec::new();
    for d in all {
        let place = format!("{what}: {} {:?} {} at {:?}", d.page, d.kind, d.ink.css(), d.at);
        if let Kind::Dot { named } = d.kind {
            let beside = |w: &Drawn| {
                let (dy, wy) = (d.at.1 + d.at.3 as i32 / 2, w.at.1 + w.at.3 as i32 / 2);
                let gap = (w.at.0 - (d.at.0 + d.at.2 as i32)).max(d.at.0 - (w.at.0 + w.at.2 as i32));
                w.kind == Kind::Words && w.page == d.page && (dy - wy).abs() <= (w.at.3 as i32).max(d.at.3 as i32) / 2 && gap <= BESIDE
            };
            if !named && !all.iter().any(beside) {
                out.push(format!("{place} is a state's dot with no words beside it and no name of its own"));
            }
            continue;
        }
        let Some(ground) = d.ground else {
            out.push(format!("{place} is drawn on nothing known"));
            continue;
        };
        let stands = contrast(d.shown, ground);
        let held = |d: &Drawn, ground: Colour| {
            (d.kind == Kind::Icon && icons.iter().any(|i| close(*i, d.ink)))
                || table.iter().any(|p| close(over(p.colour, p.ground), d.shown) && close(p.ground, ground))
                || noted.vouched.iter().any(|(ink, under)| *ink == d.ink && under.iter().any(|u| close(Colour { a: 1.0, ..*u }, ground)))
        };
        if stands < d.kind.floor() {
            out.push(format!("{place} shows as {} on {}: {stands:.2} of {}", d.shown.css(), ground.css(), d.kind.floor()));
        } else if !held(d, ground) {
            out.push(format!("{place} shows as {} on {} ({stands:.2}), a pair the table does not hold", d.shown.css(), ground.css()));
        }
    }
    out.sort();
    out.dedup();
    out
}

pub fn highlights() -> Vec<String> {
    let all = super::tests::highlights();
    if std::env::var_os("TRINDLE_PAIRS_FEW").is_none() {
        return all;
    }
    vec![all[0].clone(), "#2295f1".to_string(), "#3f51b4".to_string()]
}

pub fn in_every_look(what: &str, least: usize, paint: impl Fn()) -> Vec<String> {
    let mut all = in_look(what, false, least, &paint);
    all.extend(in_look(what, true, least, &paint));
    all
}

pub fn in_look(what: &str, light: bool, least: usize, paint: impl Fn()) -> Vec<String> {
    let thread = std::thread::current();
    let named = thread.name().unwrap_or("main");
    assert!(named == "main" || named.ends_with(NAMED), "{named} paints {what} for its pairs, so its name must end {NAMED}");
    let mut all: Vec<String> = Vec::new();
    {
        for accent in highlights() {
            crate::common::style::ASKED_FOR_TEST.with(|a| *a.borrow_mut() = Some((light, accent.clone())));
            begin();
            paint();
            let drawn = end();
            crate::common::style::ASKED_FOR_TEST.with(|a| *a.borrow_mut() = None);
            if drawn.drawn.len() < least {
                all.push(format!("{what}: only {} words and icons were drawn, of at least {least}", drawn.drawn.len()));
            }
            let look = format!("{what}, {} with {accent}", if light { "light" } else { "dark" });
            all.extend(faults(&look, &drawn, light, &accent));
        }
    }
    all
}

pub fn window<M: Clone>(state: &str, view: crate::widget::Node<M>, look: super::Theme, size: (u32, u32), clear: bool) {
    page(state);
    let mut ui = crate::ui::Ui::new(view, look);
    ui.see_through = clear;
    let mut pix = tiny_skia::Pixmap::new(size.0, size.1).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
}

pub fn none(faults: Vec<String>) {
    let shown: Vec<&str> = faults.iter().map(String::as_str).take(300).collect();
    assert!(faults.is_empty(), "{} pairs drawn are not held by the table of pairs:\n{}", faults.len(), shown.join("\n"));
}

pub const NAMED: &str = "only_pairs_the_table_holds";

#[cfg(test)]
mod tests {
    use super::*;

    fn canvas(w: u32, h: u32, c: Option<Colour>) -> tiny_skia::Pixmap {
        let mut p = tiny_skia::Pixmap::new(w, h).unwrap();
        if let Some(c) = c {
            p.fill(c.to_skia());
        }
        p
    }

    #[test]
    fn ink_is_noted_on_the_colour_under_it() {
        let grey = Colour::rgb(0.2, 0.2, 0.2);
        let mut p = canvas(40, 20, Some(grey));
        begin();
        ink(&p.as_mut(), Kind::Words, Colour::rgb(1.0, 1.0, 1.0), (4.0, 4.0, 20.0, 10.0));
        ink(&p.as_mut(), Kind::Icon, Colour { a: 0.5, ..Colour::rgb(1.0, 1.0, 1.0) }, (4.0, 4.0, 20.0, 10.0));
        let all = end().drawn;
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].ground, Some(grey));
        assert!((all[0].stands().unwrap() - contrast(Colour::rgb(1.0, 1.0, 1.0), grey)).abs() < 0.01);
        assert!((all[1].shown.r - 0.6).abs() < 0.01, "half-clear white over the grey: {:?}", all[1].shown);
        ink(&p.as_mut(), Kind::Words, Colour::rgb(1.0, 1.0, 1.0), (4.0, 4.0, 20.0, 10.0));
        assert!(end().drawn.is_empty());
    }

    #[test]
    fn ink_on_a_clear_canvas_is_measured_on_what_shows_through() {
        let navy = Colour::rgb(0.0, 0.0, 0.4);
        let mut root = canvas(40, 40, Some(navy));
        let mut own = canvas(20, 20, None);
        let white = Colour::rgb(1.0, 1.0, 1.0);
        begin();
        ink(&own.as_mut(), Kind::Words, white, (2.0, 2.0, 10.0, 10.0));
        for opacity in [1.0, 0.5] {
            let _laid = laid_over(&root.as_mut(), (10, 10, 20, 20), opacity);
            ink(&own.as_mut(), Kind::Words, white, (2.0, 2.0, 10.0, 10.0));
        }
        {
            let _laid = laid_over(&root.as_mut(), (10, 10, 20, 20), 1.0);
            let mut inner = canvas(10, 10, None);
            let _inner = laid_over(&own.as_mut(), (2, 2, 10, 10), 1.0);
            ink(&inner.as_mut(), Kind::Words, white, (0.0, 0.0, 10.0, 10.0));
        }
        ink(&own.as_mut(), Kind::Words, white, (2.0, 2.0, 10.0, 10.0));
        let all = end().drawn;
        assert_eq!(all[0].ground, None, "drawn on nothing known");
        assert_eq!(all[1].ground, Some(navy));
        assert_eq!(all[2].ground, Some(navy));
        assert!((all[2].shown.r - 0.5).abs() < 0.01, "white at half strength over navy: {:?}", all[2].shown);
        assert_eq!(all[3].ground, Some(navy));
        assert_eq!(all[3].at, (12, 12, 10, 10), "where in the window, through both canvases");
        assert_eq!(all[4].ground, None);
        begin();
        {
            let _laid = laid_over(&root.as_mut(), (30, 30, 20, 20), 1.0);
            ink(&own.as_mut(), Kind::Words, white, (12.0, 12.0, 6.0, 6.0));
            ink(&own.as_mut(), Kind::Words, white, (2.0, 2.0, 6.0, 6.0));
        }
        let all = end().drawn;
        assert_eq!(all.len(), 1, "{all:?}");
        assert_eq!(all[0].ground, Some(navy));
    }

    const OWN_PAINTING: [&str; 3] = ["the_gallery_draws_only_pairs_the_table_holds", "a_red_main_action_draws_only_pairs_the_table_holds",
                                     "selected_words_draw_only_pairs_the_table_holds"];

    #[test]
    fn the_toolkits_own_painting_tests_draw_only_pairs_the_table_holds_in_every_theme() {
        const INNER: &str = "TRINDLE_PAIRS_FEW";
        if std::env::var_os(INNER).is_some() {
            return;
        }
        let themes = crate::common::source::toolkit_root().join("share/themes");
        let mut names: Vec<std::path::PathBuf> = std::fs::read_dir(&themes).unwrap().flatten().map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "css")).collect();
        names.sort();
        assert!(names.len() >= 4, "{names:?}");
        for theme in names {
            let out = std::process::Command::new(std::env::current_exe().unwrap())
                .args([NAMED, "--test-threads=1"]).env("TRINDLE_TEST_THEME", &theme).env(INNER, "1").output().unwrap();
            let said = String::from_utf8_lossy(&out.stdout);
            assert!(out.status.success(), "in the theme {}:\n{}", theme.display(),
                    said.lines().filter(|l| !l.ends_with("... ok")).take(120).collect::<Vec<_>>().join("\n"));
            for name in OWN_PAINTING {
                assert!(said.lines().any(|l| l.starts_with("test ") && l.ends_with(&format!("::{name} ... ok"))),
                        "in the theme {}, the test {name} did not run and pass", theme.display());
            }
        }
    }

    #[test]
    fn a_pair_the_table_does_not_hold_is_a_fault_however_well_it_reads() {
        use crate::common::style::DEFAULT_ACCENT;
        let t = super::super::Theme::new(false, DEFAULT_ACCENT);
        let paint = |ground: Option<Colour>, colour: Colour| {
            let mut p = canvas(60, 30, ground);
            ink(&p.as_mut(), Kind::Words, colour, (4.0, 4.0, 40.0, 16.0));
        };
        let said = |ground: Option<Colour>, colour: Colour, vouched: bool| {
            begin();
            if vouched {
                vouch(colour, &[ground.unwrap()]);
            }
            paint(ground, colour);
            faults("a test", &end(), false, DEFAULT_ACCENT)
        };
        assert_eq!(said(Some(t.panel), t.fg, false), Vec::<String>::new());
        let wash = over(super::super::alpha(t.accent, 0.12), t.panel);
        assert!(contrast(t.fg, wash) >= WORDS);
        let found = said(Some(wash), t.fg, false);
        assert!(found.len() == 1 && found[0].ends_with("a pair the table does not hold"), "{found:?}");
        assert_eq!(said(Some(wash), t.words_on(&[wash]), true), Vec::<String>::new());
        begin();
        vouch(t.fg, &[t.raised]);
        paint(Some(wash), t.fg);
        assert_eq!(faults("a test", &end(), false, DEFAULT_ACCENT).len(), 1);
        let found = said(Some(t.panel), super::super::alpha(t.fg, 0.2), false);
        assert!(found.len() == 1 && found[0].contains(" of 4.5"), "{found:?}");
        let found = said(None, t.fg, false);
        assert!(found.len() == 1 && found[0].ends_with("is drawn on nothing known"), "{found:?}");
    }

    #[test]
    fn no_floor_is_loosened_or_written_again_as_a_number() {
        let mut found = Vec::new();
        for (name, text) in crate::common::source::every_file() {
            for (n, line) in text.lines().enumerate() {
                if breaks_a_floor(line) {
                    found.push(format!("src/{name}:{}: {}", n + 1, line.trim()));
                }
            }
        }
        assert!(found.is_empty(), "a floor is loosened, or written as a number where `WORDS` or `EDGES` is meant:\n{}", found.join("\n"));
    }

    const FLOORS: [&str; 3] = [concat!("WOR", "DS"), concat!("EDG", "ES"), concat!("floor", "()")];

    fn breaks_a_floor(line: &str) -> bool {
        if line.trim_start().starts_with("//") {
            return false;
        }
        let tight: String = line.split_whitespace().collect();
        let named = |c: char| c.is_alphanumeric() || c == '_';
        let loosened = FLOORS.iter().any(|floor| {
            tight.match_indices(floor).any(|(at, _)| {
                let whole = floor.ends_with(')') || !tight[..at].chars().next_back().is_some_and(named);
                let after = &tight[at + floor.len()..];
                let less = after.starts_with('-') && !after.starts_with("->");
                let part = after.starts_with(['*', '/']) && after[1..].starts_with(|c: char| c.is_ascii_digit());
                whole && (less || part)
            })
        });
        let speaks = [concat!("cont", "rast"), concat!("sta", "nds"), concat!("rat", "io")];
        let gives = [concat!("let", "floor"), concat!("floor", "="), concat!("floor", ":")];
        let measures = speaks.iter().any(|w| line.contains(w)) || gives.iter().any(|w| tight.contains(w));
        let own = [concat!("4", ".5"), concat!("3", ".0")];
        let again = own.iter().any(|n| ['=', '<', '>'].iter().any(|c| tight.contains(&format!("{c}{n}"))));
        loosened || (measures && (again || holds_a_lower_number(line)))
    }

    fn holds_a_lower_number(line: &str) -> bool {
        let bytes = line.as_bytes();
        let mut at = 0;
        while at < bytes.len() {
            if !bytes[at].is_ascii_digit() {
                at += 1;
                continue;
            }
            let start = at;
            while at < bytes.len() && (bytes[at].is_ascii_digit() || bytes[at] == b'.') {
                at += 1;
            }
            let own = !line[..start].chars().next_back().is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.');
            if let (true, Ok(n)) = (own && line[start..at].contains('.'), line[start..at].parse::<f32>()) {
                if (LOWEST..WORDS).contains(&n) && n != EDGES {
                    return true;
                }
            }
        }
        false
    }

    const LOWEST: f32 = 2.5;

    #[test]
    fn every_way_of_loosening_a_floor_is_known() {
        let [words, edges, floor] = FLOORS;
        let far = concat!("cont", "rast");
        let broken = [
            format!("assert!(c >= {words} - 0.01);"),
            format!("assert!(c >= {words}-0.01);"),
            format!("assert!(c >= {edges} -0.05, \"too near\");"),
            format!("let least = {edges} * 0.99;"),
            format!("let least = {words}/1.01;"),
            format!("if p.ink.{floor}-0.05 > far {{"),
            format!("assert!({far}(a, b) >= 4.46);"),
            format!("assert!({far}(a, b) >= 4.45);"),
            format!("assert!({far}(a, b) > 2.95, \"{{a:?}}\");"),
            format!("assert!({far}(a, b) >=4.5);"),
            format!("assert!({far}(a, b) < 3.0);"),
            concat!("let fl", "oor = 4.47;").to_string(),
            concat!("let fl", "oor: f32 = 4.5;").to_string(),
            concat!("Ink { fl", "oor: 2.99, ..ink }").to_string(),
        ];
        for line in &broken {
            assert!(breaks_a_floor(line), "not seen: {line}");
        }
        let sound = [
            format!("assert!({far}(a, b) >= {words}, \"{{a:?}}\");"),
            format!("let kind = if b.ink.{floor} >= {words} {{ Kind::Words }} else {{ Kind::Icon }};"),
            format!("// once: assert!({far}(a, b) >= 4.45)"),
            format!("fn {far}(a: Colour, b: Colour) -> f32 {{"),
            format!("let size = (PASS{words} - 2.0) * s;"),
            format!("let style = Style::new(({words} * s).round());"),
            "let wanted = ((start + 0.5) / tab).floor() * tab + tab;".to_string(),
            format!("p.push_str(\"Measured {far}: words need 4.5 to one, and edges 3 to one\");"),
            "let x = (w - 3.5) / 2.0;".to_string(),
        ];
        for line in &sound {
            assert!(!breaks_a_floor(line), "taken for a loosened floor: {line}");
        }
    }

    #[test]
    fn a_wash_gives_way_until_the_words_on_it_can_be_read() {
        use super::super::{alpha, Theme, WHITE};
        for light in [false, true] {
            for accent in super::super::tests::highlights() {
                let t = Theme::new(light, &accent);
                let field = over(t.field, over(t.panel, t.bg));
                for wash in [alpha(t.accent, 0.1), alpha(t.accent, 0.35), alpha(t.accent, 0.8), alpha(super::super::warn(), 0.32), alpha(WHITE, 0.9),
                             alpha(Colour::rgb(0.0, 0.0, 0.0), 0.9)] {
                    begin();
                    let first = t.wash(wash, &[field]);
                    let second = t.wash(alpha(t.accent, 0.35), &[field, over(first, field)]);
                    let noted = end();
                    for ground in [over(first, field), over(second, field), over(second, over(first, field))] {
                        assert!(contrast(t.fg, ground) >= WORDS, "light {light}, {accent}: words on {} stand {:.2}", ground.css(), contrast(t.fg, ground));
                        assert!(noted.vouched.iter().any(|(ink, under)| *ink == t.fg && under.iter().any(|u| close(*u, ground))), "not vouched on {}", ground.css());
                    }
                    if super::super::near(over(wash, super::super::kept(field))).iter().all(|g| contrast(t.fg, *g) >= WORDS + super::super::SPARE) {
                        assert_eq!(first, wash, "a wash the words can be read on was thinned");
                    }
                    let mixed = [field, over(first, field), over(alpha(t.fg, 0.07), field)];
                    assert!(mixed.iter().all(|g| contrast(t.link_on(&mixed), *g) >= WORDS), "light {light}, {accent}: a link on a mixed ground");
                }
            }
        }
    }

    #[test]
    fn words_a_little_under_their_floor_are_a_fault() {
        use crate::common::style::DEFAULT_ACCENT;
        use super::super::tests::as_shown;
        let t = super::super::Theme::new(false, DEFAULT_ACCENT);
        let ground = super::super::kept(t.panel);
        let greys: Vec<Colour> = (0..=255u8).map(|v| Colour::rgb(f32::from(v) / 255.0, f32::from(v) / 255.0, f32::from(v) / 255.0)).collect();
        let said = |kind: Kind, colour: Colour| {
            let mut p = canvas(60, 30, Some(ground));
            begin();
            ink(&p.as_mut(), kind, colour, (4.0, 4.0, 40.0, 16.0));
            faults("a test", &end(), false, DEFAULT_ACCENT)
        };
        for kind in [Kind::Words, Kind::Icon] {
            let floor = kind.floor();
            let first = greys.iter().position(|c| as_shown(*c, ground) >= floor).unwrap();
            let (short, enough) = (greys[first - 1], greys[first]);
            assert!(as_shown(short, ground) > floor - 0.2, "the grey one step short stands {}", as_shown(short, ground));
            let found = said(kind, short);
            assert!(found.len() == 1 && found[0].contains(&format!(" of {floor}")), "{kind:?} at {:.3} to one passed: {found:?}", as_shown(short, ground));
            let found = said(kind, enough);
            assert!(found.iter().all(|f| !f.contains(&format!(" of {floor}"))), "{kind:?} at {:.3} to one: {found:?}", as_shown(enough, ground));
            let between = (1..1000).map(|i| super::super::mix(short, enough, i as f32 / 1000.0))
                .find(|c| super::super::luminance(*c) > super::super::luminance(short) && {
                    let (x, y) = (super::super::luminance(*c), super::super::luminance(ground));
                    (x.max(y) + 0.05) / (x.min(y) + 0.05) >= floor
                });
            if let Some(c) = between.filter(|c| as_shown(*c, ground) < floor) {
                let found = said(kind, c);
                assert!(found.len() == 1 && found[0].contains(&format!(" of {floor}")), "{kind:?} worked out at its floor and shown at {:.3} passed: {found:?}", as_shown(c, ground));
            }
        }
    }

    #[test]
    fn a_program_on_a_page_colour_of_its_own_is_held_to_the_table_of_that_look() {
        use crate::common::style::{Layers, CSS, DEFAULT_ACCENT};
        use super::super::Theme;
        let layers = Layers::themed(CSS, "", ":root { --trindle-store-page: #3b1f4a; }");
        let t = Theme::on_page(&layers, false, DEFAULT_ACCENT, "store");
        assert_ne!(t.bg, Theme::from(&layers, false, DEFAULT_ACCENT).bg, "Apps was given no page of its own");
        let said = |asked: bool| {
            let mut p = canvas(60, 30, Some(t.bg));
            begin();
            if asked {
                asked_page("store");
            }
            ink(&p.as_mut(), Kind::Words, t.fg, (4.0, 4.0, 40.0, 16.0));
            faults_with("a page of Apps' own colour", &end(), &layers, false, DEFAULT_ACCENT)
        };
        assert_eq!(said(true), Vec::<String>::new());
        let found = said(false);
        assert!(found.len() == 1 && found[0].ends_with("a pair the table does not hold"), "{found:?}");
    }
}
