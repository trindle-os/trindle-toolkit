//! A canvas of items a program places, which the person can choose, move, zoom and pan, and which the program can find under the pointer.

use std::rc::Rc;

use tiny_skia::PixmapMut;

use super::input::{Keysym, Mods};
use super::paint::Painter;
use super::theme;
use super::tiles::{Fill, Sketch, Way};
use super::widget::{Pen, Pointer, Rect};
use super::zoom::{Drag, View};
use crate::t;

const NUDGE: (f32, f32) = (1.0, 10.0);

#[derive(Clone)]
pub struct Item {
    pub at: (f32, f32),
    pub size: (f32, f32),
    pub name: String,
    pub moves: bool,
    pub art: Rc<dyn Fn(&mut Sketch)>,
}

impl Item {
    pub fn new(name: impl Into<String>, at: (f32, f32), size: (f32, f32), art: impl Fn(&mut Sketch) + 'static) -> Item {
        Item { at, size, name: name.into(), moves: true, art: Rc::new(art) }
    }

    pub fn fixed(mut self) -> Item {
        self.moves = false;
        self
    }

    fn rect(&self) -> Rect {
        Rect { x: self.at.0, y: self.at.1, w: self.size.0, h: self.size.1 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Change {
    Chosen(Option<usize>),
    Moved(usize),
    Looked,
}

#[derive(Default)]
pub struct Scene {
    pub items: Vec<Item>,
    pub chosen: Option<usize>,
    held: Option<(usize, (f32, f32))>,
    pan: Drag,
}

impl Scene {
    pub fn new(items: Vec<Item>) -> Scene {
        Scene { items, ..Scene::default() }
    }

    pub fn hit(&self, view: &View, x: f32, y: f32) -> Option<usize> {
        let (cx, cy) = view.to_content(x, y);
        self.items.iter().rposition(|i| {
            let r = i.rect();
            cx >= r.x && cx < r.x + r.w && cy >= r.y && cy < r.y + r.h
        })
    }

    pub fn pointer(&mut self, view: &mut View, event: Pointer) -> Option<Change> {
        match event {
            Pointer::Down(x, y) => {
                let hit = self.hit(view, x, y);
                self.held = hit.filter(|&i| self.items[i].moves).map(|i| {
                    let (cx, cy) = view.to_content(x, y);
                    (i, (cx - self.items[i].at.0, cy - self.items[i].at.1))
                });
                if hit.is_none() {
                    self.pan.pointer(view, event);
                }
                (self.chosen != hit).then(|| {
                    self.chosen = hit;
                    Change::Chosen(hit)
                })
            }
            Pointer::Move(x, y) => match self.held {
                Some((i, grab)) if i < self.items.len() => {
                    let (cx, cy) = view.to_content(x, y);
                    self.items[i].at = (cx - grab.0, cy - grab.1);
                    Some(Change::Moved(i))
                }
                Some(_) => {
                    self.held = None;
                    None
                }
                None => self.pan.pointer(view, event).then_some(Change::Looked),
            },
            Pointer::Up(..) => {
                self.held = None;
                self.pan.pointer(view, event);
                None
            }
            _ => None,
        }
    }

    pub fn key(&mut self, view: &mut View, sym: Keysym, mods: Mods) -> Option<Change> {
        let n = self.items.len();
        let by = if mods.shift { NUDGE.1 } else { NUDGE.0 };
        let middle = {
            let r = view.visible();
            view.to_screen(r.x + r.w / 2.0, r.y + r.h / 2.0)
        };
        let step = match sym {
            Keysym::Tab | Keysym::ISO_Left_Tab if n > 0 => {
                let back = mods.shift || sym == Keysym::ISO_Left_Tab;
                let next = match (self.chosen, back) {
                    (None, false) => 0,
                    (None, true) => n - 1,
                    (Some(i), false) => (i + 1) % n,
                    (Some(i), true) => (i + n - 1) % n,
                };
                self.chosen = Some(next);
                return Some(Change::Chosen(self.chosen));
            }
            Keysym::Left => (-by, 0.0),
            Keysym::Right => (by, 0.0),
            Keysym::Up => (0.0, -by),
            Keysym::Down => (0.0, by),
            Keysym::plus | Keysym::equal | Keysym::KP_Add => {
                view.zoom_in(middle);
                return Some(Change::Looked);
            }
            Keysym::minus | Keysym::KP_Subtract => {
                view.zoom_out(middle);
                return Some(Change::Looked);
            }
            Keysym::_0 | Keysym::KP_0 => {
                view.fit();
                return Some(Change::Looked);
            }
            _ => return None,
        };
        let i = self.chosen.filter(|&i| self.items.get(i).is_some_and(|it| it.moves))?;
        self.items[i].at = (self.items[i].at.0 + step.0, self.items[i].at.1 + step.1);
        Some(Change::Moved(i))
    }

    pub fn paint(&self, canvas: &mut PixmapMut, pen: &Pen, view: &View) {
        let mut p = Painter::default();
        self.paint_on(&mut p, pen, (canvas.width(), canvas.height()), view);
        p.play(canvas);
    }

    pub fn paint_on<'a>(&self, p: &mut Painter<'a>, pen: &Pen<'a>, size: (u32, u32), view: &View) {
        let k = pen.scale;
        for (n, item) in self.items.iter().enumerate() {
            let r = view.rect_to_screen(item.rect());
            let mut s = Sketch::placed_on(p, pen, size, view.scale() * k, (r.x * k, r.y * k));
            (item.art)(&mut s);
            if self.chosen == Some(n) {
                let sheet = theme::sheet();
                let (out, wide) = (sheet.focus_offset + sheet.focus_width / 2.0, sheet.focus_width);
                let mut s = Sketch::placed_on(p, pen, size, k, (0.0, 0.0));
                let ring = Way::rounded((r.x - out, r.y - out, r.w + 2.0 * out, r.h + 2.0 * out), out);
                s.stroke(&ring, wide, &Fill::Flat(pen.theme.accent), &[]);
            }
        }
    }

    pub fn told(&self) -> String {
        match self.chosen.and_then(|i| self.items.get(i)) {
            Some(item) => t!("kit-scene-chosen", count = self.items.len() as i64, name = item.name.clone()),
            None => t!("kit-scene", count = self.items.len() as i64),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colour::Colour;
    use crate::icon::Icons;
    use crate::text::Fonts;
    use crate::theme::Theme;

    fn scene() -> (Scene, View) {
        let red = |s: &mut Sketch| s.rect((0.0, 0.0, 20.0, 20.0), 0.0, Colour::rgb(1.0, 0.0, 0.0));
        let items = vec![Item::new("Back", (0.0, 0.0), (100.0, 100.0), |_| {}).fixed(), Item::new("Box", (10.0, 10.0), (20.0, 20.0), red)];
        (Scene::new(items), View::new((100.0, 100.0), (100.0, 100.0)))
    }

    #[test]
    fn the_topmost_item_is_found_under_the_pointer() {
        let (s, v) = scene();
        assert_eq!(s.hit(&v, 15.0, 15.0), Some(1));
        assert_eq!(s.hit(&v, 50.0, 50.0), Some(0));
        assert_eq!(s.hit(&v, 150.0, 50.0), None);
    }

    #[test]
    fn a_press_chooses_and_a_drag_moves_in_scene_units_when_zoomed() {
        let (mut s, mut v) = scene();
        v.zoom_to(2.0, (0.0, 0.0));
        assert_eq!(s.pointer(&mut v, Pointer::Down(30.0, 30.0)), Some(Change::Chosen(Some(1))));
        assert_eq!(s.pointer(&mut v, Pointer::Move(50.0, 40.0)), Some(Change::Moved(1)));
        assert_eq!(s.items[1].at, (20.0, 15.0), "20 points at twice the size is 10 units");
        assert_eq!(s.pointer(&mut v, Pointer::Up(50.0, 40.0)), None);
        assert_eq!(s.pointer(&mut v, Pointer::Move(90.0, 90.0)), None, "let go, it stays");
    }

    #[test]
    fn a_fixed_item_is_chosen_but_not_moved() {
        let (mut s, mut v) = scene();
        assert_eq!(s.pointer(&mut v, Pointer::Down(60.0, 60.0)), Some(Change::Chosen(Some(0))));
        assert_eq!(s.pointer(&mut v, Pointer::Move(80.0, 80.0)), None);
        assert_eq!(s.items[0].at, (0.0, 0.0));
        assert_eq!(s.key(&mut v, Keysym::Right, Mods::default()), None);
    }

    #[test]
    fn a_drag_on_nothing_pans_a_zoomed_view() {
        let mut s = Scene::new(vec![Item::new("Box", (0.0, 0.0), (10.0, 10.0), |_| {})]);
        let mut v = View::new((100.0, 100.0), (100.0, 100.0));
        v.zoom_to(4.0, (50.0, 50.0));
        let before = v.visible();
        s.pointer(&mut v, Pointer::Down(90.0, 90.0));
        assert_eq!(s.pointer(&mut v, Pointer::Move(60.0, 60.0)), Some(Change::Looked));
        assert_ne!(v.visible(), before);
    }

    #[test]
    fn keys_step_through_items_move_the_chosen_one_and_zoom() {
        let (mut s, mut v) = scene();
        let shift = Mods { shift: true, ..Mods::default() };
        assert_eq!(s.key(&mut v, Keysym::Tab, Mods::default()), Some(Change::Chosen(Some(0))));
        assert_eq!(s.key(&mut v, Keysym::Tab, Mods::default()), Some(Change::Chosen(Some(1))));
        assert_eq!(s.key(&mut v, Keysym::ISO_Left_Tab, shift), Some(Change::Chosen(Some(0))));
        s.chosen = Some(1);
        assert_eq!(s.key(&mut v, Keysym::Right, Mods::default()), Some(Change::Moved(1)));
        assert_eq!(s.key(&mut v, Keysym::Down, shift), Some(Change::Moved(1)));
        assert_eq!(s.items[1].at, (11.0, 20.0));
        assert_eq!(s.key(&mut v, Keysym::plus, Mods::default()), Some(Change::Looked));
        assert!(v.scale() > 1.0);
        s.key(&mut v, Keysym::_0, Mods::default());
        assert!(v.is_fitted());
        assert_eq!(s.key(&mut v, Keysym::a, Mods::default()), None);
    }

    #[test]
    fn items_are_drawn_zoomed_and_the_chosen_one_is_ringed() {
        let (mut s, mut v) = scene();
        v.zoom_to(2.0, (0.0, 0.0));
        s.chosen = Some(1);
        let theme = Theme::new(true, "#fbc711");
        let (fonts, icons) = (Fonts::new(), Icons::new());
        let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
        let mut pix = tiny_skia::Pixmap::new(100, 100).unwrap();
        s.paint(&mut pix.as_mut(), &pen, &v);
        let p = |x, y| pix.pixel(x, y).unwrap();
        assert_eq!(p(50, 50).red(), 255, "the box covers 20 to 60 at twice the size");
        assert_eq!(p(70, 70).alpha(), 0);
        let ring = p(40, 63);
        assert!(ring.alpha() > 200 && ring.green() > 100, "a ring in the highlight just outside it");
    }

    #[test]
    fn a_screen_reader_hears_the_count_and_the_chosen_item() {
        let (mut s, _) = scene();
        assert!(s.told().contains('2'));
        s.chosen = Some(1);
        assert!(s.told().contains("Box"));
    }
}
