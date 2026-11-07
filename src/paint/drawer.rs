//! Who draws a window: the GPU when every check passes, else the processor; and what a window the GPU draws keeps.

use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU8, Ordering};
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use tiny_skia::Pixmap;

use super::gpu::{self, Gpu, Target};
use super::log::{say, Kind};
use super::remote::{self, Sending};
use super::{changes, Painter, Seen};
use crate::shadow::Shadows;
use crate::sys::{gbm, gl};
use crate::window::dmabuf::{Alloc, Drawn, Frame};
use crate::window::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    Asked,
    NoDmabuf,
    NoLibraries,
    NoNode,
    NoContext,
    Software,
    SelfTest,
    Failed,
    NoService,
    ServiceOtherGpu,
    ServiceGone,
}

impl Why {
    pub fn words(self) -> &'static str {
        match self {
            Why::Asked => "the processor was asked for, by the program or TRINDLE_DRAWN_BY_PROCESSOR=1",
            Why::NoDmabuf => "the compositor takes no Argb8888 dmabufs",
            Why::NoLibraries => "the graphics libraries are missing",
            Why::NoNode => "the render node the compositor names cannot be opened",
            Why::NoContext => "no GL context could be made",
            Why::Software => "GL is drawn by software, slower than the processor",
            Why::SelfTest => "the GPU's test frame differs from the processor's",
            Why::Failed => "the GPU failed after it was chosen",
            Why::NoService => "no drawing service answers",
            Why::ServiceOtherGpu => "the drawing service draws on another GPU than the compositor's",
            Why::ServiceGone => "the drawing service went away or stopped answering",
        }
    }
}

pub(crate) fn choose<D, G>(asked: bool, offered: bool, node: impl FnOnce() -> Option<PathBuf>, libraries: impl FnOnce() -> bool,
                           open: impl FnOnce(&Path) -> Option<D>, context: impl FnOnce(&D) -> Option<G>,
                           renderer: impl FnOnce(&G) -> String, test: impl FnOnce(&mut G) -> bool) -> Result<(D, G), Why> {
    if asked {
        return Err(Why::Asked);
    }
    if !offered {
        return Err(Why::NoDmabuf);
    }
    let node = node().ok_or(Why::NoNode)?;
    if !libraries() {
        return Err(Why::NoLibraries);
    }
    let device = open(&node).ok_or(Why::NoNode)?;
    let mut g = context(&device).ok_or(Why::NoContext)?;
    if gl::is_software(&renderer(&g)) {
        return Err(Why::Software);
    }
    if !test(&mut g) {
        return Err(Why::SelfTest);
    }
    Ok((device, g))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawnBy {
    #[default]
    Auto,
    Processor,
    Gpu,
}

static CHOICE: AtomicU8 = AtomicU8::new(0);

pub fn drawn_by(choice: DrawnBy) {
    CHOICE.store(choice as u8, Ordering::Relaxed);
}

fn chosen() -> DrawnBy {
    match CHOICE.load(Ordering::Relaxed) {
        1 => DrawnBy::Processor,
        2 => DrawnBy::Gpu,
        _ => DrawnBy::Auto,
    }
}

fn wants_gpu(choice: DrawnBy, env_processor: bool, env_gpu: bool) -> bool {
    if env_processor {
        return false;
    }
    env_gpu || match choice {
        DrawnBy::Processor => false,
        DrawnBy::Auto | DrawnBy::Gpu => true,
    }
}

fn wants_service(env_processor: bool, env_service: bool) -> bool {
    env_service && !env_processor
}

fn on(key: &str) -> bool {
    std::env::var_os(key).is_some_and(|v| v == "1")
}

pub(crate) struct Shared {
    pub(crate) device: Rc<gbm::Device>,
    gpu: RefCell<Option<Gpu>>,
    lost: Cell<bool>,
    windows: Cell<usize>,
    asleep: Cell<usize>,
    #[cfg(test)]
    _alone: crate::sys::gl::Exclusive,
}

impl Shared {
    fn new(device: Rc<gbm::Device>, gpu: Gpu) -> Shared {
        Shared { device, gpu: RefCell::new(Some(gpu)), lost: Cell::new(false), windows: Cell::new(0), asleep: Cell::new(0),
                 #[cfg(test)]
                 _alone: crate::sys::gl::Exclusive::take() }
    }

    #[cfg(test)]
    pub(crate) fn awake(&self) -> bool {
        self.gpu.borrow().is_some()
    }

    #[cfg(test)]
    pub(crate) fn gpu(&self) -> std::cell::Ref<'_, Gpu> {
        std::cell::Ref::map(self.gpu.borrow(), |g| g.as_ref().expect("the painter is made"))
    }

    #[cfg(test)]
    pub(crate) fn gpu_mut(&self) -> std::cell::RefMut<'_, Gpu> {
        std::cell::RefMut::map(self.gpu.borrow_mut(), |g| g.as_mut().expect("the painter is made"))
    }
}

thread_local! {
    static CHOSEN: RefCell<Option<(PathBuf, Result<Rc<Shared>, Why>)>> = const { RefCell::new(None) };
}

pub(crate) fn gpu_for(display: &Display) -> Result<Rc<Shared>, Why> {
    let env_gpu = on("TRINDLE_DRAWN_BY_GPU") || on("TRINDLE_DRAWN_BY_SERVICE");
    let asked = !wants_gpu(chosen(), crate::theme::drawn_by_processor(), env_gpu);
    let offered = !asked && display.takes_dmabufs();
    let node = if offered { display.dmabuf_node() } else { None };
    let known = CHOSEN.with_borrow(|c| c.as_ref().filter(|(n, _)| Some(n) == node.as_ref()).map(|(_, r)| r.clone()));
    if let Some(r) = known {
        return r.and_then(|s| if s.lost.get() { Err(Why::Failed) } else { Ok(s) });
    }
    let r = choose(asked, offered, || node.clone(), gl::libraries, |n| gbm::Device::new(gbm::open_node(n)?),
                   |d| Gpu::on(d.clone()), Gpu::renderer, gpu::agrees)
        .map(|(device, gpu)| Rc::new(Shared::new(device, gpu)));
    if let (Err(why), false) = (&r, asked) {
        say(Kind::Processor, || format!("the processor draws: {why:?}"));
    }
    if let Some(n) = node.filter(|_| !matches!(r, Err(Why::NoNode))) {
        CHOSEN.set(Some((n, r.clone())));
    }
    r
}

pub(crate) fn way_for(display: &Display, events: &crate::event::Loop) -> Result<(Painting, Alloc), Why> {
    if wants_service(crate::theme::drawn_by_processor(), on("TRINDLE_DRAWN_BY_SERVICE")) && display.takes_dmabufs() {
        match display.dmabuf_node().ok_or(Why::NoNode).and_then(|node| remote::connected(&node, events)) {
            Ok(r) => return Ok((Painting::There(Sending::new(r.clone())), Alloc::Service(r))),
            Err(why) => eprintln!("trindle: the GPU in this program draws: {why:?} ({})", why.words()),
        }
    }
    gpu_for(display).map(|gpu| (Painting::Here(Drawing::new(gpu.clone())), Alloc::Gbm(gpu.device.clone())))
}

pub(crate) trait Draws {
    fn draw(&mut self, frame: &Frame, p: Painter, shadows: &Shadows) -> Drawn;
}

impl Draws for Drawing {
    fn draw(&mut self, frame: &Frame, p: Painter, shadows: &Shadows) -> Drawn {
        Drawing::draw(self, frame, p, shadows)
    }
}

pub(crate) enum Painting {
    Here(Drawing),
    There(Sending),
}

impl Draws for Painting {
    fn draw(&mut self, frame: &Frame, p: Painter, shadows: &Shadows) -> Drawn {
        match self {
            Painting::Here(d) => d.draw(frame, p, shadows),
            Painting::There(s) => s.draw(frame, p, shadows),
        }
    }
}

impl Painting {
    pub(crate) fn sleep(&mut self, yes: bool) {
        match self {
            Painting::Here(d) => d.sleep(yes),
            Painting::There(s) => s.sleep(yes),
        }
    }
}

pub(crate) fn part_to_draw(changed: &Option<Vec<[i32; 4]>>, frame: &Frame) -> Option<[i32; 4]> {
    let again = changed.as_ref().filter(|_| frame.age > 0).map(|c| c.iter().chain(frame.behind).fold(None, |all: Option<[i32; 4]>, r| {
        let r = [r[0], r[1], r[0] + r[2], r[1] + r[3]];
        Some(all.map_or(r, |a| [a[0].min(r[0]), a[1].min(r[1]), a[2].max(r[2]), a[3].max(r[3])]))
    }));
    if again.is_none() {
        say(Kind::Whole, || format!("whole frame drawn: {}", if changed.is_none() { "too much changed to list" } else { "the buffer's age is unknown" }));
    }
    again.unwrap_or(Some([0, 0, frame.size.0 as i32, frame.size.1 as i32]))
}

pub(crate) struct Drawing {
    shared: Rc<Shared>,
    targets: Vec<(u64, Target)>,
    seen: Seen,
    scratch: Option<Pixmap>,
    asleep: bool,
}

impl Drawing {
    pub(crate) fn new(shared: Rc<Shared>) -> Drawing {
        shared.windows.set(shared.windows.get() + 1);
        Drawing { shared, targets: Vec::new(), seen: Seen::new(), scratch: None, asleep: false }
    }

    pub(crate) fn sleep(&mut self, yes: bool) {
        if yes == self.asleep {
            return;
        }
        self.asleep = yes;
        let s = &self.shared;
        if !yes {
            s.asleep.set(s.asleep.get().saturating_sub(1));
            return;
        }
        s.asleep.set(s.asleep.get() + 1);
        self.seen = Seen::new();
        let Ok(mut slot) = s.gpu.try_borrow_mut() else { return };
        if let Some(gpu) = slot.as_ref().filter(|g| g.current()) {
            for (_, t) in self.targets.drain(..) {
                gpu.free(t);
            }
        }
        self.targets.clear();
        if s.asleep.get() >= s.windows.get() && slot.take().is_some() {
            say(Kind::Whole, || "every window is out of sight: the GPU painter, its atlases and context are freed".into());
        }
    }

    pub(crate) fn draw(&mut self, frame: &Frame, p: Painter, shadows: &Shadows) -> Drawn {
        self.sleep(false);
        let (Some(d), Ok(mut slot)) = (frame.dmabuf, self.shared.gpu.try_borrow_mut()) else {
            say(Kind::Processor, || "the processor draws: no buffer to draw into, or the GPU painter in use".into());
            return Drawn::Failed;
        };
        if slot.is_none() {
            *slot = Gpu::on(self.shared.device.clone());
            say(Kind::Whole, || format!("the GPU painter made again: {}", if slot.is_some() { "ready" } else { "no context" }));
        }
        let Some(gpu) = slot.as_mut() else {
            self.shared.lost.set(true);
            return Drawn::Failed;
        };
        if self.shared.lost.get() || !gpu.current() {
            self.shared.lost.set(true);
            say(Kind::Processor, || "the processor draws: the GPU context is lost".into());
            return Drawn::Failed;
        }
        gpu.error();
        if !self.targets.iter().any(|(id, _)| *id == frame.id) {
            let Some(t) = gpu.target_of(d, frame.modifier) else {
                say(Kind::Processor, || format!("the processor draws: the buffer (layout {:?}) cannot be drawn into", frame.modifier));
                return Drawn::Failed;
            };
            if self.targets.len() >= 2 {
                gpu.free(self.targets.remove(0).1);
            }
            self.targets.push((frame.id, t));
        }
        let Some((_, t)) = self.targets.iter().find(|(id, _)| *id == frame.id) else { return Drawn::Failed };
        let (ops, changed) = changes(p, frame.size, &mut self.seen, &mut self.scratch);
        let changed: Option<Vec<[i32; 4]>> = changed.map(|c| c.iter().map(|r| [r.x(), r.y(), r.width() as i32, r.height() as i32]).collect());
        if let Some(part) = part_to_draw(&changed, frame) {
            gpu.play_part(ops, t, shadows, part);
            if let Some(fence) = gpu.fence() {
                gbm::wait_for(d.planes[0].fd.as_fd(), fence);
            }
        }
        let error = gpu.error();
        if error != 0 {
            say(Kind::Processor, || format!("the processor draws: GL error {error:#x}"));
            return Drawn::Failed;
        }
        changed.map_or(Drawn::Whole, Drawn::Parts)
    }
}

impl Drop for Drawing {
    fn drop(&mut self) {
        let (s, windows) = (&self.shared, &self.shared.windows);
        windows.set(windows.get().saturating_sub(1));
        if self.asleep {
            s.asleep.set(s.asleep.get().saturating_sub(1));
        }
        let Ok(slot) = s.gpu.try_borrow() else { return };
        if let Some(gpu) = slot.as_ref().filter(|g| g.current()) {
            for (_, t) in self.targets.drain(..) {
                gpu.free(t);
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn here() -> Rc<Shared> {
    let _alone = crate::sys::gl::Exclusive::take();
    let node = gbm::render_nodes().into_iter().next().expect("a render node");
    let device = gbm::Device::new(gbm::open_node(&node).expect("the node opens")).expect("a GBM device");
    let mut gpu = Gpu::on(device.clone()).expect("a context on the render node");
    assert!(!gl::is_software(&gpu.renderer()), "{}", gpu.renderer());
    assert!(gpu::agrees(&mut gpu), "the self-test");
    Rc::new(Shared::new(device, gpu))
}

#[cfg(test)]
mod soak;

#[cfg(test)]
mod tests {
    use super::*;

    fn with(fail: usize) -> Result<(u8, u8), Why> {
        let ok = |n: usize| n != fail;
        let renderer = if ok(6) { "AMD Radeon 760M (radeonsi)" } else { "llvmpipe (LLVM 19.1.7, 256 bits)" };
        choose(!ok(0), ok(1), || ok(2).then(|| PathBuf::from("/dev/dri/renderD128")), || ok(3), |_| ok(4).then_some(1),
               |_| ok(5).then_some(2), |_| renderer.to_string(), |_| ok(7))
    }

    #[test]
    fn each_failed_check_picks_the_processor() {
        let why = [Why::Asked, Why::NoDmabuf, Why::NoNode, Why::NoLibraries, Why::NoNode, Why::NoContext, Why::Software, Why::SelfTest];
        for (n, w) in why.iter().enumerate() {
            assert_eq!(with(n), Err(*w), "check {n}");
        }
        assert_eq!(with(usize::MAX), Ok((1, 2)));
    }

    fn buffer(s: &Shared, w: u32, h: u32) -> (gbm::Buffer, gbm::Dmabuf) {
        buffer_with(s, w, h, &[gbm::MOD_LINEAR])
    }

    pub(super) fn buffer_with(s: &Shared, w: u32, h: u32, modifiers: &[u64]) -> (gbm::Buffer, gbm::Dmabuf) {
        let b = s.device.buffer(w, h, gbm::ARGB8888, modifiers).expect("a buffer");
        let d = b.export().expect("exported");
        (b, d)
    }

    fn frame<'a>(d: &'a gbm::Dmabuf, id: u64, age: u32, behind: &'a [[i32; 4]]) -> Frame<'a> {
        Frame { dmabuf: Some(d), modifier: Some(gbm::MOD_LINEAR), id, size: (d.width, d.height), scale: 1.0, age, behind }
    }

    fn apart(want: &tiny_skia::Pixmap, d: &gbm::Dmabuf) -> u8 {
        let got = gbm::read_linear(d).expect("the buffer read");
        let (w, stride) = (want.width() as usize, d.planes[0].stride as usize);
        want.data().chunks_exact(4).enumerate().map(|(i, px)| {
            let at = d.planes[0].offset as usize + (i / w) * stride + (i % w) * 4;
            [px[2], px[1], px[0], px[3]].iter().zip(&got[at..at + 4]).map(|(a, b)| a.abs_diff(*b)).max().unwrap_or(0)
        }).max().unwrap_or(0)
    }

    #[test]
    #[ignore = "needs a GPU and Mesa"]
    fn a_window_of_widgets_is_drawn_by_the_gpu_within_two_levels_of_the_processor() {
        let shared = here();
        let (w, h) = (300, 150);
        for (name, theme) in crate::theme::tests::every_look() {
            let ui = || {
                let mut ui = crate::ui::Ui::new(crate::gallery::golden::parts(), theme.clone());
                ui.lay_at(w, h);
                ui
            };
            let mut want = tiny_skia::Pixmap::new(w, h).unwrap();
            ui().paint(&mut want.as_mut(), 1.0);
            let (_b, d) = buffer(&shared, w, h);
            assert!(matches!(ui().paint_by_gpu(&frame(&d, 1, 0, &[]), &mut Drawing::new(shared.clone())), Drawn::Whole), "{name}: drawn");
            assert!(apart(&want, &d) <= 2, "{name}: {} levels apart", apart(&want, &d));
        }
        assert_eq!(shared.gpu().error(), 0);
    }

    #[test]
    #[ignore = "needs a GPU and Mesa"]
    fn the_painter_is_freed_while_every_window_is_out_of_sight_and_made_again_by_the_next_frame() {
        let shared = here();
        let (w, h) = (300, 150);
        let theme = crate::theme::Theme::new(true, "#fbc711");
        let ui = || {
            let mut ui = crate::ui::Ui::new(crate::gallery::golden::parts(), theme.clone());
            ui.lay_at(w, h);
            ui
        };
        let mut want = Pixmap::new(w, h).unwrap();
        ui().paint(&mut want.as_mut(), 1.0);
        let (mut first, mut second) = (Drawing::new(shared.clone()), Drawing::new(shared.clone()));
        let (_b1, d1) = buffer(&shared, w, h);
        let (_b2, d2) = buffer(&shared, w, h);
        assert!(matches!(ui().paint_by_gpu(&frame(&d1, 1, 0, &[]), &mut first), Drawn::Whole));
        assert!(matches!(ui().paint_by_gpu(&frame(&d2, 2, 0, &[]), &mut second), Drawn::Whole));
        first.sleep(true);
        assert!(shared.awake(), "the second window still shows");
        assert!(first.targets.is_empty());
        second.sleep(true);
        assert!(!shared.awake(), "every window is out of sight");
        assert!(second.targets.is_empty());
        let (_b3, d3) = buffer(&shared, w, h);
        assert!(matches!(ui().paint_by_gpu(&frame(&d3, 3, 0, &[]), &mut first), Drawn::Whole), "made again by the next frame");
        assert!(shared.awake());
        assert!(apart(&want, &d3) <= 2, "{} levels apart", apart(&want, &d3));
        assert_eq!(shared.gpu().error(), 0);
        first.sleep(true);
        assert!(!shared.awake(), "the first is out of sight again");
        drop(second);
        let (_b4, d4) = buffer(&shared, w, h);
        assert!(matches!(ui().paint_by_gpu(&frame(&d4, 4, 0, &[]), &mut first), Drawn::Whole));
        assert!(shared.awake() && apart(&want, &d4) <= 2);
    }

    #[test]
    #[ignore = "needs a GPU and Mesa"]
    fn the_whole_gallery_is_drawn_by_the_gpu_within_two_levels_of_the_processor_in_every_look() {
        let shared = here();
        let (w, h) = (1000, 700);
        let mut wrong = Vec::new();
        for (name, theme) in crate::theme::tests::every_look() {
            let mut ui = crate::ui::Ui::new(crate::gallery::gallery(0), theme);
            ui.still();
            ui.lay_at(w, h);
            let mut want = tiny_skia::Pixmap::new(w, h).unwrap();
            ui.paint(&mut want.as_mut(), 1.0);
            let (_b, d) = buffer(&shared, w, h);
            assert!(matches!(ui.paint_by_gpu(&frame(&d, 1, 0, &[]), &mut Drawing::new(shared.clone())), Drawn::Whole), "{name}: drawn");
            let levels = apart(&want, &d);
            if levels > 2 {
                wrong.push(format!("{name}: {levels} levels apart"));
            }
        }
        assert!(wrong.is_empty(), "{wrong:#?}");
    }

    #[test]
    #[ignore = "needs a GPU and Mesa"]
    fn a_buffer_drawn_again_only_where_it_changed_matches_the_processor() {
        let shared = here();
        let (w, h) = (900, 300);
        let buttons = (0..4).map(|i| crate::widget::word_button(format!("Go {i}")).on_press(i).key(format!("b{i}"))).collect();
        let view = crate::widget::column(vec![crate::widget::label("Words that stay"), crate::widget::row(buttons).gap(100.0)]).padding(60.0).gap(12.0);
        let mut ui = crate::ui::Ui::new(view, crate::theme::Theme::new(true, "#fbc711"));
        ui.lay_at(w, h);
        let point = |ui: &mut crate::ui::Ui<usize>, id: &str, first: bool| {
            let r = ui.boxes().into_iter().find(|b| b.0 == id).unwrap().1;
            let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
            let at = if first { crate::input::Input::PointerIn { x, y } } else { crate::input::Input::PointerMove { x, y } };
            ui.input(&at, std::time::Instant::now());
        };
        let mut drawing = Drawing::new(shared.clone());
        let ((_a, da), (_b, db)) = (buffer(&shared, w, h), buffer(&shared, w, h));
        assert!(matches!(ui.paint_by_gpu(&frame(&da, 1, 0, &[]), &mut drawing), Drawn::Whole));
        point(&mut ui, "b0", true);
        let Drawn::Parts(missed) = ui.paint_by_gpu(&frame(&db, 2, 0, &[]), &mut drawing) else { panic!("not in parts") };
        assert_eq!(missed.len(), 1);
        point(&mut ui, "b2", false);
        let Drawn::Parts(parts) = ui.paint_by_gpu(&frame(&da, 1, 2, &missed), &mut drawing) else { panic!("not in parts") };
        assert_eq!(parts.len(), 2);
        let mut want = tiny_skia::Pixmap::new(w, h).unwrap();
        ui.paint(&mut want.as_mut(), 1.0);
        assert!(apart(&want, &da) <= 2, "{} levels apart", apart(&want, &da));
        let fence = shared.gpu().fence().expect("a fence, not a wait");
        assert!(gbm::wait_for(da.planes[0].fd.as_fd(), fence), "the compositor told to wait for it");
    }

    #[test]
    #[ignore = "needs a GPU and Mesa"]
    fn pictures_icons_and_a_clock_stay_drawn_frame_after_frame() {
        let shared = here();
        let (w, h) = (700, 400);
        let photo = Rc::new(image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(300, 200, |x, y| image::Rgba([(x % 256) as u8, (y % 256) as u8, 120, 255]))));
        let view = |n: usize| {
            let icons: Vec<crate::widget::Node<u8>> = (1..=12).map(|i| crate::widget::icon_image(format!("trindle-generic-{i:02}"), 32.0, crate::widget::Tone::Plain)).collect();
            crate::widget::column(vec![
                crate::widget::label(format!("12:{:02}", n % 60)),
                crate::widget::row(icons).gap(8.0),
                crate::widget::image_view(photo.clone(), "A photo", (300.0, 200.0)),
            ]).padding(20.0).gap(10.0)
        };
        let mut ui = crate::ui::Ui::new(view(0), crate::theme::Theme::new(true, "#fbc711"));
        ui.lay_at(w, h);
        let mut drawing = Drawing::new(shared.clone());
        let bufs = [buffer(&shared, w, h), buffer(&shared, w, h)];
        let mut behind: [Vec<[i32; 4]>; 2] = [Vec::new(), Vec::new()];
        for n in 0..40 {
            if n > 0 {
                ui.set_view(view(n));
                ui.lay_at(w, h);
            }
            let mut want = tiny_skia::Pixmap::new(w, h).unwrap();
            ui.paint(&mut want.as_mut(), 1.0);
            let slot = n % 2;
            let age = if n < 2 { 0 } else { 2 };
            let drawn = ui.paint_by_gpu(&frame(&bufs[slot].1, slot as u64 + 1, age, &behind[slot]), &mut drawing);
            behind[slot].clear();
            match drawn {
                Drawn::Parts(p) => behind[1 - slot].extend(p),
                Drawn::Whole => behind[1 - slot] = vec![[0, 0, w as i32, h as i32]],
                _ => panic!("frame {n} failed"),
            }
            assert!(apart(&want, &bufs[slot].1) <= 2, "frame {n}: {} levels apart", apart(&want, &bufs[slot].1));
        }
    }

    #[test]
    #[ignore = "needs a GPU and Mesa"]
    fn a_page_of_real_widgets_is_drawn_by_the_gpu_within_two_levels_of_the_processor() {
        use crate::big_screens_tests::page;
        let shared = here();
        let looks = crate::theme::tests::every_look().into_iter().filter(|(n, _)| n.contains("snug") || n.contains("paper") || n.starts_with("Trindle"));
        let themes = looks.map(|(n, t)| (n, t, (640, 400, 1.0))).chain([("chosen".to_string(), crate::theme::Theme::chosen(), (0, 0, 2.0))]);
        for (name, theme, (w, h, scale)) in themes {
            let sizes = if w == 0 { vec![(1280, 800), (1920, 1080), (2256, 1504), (3840, 2160)] } else { vec![(w, h)] };
            for (w, h) in sizes {
                let mut ui = crate::ui::Ui::new(page(w, h, scale, 0), theme.clone());
                ui.lay_at(w, h);
                let mut want = Pixmap::new(w, h).unwrap();
                ui.paint(&mut want.as_mut(), scale);
                let (b, d) = buffer(&shared, w, h);
                assert!(matches!(ui.paint_by_gpu(&Frame { scale, ..frame(&d, 1, 0, &[]) }, &mut Drawing::new(shared.clone())), Drawn::Whole), "{name}: drawn");
                assert!(apart(&want, &d) <= 2, "{name} {w} x {h}: {} levels apart", apart(&want, &d));
                drop(b);
            }
        }
        assert_eq!(shared.gpu().error(), 0);
    }

    #[test]
    fn the_service_is_asked_for_only_by_the_environment_and_never_over_the_processor() {
        assert!(!wants_service(false, false), "the processor stays the default");
        assert!(wants_service(false, true));
        assert!(!wants_service(true, true));
    }

    #[test]
    fn the_environment_wins_over_the_programs_choice_and_auto_is_the_toolkits() {
        use DrawnBy::*;
        for choice in [Auto, Processor, Gpu] {
            assert!(!wants_gpu(choice, true, true), "{choice:?}: the processor asked for in the environment wins over all");
            assert!(!wants_gpu(choice, true, false), "{choice:?}");
            assert!(wants_gpu(choice, false, true), "{choice:?}: the GPU asked for in the environment wins over the program");
        }
        assert!(wants_gpu(Auto, false, false), "the toolkit chooses the GPU");
        assert!(!wants_gpu(Processor, false, false));
        assert!(wants_gpu(Gpu, false, false));
        assert_eq!(DrawnBy::default(), Auto);
    }

    #[test]
    fn the_choice_is_kept_for_the_program() {
        drawn_by(DrawnBy::Gpu);
        assert_eq!(chosen(), DrawnBy::Gpu);
        drawn_by(DrawnBy::Processor);
        assert_eq!(chosen(), DrawnBy::Processor);
        drawn_by(DrawnBy::Auto);
        assert_eq!(chosen(), DrawnBy::Auto);
    }

    #[test]
    fn every_why_has_words_and_a_name_for_the_startup_line() {
        let all = [Why::Asked, Why::NoDmabuf, Why::NoLibraries, Why::NoNode, Why::NoContext, Why::Software, Why::SelfTest, Why::Failed, Why::NoService, Why::ServiceOtherGpu, Why::ServiceGone];
        for w in all {
            assert!(!w.words().is_empty());
            let line = crate::ui::Drawer::Processor(w).line("org.example.App");
            assert_eq!(line, format!("trindle: org.example.App window drawn by processor ({w:?})"));
        }
        assert_eq!(crate::ui::Drawer::Gpu.line("org.example.App"), "trindle: org.example.App window drawn by gpu (Passed)");
        assert_eq!(crate::ui::Drawer::Service.line("org.example.App"), "trindle: org.example.App window drawn by service (Passed)");
    }

    #[test]
    fn a_check_is_not_made_once_one_before_it_has_failed() {
        let made = Cell::new(false);
        let r = choose::<u8, u8>(false, false, || { made.set(true); None }, || { made.set(true); true }, |_| None, |_| None, |_| String::new(), |_| true);
        assert_eq!(r, Err(Why::NoDmabuf));
        assert!(!made.get());
    }
}
