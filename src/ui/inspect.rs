//! Tools for a program's author, off until a key: the inspector (Ctrl+Shift+F12) names the widget under the pointer, the speed overlay (Ctrl+Shift+F11) shows paint time, memory and how much was painted again.

use super::*;

#[derive(Default)]
pub(crate) struct Dev {
    inspect: bool,
    pub(super) speed: bool,
    paint: Cell<Duration>,
    damage: Cell<f32>,
    pub(super) gpu: Cell<bool>,
}

fn kind<M>(what: &What<M>) -> &'static str {
    match what {
        What::Column { .. } => "column",
        What::Row { .. } => "row",
        What::Grid { .. } => "grid",
        What::Card(..) => "card",
        What::Space => "space",
        What::Label { .. } => "label",
        What::Ring { .. } => "ring",
        What::Button { .. } => "button",
        What::Switch { .. } => "switch",
        What::Tick { .. } => "tick",
        What::Slider { .. } => "slider",
        What::Entry { .. } => "entry",
        What::Choice { .. } => "choice",
        What::Dropdown { .. } => "dropdown",
        What::Scroll(..) => "scroll",
        What::ScrollX(..) => "scroll_x",
        What::List { .. } => "list",
        What::Spinner { .. } => "spinner",
        What::Drawing { .. } => "drawing",
        What::Layers(..) => "layers",
        What::Placed { .. } => "placed",
        What::Slide { .. } => "slide",
        What::Aspect { .. } => "aspect",
        What::Rich { .. } => "rich",
        What::Split { .. } => "split",
        What::Divider { .. } => "divider",
    }
}

fn sheet_names<M>(what: &What<M>) -> &'static [&'static str] {
    match what {
        What::Button { words: None, .. } => &["trindle-icon-button-corner", "trindle-button-height"],
        What::Button { .. } => &["trindle-button-corner", "trindle-button-height"],
        What::Switch { .. } => &["trindle-switch-corner", "trindle-knob-corner"],
        What::Tick { .. } => &["trindle-tick-corner"],
        What::Slider { .. } => &["trindle-slider-corner", "trindle-knob-corner"],
        What::Entry { .. } | What::Dropdown { .. } => &["trindle-box-corner", "trindle-button-height"],
        What::Choice { .. } => &["trindle-segmented-corner", "trindle-button-height"],
        What::Card(..) => &["trindle-card-corner", "trindle-card-padding"],
        What::List { .. } => &["trindle-row-corner"],
        What::Scroll(..) | What::ScrollX(..) => &["trindle-scrollbar-corner"],
        What::Column { .. } | What::Row { .. } | What::Grid { .. } => &["trindle-gap"],
        What::Divider { .. } => &["trindle-divider", "trindle-divider-lit"],
        _ => &[],
    }
}

fn memory_mb() -> Option<f32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let kb: f32 = status.lines().find_map(|l| l.strip_prefix("VmRSS:"))?.trim().trim_end_matches("kB").trim().parse().ok()?;
    Some(kb / 1024.0)
}

impl<M: Clone> Ui<M> {
    pub(super) fn dev_key(&mut self, input: &Input) -> bool {
        let Input::Key { sym, mods: Mods { ctrl: true, shift: true, alt: false, logo: false }, repeat: false, .. } = *input else { return false };
        match sym {
            Keysym::F12 => self.dev.inspect = !self.dev.inspect,
            Keysym::F11 => self.dev.speed = !self.dev.speed,
            _ => return false,
        }
        true
    }

    pub(super) fn inspecting(&self) -> bool {
        self.dev.inspect
    }

    pub fn inspected(&mut self) -> Option<(Rect, Vec<String>)> {
        let (x, y) = self.pointer.filter(|_| self.dev.inspect)?;
        let spot = self.spots.iter().rev().find(|s| s.shows(x, y))?;
        let (id, r) = (spot.id.clone(), spot.rect);
        let n = self.tree.at(&spot.path)?;
        let (what_kind, names) = (kind(&n.what), sheet_names(&n.what));
        let states: Vec<&str> = [
            (self.focus.as_deref() == Some(&id), crate::t!("kit-inspect-keyboard")),
            (self.hover.as_deref() == Some(&id), crate::t!("kit-inspect-pointer")),
            (self.pressed.as_deref() == Some(&id), crate::t!("kit-inspect-pressed")),
            (n.focusable(), crate::t!("kit-inspect-takes-keys")),
            (n.off, crate::t!("kit-inspect-off")),
        ].into_iter().filter_map(|(on, said)| on.then_some(said)).collect();
        let state = if states.is_empty() { crate::t!("kit-inspect-rest").to_string() } else { states.join(", ") };
        let mut lines = vec![
            format!("{what_kind} {id}"),
            crate::t!("kit-inspect-size", w = r.w.round(), h = r.h.round(), x = r.x.round(), y = r.y.round()),
            crate::t!("kit-inspect-state", state = state),
        ];
        lines.push(match self.told_of(&id) {
            Some((role, name)) => crate::t!("kit-inspect-reader", role = role, name = name),
            None => crate::t!("kit-inspect-unread").to_string(),
        });
        let layers = crate::common::style::layers();
        let light = self.theme.light;
        for name in names {
            let value = layers.each(name, light).next().unwrap_or("");
            lines.push(format!("--{name}: {value}"));
        }
        Some((r, lines))
    }

    pub fn inspect(&mut self, on: bool) {
        self.dev.inspect = on;
    }

    pub fn show_speed(&mut self, on: bool) {
        self.dev.speed = on;
    }

    pub fn speed(&self) -> Option<Vec<String>> {
        self.dev.speed.then(|| vec![
            crate::t!("kit-speed-paint", ms = format!("{:.1}", self.dev.paint.get().as_secs_f64() * 1000.0)),
            crate::t!("kit-speed-memory", mb = format!("{:.0}", memory_mb().unwrap_or(0.0))),
            crate::t!("kit-speed-damage", share = format!("{:.0}", self.dev.damage.get() * 100.0)),
            if self.dev.gpu.get() { crate::t!("kit-speed-gpu") } else { crate::t!("kit-speed-processor") }.to_string(),
        ])
    }

    pub(super) fn painted_in(&self, took: Duration, share: f32) {
        self.dev.paint.set(took);
        self.dev.damage.set(share.clamp(0.0, 1.0));
    }

    pub(super) fn paint_dev(&mut self, canvas: &mut PixmapMut) {
        let inspected = self.inspected();
        self.paint_dev_with(canvas, inspected);
    }

    pub(super) fn paint_dev_with(&self, canvas: &mut PixmapMut, inspected: Option<(Rect, Vec<String>)>) {
        if let Some((r, lines)) = inspected {
            edge(canvas, r, 0.0, self.theme.accent, self.scale);
            let layout = label_layout(&self.fonts, &lines.join("\n"), self.scale, canvas.width() as f32);
            label_pill(canvas, &self.fonts, self.scale, &layout, r, canvas.height() as f32, theme::sheet().label_ink);
        }
        self.paint_speed(canvas);
    }

    pub(super) fn paint_speed(&self, canvas: &mut PixmapMut) -> Option<Rect> {
        let lines = self.speed()?;
        let (cw, ch) = (canvas.width() as f32, canvas.height() as f32);
        let layout = label_layout(&self.fonts, &lines.join("\n"), self.scale, cw);
        let corner = Rect { x: cw, y: 0.0, w: 0.0, h: 0.0 };
        let fall = theme::sheet().label_shadow;
        let b = label_rect(&layout, corner, (cw, ch), self.scale).inset(-((fall.y + fall.blur) * self.scale + self.scale.max(1.0) * 2.0).ceil());
        let whole = Rect { x: 0.0, y: 0.0, w: cw, h: ch };
        let b = b.intersect(&whole);
        let (x, y) = (b.x.floor(), b.y.floor());
        let part = Rect { x, y, w: (b.x + b.w).ceil() - x, h: (b.y + b.h).ceil() - y };
        if let Some(under) = self.paint_part(part) {
            crate::common::shm::put(canvas.data_mut(), (cw as i32, ch as i32), under.data(), [x as i32, y as i32, under.width() as i32, under.height() as i32]);
        }
        label_pill(canvas, &self.fonts, self.scale, &layout, corner, ch, theme::sheet().label_ink);
        Some(part)
    }
}
