//! The one table of every ink Trindle paints and everything it is painted on, each pair with its floor:

use crate::common::style::Layers;

use super::{alpha, clear, contrast, kin, luminance, mix, over, sided, usual, Colour, Reader, Sheet, Theme, BLACK, EDGES, WHITE, WORDS, AIM_EDGES, AIM_WORDS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub enum Ink {
    Words,
    Icon,
    Quiet,
    Red,
    RedMark,
    HighlightWords,
    OnPicked,
    OnHighlight,
    Highlight,
    Soft,
    FieldEdge,
    CurrentEdge,
    Folder,
    Knob,
    KnobOff,
    Label,
    Title,
    TitleAway,
    OnFill,
    OnGreyed,
    KnobGreyed,
    PickedWords,
    PickedQuiet,
    Hidden,
    HeldBack,
}

impl Ink {
    pub const ALL: [Ink; 25] = [Ink::Words, Ink::Icon, Ink::Quiet, Ink::Red, Ink::RedMark, Ink::HighlightWords, Ink::OnPicked, Ink::OnHighlight,
                                Ink::Highlight, Ink::Soft, Ink::FieldEdge, Ink::CurrentEdge, Ink::Folder, Ink::Knob, Ink::KnobOff, Ink::Label,
                                Ink::Title, Ink::TitleAway, Ink::OnFill, Ink::OnGreyed, Ink::KnobGreyed, Ink::PickedWords,
                                Ink::PickedQuiet, Ink::Hidden, Ink::HeldBack];

    pub fn floor(self) -> f32 {
        match self {
            Ink::Words | Ink::Quiet | Ink::Red | Ink::HighlightWords | Ink::OnPicked | Ink::OnHighlight | Ink::Label | Ink::Title
            | Ink::TitleAway | Ink::OnFill | Ink::OnGreyed | Ink::PickedWords | Ink::PickedQuiet | Ink::Hidden => WORDS,
            Ink::Icon | Ink::RedMark | Ink::Highlight | Ink::Soft | Ink::FieldEdge | Ink::CurrentEdge | Ink::Folder | Ink::Knob | Ink::KnobOff
            | Ink::KnobGreyed | Ink::HeldBack => EDGES,
        }
    }

    pub fn name(self) -> Option<&'static str> {
        Some(match self {
            Ink::Words => "trindle-text",
            Ink::Icon => "trindle-icon",
            Ink::Quiet => "trindle-text-quiet",
            Ink::Red => "trindle-shade-red",
            Ink::RedMark => "trindle-bad",
            Ink::HighlightWords => "trindle-highlight-text",
            Ink::OnPicked => "trindle-on-picked",
            Ink::OnHighlight | Ink::OnFill | Ink::OnGreyed | Ink::KnobGreyed | Ink::PickedWords | Ink::PickedQuiet | Ink::Hidden
            | Ink::HeldBack => return None,
            Ink::Highlight => "trindle-highlight",
            Ink::Soft => "trindle-highlight-soft",
            Ink::FieldEdge => "trindle-typing-box-edge",
            Ink::CurrentEdge => "trindle-current-card-edge",
            Ink::Folder => "trindle-folder-icon",
            Ink::Knob => "trindle-knob",
            Ink::KnobOff => "trindle-knob-off",
            Ink::Label => "trindle-label-ink",
            Ink::Title => "trindle-title-ink",
            Ink::TitleAway => "trindle-title-away-ink",
        })
    }

    pub fn always(self) -> bool {
        matches!(self, Ink::Red | Ink::OnHighlight | Ink::Highlight | Ink::Soft | Ink::Folder | Ink::OnFill | Ink::OnGreyed | Ink::KnobGreyed
                       | Ink::PickedWords | Ink::PickedQuiet)
    }
}

#[cfg(test)]
mod roles {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Role {
        Ink(Ink),
        Ground,
        Free(&'static str),
    }

    const PICTURE: Role = Role::Free("a colour for pictures, charts and the picker's swatches only, never for words or icons");
    const STATE: Role = Role::Free("a state's dot, ring or bar: the same kind of colour in every look so it keeps its meaning (the light look's teal and orange are deeper, held three to one by their own test), and never the only thing that says the state");
    const LINE: Role = Role::Free("a fine line or edge that only parts things already told apart by their surfaces");
    const SHADOW: Role = Role::Free("a shadow's colour, which nothing is read on");
    const CODE: Role = Role::Free("a code view's words by kind, each moved by `Theme::ink_on` just far enough to be read on what it is drawn on, whatever the sheet says");
    const TERM: Role = Role::Free("a terminal's colour, moved to four and a half to one on the very cell it is drawn on when it is drawn (term::Term::screen)");

    pub const ROLES: &[(&str, Role)] = &[
        ("trindle-page", Role::Ground), ("trindle-card", Role::Ground), ("trindle-box", Role::Ground), ("trindle-box-more", Role::Ground),
        ("trindle-typing-box", Role::Ground), ("trindle-control", Role::Ground), ("trindle-hover", Role::Ground), ("trindle-well", Role::Ground), ("trindle-ring-grey", Role::Ground),
        ("trindle-top-bar", Role::Ground), ("trindle-side-bar", Role::Ground), ("trindle-foot", Role::Ground), ("trindle-trial-foot", Role::Ground),
        ("trindle-band", Role::Ground), ("trindle-picked", Role::Ground), ("trindle-highlight-hover", Role::Ground),
        ("trindle-label-background", Role::Ground), ("trindle-title-background", Role::Ground), ("trindle-title-away-background", Role::Ground),
        ("trindle-text", Role::Ink(Ink::Words)), ("trindle-icon", Role::Ink(Ink::Icon)), ("trindle-text-quiet", Role::Ink(Ink::Quiet)), ("trindle-shade-red", Role::Ink(Ink::Red)),
        ("trindle-bad", Role::Ink(Ink::RedMark)), ("trindle-highlight-text", Role::Ink(Ink::HighlightWords)),
        ("trindle-on-picked", Role::Ink(Ink::OnPicked)), ("trindle-highlight", Role::Ink(Ink::Highlight)),
        ("trindle-typing-box-edge", Role::Ink(Ink::FieldEdge)), ("trindle-current-card-edge", Role::Ink(Ink::CurrentEdge)),
        ("trindle-highlight-soft", Role::Ink(Ink::Soft)),
        ("trindle-folder-icon", Role::Ink(Ink::Folder)), ("trindle-knob", Role::Ink(Ink::Knob)), ("trindle-knob-off", Role::Ink(Ink::KnobOff)),
        ("trindle-label-ink", Role::Ink(Ink::Label)), ("trindle-title-ink", Role::Ink(Ink::Title)),
        ("trindle-title-away-ink", Role::Ink(Ink::TitleAway)),
        ("trindle-term-black", TERM), ("trindle-term-red", TERM), ("trindle-term-green", TERM), ("trindle-term-yellow", TERM), ("trindle-term-blue", TERM),
        ("trindle-term-magenta", TERM), ("trindle-term-cyan", TERM), ("trindle-term-white", TERM), ("trindle-term-bright-black", TERM),
        ("trindle-term-bright-red", TERM), ("trindle-term-bright-green", TERM), ("trindle-term-bright-yellow", TERM), ("trindle-term-bright-blue", TERM),
        ("trindle-term-bright-magenta", TERM), ("trindle-term-bright-cyan", TERM), ("trindle-term-bright-white", TERM),
        ("trindle-red", PICTURE), ("trindle-pink", PICTURE), ("trindle-purple", PICTURE), ("trindle-deep-purple", PICTURE),
        ("trindle-indigo", PICTURE), ("trindle-blue", PICTURE), ("trindle-light-blue", PICTURE), ("trindle-cyan", PICTURE),
        ("trindle-teal", PICTURE), ("trindle-green", PICTURE), ("trindle-light-green", PICTURE), ("trindle-lime", PICTURE),
        ("trindle-yellow", PICTURE), ("trindle-amber", PICTURE), ("trindle-orange", PICTURE), ("trindle-deep-orange", PICTURE),
        ("trindle-brown", PICTURE), ("trindle-grey", PICTURE), ("trindle-blue-grey", PICTURE),
        ("trindle-calm", STATE), ("trindle-good", STATE), ("trindle-busy", STATE), ("trindle-quiet", STATE),
        ("trindle-safe-calm", STATE), ("trindle-safe-good", STATE), ("trindle-safe-busy", STATE), ("trindle-safe-bad", STATE),
        ("trindle-safe-series-1", PICTURE), ("trindle-safe-series-2", PICTURE), ("trindle-safe-series-3", PICTURE), ("trindle-safe-series-4", PICTURE),
        ("trindle-safe-series-5", PICTURE), ("trindle-safe-series-6", PICTURE),
        ("trindle-use-input", STATE), ("trindle-use-files", STATE), ("trindle-use-network", STATE), ("trindle-use-usb", STATE),
        ("trindle-code-keyword", CODE), ("trindle-code-type", CODE), ("trindle-code-string", CODE), ("trindle-code-number", CODE), ("trindle-code-comment", CODE),
        ("trindle-ring-edge", Role::Free("the ring's edge: held by its own test three to one from a white and a black window, the one place the ring meets a window")), ("trindle-edge", LINE), ("trindle-box-edge", LINE), ("trindle-control-edge", LINE), ("trindle-side-line", LINE), ("trindle-section-line", LINE),
        ("trindle-area-line", LINE), ("trindle-trial-foot-line", LINE), ("trindle-label-edge", LINE), ("trindle-label-outline", LINE),
        ("trindle-title-border", LINE), ("trindle-title-away-border", LINE), ("trindle-window-edge", LINE),
        ("trindle-shadow", SHADOW), ("trindle-knob-shadow", SHADOW),
        ("trindle-spelling", Role::Free("the wavy line under an unknown word: held three to one from the typing box in every look by its own test")),
        ("trindle-store-page", Role::Free("a program's own page colour: `Theme::for_page` lays it under the whole table, held with any value at all (see a_look_on_a_programs_own_page_colour_holds_the_whole_table)")),
    ];

    pub fn role(name: &str) -> Option<Role> {
        if name.starts_with("trindle-pair-") {
            return Some(PICTURE);
        }
        ROLES.iter().find(|(n, _)| *n == name).map(|(_, r)| *r)
    }
}

#[cfg(test)]
pub use roles::*;

const POINTED: f32 = 0.06;

fn solid(c: Colour) -> Colour {
    Colour { a: 1.0, ..c }
}

fn shows(c: Colour) -> Vec<Colour> {
    if c.a >= 1.0 { vec![c] } else { vec![over(c, BLACK), over(c, WHITE)] }
}

struct Seen {
    page: Colour,
    card: Colour,
    boxed: Colour,
    more: Colour,
    field: Colour,
    control: Colour,
    hover: Colour,
    well: Colour,
    ring: Colour,
    top: Colour,
    side: Colour,
    foot: Colour,
    trial: Colour,
}

impl Seen {
    fn all(t: &Theme) -> Vec<Seen> {
        shows(t.bg).into_iter().map(|page| {
            let card = over(t.panel, page);
            let boxed = over(t.raised, card);
            Seen { page, card, boxed, more: over(t.raised_more, boxed), field: over(t.field, card), control: over(t.control, page),
                   hover: over(t.hover, card), well: over(t.well, page), ring: solid(t.ring), top: over(t.top_bar, page), side: over(t.side_bar, page),
                   foot: over(t.foot, page), trial: over(t.trial_foot, page) }
        }).collect()
    }

    fn layers(&self) -> [(&'static str, Colour); 13] {
        [("the page", self.page), ("a card", self.card), ("a box", self.boxed), ("a box in a box", self.more), ("a typing box", self.field),
         ("a control", self.control), ("what is pointed at", self.hover), ("the well", self.well), ("the ring", self.ring), ("the top bar", self.top),
         ("the side bar", self.side), ("the foot", self.foot), ("the trial's foot", self.trial)]
    }

    fn row_grounds(&self) -> [(&'static str, Colour); 4] {
        [("on the page", self.page), ("in a side bar", self.side), ("on a card", self.card), ("in a box", self.boxed)]
    }

    fn rows(&self, t: &Theme, words: Colour) -> Vec<(String, Colour)> {
        let mut all = Vec::new();
        for (at, ground) in self.row_grounds() {
            let band = over(t.band, ground);
            all.push((format!("a banded row {at}"), band));
            all.push((format!("a row pointed at {at}"), over(alpha(words, POINTED), ground)));
            all.push((format!("a banded row pointed at {at}"), over(t.hover, band)));
        }
        all
    }
}

fn grounds(t: &Theme, label: Colour, ink: Ink, words: Colour) -> Vec<(String, Colour)> {
    let named = |all: &[(&'static str, Colour)]| all.iter().map(|(n, c)| (n.to_string(), *c)).collect::<Vec<_>>();
    let mut out = Vec::new();
    for seen in Seen::all(t) {
        match ink {
            Ink::Words | Ink::Icon | Ink::Quiet | Ink::Red => {
                out.extend(named(&seen.layers()));
                out.extend(seen.rows(t, words));
            }
            Ink::HighlightWords => out.extend(named(&seen.layers())),
            Ink::RedMark | Ink::Highlight | Ink::Soft => out.extend(named(&[("the page", seen.page), ("a card", seen.card), ("a box", seen.boxed)])),
            Ink::OnPicked => {
                for (at, ground) in seen.row_grounds() {
                    out.push((format!("a picked row {at}"), over(t.picked, ground)));
                }
            }
            Ink::OnHighlight => {
                out.push(("the highlight".to_string(), over(t.accent, seen.card)));
                out.push(("the highlight pointed at".to_string(), over(t.accent_hover, over(t.accent, seen.card))));
            }
            Ink::FieldEdge => out.extend(named(&[("a card", seen.card), ("its typing box", seen.field)])),
            Ink::CurrentEdge => out.push(("its card".to_string(), seen.card)),
            Ink::Folder => folder_grounds(&mut out, t, &seen, words),
            Ink::Knob => out.push(("a switch that is on".to_string(), over(t.soft, seen.card))),
            Ink::KnobOff => out.extend(named(&[("a switch on the page", seen.control), ("a switch on a card", seen.page),
                                               ("a switch in a box", seen.card), ("a switch in a box in a box", seen.boxed)])),
            Ink::OnGreyed | Ink::KnobGreyed => out.push(("the greyed highlight".to_string(), solid(t.muted))),
            Ink::PickedWords | Ink::PickedQuiet => {
                for (at, ground) in seen.row_grounds() {
                    for (banded, ground) in [("", ground), (" on a band", over(t.band, ground))] {
                        out.push((format!("a picked item{banded} {at}"), over(t.picked, ground)));
                    }
                }
            }
            Ink::Hidden | Ink::HeldBack => {
                out.extend(named(&seen.layers()));
                out.extend(seen.rows(t, words));
            }
            Ink::Label | Ink::Title | Ink::TitleAway | Ink::OnFill => {}
        }
    }
    match ink {
        Ink::Folder => out.extend(shows(label).into_iter().map(|c| ("the picture at the pointer".to_string(), c))),
        Ink::Label => out.extend(shows(label).into_iter().map(|c| ("its label".to_string(), c))),
        _ => {}
    }
    out
}

fn folder_grounds(out: &mut Vec<(String, Colour)>, t: &Theme, seen: &Seen, words: Colour) {
    for (at, ground) in seen.row_grounds() {
        let band = over(t.band, ground);
        out.push((format!("a row {at}"), ground));
        out.push((format!("a banded row {at}"), band));
        out.push((format!("a row pointed at {at}"), over(alpha(words, POINTED), ground)));
        out.push((format!("a picked row {at}"), over(t.picked, ground)));
        for (banded, ground) in [("", ground), (" on a band", band)] {
            out.push((format!("an item pointed at{banded} {at}"), over(t.hover, ground)));
            out.push((format!("a picked item{banded} {at}"), over(t.picked, ground)));
        }
    }
    out.push(("a row pointed at".to_string(), seen.boxed));
    out.push(("a row pointed at in a box".to_string(), seen.more));
}

fn colour_of(t: &Theme, s: &Sheet, ink: Ink, ground: Colour) -> Colour {
    match ink {
        Ink::Words => t.fg,
        Ink::Icon => t.icon,
        Ink::Quiet => t.muted,
        Ink::Red => t.danger_text,
        Ink::RedMark => t.danger,
        Ink::HighlightWords => t.accent_text,
        Ink::OnPicked => t.on_picked,
        Ink::OnHighlight => t.on_accent,
        Ink::Highlight => t.accent,
        Ink::Soft => t.soft,
        Ink::FieldEdge => t.field_edge,
        Ink::CurrentEdge => t.current_edge,
        Ink::Folder => t.folder_ink(None, &[ground]),
        Ink::Knob => t.knob,
        Ink::KnobOff => t.knob_off,
        Ink::Label => s.label_ink,
        Ink::Title | Ink::TitleAway => BLACK,
        Ink::OnFill => super::on(ground),
        Ink::OnGreyed => t.on_greyed(),
        Ink::KnobGreyed => t.knob_greyed(),
        Ink::PickedWords => t.words_on(&[ground]),
        Ink::PickedQuiet => t.quiet_on(&[ground]),
        Ink::Hidden => alpha(t.fg, t.dim(s.hidden_drive, AIM_WORDS)),
        Ink::HeldBack => alpha(t.fg, t.dim(s.apps_list.held, AIM_EDGES)),
    }
}

pub(super) fn layers_of(t: &Theme) -> Vec<Colour> {
    grounds(t, BLACK, Ink::Words, t.fg).into_iter().map(|(_, c)| c).collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct Painted {
    pub ink: Ink,
    pub on: String,
    pub colour: Colour,
    pub ground: Colour,
}

impl Painted {
    pub fn stands(&self) -> f32 {
        contrast(over(self.colour, super::kept(self.ground)), self.ground)
    }

    pub fn short(&self) -> bool {
        self.stands() < self.ink.floor()
    }
}

pub fn painted(layers: &Layers, light: bool, accent: &str) -> Vec<Painted> {
    painted_in(&Theme::from(layers, light, accent), layers, light)
}

pub fn painted_in(t: &Theme, layers: &Layers, light: bool) -> Vec<Painted> {
    let (t, s) = (t.clone(), Sheet::from(layers));
    let mut all = Vec::new();
    for ink in Ink::ALL {
        for (on, ground) in grounds(&t, s.label_back, ink, t.fg) {
            all.push(Painted { ink, on, colour: colour_of(&t, &s, ink, ground), ground });
        }
    }
    let fills = s.palette.iter().chain(s.pairs.iter().flatten()).chain([&s.calm, &s.good, &s.warn, &s.calm_light, &s.good_light, &s.warn_light, &s.problem, &s.quiet]).chain(s.safe.iter().flat_map(|x| x.states.iter().chain(&x.series)));
    for (n, fill) in fills.enumerate() {
        let ground = solid(*fill);
        all.push(Painted { ink: Ink::OnFill, on: format!("a fill, colour {n} of the palette, its pairs and the states"), colour: colour_of(&t, &s, Ink::OnFill, ground), ground });
    }
    let c = |v: [f32; 4]| Colour { r: v[0], g: v[1], b: v[2], a: v[3] };
    for (ink, (words, bar)) in [Ink::Title, Ink::TitleAway].into_iter().zip(crate::common::sheet::TITLES) {
        let (Some(i), Some(b)) = (layers.colour(words, light), layers.colour(bar, light)) else { continue };
        for ground in shows(c(b)) {
            all.push(Painted { ink, on: "its bar".to_string(), colour: c(i), ground });
        }
    }
    all
}

pub(super) fn mine(layers: &Layers, light: bool, name: &str) -> bool {
    let r = Reader::new(layers, layers.light(light), WHITE);
    layers.own.get(name, layers.light(light)).is_some_and(|said| r.colour(said, 1).is_some())
}

pub fn theirs(layers: &Layers, light: bool, ink: Ink) -> bool {
    if ink.always() {
        return false;
    }
    match ink {
        Ink::Label => mine(layers, false, "trindle-label-ink"),
        Ink::Hidden | Ink::HeldBack => mine(layers, light, "trindle-text"),
        Ink::Title | Ink::TitleAway => ink.name().is_some_and(|n| layers.own_title(n, layers.light(light))),
        _ => ink.name().is_some_and(|n| mine(layers, light, n)),
    }
}

fn takes_words(ground: Colour, far: Colour) -> bool {
    contrast(far, ground) >= WORDS
}

fn roomy(c: Colour, far: Colour) -> Colour {
    let fits = |c: &Colour| takes_words(*c, far) && takes_words(over(alpha(far, POINTED), *c), far);
    let to = if far == WHITE { BLACK } else { WHITE };
    (0..=20).map(|i| mix(c, to, i as f32 / 20.0)).find(fits).unwrap_or(to)
}

pub(super) fn hold_layers(t: &mut Theme, layers: &Layers) -> bool {
    let mine = |name: &str| mine(layers, t.light, name);
    let behind = if t.light { WHITE } else { BLACK };
    let dark = luminance(over(t.bg, behind)) < 0.18;
    let far = if dark { WHITE } else { BLACK };
    if !mine("trindle-page") {
        t.bg = roomy(sided(over(t.bg, behind), dark), far);
    }
    let held = |name: &str, c: Colour, under: Colour| if mine(name) { c } else { sided(over(c, under), dark) };
    t.panel = held("trindle-card", t.panel, t.bg);
    t.raised = held("trindle-box", t.raised, t.panel);
    if !mine("trindle-card") {
        t.panel = roomy(t.panel, far);
    }
    if !mine("trindle-box") {
        t.raised = roomy(t.raised, far);
    }
    t.raised_more = held("trindle-box-more", t.raised_more, t.raised);
    t.field = held("trindle-typing-box", t.field, t.panel);
    t.control = held("trindle-control", t.control, t.bg);
    t.hover = held("trindle-hover", t.hover, t.panel);
    t.well = held("trindle-well", t.well, t.bg);
    t.ring = held("trindle-ring-grey", t.ring, t.bg);
    t.trial_foot = held("trindle-trial-foot", t.trial_foot, t.bg);
    let page = t.bg;
    for (name, area) in [("trindle-top-bar", &mut t.top_bar), ("trindle-side-bar", &mut t.side_bar), ("trindle-foot", &mut t.foot)] {
        if area.a > 0.0 && !mine(name) {
            *area = sided(over(*area, page), dark);
            if name == "trindle-side-bar" {
                *area = roomy(*area, far);
            }
        }
    }
    dark
}

pub(super) fn highlight(chosen: Colour, t: &Theme) -> Colour {
    let on: Vec<Colour> = grounds(t, BLACK, Ink::Highlight, t.fg).into_iter().map(|(_, c)| c).collect();
    clear(solid(chosen), &on, AIM_EDGES)
}

pub(super) fn hold_inks(mut t: Theme, layers: &Layers, dark: bool) -> Theme {
    hold_rows(&mut t, layers, if dark { WHITE } else { BLACK });
    hold_words(&mut t, layers, dark);
    hold_edges(&mut t, layers);
    hold_on_accent(&mut t, layers);
    let mine = |name: &str| mine(layers, t.light, name);
    if !mine("trindle-knob") {
        t.knob = clear(solid(t.knob), &under(&t, Ink::Knob, t.fg), AIM_EDGES);
    }
    if !mine("trindle-knob-off") {
        t.knob_off = clear(solid(t.knob_off), &under(&t, Ink::KnobOff, t.fg), AIM_EDGES);
    }
    t
}

fn under(t: &Theme, ink: Ink, words: Colour) -> Vec<Colour> {
    grounds(t, BLACK, ink, words).into_iter().map(|(_, c)| c).collect()
}

fn hold_rows(t: &mut Theme, layers: &Layers, far: Colour) {
    let mine = |name: &str| mine(layers, t.light, name);
    if !mine("trindle-band") {
        let fits = |band: Colour| Seen::all(t).iter().all(|seen| seen.row_grounds().iter().all(|(_, ground)| {
            let banded = over(band, *ground);
            !takes_words(*ground, far) || (takes_words(banded, far) && (!takes_words(seen.hover, far) || takes_words(over(t.hover, banded), far)))
        }));
        t.band = (0..=20).map(|i| alpha(t.band, 1.0 - i as f32 / 20.0)).find(|b| fits(*b)).unwrap_or(Colour { a: 0.0, ..t.band });
    }
    if !mine("trindle-picked") {
        t.picked = over(t.picked, t.panel);
    }
}

fn hold_words(t: &mut Theme, layers: &Layers, dark: bool) {
    let mine = |name: &str| mine(layers, t.light, name);
    if !mine("trindle-text") {
        let least = |c: Colour| under(t, Ink::Words, c).iter().map(|g| contrast(c, *g)).fold(f32::INFINITY, f32::min);
        let fg = solid(t.fg);
        if least(fg) < AIM_WORDS {
            let to = if least(WHITE) >= least(BLACK) { WHITE } else { BLACK };
            t.fg = (1..=20).map(|i| mix(fg, to, i as f32 / 20.0)).find(|m| least(*m) >= AIM_WORDS).unwrap_or(to);
        } else {
            t.fg = fg;
        }
    }
    if !mine("trindle-icon") {
        t.icon = clear(solid(t.icon), &under(t, Ink::Icon, t.fg), AIM_EDGES);
    }
    let built = Reader::new(super::builtin(), !dark, t.accent);
    if !mine("trindle-text-quiet") {
        t.muted = clear(solid(t.muted), &under(t, Ink::Quiet, t.fg), AIM_WORDS);
    }
    let red = if mine("trindle-shade-red") { solid(t.danger_text) } else { kin(t.danger_text, built.c("trindle-shade-red")) };
    t.danger_text = clear(red, &under(t, Ink::Red, t.fg), AIM_WORDS);
    if !mine("trindle-bad") {
        t.danger = clear(kin(t.danger, built.c("trindle-bad")), &under(t, Ink::RedMark, t.fg), AIM_EDGES);
    }
    if !mine("trindle-highlight-text") {
        t.accent_text = clear(solid(t.accent_text), &under(t, Ink::HighlightWords, t.fg), AIM_WORDS);
    }
    if !mine("trindle-on-picked") {
        t.on_picked = clear(solid(t.on_picked), &under(t, Ink::OnPicked, t.fg), AIM_WORDS);
    }
}

fn hold_edges(t: &mut Theme, layers: &Layers) {
    let mine = |name: &str| mine(layers, t.light, name);
    let plain: Vec<Colour> = Seen::all(t).iter().flat_map(|s| [s.page, s.card, s.boxed, s.more]).map(solid).collect();
    if !mine("trindle-folder-icon") {
        t.folder = clear(solid(t.folder), &plain, AIM_EDGES);
    }
    t.soft = clear(solid(t.soft), &under(t, Ink::Soft, t.fg), AIM_EDGES);
    if !mine("trindle-typing-box-edge") {
        let at = under(t, Ink::FieldEdge, t.fg);
        let field = at.get(1).copied().unwrap_or(t.field);
        t.field_edge = clear(over(t.field_edge, field), &at, AIM_EDGES);
    }
    if !mine("trindle-current-card-edge") {
        let at = under(t, Ink::CurrentEdge, t.fg);
        if at.iter().any(|card| contrast(over(t.current_edge, super::kept(*card)), *card) < AIM_EDGES) {
            t.current_edge = clear(over(t.current_edge, at[0]), &at, AIM_EDGES);
        }
    }
}

fn hold_on_accent(t: &mut Theme, layers: &Layers) {
    let near = usual(t.accent);
    let other = if near == WHITE { usual(WHITE) } else { WHITE };
    let hover = solid(over(t.accent_hover, t.accent));
    let theirs = mine(layers, t.light, "trindle-highlight-hover");
    let least = |t: &Theme, c: Colour| grounds(t, BLACK, Ink::OnHighlight, t.fg).iter().map(|(_, g)| contrast(c, *g)).fold(f32::INFINITY, f32::min);
    let mut found = None;
    'inks: for c in [near, other, BLACK, WHITE] {
        for i in 0..=10 {
            if !theirs {
                t.accent_hover = mix(hover, t.accent, i as f32 / 10.0);
            }
            if least(t, c) >= AIM_WORDS {
                found = Some(c);
                break 'inks;
            }
            if theirs {
                break;
            }
        }
    }
    t.on_accent = found.unwrap_or_else(|| {
        [near, other, BLACK, WHITE].into_iter().map(|c| (c, least(t, c))).fold((near, 0.0), |a, b| if a.1 >= b.1 { a } else { b }).0
    });
}

pub(super) fn hold_label(s: &mut Sheet, layers: &Layers) {
    if !mine(layers, false, "trindle-label-background") {
        s.label_back = solid(s.label_back);
    }
    if !mine(layers, false, "trindle-label-ink") {
        s.label_ink = clear(solid(s.label_ink), &shows(s.label_back), AIM_WORDS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::style;
    use crate::theme::tests::{highlights, Dice, PROPOSED};

    const CSS: &str = style::CSS;

    fn short(layers: &Layers, light: bool, accent: &str) -> Vec<String> {
        painted(layers, light, accent).into_iter().filter(|p| p.short() && !theirs(layers, light, p.ink))
            .map(|p| format!("{:?} {} on {} {}: {:.2} of {}", p.ink, p.colour.css(), p.on, p.ground.css(), p.stands(), p.ink.floor())).collect()
    }

    fn any_colour(dice: &mut Dice) -> String {
        let (r, g, b) = (dice.below(256), dice.below(256), dice.below(256));
        match dice.below(8) {
            0 => format!("rgba({r}, {g}, {b}, 0.{})", dice.below(10)),
            1 => format!("#{0:02x}{0:02x}{0:02x}", 100 + dice.below(60)),
            2 => format!("#{r:02x}{r:02x}{r:02x}"),
            _ => format!("#{r:02x}{g:02x}{b:02x}"),
        }
    }

    fn any_sheet(dice: &mut Dice, highlight: bool) -> String {
        let names: Vec<&str> = ROLES.iter().filter(|(n, r)| !matches!(r, Role::Free(_)) && (highlight || *n != "trindle-highlight")).map(|(n, _)| *n).collect();
        let block = |dice: &mut Dice| {
            let mut css = String::new();
            let most = if dice.below(4) == 0 { names.len() } else { 6 };
            for _ in 0..(1 + dice.below(most)) {
                let name = names[dice.below(names.len())];
                css.push_str(&format!("  --{name}: {};\n", any_colour(dice)));
            }
            css
        };
        let mut css = String::new();
        if dice.below(4) != 0 {
            css.push_str(&format!(":root {{\n{}}}\n", block(dice)));
        }
        if dice.below(2) == 0 {
            css.push_str(&format!("@media (prefers-color-scheme: light) {{ :root {{\n{}}} }}\n", block(dice)));
        }
        css
    }

    fn any_highlight(dice: &mut Dice) -> String {
        format!("#{:02x}{:02x}{:02x}", dice.below(256), dice.below(256), dice.below(256))
    }

    fn went_as_far_as_it_can(t: &Theme, s: &Sheet, everything: &[Painted], p: &Painted) -> bool {
        let titles: Vec<Colour> = everything.iter().filter(|q| q.ink == p.ink).map(|q| q.ground).collect();
        let on = |c: Colour| match p.ink {
            Ink::Title | Ink::TitleAway => titles.clone(),
            ink => grounds(t, s.label_back, ink, if matches!(ink, Ink::Words | Ink::Hidden | Ink::HeldBack) { c } else { t.fg }).into_iter().map(|(_, g)| g).collect(),
        };
        let least = |c: Colour| on(c).iter().map(|g| contrast(c, *g)).fold(f32::INFINITY, f32::min);
        let c = solid(p.colour);
        least(c) >= least(BLACK).max(least(WHITE)) - 0.001
    }

    #[test]
    fn trindles_own_look_holds_the_whole_table_with_every_highlight() {
        let layers = Layers::new(CSS, "");
        let mut all = Vec::new();
        for light in [false, true] {
            for accent in highlights() {
                all.extend(short(&layers, light, &accent).into_iter().map(|s| format!("{} with {accent}: {s}", if light { "light" } else { "dark" })));
            }
        }
        assert!(all.is_empty(), "{} pairs fall short:\n{}", all.len(), all.join("\n"));
    }

    #[test]
    fn trindles_own_sheet_is_drawn_as_it_is_written() {
        let layers = Layers::new(CSS, "");
        for light in [false, true] {
            let t = Theme::from(&layers, light, style::DEFAULT_ACCENT);
            let raw = Theme::read(&layers, light, t.accent);
            for (name, drawn, written) in [("page", t.bg, raw.bg), ("card", t.panel, raw.panel), ("box", t.raised, raw.raised),
                                           ("box-more", t.raised_more, raw.raised_more), ("typing-box", t.field, raw.field),
                                           ("control", t.control, raw.control), ("hover", t.hover, raw.hover), ("well", t.well, raw.well), ("ring-grey", t.ring, raw.ring),
                                           ("text", t.fg, raw.fg), ("text-quiet", t.muted, raw.muted), ("shade-red", t.danger_text, raw.danger_text),
                                           ("bad", t.danger, raw.danger), ("typing-box-edge", t.field_edge, raw.field_edge),
                                           ("current-card-edge", t.current_edge, raw.current_edge), ("band", t.band, raw.band),
                                           ("knob", t.knob, raw.knob), ("knob-off", t.knob_off, raw.knob_off), ("trial-foot", t.trial_foot, raw.trial_foot)] {
                assert_eq!(drawn, written, "--trindle-{name}, light {light}: drawn as {} and written as {}", drawn.css(), written.css());
            }
        }
    }

    #[test]
    fn the_themes_that_come_with_trindle_hold_the_whole_table() {
        let mut all = Vec::new();
        for (name, css) in PROPOSED {
            let layers = Layers::themed(CSS, css, "");
            for light in [false, true] {
                for accent in highlights() {
                    all.extend(short(&layers, light, &accent).into_iter().map(|s| format!("{name}, light {light}, with {accent}: {s}")));
                }
            }
        }
        assert!(all.is_empty(), "{} pairs fall short:\n{}", all.len(), all.join("\n"));
    }

    #[test]
    fn any_theme_with_any_highlight_holds_the_whole_table() {
        let mut dice = Dice(0x7ab1_e5ee);
        let twenty = highlights();
        let mut all = Vec::new();
        for n in 0..3000 {
            let theme = any_sheet(&mut dice, false);
            let layers = Layers::themed(CSS, &theme, "");
            let accent = if n % 3 == 0 { any_highlight(&mut dice) } else { twenty[n % twenty.len()].clone() };
            for light in [false, true] {
                all.extend(short(&layers, light, &accent).into_iter().map(|s| format!("light {light} with {accent}: {s}\n{theme}")));
            }
            if all.len() > 20 {
                break;
            }
        }
        assert!(all.is_empty(), "{} pairs fall short:\n{}", all.len(), all.join("\n"));
    }

    #[test]
    fn any_highlight_holds_the_whole_table_on_every_look_that_comes_with_trindle() {
        let mut dice = Dice(0x51de_ca11);
        let mut looks = vec![("Trindle", Layers::new(CSS, ""))];
        looks.extend(PROPOSED.iter().map(|(name, css)| (*name, Layers::themed(CSS, css, ""))));
        let mut all = Vec::new();
        let mut tried: Vec<String> = ["#000000", "#ffffff", "#808080", "#767676", "#4a90e2", "#ff0000", "#00ff00", "#0000ff"].iter().map(|s| s.to_string()).collect();
        tried.extend((0..600).map(|_| any_highlight(&mut dice)));
        for (name, layers) in &looks {
            for accent in &tried {
                for light in [false, true] {
                    all.extend(short(layers, light, accent).into_iter().map(|s| format!("{name}, light {light}, with {accent}: {s}")));
                }
            }
        }
        assert!(all.is_empty(), "{} pairs fall short:\n{}", all.len(), all.join("\n"));
    }

    #[test]
    fn any_sheet_of_the_persons_own_holds_the_table_but_for_what_it_set_itself() {
        let mut dice = Dice(0x0b5e_55ed);
        let twenty = highlights();
        let (mut all, mut let_off, mut furthest) = (Vec::new(), 0, 0);
        for n in 0..4000 {
            let own = any_sheet(&mut dice, true);
            let theme = match n % 4 {
                0 => String::new(),
                1 => PROPOSED[n / 4 % PROPOSED.len()].1.to_string(),
                _ => any_sheet(&mut dice, false),
            };
            let layers = Layers::themed(CSS, &theme, &own);
            let accent = if n % 3 == 0 { any_highlight(&mut dice) } else { twenty[n % twenty.len()].clone() };
            let sheet = Sheet::from(&layers);
            for light in [false, true] {
                let t = Theme::from(&layers, light, &accent);
                let everything = painted_in(&t, &layers, light);
                for p in everything.iter().filter(|p| p.short()) {
                    if theirs(&layers, light, p.ink) {
                        let_off += 1;
                    } else if !layers.own.is_empty() && went_as_far_as_it_can(&t, &sheet, &everything, p) {
                        furthest += 1;
                    } else {
                        all.push(format!("light {light} with {accent}: {:?} {} on {} {}: {:.2}\nown:\n{own}theme:\n{theme}", p.ink, p.colour.css(), p.on,
                                         p.ground.css(), p.stands()));
                    }
                }
            }
            if all.len() > 10 {
                break;
            }
        }
        assert!(all.is_empty(), "{} pairs fall short:\n{}", all.len(), all.join("\n"));
        assert!(let_off > 100 && furthest > 100, "{let_off} {furthest}");
    }

    #[test]
    fn every_colour_the_sheet_names_or_the_toolkit_reads_has_its_place_in_the_table() {
        let layers = Layers::new(CSS, "");
        let (dark, light) = (Reader::new(&layers, false, WHITE), Reader::new(&layers, true, WHITE));
        let mut colours = Vec::new();
        for d in style::declarations(CSS) {
            let r = if d.light { &light } else { &dark };
            if r.colour(&d.value, 0).is_some() && !colours.contains(&d.name) {
                colours.push(d.name);
            }
        }
        assert!(colours.len() > 60, "{}", colours.len());
        for name in &colours {
            assert!(role(name).is_some(), "--{name} is a colour in the sheet with no place in the table");
        }
        for (name, _) in ROLES {
            assert!(colours.iter().any(|c| c == name), "--{name} has a place in the table and is no colour in the sheet");
            assert_eq!(ROLES.iter().filter(|(n, _)| n == name).count(), 1, "--{name} has two places");
        }
        super::super::READ.with(|r| r.borrow_mut().clear());
        let _ = Sheet::from(&layers);
        for look in [false, true] {
            let _ = Theme::from(&layers, look, style::DEFAULT_ACCENT);
        }
        let read = super::super::READ.with(|r| r.borrow().clone());
        assert!(read.len() > 40, "{}", read.len());
        for name in read {
            assert!(role(&name).is_some(), "the toolkit reads --{name} as a colour and it has no place in the table");
        }
        for ink in Ink::ALL {
            if let Some(name) = ink.name() {
                assert_eq!(role(name), Some(Role::Ink(ink)), "{ink:?}");
            }
            let on = painted(&layers, false, style::DEFAULT_ACCENT).into_iter().filter(|p| p.ink == ink).count();
            assert!(on > 0, "{ink:?} is painted on nothing");
        }
    }

    #[test]
    fn the_list_of_all_inks_lacks_none() {
        fn place(ink: Ink) -> usize {
            match ink {
                Ink::Words => 0, Ink::Icon => 1, Ink::Quiet => 2, Ink::Red => 3, Ink::RedMark => 4, Ink::HighlightWords => 5, Ink::OnPicked => 6,
                Ink::OnHighlight => 7, Ink::Highlight => 8, Ink::Soft => 9, Ink::FieldEdge => 10, Ink::CurrentEdge => 11, Ink::Folder => 12,
                Ink::Knob => 13, Ink::KnobOff => 14, Ink::Label => 15, Ink::Title => 16, Ink::TitleAway => 17, Ink::OnFill => 18, Ink::OnGreyed => 19,
                Ink::KnobGreyed => 20, Ink::PickedWords => 21, Ink::PickedQuiet => 22, Ink::Hidden => 23, Ink::HeldBack => 24,
            }
        }
        let mut seen: Vec<usize> = Ink::ALL.iter().map(|i| place(*i)).collect();
        seen.sort_unstable();
        assert_eq!(seen, (0..=place(Ink::HeldBack)).collect::<Vec<_>>());
    }

    #[test]
    fn a_colour_set_for_any_name_in_the_table_is_seen_in_its_pairs() {
        let odd = Colour::parse("#ff00fe").unwrap();
        for (name, role) in ROLES {
            let layers = Layers::new(CSS, &format!(":root {{ --{name}: #ff00fe; }}"));
            let accent = if *name == "trindle-highlight" { "#ff00fe" } else { style::DEFAULT_ACCENT };
            for light in [false, true] {
                let all = painted(&layers, light, accent);
                match role {
                    Role::Ground => assert!(all.iter().any(|p| p.ground == odd), "nothing is measured on --{name}, light {light}"),
                    Role::Ink(ink) if ink.always() => assert!(all.iter().any(|p| p.ink == *ink), "--{name}"),
                    Role::Ink(ink) => assert!(all.iter().any(|p| p.ink == *ink && p.colour == odd), "--{name} is not measured as {ink:?}, light {light}"),
                    Role::Free(why) => assert!(why.len() > 20, "--{name} is let off with no reason"),
                }
            }
        }
    }

    #[test]
    fn words_on_a_highlight_of_the_persons_own_can_be_read_with_no_theme() {
        let layers = Layers::new(CSS, ":root { --trindle-highlight: #4a90e2; }");
        for light in [false, true] {
            let t = Theme::from(&layers, light, "#4a90e2");
            assert!(contrast(t.on_accent, t.accent) >= WORDS, "light {light}: {:.2}", contrast(t.on_accent, t.accent));
            assert!(contrast(t.on_accent, t.accent_hover) >= WORDS, "light {light}, pointed at: {:.2}", contrast(t.on_accent, t.accent_hover));
            assert_eq!(short(&layers, light, "#4a90e2"), Vec::<String>::new());
        }
        let plain = Layers::new(CSS, "");
        for light in [false, true] {
            let t = Theme::from(&plain, light, "#4a90e2");
            assert!(contrast(t.on_accent, t.accent) >= WORDS, "light {light}: {:.2}", contrast(t.on_accent, t.accent));
        }
    }

    #[test]
    fn a_colour_set_for_one_look_only_is_trindles_own_in_the_other_and_is_held() {
        let own = ":root { --trindle-page: #808080; } @media (prefers-color-scheme: light) { :root { --trindle-text: #ffffff; } }";
        let layers = Layers::new(CSS, own);
        let dark = Theme::from(&layers, false, style::DEFAULT_ACCENT);
        assert!(contrast(dark.fg, dark.bg) >= WORDS, "the dark look: {:.2}", contrast(dark.fg, dark.bg));
        assert!(!theirs(&layers, false, Ink::Words) && theirs(&layers, true, Ink::Words));
        assert_eq!(short(&layers, false, style::DEFAULT_ACCENT), Vec::<String>::new());
        assert_eq!(Theme::from(&layers, true, style::DEFAULT_ACCENT).fg, WHITE);
        for theme in ["", PROPOSED[0].1] {
            for ink in Ink::ALL {
                let Some(name) = ink.name().filter(|_| !ink.always() && ink != Ink::Label) else { continue };
                let one = Layers::themed(CSS, theme, &format!("@media (prefers-color-scheme: light) {{ :root {{ --{name}: #808080; }} }}"));
                let dark_only = one.only == Some(false);
                assert!(!theirs(&one, false, ink), "{name}: theirs in the dark look");
                assert_eq!(theirs(&one, true, ink), !dark_only, "{name}: theirs in the light look");
                let both = Layers::themed(CSS, theme, &format!(":root {{ --{name}: #808080; }}"));
                assert!(theirs(&both, false, ink) && theirs(&both, true, ink), "{name}");
                let unread = Layers::themed(CSS, theme, &format!(":root {{ --{name}: porridge; }}"));
                assert!(!theirs(&unread, false, ink) && !theirs(&unread, true, ink), "{name}: an unreadable value");
            }
        }
        let label = Layers::new(CSS, ":root { --trindle-label-background: #808080; } @media (prefers-color-scheme: light) { :root { --trindle-label-ink: #7a7a7a; } }");
        assert!(!theirs(&label, false, Ink::Label) && !theirs(&label, true, Ink::Label));
        let s = Sheet::from(&label);
        assert!(contrast(s.label_ink, s.label_back) >= WORDS, "{:.2}", contrast(s.label_ink, s.label_back));
    }

    #[test]
    fn red_words_are_held_even_when_the_persons_own_sheet_sets_them() {
        for own in [":root { --trindle-shade-red: #0a111e; }", ":root { --trindle-shade-red: var(--trindle-card); }",
                    ":root { --trindle-shade-red: rgba(255, 0, 0, 0.1); }",
                    "@media (prefers-color-scheme: light) { :root { --trindle-shade-red: #ffffff; } }"] {
            let layers = Layers::new(CSS, own);
            for light in [false, true] {
                let reds: Vec<Painted> = painted(&layers, light, style::DEFAULT_ACCENT).into_iter().filter(|p| p.ink == Ink::Red).collect();
                assert!(reds.len() >= 24);
                for p in reds {
                    assert!(!p.short(), "{own}, light {light}: red words on {}: {:.2}", p.on, p.stands());
                }
            }
        }
        let layers = Layers::new(CSS, ":root { --trindle-shade-red: #ffd0d0; }");
        assert_eq!(Theme::from(&layers, false, style::DEFAULT_ACCENT).danger_text, Colour::parse("#ffd0d0").unwrap());
    }

    #[test]
    fn the_held_look_is_trindles_own_sheet_and_nothing_else() {
        let own = Layers::new(CSS, "");
        let held = super::super::held();
        assert_eq!(held, Theme::from(&own, held.light, &style::chosen_accent()));
        for light in [false, true] {
            assert_eq!(short(&own, light, &style::chosen_accent()), Vec::<String>::new());
        }
        assert_eq!(super::super::safe(), Theme::from(&own, false, style::DEFAULT_ACCENT));
    }

    #[test]
    fn a_themes_band_leaves_every_row_readable() {
        for band in ["rgba(255, 255, 255, 0.5)", "#808080", "rgba(128, 128, 128, 0.9)", "#ffffff"] {
            let layers = Layers::themed(CSS, &format!(":root {{ --trindle-band: {band}; }}"), "");
            for light in [false, true] {
                assert_eq!(short(&layers, light, style::DEFAULT_ACCENT), Vec::<String>::new(), "{band}, light {light}");
            }
        }
    }

    #[test]
    fn words_on_a_fill_of_any_colour_can_be_read() {
        let s = Sheet::from(&Layers::new(CSS, ""));
        let mut all: Vec<Colour> = s.palette.iter().chain(s.pairs.iter().flatten()).chain([&s.calm, &s.good, &s.warn, &s.calm_light, &s.good_light, &s.warn_light, &s.problem, &s.quiet]).copied().collect();
        assert!(all.len() >= 19 * 3 + 6);
        let mut dice = Dice(0x0f11_15ed);
        all.extend((0..4000).map(|_| Colour::parse(&any_highlight(&mut dice)).unwrap()));
        all.extend((90..140).map(|v| Colour::rgb(v as f32 / 255.0, v as f32 / 255.0, v as f32 / 255.0)));
        for c in all {
            let ink = super::super::on(c);
            assert!(contrast(ink, c) >= WORDS, "{} on {}: {:.2}", ink.css(), c.css(), contrast(ink, c));
            let said = Colour::parse(style::on_colour(&c.css())).unwrap();
            assert!(contrast(said, c) >= WORDS, "as words, {} on {}: {:.2}", said.css(), c.css(), contrast(said, c));
        }
        let fills = painted(&Layers::new(CSS, ""), false, style::DEFAULT_ACCENT).into_iter().filter(|p| p.ink == Ink::OnFill).count();
        assert_eq!(fills, 19 * 3 + 8 + 2 * (4 + 6));
    }

    #[test]
    fn a_colour_asked_for_by_name_is_the_one_the_table_holds() {
        let theme = ":root { --trindle-text: #3a3a3a; --trindle-text-quiet: #303030; --trindle-card: #7f7f7f; --trindle-highlight-text: #222222; }";
        let layers = Layers::themed(CSS, theme, "");
        let named = |name: &str, light: bool| super::super::named_in(&layers, name, light, style::DEFAULT_ACCENT);
        for light in [false, true] {
            let t = Theme::from(&layers, light, style::DEFAULT_ACCENT);
            for words in ["trindle-text", "trindle-text-quiet"] {
                let stands = contrast(named(words, light), named("trindle-well", light));
                assert!(stands >= WORDS, "light {light}: --{words} on the well: {stands:.2}");
            }
            for ground in ["trindle-page", "trindle-card", "trindle-box", "trindle-control"] {
                let stands = contrast(named("trindle-text", light), named(ground, light));
                assert!(stands >= WORDS, "light {light}: words on --{ground}: {stands:.2}");
            }
            assert_eq!((named("trindle-text", light), named("trindle-card", light), named("trindle-highlight", light)), (t.fg, t.panel, t.accent));
            let all = painted(&layers, light, style::DEFAULT_ACCENT);
            for (name, role) in ROLES {
                match role {
                    Role::Ink(ink) if !matches!(ink, Ink::Label | Ink::Title | Ink::TitleAway | Ink::Folder) =>
                        assert!(all.iter().any(|p| p.ink == *ink && p.colour == named(name, light)), "--{name} by name is not the {ink:?} the table holds, light {light}"),
                    Role::Ground if !name.starts_with("trindle-label") && !name.starts_with("trindle-title") => {
                        let c = named(name, light);
                        assert!(c.a < 1.0 || all.iter().any(|p| p.ground == c), "--{name} by name is no ground of the table, light {light}: {}", c.css());
                    }
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn a_look_on_a_programs_own_page_colour_holds_the_whole_table() {
        let mut dice = Dice(0x9a6e_c010);
        let mut all = Vec::new();
        let mut pages = vec!["#808080".to_string(), "#ffffff".to_string(), "#000000".to_string(), "rgba(128, 128, 128, 0.5)".to_string()];
        pages.extend((0..300).map(|_| any_colour(&mut dice)));
        for (n, page) in pages.iter().enumerate() {
            let layers = Layers::themed(CSS, &format!(":root {{ --trindle-store-page: {page}; }}"), "");
            let accent = highlights()[n % 20].clone();
            for light in [false, true] {
                let t = Theme::on_page(&layers, light, &accent, "store");
                all.extend(painted_in(&t, &layers, light).into_iter().filter(|p| p.short() && !matches!(p.ink, Ink::Title | Ink::TitleAway | Ink::Label))
                    .map(|p| format!("page {page}, light {light}, {accent}: {:?} {} on {} {}: {:.2}", p.ink, p.colour.css(), p.on, p.ground.css(), p.stands())));
                assert_eq!(Theme::on_page(&layers, light, &accent, "folders"), Theme::from(&layers, light, &accent));
            }
        }
        assert!(all.is_empty(), "{} pairs fall short:\n{}", all.len(), all[..all.len().min(40)].join("\n"));
    }

    #[test]
    fn what_is_made_quieter_can_still_be_read_in_any_look() {
        let own = Layers::new(CSS, "");
        let s = Sheet::from(&own);
        for light in [false, true] {
            let t = Theme::from(&own, light, style::DEFAULT_ACCENT);
            assert_eq!(t.dim(s.hidden_drive, WORDS), s.hidden_drive, "Trindle's own look, light {light}: a hidden drive shows as much as the sheet asks");
            assert_eq!(t.dim(super::super::HIDDEN_LEAST, WORDS), super::super::HIDDEN_LEAST, "light {light}: and at the least a sheet may ask");
        }
        let theme = ":root { --trindle-page: #2b2b2b; --trindle-card: #2b2b2b; --trindle-box: #2b2b2b; --trindle-text: #8f8f8f; }";
        let layers = Layers::themed(CSS, theme, "");
        let t = Theme::from(&layers, false, style::DEFAULT_ACCENT);
        let raw = over(alpha(t.fg, s.hidden_drive), t.panel);
        assert!(contrast(raw, t.panel) < WORDS, "the case is one where the sheet's share falls short: {:.2}", contrast(raw, t.panel));
        let held = over(alpha(t.fg, t.dim(s.hidden_drive, WORDS)), t.panel);
        assert!(contrast(held, t.panel) >= WORDS, "{:.2}", contrast(held, t.panel));
        let mut dice = Dice(0x00d1_44ed);
        for _ in 0..2000 {
            let (ink, ground) = (Colour::parse(&any_highlight(&mut dice)).unwrap(), Colour::parse(&any_highlight(&mut dice)).unwrap());
            if contrast(ink, ground) < WORDS {
                continue;
            }
            let faint = super::super::dimmed(ink, 0.2 + dice.below(60) as f32 / 100.0, &[ground], WORDS);
            assert!(contrast(over(faint, ground), ground) >= WORDS, "{} at {} on {}", ink.css(), faint.a, ground.css());
        }
    }

    #[test]
    fn what_is_on_the_greyed_highlight_is_held_to_it() {
        let mut looks = vec![("Trindle".to_string(), Layers::new(CSS, ""))];
        looks.extend(PROPOSED.iter().map(|(name, css)| (name.to_string(), Layers::themed(CSS, css, ""))));
        let mut tried = highlights();
        tried.extend(["#0d1b4c", "#000000", "#ffffff", "#808080"].iter().map(|s| s.to_string()));
        for (name, layers) in &looks {
            for light in [false, true] {
                for accent in &tried {
                    let grey = Theme::from(layers, light, accent).faded();
                    assert_eq!(grey.accent, grey.muted);
                    let (words, knob) = (contrast(grey.on_accent, grey.accent), contrast(grey.knob, grey.accent));
                    assert!(words >= WORDS, "{name}, light {light}, {accent}: {} on the greyed highlight {}: {words:.2}", grey.on_accent.css(), grey.accent.css());
                    assert!(knob >= EDGES, "{name}, light {light}, {accent}: a knob {} on the greyed highlight {}: {knob:.2}", grey.knob.css(), grey.accent.css());
                }
            }
        }
    }

    #[test]
    fn a_switchs_knob_can_be_found_on_its_track_in_both_looks() {
        let layers = Layers::new(CSS, "");
        for light in [false, true] {
            for accent in highlights() {
                let t = Theme::from(&layers, light, &accent);
                assert!(contrast(t.knob, t.soft) >= EDGES, "light {light} with {accent}: on: {:.2}", contrast(t.knob, t.soft));
                for track in [t.control, t.bg, t.panel, t.raised] {
                    assert!(contrast(t.knob_off, track) >= EDGES, "light {light}: off on {}: {:.2}", track.css(), contrast(t.knob_off, track));
                }
            }
        }
    }
}
