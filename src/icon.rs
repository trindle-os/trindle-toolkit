//! Icons by name through the freedesktop icon theme rules; a file named by path outside the computer's own places is read on a thread, never on the loop that draws.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use tiny_skia::{Pixmap, PixmapMut, Transform};

use super::colour::Colour;
use super::paint::Painter;
use super::generic;
use super::picture;
use super::window::Window;
use crate::common::files::{config_home, data_home, own_place};

const SYMBOLIC: &str = "-symbolic";
const MOST_SIZE: u32 = 8192;
const KINDS: &[&str] = &["png", "svg"];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Fit {
    Fixed,
    Scalable,
    Threshold,
}

#[derive(Debug)]
struct Dir {
    path: String,
    size: u32,
    scale: u32,
    fit: Fit,
    min: u32,
    max: u32,
    threshold: u32,
}

impl Dir {
    fn suits(&self, size: u32, scale: u32) -> bool {
        self.scale == scale && match self.fit {
            Fit::Fixed => self.size == size,
            Fit::Scalable => (self.min..=self.max).contains(&size),
            Fit::Threshold => (self.size.saturating_sub(self.threshold)..=self.size + self.threshold).contains(&size),
        }
    }

    fn distance(&self, size: u32, scale: u32) -> u32 {
        let want = size * scale;
        let (low, high) = match self.fit {
            Fit::Fixed => (self.size, self.size),
            Fit::Scalable => (self.min, self.max),
            Fit::Threshold => (self.size.saturating_sub(self.threshold), self.size + self.threshold),
        };
        let (low, high) = (low * self.scale, high * self.scale);
        if want < low { low - want } else { want.saturating_sub(high) }
    }
}

#[derive(Debug, Default)]
struct Theme {
    roots: Vec<PathBuf>,
    dirs: Vec<Dir>,
    inherits: Vec<String>,
    elsewhere: bool,
}

const ICON_SIDE: u32 = 2048;
const ICON_ALLOC: u64 = 64 << 20;

const INDEX_MOST: u64 = 1 << 20;

fn parse(text: &str) -> (Vec<Dir>, Vec<String>) {
    let mut sections: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
    let mut at = "";
    for line in text.lines().map(str::trim) {
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            at = name;
        } else if let Some((k, v)) = line.split_once('=') {
            sections.entry(at).or_default().entry(k.trim()).or_insert(v.trim());
        }
    }
    let head = sections.get("Icon Theme");
    let list = |key: &str| -> Vec<String> {
        head.and_then(|h| h.get(key)).map_or(Vec::new(), |v| {
            v.split(',').map(str::trim).filter(|s| !s.is_empty()).map(String::from).collect()
        })
    };
    let mut dirs = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for path in list("Directories").into_iter().chain(list("ScaledDirectories")) {
        let Some(s) = sections.get(path.as_str()) else { continue };
        let num = |k: &str| s.get(k).and_then(|v| v.parse::<u32>().ok()).map(|v| v.min(MOST_SIZE));
        let Some(size) = num("Size") else { continue };
        let fit = match s.get("Type").copied() {
            Some("Fixed") => Fit::Fixed,
            Some("Scalable") => Fit::Scalable,
            _ => Fit::Threshold,
        };
        if !seen.insert(path.clone()) {
            continue;
        }
        dirs.push(Dir {
            size,
            scale: num("Scale").unwrap_or(1).max(1),
            fit,
            min: num("MinSize").unwrap_or(size),
            max: num("MaxSize").unwrap_or(size),
            threshold: num("Threshold").unwrap_or(2),
            path,
        });
    }
    (dirs, list("Inherits"))
}

pub fn chosen() -> String {
    let ini = std::fs::read_to_string(config_home().join("gtk-4.0").join("settings.ini")).unwrap_or_default();
    ini.lines()
        .find_map(|l| l.trim().strip_prefix("gtk-icon-theme-name").filter(|v| v.starts_with([' ', '\t', '='])).map(|v| v.trim_start().trim_start_matches('=').trim()))
        .filter(|v| !v.is_empty())
        .unwrap_or("hicolor")
        .to_string()
}

fn bases() -> Vec<PathBuf> {
    let mut bases = vec![data_home().join("icons")];
    let sys = std::env::var("XDG_DATA_DIRS").ok().filter(|d| !d.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    bases.extend(sys.split(':').filter(|s| !s.is_empty()).map(|s| Path::new(s).join("icons")));
    let flatpak = [data_home().join("flatpak/exports/share/icons"), PathBuf::from("/var/lib/flatpak/exports/share/icons")];
    for d in flatpak {
        if !bases.contains(&d) {
            bases.push(d);
        }
    }
    bases.dedup();
    bases
}

type Key = (String, u32, u32, Option<[u8; 4]>);
type Kept = (u64, Rc<Pixmap>);

const FAR_MOST: usize = 256;
const MOST_PAINTED: usize = 4096;

static FAR_READS: crate::common::run::Slots = crate::common::run::Slots::new(8);

#[derive(Debug, PartialEq)]
enum Step<S> {
    Is(bool),
    Waits,
    Read(S),
    Full,
}

fn far_step<K: std::hash::Hash + Eq + Clone, P, S>(far: &mut HashMap<K, FarOf<P>>, key: &K, place: impl FnOnce() -> Option<S>) -> (Step<S>, Option<P>) where P: Clone {
    match far.get(key) {
        Some(FarOf::Back(painted)) => (Step::Is(painted.is_some()), painted.clone()),
        Some(FarOf::Out) => (Step::Waits, None),
        None => {
            let Some(place) = place() else { return (Step::Full, None) };
            if far.len() >= FAR_MOST {
                far.retain(|_, f| matches!(f, FarOf::Out));
            }
            far.insert(key.clone(), FarOf::Out);
            (Step::Read(place), None)
        }
    }
}

enum FarOf<P> {
    Out,
    Back(Option<P>),
}

type Sought = (String, String, u32, u32);

thread_local! {
    static FAR: RefCell<HashMap<(PathBuf, u32), FarOf<Rc<Pixmap>>>> = RefCell::new(HashMap::new());
    static FAR_FOUND: RefCell<HashMap<Sought, FarOf<PathBuf>>> = RefCell::new(HashMap::new());
    static WINDOWS: RefCell<Vec<std::rc::Weak<Window>>> = const { RefCell::new(Vec::new()) };
}

pub fn draw_again_when_one_is_back(window: &Rc<Window>) {
    WINDOWS.with_borrow_mut(|windows| {
        windows.retain(|w| w.strong_count() > 0);
        windows.push(Rc::downgrade(window));
    });
}

pub fn forget_far() {
    FAR.with_borrow_mut(|far| far.retain(|_, f| matches!(f, FarOf::Out)));
    FAR_FOUND.with_borrow_mut(|far| far.retain(|_, f| matches!(f, FarOf::Out)));
}

fn draw_again() {
    let windows: Vec<Rc<Window>> = WINDOWS.with_borrow(|windows| windows.iter().filter_map(|w| w.upgrade()).collect());
    windows.iter().for_each(|w| w.redraw());
}

fn is_far(file: &Path) -> bool {
    file.is_absolute() && !own_place(file)
}

fn far(file: &Path, px: u32) -> Option<Rc<Pixmap>> {
    let key = (file.to_path_buf(), px);
    let (step, painted) = FAR.with_borrow_mut(|far| far_step(far, &key, || FAR_READS.take()));
    if let Step::Read(place) = step {
        let file = file.to_path_buf();
        crate::common::run::on_thread(move || {
            let _place = place;
            read_far(&file, px)
        }, move |painted| {
            FAR.with_borrow_mut(|far| far.insert(key, FarOf::Back(painted.map(Rc::new))));
            draw_again();
        });
    }
    painted
}

fn read_far(file: &Path, px: u32) -> Option<Pixmap> {
    file.is_file().then(|| paint(file, px)).flatten()
}

fn seek_far(theme: &str, bases: Vec<PathBuf>, pixmaps: PathBuf, name: &str, size: u32, scale: u32) -> Option<PathBuf> {
    let anywhere = Icons { anywhere: true, ..Icons::with(theme, bases, pixmaps) };
    fallbacks(name).iter().find_map(|n| anywhere.search(n, size, scale))
}

pub struct Icons {
    theme: String,
    own: Option<&'static str>,
    bases: Vec<PathBuf>,
    pixmaps: PathBuf,
    anywhere: bool,
    here: RefCell<HashMap<PathBuf, bool>>,
    mounts: std::cell::Cell<usize>,
    elsewhere: std::cell::Cell<bool>,
    themes: RefCell<HashMap<String, Rc<Theme>>>,
    found: RefCell<HashMap<(String, u32, u32), Option<PathBuf>>>,
    painted: RefCell<HashMap<Key, Option<Kept>>>,
    ink: std::cell::Cell<Option<(Colour, Colour)>>,
}

impl Default for Icons {
    fn default() -> Self {
        Self::new()
    }
}

impl Icons {
    pub fn new() -> Icons {
        Icons::with(&chosen(), bases(), PathBuf::from("/usr/share/pixmaps"))
    }

    fn with(theme: &str, bases: Vec<PathBuf>, pixmaps: PathBuf) -> Icons {
        Icons {
            theme: theme.to_string(),
            own: crate::host::host().own_icon_set(),
            bases,
            pixmaps,
            anywhere: false,
            here: RefCell::default(),
            mounts: std::cell::Cell::new(0),
            elsewhere: std::cell::Cell::new(false),
            themes: RefCell::default(),
            found: RefCell::default(),
            painted: RefCell::default(),
            ink: std::cell::Cell::new(None),
        }
    }

    fn is_here(&self, dir: &Path) -> bool {
        if self.anywhere {
            return true;
        }
        let known = self.here.borrow().get(dir).copied();
        let here = known.unwrap_or_else(|| {
            let here = own_place(dir);
            self.here.borrow_mut().insert(dir.to_path_buf(), here);
            here
        });
        if !here {
            self.elsewhere.set(true);
        }
        here
    }

    fn folder(&self, dir: &Path) -> bool {
        self.is_here(dir) && {
            #[cfg(test)]
            tests::looked(dir, true);
            dir.is_dir()
        }
    }

    fn file_in(&self, dir: &Path, name: &str) -> Option<PathBuf> {
        if !self.is_here(dir) {
            return None;
        }
        let file = dir.join(name);
        #[cfg(test)]
        tests::looked(&file, false);
        let kind = std::fs::symlink_metadata(&file).ok()?.file_type();
        if kind.is_symlink() {
            if !self.anywhere && !own_place(&file) {
                self.elsewhere.set(true);
                return None;
            }
            #[cfg(test)]
            tests::looked(&file, true);
            return file.is_file().then_some(file);
        }
        kind.is_file().then_some(file)
    }

    fn theme(&self, name: &str) -> Rc<Theme> {
        if let Some(t) = self.themes.borrow().get(name) {
            if t.elsewhere {
                self.elsewhere.set(true);
            }
            return t.clone();
        }
        let before = self.elsewhere.replace(false);
        let roots: Vec<PathBuf> = self.bases.iter().map(|b| b.join(name)).filter(|p| self.folder(p)).collect();
        let index = roots.iter().find_map(|r| self.file_in(r, "index.theme").and_then(|f| crate::common::files::read_within(&f, INDEX_MOST).ok()));
        let (dirs, inherits) = index.as_deref().map(parse).unwrap_or_default();
        let theme = Rc::new(Theme { roots, dirs, inherits, elsewhere: self.elsewhere.get() });
        self.elsewhere.set(before || theme.elsewhere);
        self.themes.borrow_mut().insert(name.to_string(), theme.clone());
        theme
    }

    fn sought(&self, name: &str, size: u32, scale: u32) -> Option<Option<PathBuf>> {
        let key: Sought = (self.theme.clone(), name.to_string(), size, scale);
        let (step, found) = FAR_FOUND.with_borrow_mut(|far| far_step(far, &key, || FAR_READS.take()));
        match step {
            Step::Is(_) => Some(found),
            Step::Read(place) => {
                let (theme, bases, pixmaps, name) = (self.theme.clone(), self.bases.clone(), self.pixmaps.clone(), name.to_string());
                crate::common::run::on_thread(move || {
                    let _place = place;
                    seek_far(&theme, bases, pixmaps, &name, size, scale)
                }, move |found| {
                    FAR_FOUND.with_borrow_mut(|far| far.insert(key, FarOf::Back(found)));
                    draw_again();
                });
                None
            }
            Step::Waits | Step::Full => None,
        }
    }

    pub fn find(&self, name: &str, size: u32, scale: u32) -> Option<PathBuf> {
        let name = generic::split(name).0;
        let key = (name.to_string(), size, scale);
        if let Some(f) = self.found.borrow().get(&key) {
            return f.clone();
        }
        let file = if is_far(Path::new(name)) {
            Some(PathBuf::from(name))
        } else if Path::new(name).is_absolute() {
            Some(PathBuf::from(name)).filter(|p| p.is_file())
        } else if name.contains('/') {
            None
        } else {
            let mounts = crate::common::files::mounts_seen();
            if self.mounts.replace(mounts) != mounts {
                self.here.borrow_mut().clear();
                self.themes.borrow_mut().clear();
            }
            self.elsewhere.set(false);
            match fallbacks(name).iter().find_map(|n| self.search(n, size, scale.max(1))) {
                Some(file) => Some(file),
                None if self.elsewhere.get() => self.sought(name, size, scale.max(1))?,
                None => None,
            }
        };
        let mut found = self.found.borrow_mut();
        if found.len() >= MOST_PAINTED {
            found.clear();
        }
        found.insert(key, file.clone());
        file
    }

    fn search(&self, name: &str, size: u32, scale: u32) -> Option<PathBuf> {
        let mut seen = Vec::new();
        self.in_theme(&self.theme.clone(), name, size, scale, &mut seen)
            .or_else(|| self.own.and_then(|own| self.in_theme(own, name, size, scale, &mut seen)))
            .or_else(|| self.in_theme("hicolor", name, size, scale, &mut seen))
            .or_else(|| KINDS.iter().find_map(|k| self.file_in(&self.pixmaps, &format!("{name}.{k}"))))
    }

    fn in_theme(&self, theme: &str, name: &str, size: u32, scale: u32, seen: &mut Vec<String>) -> Option<PathBuf> {
        if seen.iter().any(|s| s == theme) {
            return None;
        }
        seen.push(theme.to_string());
        let t = self.theme(theme);
        self.lookup(&t, name, size, scale).or_else(|| {
            t.inherits.iter().find_map(|parent| self.in_theme(parent, name, size, scale, seen))
        })
    }

    fn lookup(&self, theme: &Theme, name: &str, size: u32, scale: u32) -> Option<PathBuf> {
        let files_in = |dir: &Dir| -> Option<PathBuf> {
            theme.roots.iter().find_map(|root| {
                let folder = root.join(&dir.path);
                KINDS.iter().find_map(|k| self.file_in(&folder, &format!("{name}.{k}")))
            })
        };
        if let Some(f) = theme.dirs.iter().filter(|d| d.suits(size, scale)).find_map(files_in) {
            return Some(f);
        }
        let mut best: Option<(u32, PathBuf)> = None;
        for dir in &theme.dirs {
            let d = dir.distance(size, scale);
            if best.as_ref().is_some_and(|(b, _)| d >= *b) {
                continue;
            }
            if let Some(f) = files_in(dir) {
                best = Some((d, f));
            }
        }
        best.map(|(_, f)| f)
    }

    pub fn get(&self, name: &str, size: u32, scale: u32, colour: Colour) -> Option<Rc<Pixmap>> {
        self.kept(name, size, scale, colour).map(|(_, px)| px)
    }

    fn kept(&self, name: &str, size: u32, scale: u32, colour: Colour) -> Option<Kept> {
        let scale = scale.max(1);
        let (plain, who) = generic::split(name);
        let name = plain;
        let Some(file) = self.find(name, size, scale) else {
            let n = generic::number(name)?;
            if let Some((id, shown)) = who {
                crate::host::host().missing_icon(id, shown);
            }
            let key = (format!("generic {name}"), size, scale, None);
            if let Some(p) = self.painted.borrow().get(&key) {
                return p.clone();
            }
            let painted = generic::paint(n, size * scale).map(|px| (crate::paint::new_id(), Rc::new(px)));
            self.keep(key, painted.clone());
            return painted;
        };
        if let Some((id, shown)) = who {
            if generic::number(name).is_some() {
                crate::host::host().missing_icon(id, shown);
            }
        }
        let symbolic = is_symbolic(name) || file.file_stem().is_some_and(|s| is_symbolic(&s.to_string_lossy()));
        let tint = symbolic.then(|| colour.to_skia().to_color_u8()).map(|c| [c.red(), c.green(), c.blue(), c.alpha()]);
        let key = (name.to_string(), size, scale, tint);
        if let Some(p) = self.painted.borrow().get(&key) {
            return p.clone();
        }
        let painted = if is_far(&file) {
            let px = far(&file, size * scale)?;
            Some(if symbolic {
                let mut px = (*px).clone();
                recolour(&mut px, colour);
                Rc::new(px)
            } else {
                px
            })
        } else {
            paint(&file, size * scale).map(|mut px| {
                if symbolic {
                    recolour(&mut px, colour);
                }
                Rc::new(px)
            })
        }.map(|px| (crate::paint::new_id(), px));
        self.keep(key, painted.clone());
        painted
    }

    fn keep(&self, key: Key, painted: Option<Kept>) {
        let mut kept = self.painted.borrow_mut();
        if kept.len() >= MOST_PAINTED {
            kept.clear();
        }
        kept.insert(key, painted);
    }

    pub fn has(&self, name: &str, size: u32, scale: u32) -> bool {
        generic::number(name).is_some() || self.find(name, size, scale).is_some()
    }

    pub fn set_ink(&self, words: Colour, icon: Colour) {
        self.ink.set(Some((words, icon)));
    }

    pub fn draw(&self, canvas: &mut PixmapMut, name: &str, x: i32, y: i32, size: u32, scale: u32, colour: Colour) -> Option<()> {
        let mut drawn = None;
        crate::paint::played(canvas, &crate::shadow::Shadows::default(), |p| drawn = self.draw_on(p, name, x, y, size, scale, colour));
        drawn
    }

    pub(crate) fn draw_on(&self, p: &mut Painter, name: &str, x: i32, y: i32, size: u32, scale: u32, colour: Colour) -> Option<()> {
        let colour = match self.ink.get() {
            Some((words, icon)) if words == colour => icon,
            _ => colour,
        };
        let (id, px) = self.kept(name, size, scale, colour)?;
        #[cfg(any(test, feature = "testing"))]
        if let Some(at) = tiny_skia::IntRect::from_xywh(x, y, px.width(), px.height()).filter(|_| is_symbolic(name)) {
            p.pixels(at, move |c| super::theme::drawn::ink(c, super::theme::drawn::Kind::Icon, colour, (x as f32, y as f32, at.width() as f32, at.height() as f32)));
        }
        p.image(id, px, x, y);
        Some(())
    }
}

fn is_symbolic(name: &str) -> bool {
    name.ends_with(SYMBOLIC)
}

fn fallbacks(name: &str) -> Vec<String> {
    let mut names = vec![name.to_string()];
    if let Some(stem) = name.strip_suffix(SYMBOLIC) {
        let mut stem = stem;
        while let Some((shorter, _)) = stem.rsplit_once('-') {
            names.push(format!("{shorter}{SYMBOLIC}"));
            stem = shorter;
        }
    }
    names
}

fn paint(file: &Path, px: u32) -> Option<Pixmap> {
    let mut out = Pixmap::new(px.max(1), px.max(1))?;
    if file.extension().is_some_and(|e| e == "svg") {
        let tree = picture::svg_bytes(file).and_then(|data| picture::svg_tree(&data));
        let tree = tree.or_else(|| {
            eprintln!("trindle: {}: this icon could not be read", file.display());
            None
        })?;
        let s = tree.size();
        let k = px as f32 / s.width().max(s.height());
        let (dx, dy) = ((px as f32 - s.width() * k) / 2.0, (px as f32 - s.height() * k) / 2.0);
        resvg::render(&tree, Transform::from_row(k, 0.0, 0.0, k, dx, dy), &mut out.as_mut());
    } else {
        let mut limits = image::Limits::default();
        (limits.max_image_width, limits.max_image_height, limits.max_alloc) = (Some(ICON_SIDE), Some(ICON_SIDE), Some(ICON_ALLOC));
        let img = picture::load_within(file, limits)?;
        let (w, h) = (img.width().max(1), img.height().max(1));
        let (fw, fh) = if w >= h { (px, (px * h / w).max(1)) } else { ((px * w / h).max(1), px) };
        picture::draw(&mut out.as_mut(), &img, ((px - fw) / 2) as i32, ((px - fh) / 2) as i32, fw, fh)?;
    }
    Some(out)
}

fn recolour(px: &mut Pixmap, colour: Colour) {
    let c = colour.to_skia().to_color_u8();
    for p in px.data_mut().chunks_exact_mut(4) {
        let a = (p[3] as u32 * c.alpha() as u32 + 127) / 255;
        let weigh = |v: u8| ((v as u32 * a + 127) / 255) as u8;
        p.copy_from_slice(&[weigh(c.red()), weigh(c.green()), weigh(c.blue()), a as u8]);
    }
}

pub fn silhouette(px: &Pixmap, colour: Colour, strip_plate: bool) -> Pixmap {
    let mut out = px.clone();
    if let Some(mask) = strip_plate.then(|| drawing_on_plate(px.data())).flatten() {
        for (p, a) in out.data_mut().chunks_exact_mut(4).zip(mask) {
            p[3] = a;
        }
    }
    recolour(&mut out, colour);
    out
}

fn drawing_on_plate(pixels: &[u8]) -> Option<Vec<u8>> {
    let colour = |p: &[u8]| -> Option<[f32; 3]> {
        let a = p[3] as f32;
        (a >= 128.0).then(|| [p[0] as f32 * 255.0 / a, p[1] as f32 * 255.0 / a, p[2] as f32 * 255.0 / a])
    };
    let total = pixels.len() / 4;
    let mut counts = HashMap::<[u8; 3], usize>::new();
    for p in pixels.chunks_exact(4) {
        if let Some(c) = colour(p) {
            *counts.entry(c.map(|v| v as u8 >> 4)).or_default() += 1;
        }
    }
    let (plate, n) = counts.into_iter().max_by_key(|(_, n)| *n)?;
    if n * 10 < total * 4 {
        return None;
    }
    let (mut sum, mut k) = ([0f32; 3], 0f32);
    for c in pixels.chunks_exact(4).filter_map(colour).filter(|c| c.map(|v| v as u8 >> 4) == plate) {
        (0..3).for_each(|i| sum[i] += c[i]);
        k += 1.0;
    }
    let plate = sum.map(|v| v / k);
    let apart = |p: &[u8]| -> f32 {
        let a = p[3] as f32;
        if a == 0.0 {
            return 0.0;
        }
        (0..3).map(|i| (p[i] as f32 * 255.0 / a - plate[i]).powi(2)).sum::<f32>().sqrt()
    };
    let most = pixels.chunks_exact(4).map(apart).fold(0f32, f32::max);
    if most < 32.0 {
        return None;
    }
    Some(pixels.chunks_exact(4).map(|p| ((apart(p) / most).min(1.0) * p[3] as f32) as u8).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_icon_named_by_a_path_that_may_be_on_a_share_is_never_looked_at_as_the_window_draws() {
        let dir = outside_own_places("far-icon");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("far.svg");
        std::fs::write(&file, r##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="#ff0000"/></svg>"##).unwrap();
        for (path, aside) in [("/mnt/share/icons/a.png", true), ("/home/sam/Pictures/a.png", true), ("/home/sam/.appimage/a.png", true),
                              ("/tmp/a.png", true), ("/run/user/1000/gvfs/smb-share:server=nas,share=films/a.png", true),
                              ("/usr/share/pixmaps/a.png", false), ("folder-symbolic", false)] {
            assert_eq!(is_far(Path::new(path)), aside, "{path}");
        }
        assert!(is_far(&file), "the made-up icon is in a place of its own, and proves nothing");
        let name = file.to_string_lossy().into_owned();
        let icons = Icons::new();
        assert_eq!(icons.find(&name, 16, 1), Some(file.clone()));
        assert!(icons.get(&name, 16, 1, Colour::rgb(1.0, 1.0, 1.0)).is_none(), "an icon that may be on a share was read as the window drew");
        assert!(icons.painted.borrow().is_empty());
        let white = Colour::rgb(1.0, 1.0, 1.0);
        assert!(sent_out(|| icons.get(&name, 16, 1, white).is_none() && FAR.with_borrow(|far| matches!(far.get(&(file.clone(), 16)), Some(FarOf::Out)))));
        let painted = read_far(&file, 16).expect("the thread did not paint the icon");
        assert_eq!(painted.pixel(8, 8).map(|p| (p.red(), p.alpha())), Some((255, 255)));
        FAR.with_borrow_mut(|far| far.insert((file.clone(), 16), FarOf::Back(Some(Rc::new(painted)))));
        assert!(icons.get(&name, 16, 1, Colour::rgb(1.0, 1.0, 1.0)).is_some());
        assert!(Icons::new().get(&name, 16, 1, Colour::rgb(1.0, 1.0, 1.0)).is_some());
        assert!(read_far(&dir, 16).is_none() && read_far(&dir.join("gone.svg"), 16).is_none());
        std::fs::remove_dir_all(&dir).unwrap();
        let code = crate::common::source::code(include_str!("icon.rs"));
        let code = code.split("\n#[cfg(test)]").next().unwrap();
        let body = |from: &str| {
            let rest = &code[code.find(from).unwrap_or_else(|| panic!("{from} is gone"))..];
            &rest[..rest.find("\n}\n").expect("the end of it")]
        };
        for looks in [".is_file()", ".exists()", ".is_dir()", "metadata(", "std::fs::", "paint(", "picture::load("] {
            assert!(!body("\nfn far(").replace("read_far(", "").contains(looks), "far: {looks}");
            assert!(!body("\nfn is_far(").contains(looks), "is_far: {looks}");
        }
        assert_eq!(code.matches("read_far(").count(), 2, "read_far is called from somewhere but the thread");
        assert!(code.contains("let _place = place;"));
        let find = &code[code.find("    pub fn find(").unwrap()..];
        let find = &find[..find.find("\n    }\n").unwrap()];
        assert!(find.find("is_far(Path::new(name))").unwrap() < find.find(".is_file()").unwrap(), "a path is looked at before it is known to be a place of its own");
    }

    #[test]
    fn an_icon_read_on_a_thread_is_read_once() {
        let mut far: HashMap<u32, FarOf<u8>> = HashMap::new();
        let free = || Some(());
        assert_eq!(far_step(&mut far, &1, free), (Step::Read(()), None));
        assert_eq!(far_step(&mut far, &1, free), (Step::Waits, None));
        far.insert(1, FarOf::Back(Some(7)));
        assert_eq!(far_step(&mut far, &1, free), (Step::Is(true), Some(7)));
        far.insert(2, FarOf::Back(None));
        assert_eq!(far_step(&mut far, &2, free), (Step::Is(false), None));
        far.insert(3, FarOf::Out);
        for n in 10..10 + FAR_MOST as u32 {
            far.insert(n, FarOf::Back(Some(0)));
        }
        assert_eq!(far_step(&mut far, &9, free), (Step::Read(()), None));
        assert_eq!(far.len(), 2, "the icons that were back were not forgotten");
        assert!(matches!((far.get(&3), far.get(&9)), (Some(FarOf::Out), Some(FarOf::Out))));
    }

    #[test]
    fn no_more_icons_are_out_being_read_at_once_than_there_are_places() {
        static PLACES: crate::common::run::Slots = crate::common::run::Slots::new(2);
        let mut far: HashMap<u32, FarOf<u8>> = HashMap::new();
        let mut held = Vec::new();
        for n in 0..5 {
            match far_step(&mut far, &n, || PLACES.take()) {
                (Step::Read(place), None) => held.push(place),
                (Step::Full, None) => {}
                _ => panic!("icon {n} was neither sent out nor turned away"),
            }
        }
        assert_eq!((held.len(), far.len()), (2, 2), "reads went out past the places there are");
        assert!(matches!(far_step(&mut far, &4, || PLACES.take()), (Step::Full, None)));
        assert!(!far.contains_key(&4));
        held.pop();
        assert!(matches!(far_step(&mut far, &4, || PLACES.take()), (Step::Read(_), None)));
        let code = crate::common::source::code(include_str!("icon.rs"));
        let code = code.split("\n#[cfg(test)]").next().unwrap();
        assert_eq!(code.matches("far_step(far, &key, || FAR_READS.take())").count(), 2);
        assert_eq!(code.matches("far_step(").count(), 2, "an icon is sent out to be read from a place with no count");
    }

    thread_local! {
        pub(super) static LOOKED: RefCell<Vec<(PathBuf, bool)>> = const { RefCell::new(Vec::new()) };
    }

    pub(super) fn looked(path: &Path, followed: bool) {
        LOOKED.with_borrow_mut(|l| l.push((path.to_path_buf(), followed)));
    }

    fn sent_out(mut out: impl FnMut() -> bool) -> bool {
        for _ in 0..500 {
            if out() {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        false
    }

    #[test]
    fn a_set_kept_outside_the_computers_own_places_is_only_looked_into_on_a_thread() {
        let (root, _) = sets("elsewhere");
        let far = outside_own_places("far-sets");
        let _ = std::fs::remove_dir_all(&far);
        let put = |path: &str, text: &[u8]| {
            let p = far.join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        };
        put("icons/Mine/index.theme", b"[Icon Theme]\nDirectories=24x24/apps\n\n[24x24/apps]\nSize=24\nType=Fixed\n");
        put("icons/Mine/24x24/apps/remote.svg", SQUARE.as_bytes());
        put("icons/Mine/24x24/apps/editor.svg", SQUARE.as_bytes());
        std::os::unix::fs::symlink("/proc/version", root.join("icons/Mine/24x24/apps/linked.svg")).unwrap();
        std::os::unix::fs::symlink("/proc/self", root.join("icons/Linked")).unwrap();
        assert!(!own_place(&root.join("icons/Mine/24x24/apps/linked.svg")) && !own_place(&root.join("icons/Linked")), "the made-up links prove nothing");
        std::os::unix::fs::symlink("editor.png", root.join("icons/Mine/24x24/apps/writer.png")).unwrap();
        assert!(is_far(&far.join("icons")) && !is_far(&root.join("icons")), "the made-up places prove nothing");
        let bases = vec![far.join("icons"), root.join("icons")];
        let icons = Icons::with("Mine", bases.clone(), root.join("pixmaps"));
        LOOKED.with_borrow_mut(Vec::clear);
        let out = |name: &str| FAR_FOUND.with_borrow(|f| matches!(f.get(&("Mine".to_string(), name.to_string(), 24, 1)), Some(FarOf::Out)));
        assert_eq!(icons.find("editor", 24, 1), Some(root.join("icons/Mine/24x24/apps/editor.png")));
        assert_eq!(icons.find("writer", 24, 1), Some(root.join("icons/Mine/24x24/apps/writer.png")));
        assert!(!out("editor") && !out("writer"));
        for name in ["remote", "linked", "nowhere"] {
            assert!(sent_out(|| icons.find(name, 24, 1).is_none() && out(name)), "{name}: no thread went to look for it");
            assert!(!icons.found.borrow().contains_key(&(name.to_string(), 24, 1)), "{name} was remembered as missing before the thread was back");
        }
        let looked = LOOKED.with_borrow(|l| l.clone());
        assert!(looked.len() > 4, "the looks were not noted");
        for (path, followed) in &looked {
            assert!(!path.starts_with(&far), "{} was looked at as the window drew", path.display());
            assert!(!(*followed && (path.ends_with("linked.svg") || path.starts_with(root.join("icons/Linked")))), "the link {} was followed as the window drew", path.display());
        }
        let seek = |name: &str| seek_far("Mine", bases.clone(), root.join("pixmaps"), name, 24, 1);
        assert_eq!(seek("remote"), Some(far.join("icons/Mine/24x24/apps/remote.svg")));
        assert_eq!(seek("linked"), Some(root.join("icons/Mine/24x24/apps/linked.svg")));
        let linked = Icons::with("Linked", vec![root.join("icons")], root.join("pixmaps"));
        LOOKED.with_borrow_mut(Vec::clear);
        assert!(sent_out(|| linked.find("inside", 24, 1).is_none() && FAR_FOUND.with_borrow(|f| f.contains_key(&("Linked".to_string(), "inside".to_string(), 24, 1)))));
        assert!(LOOKED.with_borrow(|l| l.iter().all(|(path, _)| !path.starts_with(root.join("icons/Linked")))), "a set that is a link to elsewhere was looked into as the window drew");
        assert_eq!(seek("nowhere"), None);
        for name in ["remote", "nowhere"] {
            FAR_FOUND.with_borrow_mut(|f| f.insert(("Mine".to_string(), name.to_string(), 24, 1), FarOf::Back(seek(name))));
        }
        assert_eq!(icons.find("remote", 24, 1), seek("remote"));
        assert_eq!(Icons::with("Mine", bases.clone(), root.join("pixmaps")).find("remote", 24, 1), seek("remote"));
        assert_eq!(icons.find("nowhere", 24, 1), None);
        assert!(icons.found.borrow().contains_key(&("nowhere".to_string(), 24, 1)));
        let file = seek("remote").unwrap();
        assert!(sent_out(|| icons.get("remote", 24, 1, Colour::rgb(1.0, 1.0, 1.0)).is_none() && FAR.with_borrow(|f| matches!(f.get(&(file.clone(), 24)), Some(FarOf::Out)))));
        for dir in [&root, &far] {
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn a_search_on_this_computer_sends_nothing_out_and_every_look_asks_first() {
        let (plain, here) = sets("elsewhere-none");
        assert_eq!(here.find("nowhere-at-all", 24, 1), None);
        assert!(here.found.borrow().contains_key(&("nowhere-at-all".to_string(), 24, 1)));
        assert!(FAR_FOUND.with_borrow(|f| !f.contains_key(&("Mine".to_string(), "nowhere-at-all".to_string(), 24, 1))));
        let code = crate::common::source::code(include_str!("icon.rs"));
        let code = code.split("\n#[cfg(test)]").next().unwrap();
        let body = |from: &str| {
            let rest = &code[code.find(from).unwrap_or_else(|| panic!("{from} is gone"))..];
            &rest[..rest.find("\n    }\n").expect("the end of it")]
        };
        for (part, most) in [("    fn theme(", 0), ("    fn search(", 0), ("    fn in_theme(", 0), ("    fn lookup(", 0), ("    fn sought(", 0)] {
            let looks: usize = [".is_file()", ".exists()", ".is_dir()", "metadata(", "std::fs::", "read_dir("].iter().map(|l| body(part).matches(l).count()).sum();
            assert_eq!(looks, most, "{part} looks at the disk for itself");
        }
        assert!(body("    fn theme(").contains("self.file_in(r, \"index.theme\").and_then(|f| crate::common::files::read_within(&f, INDEX_MOST).ok())"));
        for gate in ["    fn folder(", "    fn file_in("] {
            assert!(body(gate).find("self.is_here(dir)").unwrap() < body(gate).find("looked(").unwrap(), "{gate} looks before it asks");
        }
        assert_eq!(code.matches("seek_far(").count(), 2, "seek_far is called from somewhere but the thread");
        assert!(code.contains("let _place = place;"));
        let bases_fn = &code[code.find("\nfn bases(").unwrap()..];
        assert!(!bases_fn[..bases_fn.find("\n}\n").unwrap()].contains(".is_dir()"), "the folders are looked at as they are listed");
        std::fs::remove_dir_all(&plain).unwrap();
    }

    fn picture(px: &[(u8, u8, u8, u8)]) -> Pixmap {
        let mut out = Pixmap::new(px.len() as u32, 1).unwrap();
        for (dst, &(r, g, b, a)) in out.data_mut().chunks_exact_mut(4).zip(px) {
            let m = |v: u8| (v as u32 * a as u32 / 255) as u8;
            dst.copy_from_slice(&[m(r), m(g), m(b), a]);
        }
        out
    }

    #[test]
    fn a_silhouette_keeps_the_shape_in_one_colour() {
        let px = picture(&[(250, 10, 10, 255), (10, 250, 10, 128), (0, 0, 0, 0)]);
        let out = silhouette(&px, Colour::rgb(1.0, 1.0, 1.0), false);
        assert_eq!(out.data(), &[255, 255, 255, 255, 128, 128, 128, 128, 0, 0, 0, 0]);
    }

    #[test]
    fn a_dark_drawing_on_a_yellow_disc_keeps_only_the_drawing() {
        let yellow = (250, 200, 30, 255);
        let mut px = vec![yellow; 12];
        px.extend([(20, 20, 30, 255), (20, 20, 30, 255), (0, 0, 0, 0), (0, 0, 0, 0)]);
        let out = silhouette(&picture(&px), Colour::rgb(1.0, 1.0, 1.0), true);
        let a = |i: usize| out.data()[i * 4 + 3];
        assert_eq!(a(0), 0, "the disc goes");
        assert_eq!(a(12), 255, "the drawing stays solid");
        assert_eq!(a(14), 0, "outside the disc stays clear");
    }

    #[test]
    fn an_outline_or_a_plain_square_keeps_its_shape() {
        let mut px = vec![(0, 0, 0, 0); 16];
        for i in [0, 3, 12, 15] {
            px[i] = (80, 80, 80, 255);
        }
        assert!(drawing_on_plate(picture(&px).data()).is_none());
        assert!(drawing_on_plate(picture(&[(250, 200, 30, 255); 16]).data()).is_none());
    }

    const SQUARE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect x="4" y="4" width="8" height="8" fill="#bebebe"/></svg>"##;

    fn outside_own_places(name: &str) -> PathBuf {
        let spots = [std::env::temp_dir(), PathBuf::from("/tmp"), PathBuf::from("/var/tmp"), crate::common::source::toolkit_root().join("target")];
        spots.iter().map(|spot| spot.join(format!("{name}-{}", std::process::id()))).find(|dir| !own_place(dir)).expect("a folder outside the computer's own places")
    }

    fn sets(name: &str) -> (PathBuf, Icons) {
        let beside = std::env::current_exe().unwrap().ancestors().nth(2).expect("the folder the build is in").join("share");
        let root = beside.join(format!("trindle-icon-{name}-{}", std::process::id()));
        assert!(!is_far(&root.join("icons/Mine/24x24/apps/editor.png")), "the made-up icon sets are not in a place of Trindle's own");
        let _ = std::fs::remove_dir_all(&root);
        let put = |path: &str, text: &[u8]| {
            let p = root.join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        };
        let png = |size: u32| {
            let mut out = std::io::Cursor::new(Vec::new());
            image::RgbaImage::from_pixel(size, size, image::Rgba([200, 0, 0, 255]))
                .write_to(&mut out, image::ImageFormat::Png).unwrap();
            out.into_inner()
        };
        put("icons/Mine/index.theme", b"[Icon Theme]\nInherits=Base\nDirectories=24x24/apps\n\n[24x24/apps]\nSize=24\nType=Fixed\n");
        put("icons/Mine/24x24/apps/editor.png", &png(24));
        put("icons/Base/index.theme", b"[Icon Theme]\nInherits=Mine\nDirectories=16x16/apps,48x48/apps,symbolic/actions\nScaledDirectories=16x16@2x/apps\n\n\
            [16x16/apps]\nSize=16\nType=Fixed\n\n[16x16@2x/apps]\nSize=16\nScale=2\nType=Fixed\n\n[48x48/apps]\nSize=48\nType=Fixed\n\n\
            [symbolic/actions]\nSize=16\nMinSize=8\nMaxSize=512\nType=Scalable\n");
        put("icons/Base/16x16/apps/editor.png", &png(16));
        put("icons/Base/16x16/apps/viewer.png", &png(16));
        put("icons/Base/16x16@2x/apps/viewer.png", &png(32));
        put("icons/Base/48x48/apps/viewer.png", &png(48));
        put("icons/Base/symbolic/actions/go-next-symbolic.svg", SQUARE.as_bytes());
        put("icons/hicolor/index.theme", b"[Icon Theme]\nDirectories=scalable/apps\n\n[scalable/apps]\nSize=64\nMinSize=1\nMaxSize=512\nType=Scalable\n");
        put("icons/hicolor/scalable/apps/player.svg", SQUARE.as_bytes());
        put("pixmaps/old.png", &png(32));
        let icons = Icons::with("Mine", vec![root.join("icons")], root.join("pixmaps"));
        (root, icons)
    }

    #[test]
    fn the_chosen_set_then_what_it_builds_on_then_hicolor_then_pixmaps() {
        let (root, icons) = sets("order");
        let at = |p: &str| Some(root.join("icons").join(p));
        assert_eq!(icons.find("editor", 24, 1), at("Mine/24x24/apps/editor.png"));
        assert_eq!(icons.find("viewer", 16, 1), at("Base/16x16/apps/viewer.png"));
        assert_eq!(icons.find("player", 24, 1), at("hicolor/scalable/apps/player.svg"));
        assert_eq!(icons.find("old", 24, 1), Some(root.join("pixmaps/old.png")));
        assert_eq!(icons.find("nothing", 24, 1), None);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn with_no_set_chosen_the_default_set_still_gives_its_icons() {
        let (root, _) = sets("default");
        let set = root.join("icons").join("Own-Set");
        std::fs::create_dir_all(set.join("symbolic/actions")).unwrap();
        std::fs::write(set.join("index.theme"), "[Icon Theme]\nInherits=hicolor\nDirectories=symbolic/actions\n\n\
            [symbolic/actions]\nSize=16\nMinSize=8\nMaxSize=512\nType=Scalable\n").unwrap();
        std::fs::write(set.join("symbolic/actions/edit-copy-symbolic.svg"), SQUARE).unwrap();
        let icons = Icons { own: Some("Own-Set"), ..Icons::with("hicolor", vec![root.join("icons")], root.join("pixmaps")) };
        assert_eq!(icons.find("edit-copy-symbolic", 16, 1), Some(set.join("symbolic/actions/edit-copy-symbolic.svg")));
        assert_eq!(icons.find("player", 24, 1), Some(root.join("icons/hicolor/scalable/apps/player.svg")));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_nearest_size_and_the_screen_scale_are_chosen() {
        let (root, icons) = sets("size");
        let at = |p: &str| Some(root.join("icons").join(p));
        assert_eq!(icons.find("viewer", 40, 1), at("Base/48x48/apps/viewer.png"));
        assert_eq!(icons.find("viewer", 20, 1), at("Base/16x16/apps/viewer.png"));
        assert_eq!(icons.find("viewer", 16, 2), at("Base/16x16@2x/apps/viewer.png"));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_missing_symbolic_icon_falls_back_to_a_shorter_name() {
        assert_eq!(fallbacks("go-next-rtl-symbolic"), ["go-next-rtl-symbolic", "go-next-symbolic", "go-symbolic"]);
        assert_eq!(fallbacks("editor"), ["editor"]);
        let (root, icons) = sets("fallback");
        assert!(icons.find("go-next-rtl-symbolic", 16, 1).is_some());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn symbolic_icons_take_the_colour_asked_for_and_others_keep_theirs() {
        let (root, icons) = sets("paint");
        let white = Colour::rgb(1.0, 1.0, 1.0);
        let px = icons.get("go-next-symbolic", 32, 1, white).unwrap();
        assert_eq!((px.width(), px.height()), (32, 32));
        let mid = px.pixel(16, 16).unwrap();
        assert_eq!((mid.red(), mid.green(), mid.blue(), mid.alpha()), (255, 255, 255, 255));
        assert_eq!(px.pixel(2, 2).unwrap().alpha(), 0);
        let player = icons.get("player", 32, 1, white).unwrap();
        assert_eq!(player.pixel(16, 16).unwrap().red(), 0xbe);
        let editor = icons.get("editor", 24, 2, white).unwrap();
        assert_eq!((editor.width(), editor.pixel(24, 24).unwrap().red()), (48, 200));
        let mut canvas = Pixmap::new(40, 40).unwrap();
        icons.draw(&mut canvas.as_mut(), "go-next-symbolic", 4, 4, 32, 1, white).unwrap();
        assert_eq!(canvas.pixel(20, 20).unwrap().alpha(), 255);
        let mut p = crate::paint::Painter::default();
        for _ in 0..2 {
            icons.draw_on(&mut p, "player", 3, 5, 32, 1, white).unwrap();
        }
        let ids: Vec<u64> = p.ops.iter().filter_map(|op| match op { crate::paint::Op::Image { id, .. } => Some(*id), _ => None }).collect();
        assert!(ids.len() == 2 && ids[0] == ids[1], "a kept icon is drawn by the same id each time");
        let mut q = crate::paint::Painter::default();
        for _ in 0..2 {
            icons.draw_on(&mut q, "trindle-generic-03", 3, 5, 32, 1, white).unwrap();
        }
        let ids: Vec<u64> = q.ops.iter().filter_map(|op| match op { crate::paint::Op::Image { id, .. } => Some(*id), _ => None }).collect();
        assert!(ids.len() == 2 && ids[0] == ids[1], "a generic icon is drawn by the same id each time");
        let (mut played, mut direct) = (Pixmap::new(40, 40).unwrap(), Pixmap::new(40, 40).unwrap());
        crate::paint::cpu::play(p.ops, &mut played.as_mut(), &crate::shadow::Shadows::default());
        direct.draw_pixmap(3, 5, player.as_ref().as_ref(), &tiny_skia::PixmapPaint::default(), Transform::identity(), None);
        assert!(played.data() == direct.data());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_index_theme_reads() {
        let (dirs, inherits) = parse("[Icon Theme]\nInherits=A, B\nDirectories=x,y\n\n[x]\nSize=22\nScale=2\nType=Threshold\n\n[y]\nType=Fixed\n");
        assert_eq!(inherits, ["A", "B"]);
        assert_eq!(dirs.len(), 1);
        assert_eq!((dirs[0].size, dirs[0].scale, dirs[0].fit), (22, 2, Fit::Threshold));
        assert!(dirs[0].suits(20, 2) && !dirs[0].suits(20, 1) && !dirs[0].suits(25, 2));
    }

    #[test]
    fn sizes_in_an_index_theme_cannot_overflow() {
        let (dirs, _) = parse("[Icon Theme]\nDirectories=x\n\n[x]\nSize=4294967295\nScale=4294967295\nThreshold=4294967295\n");
        assert!(!dirs[0].suits(16, 1));
        assert!(dirs[0].distance(u32::MAX >> 14, 3) < u32::MAX);
    }
}
