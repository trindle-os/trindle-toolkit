//! What a look does: how `Theme` is read and held from the sheets, and the colours it hands to what is drawn.

use crate::common::style::{self, Layers};

use super::{alpha, as_laid, clear, contrast, default_accent, dim_share, kept, near, on, over, pairs, ring_edge_seen, usual, vouched, Colour, Reader, Theme, AIM_EDGES, AIM_WORDS, EDGES, HIGH_EDGES, HIGH_WORDS, LOOK};

impl Theme {
    pub fn faded(&self) -> Theme {
        let quiet = self.muted;
        Theme { fg: quiet, accent: quiet, lit_edge: quiet, accent_text: quiet, folder: quiet, soft: quiet, icon: quiet, danger: quiet, danger_text: quiet,
                on_accent: self.on_greyed(), knob: self.knob_greyed(), ..self.clone() }
    }

    pub(crate) fn on_greyed(&self) -> Colour {
        let quiet = Colour { a: 1.0, ..self.muted };
        if contrast(self.on_accent, quiet) >= AIM_WORDS { self.on_accent } else { on(quiet) }
    }

    pub(crate) fn knob_greyed(&self) -> Colour {
        let quiet = Colour { a: 1.0, ..self.muted };
        clear(Colour { a: 1.0, ..self.knob }, &[quiet], AIM_EDGES)
    }

    pub fn words_on(&self, under: &[Colour]) -> Colour {
        vouched(clear(Colour { a: 1.0, ..self.fg }, &as_laid(under), AIM_WORDS), under)
    }

    pub fn quiet_on(&self, under: &[Colour]) -> Colour {
        vouched(clear(Colour { a: 1.0, ..self.muted }, &as_laid(under), AIM_WORDS), under)
    }

    pub fn link_on(&self, under: &[Colour]) -> Colour {
        vouched(clear(Colour { a: 1.0, ..self.accent_text }, &as_laid(under), AIM_WORDS), under)
    }

    pub fn ink_on(&self, ink: Colour, under: &[Colour]) -> Colour {
        vouched(clear(Colour { a: 1.0, ..ink }, &as_laid(under), AIM_WORDS), under)
    }

    pub fn wash(&self, wash: Colour, under: &[Colour]) -> Colour {
        self.wash_under(self.fg, wash, under)
    }

    pub fn wash_under(&self, ink: Colour, wash: Colour, under: &[Colour]) -> Colour {
        #[cfg(any(test, feature = "testing"))]
        if super::WASH_AS_WAS.get() {
            return wash;
        }
        let solid = Colour { a: 1.0, ..ink };
        let reads = |w: Colour| under.iter().all(|u| near(over(w, kept(*u))).iter().all(|g| contrast(solid, *g) >= AIM_WORDS));
        let thinned = (0..=20).map(|i| alpha(wash, 1.0 - i as f32 / 20.0)).find(|w| reads(*w)).unwrap_or(alpha(wash, 0.0));
        #[cfg(any(test, feature = "testing"))]
        super::drawn::vouch(ink, &under.iter().map(|u| over(thinned, *u)).collect::<Vec<_>>());
        thinned
    }

    pub fn wash_on(&self, ink: Colour, wash: Colour, ground: Option<Colour>) -> Colour {
        match ground {
            Some(ground) => self.wash_under(ink, wash, &[ground]),
            None => self.wash_under(ink, wash, &pairs::layers_of(self)),
        }
    }

    pub fn dim(&self, share: f32, floor: f32) -> f32 {
        dim_share(self.fg, share, &pairs::layers_of(self), floor)
    }

    pub fn chosen() -> Theme {
        let light = style::light();
        let _ = LOOK.compare_exchange(0, 1 + u8::from(light), std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed);
        Theme::new(light, &style::accent())
    }

    pub fn for_page(page: &str) -> Theme {
        #[cfg(any(test, feature = "testing"))]
        super::drawn::asked_page(page);
        Theme::on_page(&style::layers(), style::light(), &style::accent(), page)
    }

    pub(super) fn on_page(layers: &Layers, light: bool, accent: &str, page: &str) -> Theme {
        let plain = Theme::from(layers, light, accent);
        let r = Reader::new(layers, plain.light, plain.accent);
        match r.named(&format!("trindle-{page}-page"), 0) {
            Some(bg) => Theme::held_on(layers, light, accent, Some(bg)),
            None => plain,
        }
    }

    pub fn new(light: bool, accent: &str) -> Theme {
        #[cfg(any(test, feature = "testing"))]
        if let Some((light, accent)) = style::asked_for_test() {
            return Theme::from(&style::layers(), light, &accent);
        }
        Theme::from(&style::layers(), light, accent)
    }

    pub(super) fn read(layers: &Layers, light: bool, accent: Colour) -> Theme {
        let r = Reader::new(layers, light, accent);
        let field = r.falls("trindle-typing-box-shadow");
        Theme {
            light,
            bg: r.c("trindle-page"),
            panel: r.c("trindle-card"),
            ring: Colour { a: 1.0, ..r.c("trindle-ring-grey") },
            ring_edge: ring_edge_seen(Colour { a: 1.0, ..r.c("trindle-ring-edge") }),
            raised: r.c("trindle-box"),
            raised_more: r.c("trindle-box-more"),
            hover: r.c("trindle-hover"),
            control: r.c("trindle-control"),
            control_edge: r.c("trindle-control-edge"),
            field: r.c("trindle-typing-box"),
            field_edge: r.c("trindle-typing-box-edge"),
            well: r.c("trindle-well"),
            fg: r.c("trindle-text"),
            muted: r.c("trindle-text-quiet"),
            on_accent: usual(accent),
            accent_text: r.c("trindle-highlight-text"),
            picked: r.c("trindle-picked"),
            on_picked: r.c("trindle-on-picked"),
            folder: r.c("trindle-folder-icon"),
            soft: r.c("trindle-highlight-soft"),
            icon: r.c("trindle-icon"),
            accent,
            lit_edge: accent,
            danger: r.c("trindle-bad"),
            danger_text: r.c("trindle-shade-red"),
            edge: r.c("trindle-edge"),
            card_edge: r.c("trindle-box-edge"),
            corrected: false,
            band: r.c("trindle-band"),
            side_line: r.c("trindle-side-line"),
            section_line: r.c("trindle-section-line"),
            top_bar: r.c("trindle-top-bar"),
            side_bar: r.c("trindle-side-bar"),
            foot: r.c("trindle-foot"),
            area_line: r.c("trindle-area-line"),
            trial_foot: r.c("trindle-trial-foot"),
            trial_foot_line: r.c("trindle-trial-foot-line"),
            current_edge: r.c("trindle-current-card-edge"),
            accent_hover: r.c("trindle-highlight-hover"),
            knob: r.c("trindle-knob"),
            knob_off: r.c("trindle-knob-off"),
            spelling: r.c("trindle-spelling"),
            shadow: r.c("trindle-shadow"),
            float: r.falls("trindle-float-shadow"),
            panel_falls: r.falls("trindle-card-shadow"),
            raised_falls: r.falls("trindle-box-shadow"),
            field_inset: field.first().copied(),
        }
    }

    pub(super) fn from(layers: &Layers, light: bool, accent: &str) -> Theme {
        Theme::held_on(layers, light, accent, None)
    }

    fn held_on(layers: &Layers, light: bool, accent: &str, page: Option<Colour>) -> Theme {
        let page = |mut t: Theme| {
            if let Some(bg) = page {
                t.bg = bg;
            }
            t
        };
        let light = layers.light(light);
        let chosen = Colour::parse(accent).unwrap_or_else(default_accent);
        let chosen = Colour { a: 1.0, ..chosen };
        let raw = page(Theme::read(layers, light, chosen));
        let mut held = raw.clone();
        let dark = pairs::hold_layers(&mut held, layers);
        let accent = pairs::highlight(chosen, &held);
        let rest = if accent == chosen { raw.clone() } else { page(Theme::read(layers, light, accent)) };
        let layered = |from: &Theme| Theme { bg: from.bg, panel: from.panel, ring: from.ring, raised: from.raised, raised_more: from.raised_more, hover: from.hover,
                                             control: from.control, field: from.field, well: from.well, top_bar: from.top_bar,
                                             side_bar: from.side_bar, foot: from.foot, trial_foot: from.trial_foot, ..rest.clone() };
        let (was, mut t) = (layered(&raw), pairs::hold_inks(layered(&held), layers, dark));
        let own = |t: &Theme| [t.bg, t.panel, t.ring, t.raised, t.raised_more, t.field, t.control, t.hover, t.well, t.fg, t.muted, t.danger_text, t.danger,
                               t.field_edge, t.top_bar, t.side_bar, t.foot, t.trial_foot, t.band];
        t.corrected = own(&t) != own(&was);
        if Reader::new(layers, light, chosen).high_contrast() {
            t.raise_contrast();
        }
        t
    }

    pub(crate) fn raise_contrast(&mut self) {
        let area = |c: Colour| if c.a > 0.0 { over(c, self.bg) } else { self.bg };
        let grounds = [self.bg, self.panel, self.raised, self.raised_more, self.field, self.control, self.hover, self.well, area(self.top_bar), area(self.side_bar),
                       area(self.foot), self.trial_foot, area(self.band)];
        let solid = |c: Colour| Colour { a: 1.0, ..c };
        for ink in [&mut self.fg, &mut self.muted, &mut self.icon, &mut self.accent_text, &mut self.danger_text] {
            *ink = clear(solid(*ink), &grounds, HIGH_WORDS);
        }
        for line in [&mut self.edge, &mut self.card_edge, &mut self.control_edge, &mut self.field_edge, &mut self.area_line, &mut self.side_line,
                     &mut self.section_line, &mut self.current_edge, &mut self.trial_foot_line] {
            *line = clear(solid(*line), &grounds, HIGH_EDGES);
        }
        self.lit_edge = clear(solid(self.accent), &grounds, HIGH_EDGES);
        self.on_picked = clear(self.on_picked, &[self.picked], HIGH_WORDS);
        self.on_accent = clear(self.on_accent, &[self.accent], HIGH_WORDS);
    }

    pub fn folder_ink(&self, own: Option<Colour>, under: &[Colour]) -> Colour {
        if let Some(own) = own {
            return own;
        }
        let found = |c: &Colour| under.iter().all(|u| contrast(over(*c, kept(*u)), *u) >= EDGES);
        let chosen = [self.folder, self.on_picked].into_iter().find(found).unwrap_or(self.folder);
        vouched(clear(Colour { a: 1.0, ..chosen }, &as_laid(under), AIM_EDGES), under)
    }

    pub fn card(&self, depth: usize) -> Colour {
        match depth {
            0 => self.panel,
            1 => self.raised,
            _ => self.raised_more,
        }
    }
}
