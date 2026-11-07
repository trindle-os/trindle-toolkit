//! Ready-made layer-shell panels (bar, dock, on-screen display, wallpaper) and the place of an applet's popup beside its panel button.

use super::{Edges, Keys, Layer, Panel};

impl Edges {
    pub fn has(self, edge: Edges) -> bool {
        self.0 & edge.0 == edge.0
    }
}

impl Panel {
    fn plain(name: &str, layer: Layer, edges: Edges, keep: i32, keys: Keys) -> Panel {
        Panel { layer, edges, keep, margins: [0; 4], keys, name: name.into(), screen: None }
    }

    pub fn bar(name: &str, edge: Edges, thick: i32) -> Panel {
        let across = if edge.has(Edges::TOP) || edge.has(Edges::BOTTOM) { Edges::LEFT | Edges::RIGHT } else { Edges::TOP | Edges::BOTTOM };
        Panel::plain(name, Layer::Top, edge | across, thick, Keys::OnClick)
    }

    pub fn dock(name: &str, edge: Edges) -> Panel {
        Panel::plain(name, Layer::Top, edge, 0, Keys::OnClick)
    }

    pub fn display(name: &str) -> Panel {
        Panel::plain(name, Layer::Overlay, Edges::NONE, 0, Keys::Never)
    }

    pub fn wallpaper(name: &str, screen: Option<String>) -> Panel {
        Panel { screen, ..Panel::plain(name, Layer::Background, Edges::ALL, -1, Keys::Never) }
    }

    pub fn popup(&self, name: &str, panel_size: (u32, u32), button: [f32; 4], size: (u32, u32), room: (u32, u32), gap: i32) -> Panel {
        let [mt, mr, mb, ml] = self.margins;
        let (pw, ph, rw, rh) = (panel_size.0 as i32, panel_size.1 as i32, room.0 as i32, room.1 as i32);
        let place = |near: bool, far: bool, from: i32, to: i32, len: i32, whole: i32| match (near, far) {
            (true, _) => from,
            (false, true) => whole - len - to,
            (false, false) => (whole - len) / 2,
        };
        let (px, py) = (place(self.edges.has(Edges::LEFT), self.edges.has(Edges::RIGHT), ml, mr, pw, rw),
                        place(self.edges.has(Edges::TOP), self.edges.has(Edges::BOTTOM), mt, mb, ph, rh));
        let (w, h) = (size.0 as i32, size.1 as i32);
        let [bx, by, bw, bh] = button.map(|v| v.round() as i32);
        let fit = |v: i32, len: i32, whole: i32| v.min(whole - len - gap).max(gap);
        let (x, y) = match self.edges {
            e if e.has(Edges::TOP) && !e.has(Edges::BOTTOM) => (fit(px + bx + (bw - w) / 2, w, rw), py + ph + gap),
            e if e.has(Edges::BOTTOM) && !e.has(Edges::TOP) => (fit(px + bx + (bw - w) / 2, w, rw), py - h - gap),
            e if e.has(Edges::RIGHT) && !e.has(Edges::LEFT) => (px - w - gap, fit(py + by + (bh - h) / 2, h, rh)),
            _ => (px + pw + gap, fit(py + by + (bh - h) / 2, h, rh)),
        };
        Panel { margins: [y.max(0), 0, 0, x.max(0)], screen: self.screen.clone(),
                ..Panel::plain(name, self.layer, Edges::TOP | Edges::LEFT, -1, Keys::OnClick) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ready_made_panels_sit_where_their_names_say() {
        let bar = Panel::bar("bar", Edges::TOP, 32);
        assert_eq!((bar.edges, bar.keep, bar.layer), (Edges::TOP | Edges::LEFT | Edges::RIGHT, 32, Layer::Top));
        assert_eq!(Panel::bar("side", Edges::LEFT, 48).edges, Edges::LEFT | Edges::TOP | Edges::BOTTOM);
        let dock = Panel::dock("dock", Edges::BOTTOM);
        assert_eq!((dock.edges, dock.keep, dock.keys), (Edges::BOTTOM, 0, Keys::OnClick));
        let osd = Panel::display("volume");
        assert_eq!((osd.layer, osd.edges, osd.keys), (Layer::Overlay, Edges::NONE, Keys::Never));
        let paper = Panel::wallpaper("paper", Some("DP-1".into()));
        assert_eq!((paper.layer, paper.edges, paper.keep, paper.screen.as_deref()), (Layer::Background, Edges::ALL, -1, Some("DP-1")));
    }

    #[test]
    fn a_popup_opens_under_a_top_bar_button_and_stays_on_screen() {
        let bar = Panel::bar("bar", Edges::TOP, 32);
        let p = bar.popup("menu", (1000, 32), [500.0, 4.0, 24.0, 24.0], (200, 300), (1000, 800), 8);
        assert_eq!((p.edges, p.keep, p.margins), (Edges::TOP | Edges::LEFT, -1, [40, 0, 0, 412]));
        assert_eq!(bar.popup("menu", (1000, 32), [970.0, 4.0, 24.0, 24.0], (200, 300), (1000, 800), 8).margins, [40, 0, 0, 792]);
        assert_eq!(bar.popup("menu", (1000, 32), [0.0, 4.0, 24.0, 24.0], (200, 300), (1000, 800), 8).margins, [40, 0, 0, 8]);
    }

    #[test]
    fn a_popup_opens_above_a_dock_and_beside_a_side_bar() {
        let dock = Panel { margins: [0, 0, 6, 0], ..Panel::dock("dock", Edges::BOTTOM) };
        let p = dock.popup("menu", (400, 60), [100.0, 8.0, 44.0, 44.0], (120, 200), (1000, 800), 8);
        assert_eq!(p.margins, [800 - 6 - 60 - 8 - 200, 0, 0, 300 + 100 + 22 - 60]);
        let side = Panel::bar("side", Edges::RIGHT, 48);
        let p = side.popup("menu", (48, 800), [4.0, 100.0, 40.0, 40.0], (200, 100), (1000, 800), 8);
        assert_eq!(p.margins, [100 + 20 - 50, 0, 0, 1000 - 48 - 8 - 200]);
        let left = Panel::bar("left", Edges::LEFT, 48);
        assert_eq!(left.popup("menu", (48, 800), [4.0, 760.0, 40.0, 40.0], (200, 100), (1000, 800), 8).margins, [692, 0, 0, 56]);
    }
}
