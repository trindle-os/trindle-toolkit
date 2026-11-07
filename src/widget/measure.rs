//! Working out how much room a node needs: its own size, and how a row or column shares what it has.

use crate::theme;

use super::{Area, CHEVRON, CHEVRON_GAP, Lay, Length, NAME_GAP, NAME_WIDTH, Node, QUIET_PADDING, Rect, SLIDE_ROOM, Sub, What, WINDOW_MARGIN, Words, action_fixed, action_icon_room, action_room, action_words, is_word_button, name_style, nothing_chosen, pill_widths, word_button_style};

const WHOLE_ROOM: f32 = 1_000_000.0;

const SCROLL_X_ROOM: f32 = 8_000_000.0;

const ROUND_PASSES: usize = 8;

const ROUND_LEAST_LIST: f32 = 64.0;

fn line_width(line: &[(usize, f32, f32)], gap: f32) -> f32 {
    line.iter().map(|p| p.1).sum::<f32>() + gap * line.len().saturating_sub(1) as f32
}

pub(super) fn line_height(line: &[(usize, f32, f32)]) -> f32 {
    line.iter().map(|p| p.2).fold(0.0, f32::max)
}

impl Lay<'_> {
    pub(super) fn scroll_room<M>(&self, inner: &Node<M>, rect: Rect) -> (f32, f32) {
        let full = self.measure(inner, rect.w).1;
        if full <= rect.h + 0.5 {
            return (rect.w, full.max(rect.h));
        }
        let own = match &inner.what {
            What::Column { padding, .. } | What::Row { padding, .. } => self.px(*padding),
            _ => 0.0,
        };
        let bar = (self.px(theme::sheet().scrollbar_hover + 2.0 * theme::sheet().scrollbar_inset) - own).max(0.0);
        if bar < 0.5 {
            return (rect.w, full.max(rect.h));
        }
        let w = (rect.w - bar).max(0.0);
        (w, self.measure(inner, w).1.max(rect.h))
    }

    pub(super) fn scroll_x_room<M>(&self, inner: &Node<M>, rect: Rect) -> (f32, f32) {
        let outer = self.natural.replace(true);
        let full = self.measure(inner, SCROLL_X_ROOM).0.max(rect.w);
        self.natural.set(outer);
        if full <= rect.w + 0.5 {
            return (rect.w, rect.h);
        }
        let bar = self.px(theme::sheet().scrollbar_hover + 2.0 * theme::sheet().scrollbar_inset);
        (full, (rect.h - bar).max(0.0))
    }

    pub(super) fn card_padding<M>(&self, n: &Node<M>) -> (f32, f32) {
        if n.chip {
            (self.px(theme::sheet().chip_padding.0), self.px(theme::sheet().chip_padding.1))
        } else if matches!(n.area, Some((Area::Top | Area::Foot | Area::Controls | Area::Between, _))) {
            (0.0, 0.0)
        } else {
            let p = self.px(if n.snug { theme::sheet().snug_padding } else if n.flat { theme::sheet().line_padding } else { theme::sheet().card_padding });
            (p, p)
        }
    }

    pub(crate) fn pills(&self, options: &[String], arrows: Option<&[Sub]>) -> Vec<f32> {
        pill_widths(self.fonts, self.texts, options, arrows, self.scale)
    }

    pub(crate) fn measure<M>(&self, n: &Node<M>, max_w: f32) -> (f32, f32) {
        let max_w = if let Length::Fixed(p) = n.width { self.px(p) } else { max_w };
        let outer = self.natural.get();
        if n.width == Length::Shrink && matches!(n.what, What::Card(..) | What::Column { .. } | What::Row { .. }) {
            self.natural.set(true);
        }
        let (w, h) = self.measure_what(n, max_w);
        self.natural.set(outer);
        let w = match n.width {
            Length::Fixed(p) => self.px(p),
            Length::Fill if outer => w.min(max_w.max(0.0)),
            Length::Fill => max_w.max(w.min(max_w)),
            Length::Shrink => w.min(max_w.max(0.0)),
        };
        let h = if let Length::Fixed(p) = n.height { self.px(p) } else { h };
        (w, h)
    }

    pub(super) fn measure_within<M>(&self, inner: &Node<M>, room: (f32, f32)) -> (f32, f32) {
        let margin = 2.0 * self.px(WINDOW_MARGIN);
        let (rw, rh) = ((room.0 - margin).max(0.0), (room.1 - margin).max(0.0));
        let was = self.whole.replace(true);
        let (mut w, mut h) = self.measure_floating(inner, rw);
        if w > rw {
            (w, h) = (rw, self.measure_what(inner, rw).1);
        }
        self.whole.set(was);
        (w, h.min(rh))
    }

    pub(super) fn measure_floating<M>(&self, inner: &Node<M>, max_w: f32) -> (f32, f32) {
        if inner.width != Length::Fill || !matches!(inner.what, What::Card(..)) {
            return self.measure(inner, max_w);
        }
        let outer = self.natural.replace(true);
        let size = self.measure(inner, max_w);
        self.natural.set(outer);
        size
    }

    pub(super) fn measure_what<M>(&self, n: &Node<M>, max_w: f32) -> (f32, f32) {
        let (w, h) = match &n.what {
            What::Space | What::Divider { .. } => (0.0, 0.0),
            What::Label { .. } => self.measure_label(n, max_w),
            What::Rich { spans, words, .. } => {
                let f = crate::rich::flow(&|t, s| self.texts.get(self.fonts, t, s, None), spans, &words.style(self.scale), Some(max_w.max(1.0)));
                (f.width.ceil(), f.height.ceil())
            }
            What::Ring { size, .. } | What::Spinner { size } => (self.px(*size), self.px(*size)),
            What::Drawing { size, .. } => (self.px(size.0), self.px(size.1)),
            What::Button { .. } => self.measure_button(n, max_w),
            What::Switch { .. } => (self.px(theme::sheet().switch.0), self.px(theme::sheet().switch.1)),
            What::Tick { .. } => (self.px(crate::theme::sheet().tick), self.px(crate::theme::sheet().tick)),
            What::Slider { .. } => (self.px(theme::SLIDER_MIN_WIDTH), self.px(theme::sheet().switch.1)),
            What::Entry { .. } => (self.px(theme::ENTRY_MIN_WIDTH), self.px(theme::sheet().entry_height)),
            What::List { rows, row, .. } => (self.px(theme::ENTRY_MIN_WIDTH), self.px(*row) * rows.len() as f32),
            What::Dropdown { options, chosen, .. } => {
                let shown = chosen.and_then(|i| options.get(i)).map_or(nothing_chosen(), String::as_str);
                let l = self.texts.get(self.fonts, shown, &Words::Plain.style(self.scale), None);
                let wide = l.width.ceil() + 2.0 * self.px(theme::sheet().entry_padding) + self.px(CHEVRON_GAP) + self.px(CHEVRON);
                (wide.max(self.px(theme::ENTRY_MIN_WIDTH)), self.px(theme::sheet().dropdown_height))
            }
            What::Choice { options, arrows, .. } => ((self.pills(options, arrows.as_ref().map(|a| a.subs.as_slice())).iter().sum::<f32>() + 2.0 * self.px(theme::sheet().tabs_padding)).min(max_w.max(0.0)),
                                             self.px(theme::sheet().button)),
            What::Scroll(inner) => {
                let (w, h) = self.measure(inner, max_w);
                (w, if n.height == Length::Fill && !self.whole.get() { 0.0 } else { h })
            }
            What::ScrollX(inner) => {
                let h = self.measure(inner, SCROLL_X_ROOM).1;
                (max_w, if n.height == Length::Fill && !self.whole.get() { 0.0 } else { h })
            }
            What::Card(inner, _) => {
                let (px, py) = self.card_padding(n);
                let (w, h) = self.measure(inner, max_w - 2.0 * px);
                (w + 2.0 * px, h + 2.0 * py)
            }
            What::Split { .. } => self.measure_split(n, max_w),
            What::Layers(items) => items.iter().map(|i| self.measure(i, max_w)).fold((0.0f32, 0.0f32), |a, s| (a.0.max(s.0), a.1.max(s.1))),
            What::Placed { inner, .. } => self.measure_floating(inner, max_w),
            What::Slide { inner, .. } => self.measure(inner, SLIDE_ROOM),
            What::Aspect { inner, ratio } => {
                let w = if self.natural.get() { self.measure(inner, max_w).0 } else { max_w };
                (w, (w / ratio).round())
            }
            What::Column { items, gap, padding, .. } => {
                let (gap, pad) = (self.px(*gap), self.px(*padding));
                let sizes = self.column_sizes(n, items, max_w - 2.0 * pad, gap);
                let w = sizes.iter().map(|s| s.0).fold(0.0, f32::max);
                let h = sizes.iter().map(|s| s.1).sum::<f32>() + gap * items.len().saturating_sub(1) as f32;
                (w + 2.0 * pad, h + 2.0 * pad)
            }
            What::Row { .. } => self.measure_row(n, max_w),
            What::Grid { .. } => self.measure_grid(n, max_w),
        };
        (w, h)
    }

    fn measure_label<M>(&self, n: &Node<M>, max_w: f32) -> (f32, f32) {
        match &n.what {
            What::Label { text, words, .. } if n.cut == 1 => {
                let l = self.texts.get(self.fonts, text, &words.style(self.scale), None);
                (l.width.min(max_w.max(1.0)).ceil(), l.lines.first().map_or(l.height, |f| f.height).ceil())
            }
            What::Label { text, words, .. } if n.cut > 1 => {
                let l = self.texts.get(self.fonts, text, &words.style(self.scale), Some(max_w.max(1.0)));
                let kept = &l.lines[..l.lines.len().min(n.cut as usize)];
                let tall = kept.last().map_or(l.height, |k| k.top + k.height);
                let line = l.lines.first().map_or(l.height, |f| f.height);
                let tall = if kept.len() < n.keep_lines as usize { line * n.keep_lines as f32 } else { tall };
                (kept.iter().map(|k| k.width).fold(0.0, f32::max).ceil(), tall.ceil())
            }
            What::Label { text, words, .. } => {
                let l = self.texts.get(self.fonts, text, &words.style(self.scale), Some(max_w.max(1.0)));
                (l.widest().ceil(), l.height.ceil())
            }
            _ => unreachable!(),
        }
    }

    fn measure_button<M>(&self, n: &Node<M>, max_w: f32) -> (f32, f32) {
        match &n.what {
            What::Button { words: Some(words), icon, on, .. } if is_word_button(icon, *on) => {
                let s = theme::sheet();
                let l = self.texts.get(self.fonts, words, &word_button_style(self.scale), None);
                ((2.0 * self.px(s.word_button_padding) + l.width.ceil()).max(self.px(s.word_button.0)), self.px(s.word_button.1))
            }
            What::Button { words: Some(words), icon, .. } if n.quiet => {
                let l = self.texts.get(self.fonts, words, &Words::Plain.style(self.scale), None);
                let before = if icon.is_empty() { 0.0 } else { self.px(theme::sheet().icon) + self.px(theme::sheet().action_gap) };
                (2.0 * self.px(QUIET_PADDING) + before + l.width.ceil(), self.px(theme::sheet().button))
            }
            What::Button { words: Some(words), icon, .. } => {
                let spared = if icon.is_empty() { action_icon_room(self.scale) } else { 0.0 };
                let l = action_words(self.fonts, self.texts, words, max_w + spared, self.scale);
                let wide = l.widest().ceil();
                let tall = if l.lines.len() > 1 { l.height.ceil() + 2.0 * action_room(self.scale) } else { 0.0 };
                (action_fixed(self.scale) - spared + wide, self.px(theme::sheet().action_height).max(tall))
            }
            What::Button { size, tip, .. } => {
                let s = self.px(size.unwrap_or(theme::icon_button()));
                if self.named && !tip.is_empty() {
                    let l = self.texts.get(self.fonts, tip, &name_style(self.scale), Some(self.px(NAME_WIDTH)));
                    let w = l.widest().ceil();
                    (s.max(w), s + self.px(NAME_GAP) + l.height.ceil())
                } else {
                    (s, s)
                }
            }
            _ => unreachable!(),
        }
    }

    fn measure_row<M>(&self, n: &Node<M>, max_w: f32) -> (f32, f32) {
        match &n.what {
            What::Row { items, gap, padding } if n.stacks && self.row_stacks(items, max_w - 2.0 * self.px(*padding), self.px(*gap)) => {
                let (gap, pad) = (self.px(*gap), self.px(*padding));
                let sizes: Vec<_> = items.iter().filter(|i| !matches!(i.what, What::Space)).map(|i| self.measure(i, max_w - 2.0 * pad)).collect();
                let h = sizes.iter().map(|s| s.1).sum::<f32>() + gap * sizes.len().saturating_sub(1) as f32;
                (sizes.iter().map(|s| s.0).fold(0.0, f32::max) + 2.0 * pad, h + 2.0 * pad)
            }
            What::Row { items, gap, padding } => {
                let (gap, pad) = (self.px(*gap), self.px(*padding));
                if let Some(lines) = self.row_lines(items, max_w - 2.0 * pad, gap, n.wraps) {
                    let wide = lines.iter().map(|l| line_width(l, gap)).fold(0.0, f32::max);
                    let tall = lines.iter().map(|l| line_height(l)).sum::<f32>() + gap * lines.len().saturating_sub(1) as f32;
                    (wide + 2.0 * pad, tall + 2.0 * pad)
                } else {
                    let widths = self.row_widths(items, max_w - 2.0 * pad, gap);
                    let h = items.iter().zip(&widths).map(|(i, w)| self.measure(i, *w).1).fold(0.0, f32::max);
                    (widths.iter().sum::<f32>() + gap * items.len().saturating_sub(1) as f32 + 2.0 * pad, h + 2.0 * pad)
                }
            }
            _ => unreachable!(),
        }
    }

    fn measure_grid<M>(&self, n: &Node<M>, max_w: f32) -> (f32, f32) {
        match &n.what {
            What::Grid { items, gap, flow: true, .. } => (max_w, self.flowed(items, *gap, max_w).1),
            What::Grid { items, min, gap, even: true, grow: true, .. } => {
                let (cols, _, high, gap) = self.grown_cells(items, max_w, *min, *gap, None);
                let rows = items.len().div_ceil(cols);
                (max_w, if rows == 0 { 0.0 } else { rows as f32 * high + gap * (rows - 1) as f32 })
            }
            What::Grid { items, min, gap, even: true, .. } => {
                let (cols, cell, gap) = self.grid_cells(max_w, *min, *gap, items.len());
                let h = items.chunks(cols)
                    .map(|r| r.iter().map(|i| self.measure(i, cell).1).fold(0.0, f32::max))
                    .collect::<Vec<_>>();
                (max_w, h.iter().sum::<f32>() + gap * h.len().saturating_sub(1) as f32)
            }
            What::Grid { items, min, gap, even: false, puzzle, .. } => {
                let w = if self.natural.get() { self.packed_natural(items, *min, *gap, max_w) } else { max_w };
                (w, self.packed(items, *min, *gap, w, *puzzle).height)
            }
            _ => unreachable!(),
        }
    }

    pub(super) fn column_sizes<M>(&self, n: &Node<M>, items: &[Node<M>], inner: f32, gap: f32) -> Vec<(f32, f32)> {
        let measured = |rooms: &[f32], cut: f32| -> Vec<(f32, f32)> {
            items.iter().zip(rooms).map(|(i, w)| {
                let (w, h) = self.measure(i, *w);
                if matches!(i.what, What::Scroll(_)) { (w, (h - cut).max(self.px(ROUND_LEAST_LIST).min(h))) } else { (w, h) }
            }).collect()
        };
        let Some((across, margin)) = n.round else { return measured(&vec![inner; items.len()], 0.0) };
        let (radius, margin) = (self.px(across) / 2.0, self.px(margin));
        let gaps = gap * items.len().saturating_sub(1) as f32;
        let lists = items.iter().filter(|i| matches!(i.what, What::Scroll(_))).count();
        let wants: Vec<Option<f32>> = items.iter().map(|i| {
            let fixed = i.width != Length::Fill && !matches!(i.what, What::Label { .. } | What::Rich { .. } | What::Scroll(_) | What::Space);
            fixed.then(|| self.measure(i, inner).0)
        }).collect();
        let mut cut = 0.0;
        let mut sizes = Vec::new();
        for _ in 0..ROUND_PASSES {
            let mut rooms = vec![inner; items.len()];
            for _ in 0..ROUND_PASSES {
                sizes = measured(&rooms, cut);
                let tall = sizes.iter().map(|s| s.1).sum::<f32>() + gaps;
                let (mut y, mut changed) = (-tall / 2.0, false);
                for ((room, (_, h)), want) in rooms.iter_mut().zip(&sizes).zip(&wants) {
                    let far = y.abs().max((y + h).abs());
                    let chord = (2.0 * ((radius * radius - far * far).max(0.0).sqrt() - margin)).floor().clamp(0.0, inner);
                    let chord = chord.max(want.unwrap_or(0.0).min(inner));
                    if chord < *room {
                        (*room, changed) = (chord, true);
                    }
                    y += h + gap;
                }
                if !changed {
                    break;
                }
            }
            if lists == 0 {
                break;
            }
            let tall = sizes.iter().map(|s| s.1).sum::<f32>() + gaps;
            let (mut y, mut over) = (-tall / 2.0, 0.0f32);
            for (want, (_, h)) in wants.iter().zip(&sizes) {
                if let Some(want) = want {
                    let half = want / 2.0 + margin + 1.0;
                    let reach = (radius * radius - half * half).max(0.0).sqrt();
                    over = over.max(y.abs().max((y + h).abs()) - reach);
                }
                y += h + gap;
            }
            if over <= 0.5 {
                break;
            }
            cut += (2.0 * over / lists as f32).ceil();
        }
        sizes
    }

    pub(super) fn row_widths<M>(&self, items: &[Node<M>], inner: f32, gap: f32) -> Vec<f32> {
        let mut left = inner - gap * items.len().saturating_sub(1) as f32;
        let mut widths = vec![0.0; items.len()];
        let natural = self.natural.get();
        let set = |i: &Node<M>| matches!(i.what, What::Button { words: None, .. } | What::Switch { .. } | What::Tick { .. } | What::Drawing { .. }) || matches!(i.width, Length::Fixed(_));
        for first in [true, false] {
            let group: Vec<usize> = items.iter().enumerate().filter(|(_, i)| (natural || i.width != Length::Fill) && set(i) == first).map(|(i, _)| i).collect();
            if !first {
                let mut pending = group.clone();
                let mut wanted = vec![0.0; items.len()];
                for &i in &group {
                    wanted[i] = self.measure(&items[i], WHOLE_ROOM).0;
                }
                let want = |i: usize| wanted[i];
                let mut squeezed = Vec::new();
                while pending.iter().map(|&i| want(i)).sum::<f32>() > left.max(0.0) && pending.len() > 1 {
                    let widest = pending.iter().copied().max_by(|&a, &b| want(a).total_cmp(&want(b))).unwrap_or(0);
                    pending.retain(|&p| p != widest);
                    squeezed.push(widest);
                }
                for &i in &pending {
                    widths[i] = want(i).min(left.max(0.0));
                    left -= widths[i];
                }
                squeezed.sort_unstable();
                for i in squeezed {
                    widths[i] = self.measure(&items[i], left.max(0.0)).0;
                    left -= widths[i];
                }
                continue;
            }
            for (i, item) in group.into_iter().map(|i| (i, &items[i])) {
                widths[i] = self.measure(item, WHOLE_ROOM).0;
                left -= widths[i];
            }
        }
        if natural {
            return widths;
        }
        let fills = items.iter().filter(|i| i.width == Length::Fill).count();
        for (i, _) in items.iter().enumerate().filter(|(_, i)| i.width == Length::Fill) {
            widths[i] = (left / fills as f32).floor().max(0.0);
        }
        widths
    }

    pub(super) fn row_lines<M>(&self, items: &[Node<M>], inner: f32, gap: f32, loose: bool) -> Option<Vec<Vec<(usize, f32, f32)>>> {
        let set = |i: &Node<M>| i.width != Length::Fill && match i.what {
            What::Button { words: None, .. } | What::Switch { .. } | What::Tick { .. } | What::Drawing { .. } | What::Space => true,
            What::Button { .. } | What::Entry { .. } | What::Choice { .. } | What::Dropdown { .. } | What::Slider { .. } | What::Label { .. } | What::Rich { .. } => loose,
            _ => false,
        };
        let fills = |i: &Node<M>| i.width == Length::Fill && matches!(i.what, What::Space);
        if !items.iter().all(|i| set(i) || fills(i)) {
            return None;
        }
        let parts: Vec<(usize, f32, f32)> = items.iter().enumerate().filter(|(_, i)| !fills(i))
            .map(|(at, i)| {
                let w = self.measure(i, WHOLE_ROOM).0.min(inner.max(0.0));
                (at, w, self.measure(i, w).1)
            }).collect();
        let fits = |line: &[(usize, f32, f32)]| line_width(line, gap) <= inner + self.rounding(line.len());
        if parts.len() < 2 || fits(&parts) {
            return None;
        }
        let mut lines = 1;
        let mut from = 0;
        for at in 0..parts.len() {
            if at > from && !fits(&parts[from..=at]) {
                lines += 1;
                from = at;
            }
        }
        loop {
            let (each, more) = (parts.len() / lines, parts.len() % lines);
            let mut shared = Vec::with_capacity(lines);
            let mut from = 0;
            for line in 0..lines {
                let n = each + usize::from(line < more);
                shared.push(parts[from..from + n].to_vec());
                from += n;
            }
            if lines >= parts.len() || shared.iter().all(|l| l.len() == 1 || fits(l)) {
                return Some(shared);
            }
            lines += 1;
        }
    }

    pub(super) fn row_stacks<M>(&self, items: &[Node<M>], inner: f32, gap: f32) -> bool {
        let words = items.iter().filter(|i| i.width == Length::Fill && !matches!(i.what, What::Space)).count();
        if words == 0 || items.len() < 2 {
            return false;
        }
        let widths = self.row_widths(items, inner, gap);
        items.iter().zip(&widths).any(|(i, w)| i.width == Length::Fill && !matches!(i.what, What::Space) && *w < self.px(theme::ROW_WORDS))
    }

    pub(super) fn grown_cells<M>(&self, items: &[Node<M>], width: f32, min: f32, gap: f32, room: Option<f32>) -> (usize, f32, f32, f32) {
        let (most, _, gap) = self.grid_cells(width, min, gap, items.len());
        let least = self.px(min).max(1.0);
        let shape = items.first().map_or(1.0, |i| self.measure(i, least).1 / least);
        let made = |cols: usize| {
            let cell = ((width - gap * (cols - 1) as f32) / cols as f32).floor();
            let high = (cell * shape).ceil();
            let rows = items.len().div_ceil(cols);
            (cols, cell, high, rows as f32 * high + gap * rows.saturating_sub(1) as f32)
        };
        let fits = |t: &(usize, f32, f32, f32)| room.is_some_and(|room| t.3 <= room + 0.5);
        let chosen = (1..=most).map(made).filter(fits).max_by(|a, b| a.3.total_cmp(&b.3)).unwrap_or_else(|| made(most));
        (chosen.0, chosen.1, chosen.2, gap)
    }

    pub(super) fn grid_cells(&self, width: f32, min: f32, gap: f32, count: usize) -> (usize, f32, f32) {
        let (min, gap) = (self.px(min).max(1.0), self.px(gap));
        let cols = (((width + gap) / (min + gap)).floor() as usize).min(count).max(1);
        (cols, ((width - gap * (cols - 1) as f32) / cols as f32).floor(), gap)
    }
}
