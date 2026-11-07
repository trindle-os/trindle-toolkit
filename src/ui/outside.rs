//! Things dragged between this window and other programs; a secret box takes words dropped into it and never lets any be dragged out.

use tiny_skia::Pixmap;

use super::*;
use crate::dnd::{Given, Kind, Offered};
use crate::widget::{Held, Outside, Part};

const PICTURE_LETTERS: usize = 32;

pub(super) struct Lift {
    id: String,
    from: (f32, f32),
    at: usize,
    out: bool,
}

struct Target {
    id: String,
    held: Held,
    way: Way,
    kind: Kind,
    entry: bool,
}

pub(crate) fn one_line(text: &str) -> String {
    text.trim_end_matches(['\n', '\r']).chars().map(|c| if c == '\n' || c == '\t' { ' ' } else { c })
        .filter(|c| !c.is_control() && !matches!(c, '\u{61c}' | '\u{200b}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}' | '\u{2066}'..='\u{2069}' | '\u{feff}'))
        .collect()
}

fn kind_taken<M>(o: &Outside<M>, offered: Offered, kinds: &[String]) -> Option<Kind> {
    if o.kinds.is_empty() {
        return if offered.files { Some(Kind::Files) } else { offered.text.then_some(Kind::Text) };
    }
    o.kinds.iter().find(|k| kinds.contains(k)).cloned().map(Kind::Typed)
}

fn solid<M>(node: &Node<M>) -> bool {
    !matches!(node.what, What::Drawing { passes: true, .. }) && !matches!(node.what, What::Column { .. } | What::Row { .. } | What::Grid { .. } | What::Space | What::Scroll(_) | What::ScrollX(_)
        | What::Layers(_) | What::Split { .. } | What::Placed { .. } | What::Slide { .. } | What::Aspect { .. } | What::Label { .. } | What::Rich { .. }
        | What::Ring { .. }
        | What::Spinner { .. })
}

impl<M: Clone> Ui<M> {
    fn outside_target(&self, x: f32, y: f32, offered: Offered) -> Option<Target> {
        let mut under: Option<&[usize]> = None;
        let k = self.scale.max(f32::EPSILON);
        for s in self.spots.iter().rev().filter(|s| s.shows(x, y)) {
            if under.is_some_and(|u| !u.starts_with(&s.path)) {
                continue;
            }
            let Some(node) = self.tree.at(&s.path) else { continue };
            if matches!(node.what, What::Entry { .. }) && !node.off && offered.text {
                let way = [Way::Copy, Way::Move].into_iter().find(|w| offered.ways.has(*w));
                if let Some(way) = way {
                    let held = Held { offered, part: Part::Whole, mods: self.mods };
                    return Some(Target { id: s.id.clone(), held, way, kind: Kind::Text, entry: true });
                }
            }
            let taker = node.outside.as_ref().filter(|_| !node.off).and_then(|o| Some((o, kind_taken(o, offered, &self.held_kinds)?)));
            if let Some((o, kind)) = taker {
                let part = match &node.what {
                    What::List { .. } => self.row_at(&s.id, y).map_or(Part::Whole, Part::Row),
                    What::Choice { .. } => self.pill(&s.id, x).map_or(Part::Whole, Part::Row),
                    What::Drawing { .. } => Part::Point((x - s.rect.x) / k, (y - s.rect.y) / k),
                    _ => Part::Whole,
                };
                let gap = match &node.what {
                    What::List { .. } => self.gap_at(&s.id, y, false).map(Part::Gap),
                    _ => None,
                };
                for part in gap.into_iter().chain([part]) {
                    let held = Held { offered, part, mods: self.mods };
                    if let Some(way) = (o.would)(&held).filter(|w| offered.ways.has(*w)) {
                        return Some(Target { id: s.id.clone(), held, way, kind, entry: false });
                    }
                }
            }
            if under.is_none() && solid(node) {
                under = Some(&s.path);
            }
        }
        None
    }

    fn outside_over(&mut self, x: f32, y: f32, reply: &mut Reply<M>) {
        let before = (self.hover.clone(), self.hover_row(), self.gap.clone());
        self.pointer = Some((x, y));
        let Some(offered) = self.outside else { return };
        let target = self.outside_target(x, y, offered);
        let over = self.drawing_at(x, y).filter(|o| self.hovers(o));
        self.hover_drawing(over.clone(), reply);
        if let Some(o) = over {
            self.point(&o, x, y, w::Pointer::Hover, reply);
        }
        self.gap = target.as_ref().and_then(|t| match t.held.part {
            Part::Gap(gap) => Some((t.id.clone(), gap)),
            _ => None,
        });
        self.hover = target.as_ref().filter(|_| self.gap.is_none()).map(|t| t.id.clone());
        self.tip = None;
        reply.redraw |= before != (self.hover.clone(), self.hover_row(), self.gap.clone());
        reply.wanted = Some(target.map(|t| (t.way, t.kind)));
    }

    fn outside_gone(&mut self, reply: &mut Reply<M>) {
        self.outside = None;
        self.held_kinds.clear();
        self.pointer = None;
        self.hover_drawing(None, reply);
        reply.redraw |= self.gap.take().is_some();
        reply.redraw |= self.hover.take().is_some();
    }

    fn drop_text(&mut self, id: &str, x: f32, text: &str, copy: bool, reply: &mut Reply<M>) {
        let line = one_line(text);
        let mut at = self.entry_hit(id, x);
        let own = !copy && self.lift.as_ref().is_some_and(|l| l.out && l.id == id);
        self.focus = Some(id.to_string());
        let Some(e) = self.edit(id) else { return };
        let before = e.text.clone();
        if own {
            let picked = e.selection();
            if (picked.start..=picked.end).contains(&at) {
                return;
            }
            e.delete_to(picked.start);
            if at > picked.end {
                at -= picked.len();
            }
        }
        e.move_to(at, false);
        if !line.is_empty() {
            e.insert(&line);
        }
        self.changed(id, &before, reply);
    }

    pub(super) fn outside_input(&mut self, input: &Input, reply: &mut Reply<M>) -> bool {
        let k = self.scale;
        match input {
            Input::DragIn { x, y, offered, kinds } => {
                self.outside = Some(*offered);
                self.held_kinds.clone_from(kinds);
                self.outside_over(x * k, y * k, reply);
            }
            Input::DragOver { x, y } => self.outside_over(x * k, y * k, reply),
            Input::DragGone => self.outside_gone(reply),
            Input::Dropped { x, y, given, way, id } => {
                let (x, y) = (x * k, y * k);
                let offered = self.outside.unwrap_or(Offered {
                    files: matches!(given, Given::Files(_)),
                    text: matches!(given, Given::Text(_)),
                    ways: crate::dnd::Ways::from_bits(way.bits()),
                });
                if let Given::Typed(kind, _) = given {
                    if self.held_kinds.is_empty() {
                        self.held_kinds.push(kind.clone());
                    }
                }
                let target = self.outside_target(x, y, offered);
                self.outside_gone(reply);
                reply.redraw = true;
                let mut taken = false;
                match target {
                    Some(t) if t.entry => {
                        if let Given::Text(text) = given {
                            self.drop_text(&t.id, x, text, *way == Way::Copy && offered.ways.has(Way::Move), reply);
                            taken = true;
                        }
                    }
                    Some(t) => {
                        if let Some(o) = self.node(&t.id).and_then(|n| n.outside.as_ref()) {
                            reply.messages.push((o.take)(&t.held, given.clone(), *way));
                            taken = !matches!(given, Given::Nothing(_));
                        }
                    }
                    None => {}
                }
                reply.dropped = Some((*id, *way, taken));
            }
            Input::DragEnded { .. } => {
                self.lift = None;
                self.dragging = None;
                self.right = None;
                self.pressed = None;
                self.touch = None;
                self.hold = None;
                self.swipe = None;
                self.swipe_x = None;
                self.drop_hover(None, reply);
                self.hover_drawing(None, reply);
                reply.redraw = true;
            }
            _ => return false,
        }
        true
    }

    pub(super) fn lift_press(&mut self, id: &str, from: (f32, f32), at: usize) -> bool {
        if self.touch.is_some() || self.entry_secret(id) {
            return false;
        }
        let selected = if self.is_entry(id) {
            self.edits.get(id).map(Edit::selection)
        } else {
            self.picked.as_ref().filter(|(p, _)| p == id).map(|(_, e)| e.selection())
        };
        if !selected.is_some_and(|s| s.contains(&at)) {
            return false;
        }
        self.lift = Some(Lift { id: id.to_string(), from, at, out: false });
        true
    }

    fn lift_text(&self, id: &str) -> Option<String> {
        if self.entry_secret(id) {
            return None;
        }
        let e = if self.is_entry(id) { self.edits.get(id) } else { self.picked.as_ref().filter(|(p, _)| p == id).map(|(_, e)| e) }?;
        e.text.get(e.selection()).filter(|t| !t.is_empty()).map(str::to_string)
    }

    pub(super) fn lift_moved(&mut self, x: f32, y: f32, reply: &mut Reply<M>) -> bool {
        let slop = self.px(RIGHT_SLOP);
        let Some(l) = &mut self.lift else { return false };
        if !l.out && ((x - l.from.0).abs() > slop || (y - l.from.1).abs() > slop) {
            l.out = true;
            let id = l.id.clone();
            reply.lift = self.lift_text(&id);
            if reply.lift.is_none() {
                self.lift = None;
            }
        }
        true
    }

    pub(super) fn lift_let_go(&mut self) -> bool {
        let Some(l) = self.lift.take() else { return false };
        self.pressed = None;
        if !l.out {
            if self.is_entry(&l.id) {
                if let Some(e) = self.edit(&l.id) {
                    e.move_to(l.at, false);
                }
            } else if let Some((_, e)) = self.picked.as_mut().filter(|(p, _)| *p == l.id) {
                e.move_to(l.at, false);
            }
        }
        true
    }

    pub fn lifting(&self) -> bool {
        self.lift.as_ref().is_some_and(|l| l.out)
    }

    pub fn drag_picture(&self, icon: Option<&str>, folder: bool, words: &str) -> Option<(Pixmap, u32)> {
        let scale = self.scale.ceil().max(1.0);
        let k = theme::sheet();
        let words = one_line(words);
        let mut shown: String = words.chars().take(PICTURE_LETTERS).collect();
        if shown.chars().count() < words.chars().count() {
            shown.push('…');
        }
        let layout = self.fonts.layout(&shown, &tip_style(scale), None);
        let (pad_x, pad_y) = ((k.label_padding.0 * scale).round(), (k.label_padding.1 * scale).round());
        let line = scale.round().max(1.0);
        let size = (layout.height / scale).floor().max(1.0);
        let lead = if icon.is_some() { size * scale + pad_x / 2.0 } else { 0.0 };
        let (bw, bh) = (layout.width.ceil() + lead + 2.0 * pad_x, layout.height.ceil() + 2.0 * pad_y);
        let mut picture = Pixmap::new((bw + 2.0 * line) as u32, (bh + 2.0 * line) as u32)?;
        let mut canvas = picture.as_mut();
        let b = Rect { x: line, y: line, w: bw, h: bh };
        let radius = theme::corner_px(theme::Part::Tooltip, b.h, scale);
        fill(&mut canvas, b.inset(-line), radius + line, k.label_outline);
        fill(&mut canvas, b, radius, k.label_back);
        edge(&mut canvas, b, radius, k.label_edge, scale);
        if let Some(name) = icon {
            let y = b.y + (b.h - size * scale) / 2.0;
            let ink = if folder { self.theme.folder_ink(None, &[k.label_back]) } else { k.label_ink };
            self.icons.draw(&mut canvas, name, (b.x + pad_x) as i32, y as i32, size as u32, scale as u32, ink);
        }
        let left = layout.lines.iter().map(|l| l.x).reduce(f32::min).unwrap_or(0.0);
        self.fonts.draw(&mut canvas, &layout, b.x + pad_x + lead - left, b.y + pad_y, k.label_ink);
        Some((picture, scale as u32))
    }
}
