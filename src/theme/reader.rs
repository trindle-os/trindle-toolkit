//! Reading the sheet: named values from the layers, and the toolkit's measures made from them.

#[cfg(any(test, feature = "testing"))]
use std::cell::RefCell;
use std::time::Duration;

use crate::common::style::{self, split_top, Layers};

use super::{alpha, builtin, fall, kin, mix, pairs, shade, AppPage, AppsList, Bento, Colour, Falls, FoldersRoom, Gaps, Palette, Part, Safe, Sheet, Welcome, FULL_ROUND, HIDDEN_LEAST, MOST_FALLS, TARGET_LEAST, TERM, UNREAD};

pub(super) struct Reader<'a> {
    pub(super) layers: &'a Layers,
    pub(super) light: bool,
    accent: Colour,
    #[cfg(any(test, feature = "testing"))]
    pub(super) missing: RefCell<Vec<String>>,
    #[cfg(any(test, feature = "testing"))]
    pub(super) limits: RefCell<Vec<(String, f32, f32)>>,
}

impl<'a> Reader<'a> {
    pub(super) fn new(layers: &'a Layers, light: bool, accent: Colour) -> Reader<'a> {
        Reader {
            layers,
            light,
            accent,
            #[cfg(any(test, feature = "testing"))]
            missing: RefCell::default(),
            #[cfg(any(test, feature = "testing"))]
            limits: RefCell::default(),
        }
    }

    pub(super) fn miss<T>(&self, name: &str, instead: T) -> T {
        #[cfg(any(test, feature = "testing"))]
        self.missing.borrow_mut().push(name.to_string());
        #[cfg(not(any(test, feature = "testing")))]
        let _ = name;
        instead
    }

    pub(super) fn n(&self, name: &str, min: f32, max: f32) -> f32 {
        #[cfg(any(test, feature = "testing"))]
        self.limits.borrow_mut().push((name.to_string(), min, max));
        self.layers.number(name, self.light, min, max).unwrap_or_else(|| self.miss(name, min))
    }

    pub(super) fn ms(&self, name: &str, min: f32, max: f32) -> Duration {
        Duration::from_millis(self.n(name, min, max) as u64)
    }

    fn step(&self, name: &str) -> f32 {
        self.layers.step(name, self.light).unwrap_or_else(|| self.miss(name, 0.0))
    }

    fn span(&self, least: (&str, f32, f32), most: (&str, f32, f32)) -> (u8, u8) {
        let low = self.n(least.0, least.1, least.2) as u8;
        (low, (self.n(most.0, most.1, most.2) as u8).max(low))
    }

    pub(super) fn weight(&self, name: &str) -> u16 {
        ((self.n(name, 100.0, 900.0) / 100.0).round() * 100.0) as u16
    }

    pub(super) fn type_of(&self, name: &str, min: f32, max: f32) -> (f32, u16) {
        (self.n(&format!("{name}-size"), min, max) * self.large(), self.weight(&format!("{name}-weight")))
    }

    pub(super) fn large(&self) -> f32 {
        self.n("trindle-large-text", 1.0, 2.0)
    }

    pub(super) fn high_contrast(&self) -> bool {
        self.n("trindle-high-contrast", 0.0, 1.0) >= 0.5
    }

    pub(super) fn c(&self, name: &str) -> Colour {
        #[cfg(any(test, feature = "testing"))]
        super::READ.with(|r| r.borrow_mut().push(name.to_string()));
        self.named(name, 0).unwrap_or_else(|| self.miss(name, UNREAD))
    }

    pub(super) fn named(&self, name: &str, depth: u8) -> Option<Colour> {
        if name == "trindle-highlight" {
            return Some(self.accent);
        }
        self.layers.each(name, self.light).find_map(|text| self.colour(text, depth + 1))
    }

    pub(super) fn colour(&self, text: &str, depth: u8) -> Option<Colour> {
        let text = text.trim();
        if depth > 8 {
            return None;
        }
        let call = |f: &str| text.strip_prefix(f).and_then(|s| s.trim_start().strip_prefix('(')).and_then(|s| s.strip_suffix(')'));
        if let Some(inner) = call("var") {
            return self.named(inner.trim().strip_prefix("--")?, depth + 1);
        }
        let num = |s: &str| s.trim().parse::<f32>().ok().filter(|v| v.is_finite());
        for (f, two) in [("shade", false), ("alpha", false), ("mix", true)] {
            let Some(inner) = call(f) else { continue };
            let args = split_top(inner, ',');
            return match (args.as_slice(), two) {
                ([c, by], false) if f == "shade" => Some(shade(self.colour(c, depth + 1)?, num(by)?.clamp(0.0, 4.0))),
                ([c, by], false) => Some(alpha(self.colour(c, depth + 1)?, num(by)?)),
                ([a, b, by], true) => Some(mix(self.colour(a, depth + 1)?, self.colour(b, depth + 1)?, num(by)?.clamp(0.0, 1.0))),
                _ => None,
            };
        }
        Colour::parse(text)
    }

    pub(super) fn falls(&self, name: &str) -> Falls {
        self.layers.each(name, self.light).find_map(Falls::parse)
            .unwrap_or_else(|| self.miss(name, Falls { each: [fall(0.0, 0.0, 0.0, 0.0); MOST_FALLS], count: 0 }))
    }
}

impl Sheet {
    pub(super) fn read(r: &Reader) -> Sheet {
        let n = |name: &str, min: f32, max: f32| r.n(name, min, max);
        let step = |name: &str| r.step(name);
        let big = r.large();
        let button = n("trindle-button-height", 28.0, 52.0) * big;
        let switch = (n("trindle-switch-width", 32.0, 64.0) * big, n("trindle-switch-height", 16.0, 36.0) * big);
        let text = r.type_of("trindle-text", 12.0, 22.0);
        let card_gap = n("trindle-card-gap", 4.0, 32.0);
        let entry_height = n("trindle-entry-height", 28.0, 52.0) * big;
        let safe_on = n("trindle-colour-blind-safe", 0.0, 1.0) >= 0.5;
        let safe = read_safe(r);
        let state = |light: bool, name: &str| read_state(r, safe_on, &safe, light, name);
        Sheet {
            button,
            icon: (n("trindle-icon-size", 12.0, 32.0) * big).min(button - 4.0),
            ring_button: n("trindle-ring-button-size", 20.0, 40.0),
            ring_app_edge: n("trindle-ring-app-edge", 1.0, 6.0),
            ring_button_gap: n("trindle-ring-button-gap", 2.0, 12.0),
            ring_band_room: n("trindle-ring-band-room", 2.0, 12.0),
            ring_band_gap: n("trindle-ring-band-gap", 2.0, 12.0),
            ring_job_programs: n("trindle-ring-job-programs", 0.0, 0.3),
            ring_job_computer: n("trindle-ring-job-computer", 0.0, 0.3),
            ring_job_tools: n("trindle-ring-job-tools", 0.0, 0.3),
            ring_job_tray: n("trindle-ring-job-tray", 0.0, 0.3),
            ring_windows_width: n("trindle-ring-windows-width", 56.0, 140.0),
            ring_windows_width_least: n("trindle-ring-windows-width-least", 40.0, 80.0),
            ring_windows_gap: n("trindle-ring-windows-gap", 0.0, 20.0),
            ring_windows_part_gap: n("trindle-ring-windows-part-gap", 2.0, 12.0),
            ring_windows_icon_alone: n("trindle-ring-windows-icon-alone", 20.0, 48.0),
            ring_windows_track: n("trindle-ring-windows-track", 0.0, 24.0),
            ring_edge_width: n("trindle-ring-edge-width", 1.0, 3.0),
            ring_edge_strength: n("trindle-ring-edge-strength", 0.1, 1.0),
            ring_windows_tab_firm: n("trindle-ring-windows-tab-firm", 0.0, 20.0),
            ring_windows_tab_front: n("trindle-ring-windows-tab-front", 6.0, 40.0),
            ring_windows_tab_pad: n("trindle-ring-windows-tab-pad", 2.0, 16.0),
            ring_windows_name_width: n("trindle-ring-windows-name-width", 40.0, 120.0) * big,
            ring_windows_title: n("trindle-ring-windows-title-size", 11.0, 20.0) * big,
            ring_windows_title_gap: n("trindle-ring-windows-title-gap", 4.0, 16.0),
            ring_windows_dot: n("trindle-ring-windows-dot", 3.0, 12.0),
            ring_windows_picture: n("trindle-ring-windows-picture", 120.0, 320.0),
            ring_windows_picture_margin: n("trindle-ring-windows-picture-margin", 6.0, 40.0),
            ring_windows_picture_corner: n("trindle-ring-windows-picture-corner", 0.0, 40.0),
            ring_windows_picture_icon: n("trindle-ring-windows-picture-icon", 48.0, 160.0),
            label_back: r.c("trindle-label-background"),
            label_ink: r.c("trindle-label-ink"),
            label_edge: r.c("trindle-label-edge"),
            label_outline: r.c("trindle-label-outline"),
            label_shadow: fall(n("trindle-label-shadow-y", 0.0, 12.0), n("trindle-label-shadow-blur", 0.0, 24.0), 0.0,
                               n("trindle-label-shadow-strength", 0.0, 1.0)),
            label_text: n("trindle-label-size", 10.0, 18.0) * big,
            label_weight: r.weight("trindle-label-weight"),
            label_padding: (n("trindle-label-padding-x", 2.0, 16.0), n("trindle-label-padding-y", 1.0, 10.0)),
            label_gap: n("trindle-label-gap", 0.0, 16.0),
            label_width_most: n("trindle-label-width-most", 160.0, 960.0),
            label_lines_most: n("trindle-label-lines-most", 3.0, 40.0) as usize,
            label_rest: r.ms("trindle-label-rest", 0.0, 2000.0),
            label_stays: r.ms("trindle-label-stays", 500.0, 10000.0),
            slide_time: r.ms("trindle-slide-time", 0.0, 1000.0),
            fade_time: r.ms("trindle-fade-time", 0.0, 1000.0),
            reduce_motion: n("trindle-reduce-motion", 0.0, 1.0) >= 0.5,
            notice_stays: r.ms("trindle-notice-stays", 2000.0, 20000.0),
            hover_edge: n("trindle-hover-edge", 1.0, 3.0),
            armed: r.ms("trindle-remove-armed", 2000.0, 10000.0),
            work_done: r.ms("trindle-work-done", 1000.0, 10000.0),
            work_heavy: (n("trindle-work-heavy-mb", 100.0, 100000.0), r.ms("trindle-work-heavy-after", 2000.0, 120000.0)),
            bento: read_bento(r, card_gap),
            welcome: read_welcome(r),
            app_page: read_app_page(r),
            apps_list: read_apps_list(r),
            folders: ["compact", "normal", "roomy"].map(|level| read_folders(r, level, big)),
            palette: Palette::ALL.map(|p| r.c(&format!("trindle-{}", p.name()))),
            term: TERM.map(|n| r.c(n)),
            pairs: Palette::ALL.map(|p| [r.c(&format!("trindle-pair-{}-first", p.name())), r.c(&format!("trindle-pair-{}-second", p.name()))]),
            info: n("trindle-i-size", 18.0, 34.0) * big,
            calm: state(false, "calm"),
            good: state(false, "good"),
            warn: state(false, "busy"),
            calm_light: state(true, "calm"),
            good_light: state(true, "good"),
            warn_light: state(true, "busy"),
            problem: state(false, "bad"),
            colour_blind_safe: safe_on,
            safe,
            quiet: r.c("trindle-quiet"),
            code: ["keyword", "type", "string", "number", "comment"].map(|k| r.c(&format!("trindle-code-{k}"))),
            corner: n("trindle-corner", 0.0, 16.0),
            corners: Part::ALL.map(|p| n(p.name(), 0.0, FULL_ROUND)),
            panel_gap: card_gap,
            page_padding: n("trindle-page-padding", 8.0, 48.0),
            gap: n("trindle-gap", 6.0, 18.0),
            card_padding: n("trindle-card-padding", 8.0, 40.0),
            snug_padding: n("trindle-snug-padding", 0.0, 12.0),
            sidebar_room: n("trindle-sidebar-room", 4.0, 20.0),
            line_padding: n("trindle-line-padding", 2.0, 12.0),
            chip_padding: (n("trindle-chip-padding-x", 4.0, 20.0), n("trindle-chip-padding-y", 0.0, 8.0)),
            tabs_padding: n("trindle-tabs-padding", 2.0, 12.0),
            focus_width: n("trindle-focus-width", 1.0, 4.0),
            focus_offset: n("trindle-focus-offset", 0.0, 4.0),
            switch,
            knob: (n("trindle-switch-knob", 10.0, 30.0) * big).min(switch.1 - 2.0),
            knob_colour: r.c("trindle-knob"),
            knob_shadow: r.c("trindle-knob-shadow"),
            knob_off: r.c("trindle-knob-off"),
            track: n("trindle-slider-track", 2.0, 12.0),
            entry_height,
            entry_padding: n("trindle-entry-padding", 4.0, 24.0),
            entry_eye: (n("trindle-entry-eye", 24.0, 44.0) * big).min(entry_height - 4.0),
            caret: n("trindle-caret", 1.0, 4.0),
            selection: n("trindle-selection", 0.1, 0.8),
            mark: n("trindle-mark", 0.1, 0.8),
            row: n("trindle-row-height", 28.0, 64.0) * big,
            rich_row: n("trindle-rich-row-height", 40.0, 80.0) * big,
            rich_gap: n("trindle-rich-row-gap", 0.0, 8.0),
            rich_icon: n("trindle-rich-row-icon", 16.0, 48.0) * big,
            rich_weight: r.weight("trindle-rich-row-weight"),
            tree_indent: n("trindle-tree-indent", 8.0, 40.0) * big,
            title_bar_room: n("trindle-title-bar-room", 2.0, 12.0),
            frame_edge: n("trindle-frame-edge", 2.0, 12.0),
            frame_shadow: n("trindle-frame-shadow", 0.0, 48.0),
            divider: n("trindle-divider", 1.0, 4.0),
            divider_lit: n("trindle-divider-lit", 2.0, 8.0),
            scrollbar: n("trindle-scrollbar", 2.0, 10.0),
            scrollbar_hover: n("trindle-scrollbar-hover", 4.0, 16.0),
            scrollbar_rest: n("trindle-scrollbar-rest", 0.2, 0.8),
            scrollbar_lit: n("trindle-scrollbar-lit", 0.5, 1.0),
            scrollbar_inset: n("trindle-scrollbar-inset", 0.0, 6.0),
            spinner: n("trindle-spinner", 20.0, 64.0) * big,
            spinner_line: n("trindle-spinner-line", 2.0, 8.0),
            preview_panel: n("trindle-preview-panel", 240.0, 480.0),
            preview_picture: n("trindle-preview-picture", 120.0, 400.0),
            sidebar_width: n("trindle-sidebar-width", 180.0, 360.0) * big,
            split_narrow: n("trindle-split-narrow", 360.0, 1200.0),
            drawer_width: n("trindle-drawer-width", 240.0, 480.0) * big,
            drawer_room: n("trindle-drawer-room", 44.0, 120.0),
            icon_tile: n("trindle-icon-tile", 88.0, 160.0) * big,
            icon_tile_icon: n("trindle-icon-tile-icon", 32.0, 96.0) * big,
            status_icon: n("trindle-status-icon", 48.0, 160.0) * big,
            dot: n("trindle-dot", 6.0, 16.0),
            dot_target: n("trindle-dot-target", 24.0, 44.0),
            fling_friction: n("trindle-fling-friction", 1.0, 12.0),
            drive_card: n("trindle-drive-card", 240.0, 480.0),
            fill_bar: n("trindle-fill-bar", 4.0, 20.0),
            chart_line: n("trindle-chart-line", 1.0, 6.0),
            radio: n("trindle-radio-size", 16.0, 28.0) * big,
            tick: n("trindle-tick-size", 14.0, 28.0) * big,
            gaps: read_gaps(r),
            colour_square: (step("trindle-colour-square-width"), step("trindle-colour-square-height")),
            colour_swatch: step("trindle-colour-swatch"),
            colour_swatch_corner: n("trindle-colour-swatch-corner", 0.0, 999.0),
            colour_code: step("trindle-colour-code-width"),
            gradient_bar: step("trindle-gradient-bar"),
            volume: step("trindle-volume-width"),
            thumb: step("trindle-photo-thumb"),
            lane: (step("trindle-board-lane"), step("trindle-board-card"), step("trindle-board-head")),
            table: (step("trindle-table-row"), step("trindle-table-head")),
            info_icon: step("trindle-info-icon"),
            bars: n("trindle-bars-height", 24.0, 80.0),
            bars_width: n("trindle-bars-width", 6.0, 32.0),
            bars_least: n("trindle-bars-least", 2.0, 12.0),
            bars_gap: n("trindle-bars-gap", 2.0, 12.0),
            bars_corner: n("trindle-bars-corner", 0.0, 999.0),
            skeleton_line: n("trindle-skeleton-line", 8.0, 24.0),
            skeleton_gap: n("trindle-skeleton-gap", 4.0, 20.0),
            skeleton_last: n("trindle-skeleton-last", 0.3, 1.0),
            hidden_drive: n("trindle-hidden-drive", HIDDEN_LEAST, 0.9),
            glow: r.ms("trindle-glow-time", 300.0, 5000.0),
            glow_width: n("trindle-glow-width", 1.0, 8.0),
            bell: r.ms("trindle-bell-time", 100.0, 1000.0),
            glow_strength: n("trindle-glow-strength", 0.1, 1.0),
            lit_width: n("trindle-lit-edge", 1.0, 6.0),
            working_line: n("trindle-working-line", 1.0, 6.0),
            action_height: button,
            action_padding: n("trindle-action-padding", 8.0, 32.0),
            action_gap: n("trindle-action-gap", 4.0, 16.0),
            action_text: n("trindle-action-size", 12.0, 20.0) * big,
            action_weight: r.weight("trindle-action-weight"),
            word_button: (n("trindle-word-button-width", 44.0, 72.0) * big, button),
            word_button_padding: n("trindle-word-button-padding", 6.0, 16.0),
            word_button_text: n("trindle-word-button-size", 12.0, 16.0) * big,
            page_title: r.type_of("trindle-page-title", 22.0, 44.0),
            section: r.type_of("trindle-section", 18.0, 36.0),
            lead: r.type_of("trindle-lead", 16.0, 30.0),
            heading: r.type_of("trindle-heading", 14.0, 24.0),
            headline: r.type_of("trindle-headline", 14.0, 24.0),
            figure: r.type_of("trindle-figure", 14.0, 48.0),
            big_figure: r.type_of("trindle-big-figure", 24.0, 72.0),
            text,
            form_label: r.type_of("trindle-form-label", 12.0, 20.0),
            note: read_note(r, text.0),
            column_heading: r.type_of("trindle-column-heading", 12.0, 18.0),
            side_heading: r.type_of("trindle-side-heading", 10.0, 16.0),
            tighten: n("trindle-title-tighten", 0.0, 0.06),
            menu_width: n("trindle-menu-width", 200.0, 360.0) * big,
            menu_row: n("trindle-menu-row", 24.0, 44.0) * big,
            menu_padding: n("trindle-menu-padding", 2.0, 12.0),
            menu_icon: n("trindle-menu-icon", 12.0, 24.0) * big,
            menu_sub_pause: r.ms("trindle-menu-sub-pause", 0.0, 1000.0),
            menu_sub_overlap: n("trindle-menu-sub-overlap", 0.0, 12.0),
            menu_sub_arrow: n("trindle-menu-sub-arrow", 8.0, 24.0),
            menu_title_pad: n("trindle-menu-title-pad", 0.0, 32.0),
            menu_group_gap: n("trindle-menu-group-gap", 0.0, 24.0),
            menu_edge: n("trindle-menu-edge", 0.0, 16.0),
            menu_row_least: n("trindle-menu-row-least", 16.0, 44.0) * big,
            tab_height: n("trindle-tab-height", 28.0, 72.0) * big,
            tab_least: n("trindle-tab-least", 60.0, 240.0) * big,
            tab_most: n("trindle-tab-most", 120.0, 480.0) * big,
            tab_close: n("trindle-tab-close", 24.0, 48.0) * big,
            tab_more: n("trindle-tab-more", 24.0, 64.0) * big,
            tab_pad: n("trindle-tab-pad", 0.0, 24.0),
            tab_slop: n("trindle-tab-slop", 0.0, 16.0),
            tab_tear: n("trindle-tab-tear", 16.0, 160.0),
            tab_count: n("trindle-tab-count", 16.0, 64.0) * big,
            pill_height: n("trindle-pill-height", 24.0, 64.0) * big,
            pill_side: n("trindle-pill-side", 4.0, 40.0),
            pill_widest: n("trindle-pill-widest", 240.0, 1200.0),
            pill_room: n("trindle-pill-room", 8.0, 120.0),
            pill_foot: n("trindle-pill-foot", 8.0, 160.0),
            pill_words: n("trindle-pill-words", 10.0, 24.0) * big,
            dropdown_height: n("trindle-dropdown-height", 28.0, 52.0) * big,
            dropdown_row: n("trindle-dropdown-row", 24.0, 44.0) * big,
            dropdown_rows: n("trindle-dropdown-rows", 3.0, 16.0).round(),
            dropdown_width: n("trindle-dropdown-width", 200.0, 720.0) * big,
            state_dot: n("trindle-state-dot", 10.0, 24.0),
            badge: n("trindle-badge", 28.0, 56.0) * big,
            badge_icon: n("trindle-badge-icon", 12.0, 32.0) * big,
            area_line: n("trindle-area-line-width", 1.0, 3.0),
            section_line_width: n("trindle-section-line-width", 1.0, 3.0),
            trial_foot: n("trindle-trial-foot-height", 56.0, 96.0) * big,
            trial_foot_padding: n("trindle-trial-foot-padding", 4.0, 12.0),
            current_edge: n("trindle-current-card-edge-width", 1.0, 3.0),
            pane_strip: n("trindle-folders-strip", 36.0, 72.0),
            note_title_weight: r.weight("trindle-note-title-weight"),
            circle: n("trindle-circle-size", 144.0, 176.0),
            circle_rest_least: n("trindle-circle-rest-least", 48.0, 144.0),
            circle_gap: n("trindle-circle-gap", 4.0, 24.0),
            circle_margin: n("trindle-circle-margin", 16.0, 48.0),
            circle_button: n("trindle-circle-button-share", 0.5, 0.9),
            circle_most: n("trindle-circle-most", 0.25, 0.6),
            circle_pointed_most: n("trindle-circle-pointed-most", 0.45, 0.95),
            circle_hold: n("trindle-circle-hold", 1000.0, 4000.0),
            circle_step: n("trindle-circle-step", 8.0, 96.0),
            note_slide: n("trindle-note-slide", 0.0, 600.0),
        }
    }

    pub(super) fn from(layers: &Layers) -> Sheet {
        let accent = Colour::parse(style::DEFAULT_ACCENT).unwrap_or(UNREAD);
        let mut sheet = Sheet::read(&Reader::new(layers, false, accent));
        pairs::hold_label(&mut sheet, layers);
        if !layers.theme.is_empty() && !sheet.colour_blind_safe {
            let ours = Reader::new(builtin(), false, accent);
            for (name, c) in [("trindle-calm", &mut sheet.calm), ("trindle-good", &mut sheet.good), ("trindle-busy", &mut sheet.warn),
                              ("trindle-bad", &mut sheet.problem)] {
                if !pairs::mine(layers, false, name) {
                    *c = kin(*c, ours.c(name));
                }
            }
            let ours = Reader::new(builtin(), true, accent);
            for (name, c) in [("trindle-calm", &mut sheet.calm_light), ("trindle-good", &mut sheet.good_light), ("trindle-busy", &mut sheet.warn_light)] {
                if !pairs::mine(layers, true, name) {
                    *c = kin(*c, ours.c(name));
                }
            }
        }
        sheet
    }
}

fn read_safe(r: &Reader) -> [Safe; 2] {
    [false, true].map(|light| {
        let r = Reader::new(r.layers, light, r.accent);
        Safe { states: ["calm", "good", "busy", "bad"].map(|k| r.c(&format!("trindle-safe-{k}"))), series: [1, 2, 3, 4, 5, 6].map(|i| r.c(&format!("trindle-safe-series-{i}"))) }
    })
}

fn read_state(r: &Reader, safe_on: bool, safe: &[Safe; 2], light: bool, name: &str) -> Colour {
    match (safe_on, name) {
        (true, "calm") => safe[usize::from(light)].states[0],
        (true, "good") => safe[usize::from(light)].states[1],
        (true, "busy") => safe[usize::from(light)].states[2],
        (true, _) => safe[usize::from(light)].states[3],
        _ if light => Reader::new(r.layers, true, r.accent).c(&format!("trindle-{name}")),
        _ => r.c(&format!("trindle-{name}")),
    }
}

fn read_bento(r: &Reader, gap: f32) -> Bento {
    Bento {
        gap,
        page_gap: gap,
        card: r.span(("trindle-bento-card-least", 3.0, 6.0), ("trindle-bento-card-most", 4.0, 8.0)),
        figure: r.span(("trindle-bento-figure-least", 2.0, 4.0), ("trindle-bento-figure-most", 2.0, 6.0)),
        picture: r.span(("trindle-bento-picture-least", 4.0, 8.0), ("trindle-bento-picture-most", 5.0, 9.0)),
        words: r.n("trindle-bento-words", 200.0, 420.0),
        figure_words: r.n("trindle-bento-figure-words", 100.0, 260.0),
        picture_least: r.n("trindle-picture-height", 160.0, 360.0),
    }
}

fn read_welcome(r: &Reader) -> Welcome {
    let least_width = r.n("trindle-welcome-least-width", 640.0, 1200.0);
    let least_height = r.n("trindle-welcome-least-height", 440.0, 900.0);
    Welcome {
        width: r.n("trindle-welcome-width", 640.0, 1800.0).max(least_width),
        height: r.n("trindle-welcome-height", 440.0, 1200.0).max(least_height),
        least_width,
        least_height,
        header: r.n("trindle-welcome-header", 80.0, 160.0),
        tile: r.n("trindle-welcome-tile", 40.0, 72.0),
        tile_least: r.n("trindle-welcome-tile-least", 36.0, 52.0),
        run: r.n("trindle-welcome-run", 4.0, 14.0),
    }
}

fn read_app_page(r: &Reader) -> AppPage {
    let least = r.n("trindle-app-picture-least", 120.0, 360.0);
    AppPage {
        narrow: r.n("trindle-app-page-narrow", 480.0, 1000.0),
        icon: r.n("trindle-app-icon", 32.0, 96.0),
        action: r.n("trindle-app-action-width", 96.0, 240.0),
        picture_shape: r.n("trindle-app-picture-shape", 0.3, 0.8),
        picture_least: least,
        picture_most: r.n("trindle-app-picture-most", 240.0, 720.0).max(least),
        ring: r.n("trindle-app-rating-ring", 48.0, 120.0),
        figure: r.n("trindle-app-rating-figure", 40.0, 96.0),
        bar: r.n("trindle-app-rating-bar", 4.0, 16.0),
        bar_name: r.n("trindle-app-rating-bar-name", 24.0, 64.0),
        bar_count: r.n("trindle-app-rating-bar-count", 32.0, 96.0),
    }
}

fn read_apps_list(r: &Reader) -> AppsList {
    AppsList {
        tile_least: r.n("trindle-apps-tile-least", 220.0, 420.0),
        tile_icon: r.n("trindle-apps-tile-icon", 32.0, 72.0),
        row_icon: r.n("trindle-apps-row-icon", 24.0, 56.0),
        card_least: r.n("trindle-apps-card-least", 280.0, 520.0),
        held: r.n("trindle-apps-held", 0.2, 0.8),
    }
}

fn read_folders(r: &Reader, level: &str, big: f32) -> FoldersRoom {
    let of = |part: &str, min: f32, max: f32| r.n(&format!("trindle-folders-{level}-{part}"), min, max);
    FoldersRoom {
        row: of("row", TARGET_LEAST, 64.0) * big,
        rich_row: of("rich-row", 40.0, 80.0) * big,
        line: of("line", TARGET_LEAST, 48.0) * big,
        side_row: of("side-row", TARGET_LEAST, 48.0) * big,
        gap: of("gap", 2.0, 16.0),
        icon_room: (of("icon-across", 40.0, 80.0), of("icon-down", 44.0, 72.0)),
    }
}

fn read_gaps(r: &Reader) -> Gaps {
    Gaps {
        tiny: r.step("trindle-gap-tiny"),
        small: r.step("trindle-gap-small"),
        medium: r.step("trindle-gap-medium"),
        roomy: r.step("trindle-gap-roomy"),
        large: r.step("trindle-gap-large"),
        huge: r.step("trindle-gap-huge"),
    }
}

fn read_note(r: &Reader, text: f32) -> (f32, u16) {
    let (size, weight) = r.type_of("trindle-note", 12.0, 18.0);
    (size.min(text), weight)
}
