//! The style sheet and theme checks. Trindle's tests run some of them too (feature `testing`).

use super::*;

#[cfg(test)]
mod access;
#[cfg(test)]
mod reading;
#[cfg(test)]
mod themes;

fn byte(v: f32) -> u8 {
    (v * 255.0).round() as u8
}

fn hex(c: Colour) -> (u8, u8, u8) {
    (byte(c.r), byte(c.g), byte(c.b))
}

const CSS: &str = include_str!("../../share/style.css");

#[test]
fn the_small_state_dots_stand_three_to_one_in_both_looks() {
    let own = Layers::new(style::CSS, "");
    for light in [false, true] {
        style::ASKED_FOR_TEST.with(|a| *a.borrow_mut() = Some((light, style::DEFAULT_ACCENT.to_string())));
        let t = Theme::from(&own, light, style::DEFAULT_ACCENT);
        let rd = Reader::new(&own, light, Colour::parse(style::DEFAULT_ACCENT).unwrap_or(UNREAD));
        let marks = [("calm", calm()), ("good", good()), ("busy", warn()), ("use-input", rd.c("trindle-use-input")),
                     ("use-files", rd.c("trindle-use-files")), ("use-network", rd.c("trindle-use-network")), ("use-usb", rd.c("trindle-use-usb"))];
        let mut short = Vec::new();
        for (name, mark) in marks {
            for (on, ground) in [("the page", t.bg), ("a card", t.panel), ("a box", t.raised), ("the well", t.well)] {
                let stands = contrast(mark, ground);
                if stands < EDGES {
                    short.push(format!("light {light}: {name} on {on}: {stands:.2}"));
                }
            }
        }
        style::ASKED_FOR_TEST.with(|a| *a.borrow_mut() = None);
        assert!(short.is_empty(), "{short:?}");
    }
}

#[test]
fn the_inks_rules_make_aim_past_their_floor() {
    let own = Layers::new(style::CSS, "");
    assert_eq!((AIM_WORDS, AIM_EDGES), (WORDS + SPARE, EDGES + SPARE));
    for light in [false, true] {
        let t = Theme::from(&own, light, style::DEFAULT_ACCENT);
        let mut short = Vec::new();
        for (name, ink) in [("words", t.fg), ("quiet words", t.muted), ("link words", t.accent_text), ("red words", t.danger_text)] {
            for (on, ground) in [("the page", t.bg), ("a card", t.panel), ("a box", t.raised)] {
                let stands = contrast(ink, ground);
                if stands < AIM_WORDS {
                    short.push(format!("light {light}: {name} on {on}: {stands:.3}"));
                }
            }
        }
        for (on, ground) in [("the page", t.bg), ("a card", t.panel), ("a box", t.raised)] {
            let stands = contrast(t.danger, ground);
            if stands < AIM_EDGES {
                short.push(format!("light {light}: the red mark on {on}: {stands:.3}"));
            }
        }
        assert!(short.is_empty(), "{short:?}");
    }
}

#[test]
fn colours_come_from_the_one_list() {
    let dark = Theme::new(false, "#fbc711");
    assert_eq!(hex(dark.bg), (0x0a, 0x11, 0x1e));
    assert_eq!(hex(dark.panel), (0x17, 0x26, 0x40));
    assert_eq!(hex(dark.raised), (0x23, 0x36, 0x55));
    assert_eq!(hex(dark.raised_more), (0x2e, 0x44, 0x68));
    assert_eq!(hex(dark.fg), (0xec, 0xf2, 0xfb));
    assert_eq!(hex(dark.muted), (0xae, 0xbd, 0xd4));
    assert_eq!(hex(dark.field), (0x0c, 0x15, 0x24));
    assert_eq!(hex(dark.field_edge), (0x73, 0x87, 0xa8));
    assert_eq!(dark.control_edge.a, 0.0);
    assert_eq!(hex(dark.danger_text), (0xff, 0xa5, 0x9c));
    assert_eq!(byte(dark.on_accent.r), 0x11);
    assert_eq!(dark.accent_text, dark.accent);
    assert_eq!(hex(dark.picked), (0x39, 0x3e, 0x39));
    let light = Theme::new(true, "#fbc711");
    assert_eq!(hex(light.bg), (0xed, 0xed, 0xed));
    assert_eq!(hex(light.panel), (0xff, 0xff, 0xff));
    assert_eq!(hex(light.raised), (0xf2, 0xf2, 0xf2));
    assert!(light.accent_text.r < light.accent.r);
    assert_eq!(byte(Theme::new(true, "#3d51b4").on_accent.r), 0xff);
    let indigo = Theme::new(false, "#3d51b4");
    assert_eq!(byte(indigo.on_accent.r), 0x11);
    assert!(contrast(indigo.on_accent, indigo.accent) >= WORDS && contrast(indigo.on_accent, indigo.accent_hover) >= WORDS);
    assert!(contrast(indigo.accent, indigo.panel) >= EDGES && contrast(indigo.accent, indigo.raised) >= EDGES);
    assert!(luminance(dark.panel) > luminance(dark.bg) && luminance(dark.raised) > luminance(dark.panel));
    assert!(luminance(dark.raised_more) > luminance(dark.raised));
    assert!(luminance(dark.field) < luminance(dark.panel));
    for t in [&dark, &light] {
        assert!(t.panel != t.bg && t.raised != t.panel);
        assert!(contrast(t.field_edge, t.panel) >= EDGES);
    }
    assert!(dark.edge.a > 0.0 && dark.edge.a < 0.3 && dark.card_edge.a > dark.edge.a && dark.card_edge.a < 0.3);
    assert!(light.edge.a == 0.0 && light.card_edge.a == 0.0);
}

#[test]
fn trindles_words_can_be_read_on_layers_only_the_persons_own_sheet_sets() {
    let sheets = [":root { --trindle-page: #808080; }",
                  ":root { --trindle-page: #f4f4f4; --trindle-card: #ffffff; --trindle-box: #eeeeee; }",
                  ":root { --trindle-page: #dfe6ee; }",
                  "@media (prefers-color-scheme: light) { :root { --trindle-page: #20242c; --trindle-card: #2a3038; } }",
                  ":root { --trindle-picked: #808080; }"];
    for own in sheets {
        let layers = Layers::themed(CSS, "", own);
        assert!(layers.theme.is_empty());
        for light in [false, true] {
            for accent in highlights() {
                let t = Theme::from(&layers, light, &accent);
                let under = over(t.bg, if luminance(t.bg) < 0.18 { BLACK } else { WHITE });
                let card = over(t.panel, under);
                let seen = [under, card, over(t.raised, card), over(t.raised_more, card), over(t.field, card), over(t.control, under),
                            over(t.hover, card), over(t.well, under)];
                for (i, l) in seen.iter().enumerate() {
                    for (name, ink) in [("words", t.fg), ("quiet words", t.muted), ("red words", t.danger_text), ("highlighted words", t.accent_text)] {
                        assert!(contrast(ink, *l) >= WORDS, "{own} light {light} {accent}: {name} on layer {i}: {}", contrast(ink, *l));
                    }
                }
                assert!(contrast(t.on_picked, over(t.picked, card)) >= WORDS, "{own} light {light}: a picked row");
                assert!(contrast(t.on_accent, t.accent) >= WORDS, "{own} light {light} {accent}: words on the highlight");
            }
        }
    }
    for own in [":root { --trindle-card: #7a7a7a; --trindle-box: #777777; }",
                ":root { --trindle-typing-box: #9a9a9a; --trindle-well: #ffffff; --trindle-control: #8c8c8c; }"] {
        let layers = Layers::themed(CSS, "", own);
        for light in [false, true] {
            let t = Theme::from(&layers, light, "#fbc711");
            let under = over(t.bg, if luminance(t.bg) < 0.18 { BLACK } else { WHITE });
            let card = over(t.panel, under);
            let seen = [under, card, over(t.raised, card), over(t.raised_more, card), over(t.field, card), over(t.control, under),
                        over(t.hover, card), over(t.well, under)];
            for (name, ink) in [("words", t.fg), ("quiet words", t.muted)] {
                let least = seen.iter().map(|l| contrast(ink, *l)).fold(f32::INFINITY, f32::min);
                let furthest = hex(ink) == (255, 255, 255) || hex(ink) == (0, 0, 0);
                assert!(least >= WORDS || furthest, "{own} light {light}: {name} stand {least} and could go further: {:?}", hex(ink));
            }
        }
    }
    let own = Layers::themed(CSS, "", ":root { --trindle-page: #808080; --trindle-text: #838383; }");
    assert_eq!(hex(Theme::from(&own, false, "#fbc711").fg), (0x83, 0x83, 0x83));
    let sized = Layers::themed(CSS, "", ":root { --trindle-gap: 14; --trindle-card-corner: 6; }");
    let plain = Layers::new(CSS, "");
    for light in [false, true] {
        for accent in highlights() {
            let (a, b) = (Theme::from(&sized, light, &accent), Theme::from(&plain, light, &accent));
            assert_eq!(format!("{:?}", (a.bg, a.panel, a.raised, a.fg, a.muted, a.accent, a.danger, a.danger_text, a.field_edge, a.on_picked)),
                       format!("{:?}", (b.bg, b.panel, b.raised, b.fg, b.muted, b.accent, b.danger, b.danger_text, b.field_edge, b.on_picked)));
        }
    }
}

#[test]
fn trindles_own_look_holds_every_floor_with_every_highlight() {
    let layers = Layers::new(CSS, "");
    let s = Sheet::from(&layers);
    assert!(contrast(s.label_ink, s.label_back) >= WORDS && s.label_back.a >= 1.0);
    for light in [false, true] {
        for accent in highlights() {
            let t = Theme::from(&layers, light, &accent);
            let look = format!("Trindle, {}, with {accent}", if light { "light" } else { "dark" });
            for (name, c) in [("page", t.bg), ("card", t.panel), ("box", t.raised), ("box-more", t.raised_more), ("typing box", t.field),
                              ("control", t.control), ("hover", t.hover), ("well", t.well)] {
                assert!(c.a >= 1.0, "{look}: the {name} is solid");
                assert!(if light { luminance(c) >= LIGHT_ABOVE } else { luminance(c) <= DARK_BELOW }, "{look}: the {name} keeps its side");
                for (words, ink) in [("words", t.fg), ("quiet words", t.muted), ("red words", t.danger_text)] {
                    assert!(contrast(ink, c) >= WORDS, "{look}: {words} on the {name}: {}", contrast(ink, c));
                }
            }
            for (name, c) in [("page", t.bg), ("card", t.panel), ("box", t.raised)] {
                assert!(contrast(t.danger, c) >= EDGES, "{look}: the red mark on the {name}: {}", contrast(t.danger, c));
            }
            assert!(contrast(t.on_picked, over(t.picked, t.panel)) >= WORDS, "{look}: words on a picked row");
            assert!(contrast(over(t.field_edge, t.field), t.panel) >= EDGES, "{look}: a typing box's edge on a card");
            assert!(contrast(over(t.field_edge, t.field), t.field) >= EDGES, "{look}: a typing box's edge on its box");
            if !light {
                assert!(contrast(t.accent, t.panel) >= EDGES, "{look}: the highlight on a card");
                assert!(contrast(s.knob_colour, t.accent) >= EDGES, "{look}: a switch's knob while on");
            }
        }
        let t = Theme::from(&layers, light, style::DEFAULT_ACCENT);
        if !light {
            assert!(contrast(s.knob_off, t.bg) >= EDGES && contrast(s.knob_off, t.panel) >= EDGES, "a switch's knob while off");
        }
        for (ink, bar) in [("trindle-title-ink", "trindle-title-background"), ("trindle-title-away-ink", "trindle-title-away-background")] {
            let r = Reader::new(&layers, light, t.accent);
            assert!(contrast(r.c(ink), r.c(bar)) >= WORDS, "{ink} on {bar}, light {light}");
        }
    }
}

fn red_grounds(t: &Theme) -> Vec<(String, Colour)> {
    let area = |c: Colour| over(c, t.bg);
    let mut all: Vec<(String, Colour)> = [("the page", t.bg), ("a card", t.panel), ("a box", t.raised), ("a box in a box", t.raised_more),
        ("a typing box", t.field), ("a control", t.control), ("what is pointed at", t.hover), ("the well", t.well),
        ("the top bar", area(t.top_bar)), ("the side bar", area(t.side_bar)), ("the foot", area(t.foot)), ("the trial's foot", area(t.trial_foot))]
        .into_iter().map(|(name, c)| (name.to_string(), c)).collect();
    for (depth, ground) in [(0, t.bg), (0, area(t.side_bar)), (1, t.panel), (2, t.raised)] {
        for (name, c) in [("a banded row", over(t.band, ground)), ("a row pointed at", t.card(depth.max(1))),
                          ("a side bar's row pointed at", over(alpha(t.fg, 0.06), ground)),
                          ("a banded item pointed at", over(t.hover, over(t.band, ground)))] {
            all.push((format!("{name}, {depth} cards deep"), c));
        }
    }
    all
}

#[test]
fn red_words_can_be_read_wherever_they_are_written() {
    let mut looks = vec![("Trindle, dark".to_string(), Layers::new(CSS, ""), false), ("Trindle, light".to_string(), Layers::new(CSS, ""), true)];
    for (name, css) in PROPOSED {
        let layers = Layers::themed(CSS, css, "");
        for light in [false, true] {
            if layers.only.is_none_or(|only| only == light) {
                looks.push((format!("{name}, {}", if light { "light" } else { "dark" }), layers.clone(), light));
            }
        }
    }
    assert!(looks.len() >= 2 + PROPOSED.len());
    let mut short = Vec::new();
    for (look, layers, light) in &looks {
        for accent in highlights() {
            let t = Theme::from(layers, *light, &accent);
            assert_eq!(t.danger_text.a, 1.0, "{look}: red words are solid");
            for (name, ground) in red_grounds(&t) {
                assert_eq!(ground.a, 1.0, "{look} with {accent}: {name} is solid");
                let got = contrast(t.danger_text, ground);
                if got < WORDS {
                    short.push(format!("{look} with {accent}: {name}: {got:.2}"));
                }
            }
        }
    }
    assert!(short.is_empty(), "red words fall short in {} places:\n{}", short.len(), short.join("\n"));
}

#[test]
fn the_light_look_takes_nothing_from_the_dark() {
    let light = Theme::new(true, "#fbc711");
    for (name, c, want) in [("page", light.bg, (0xed, 0xed, 0xed)), ("card", light.panel, (0xff, 0xff, 0xff)), ("box", light.raised, (0xf2, 0xf2, 0xf2)),
                            ("typing box", light.field, (0xff, 0xff, 0xff)), ("typing box edge", light.field_edge, (0x7a, 0x7a, 0x7a)),
                            ("words", light.fg, (0x11, 0x11, 0x11)), ("quiet words", light.muted, (0x4f, 0x4f, 0x4f)),
                            ("red words", light.danger_text, (0xb7, 0x1c, 0x1c)), ("well", light.well, (0xe0, 0xe0, 0xe0)),
                            ("side line", light.side_line, (0xd6, 0xd6, 0xd6)), ("shadow", light.shadow, (16, 18, 30))] {
        assert_eq!(hex(c), want, "the light look's {name}");
        assert_eq!(c.a, 1.0, "the light look's {name}");
    }
    assert_eq!(light.raised_more, mix(light.raised, light.fg, 0.06));
    assert_eq!((light.control, light.hover), (light.panel, light.raised));
    assert_eq!(light.band, Colour { a: 0.04, ..BLACK });
    for (name, c) in [("edge", light.edge), ("box edge", light.card_edge), ("control edge", light.control_edge), ("area line", light.area_line)] {
        assert_eq!(c.a, 0.0, "the light look has no {name}");
    }
    assert!(light.side_bar.a == 1.0 && light.side_bar != light.bg && light.top_bar.a == 1.0 && light.top_bar != light.bg && light.top_bar != light.side_bar,
            "the light look's top bar and side bar are set apart by their grey");
    assert_eq!((hex(light.foot), light.foot.a), ((0xed, 0xed, 0xed), 1.0), "the light look's foot");
    assert!(light.panel_falls.is_empty() && light.raised_falls.is_empty() && light.field_inset.is_none());
    assert_eq!(&*light.float, &[fall(10.0, 28.0, 0.0, 0.18)]);
    let layers = Layers::new(CSS, "");
    let c = |name: &str| layers.colour(name, true).map(|c| (byte(c[0]), byte(c[1]), byte(c[2]), byte(c[3])));
    assert_eq!(c("trindle-title-background"), Some((0xff, 0xff, 0xff, 0xff)));
    assert_eq!(c("trindle-title-ink"), Some((0x11, 0x11, 0x11, 0xff)));
    assert_eq!(c("trindle-title-border"), Some((0xd6, 0xd6, 0xd6, 0xff)));
    assert_eq!(c("trindle-title-away-background"), Some((0xed, 0xed, 0xed, 0xff)));
    assert_eq!(c("trindle-title-away-ink"), Some((0x52, 0x52, 0x5b, 0xff)));
    assert_eq!(c("trindle-title-away-border"), Some((0xd6, 0xd6, 0xd6, 0xff)));
    assert_eq!(c("trindle-window-edge"), Some((0, 0, 0, 26)));
    let own = crate::common::sheet::Values::parse(CSS);
    for d in style::declarations(CSS).iter().filter(|d| !d.light) {
        let night = ["rgba(190, 215, 255", "rgba(200, 222, 255", "rgba(4, 9, 18", "inset "].iter().any(|n| d.value.contains(n));
        let shared = d.name.starts_with("trindle-label-") || d.name.starts_with("trindle-knob");
        if night && !shared {
            assert!(own.get(&d.name, true) != Some(d.value.as_str()), "--{} shows through the light look", d.name);
        }
    }
}

#[test]
fn the_default_look_is_the_house_style() {
    let dark = Theme::new(false, "#fbc711");
    assert_eq!(dark.accent_hover, shade(dark.accent, 0.88));
    assert_eq!(dark.on_picked, mix(dark.accent, WHITE, 0.35));
    assert_eq!(hex(dark.shadow), (0x02, 0x06, 0x0e));
    assert_eq!(&*dark.panel_falls, &[fall(8.0, 22.0, -6.0, 0.42)]);
    assert!(dark.raised_falls.is_empty());
    assert_eq!(dark.field_inset, Some(fall(2.0, 5.0, 0.0, 0.45)));
    assert_eq!(&*dark.float, &[fall(22.0, 54.0, 0.0, 0.55), fall(3.0, 10.0, 0.0, 0.40)]);
    let light = Theme::new(true, "#fbc711");
    assert_eq!(light.raised_more, mix(light.raised, light.fg, 0.06));
    assert_eq!(light.picked, mix(light.accent, WHITE, 0.88));
    assert_eq!(light.on_picked, mix(light.accent, BLACK, 0.62));
    assert_eq!(hex(light.shadow), (16, 18, 30));
    assert!(light.panel_falls.is_empty() && light.raised_falls.is_empty() && light.field_inset.is_none());
    assert_eq!(&*light.float, &[fall(10.0, 28.0, 0.0, 0.18)]);
    let s = Sheet::from(&Layers::new(CSS, ""));
    assert_eq!((s.corner_of(Part::Card), s.corner_of(Part::Box), s.panel_gap, s.gap, s.card_padding), (8.0, 8.0, 12.0, 12.0, 18.0));
    assert_eq!(s.corner, 8.0);
    assert_eq!((s.page_padding, s.corner_of(Part::Menu), s.tabs_padding, s.corner_of(Part::Row)), (24.0, 8.0, 3.0, 8.0));
    assert_eq!((s.button, s.icon, s.action_height, s.entry_height, s.menu_row), (34.0, 16.0, 34.0, 34.0, 34.0));
    assert_eq!((s.switch, s.knob, s.row, s.rich_row), ((44.0, 24.0), 18.0, 40.0, 54.0));
    assert_eq!((s.text, s.note, s.page_title, s.heading), ((14.0, 400), (13.0, 400), (36.0, 400), (16.0, 600)));
    assert_eq!((s.headline, s.rich_weight), ((15.0, 400), 600));
    assert_eq!(s.bento.gap, 12.0);
    assert_eq!(s.glow, Duration::from_millis(1600));
    assert_eq!(hex(s.calm), (0x22, 0x95, 0xf1));
    assert_eq!(hex(s.good), (0x01, 0x95, 0x87));
}

#[test]
fn every_value_read_is_in_the_sheet_and_its_limits_are_written_beside_it() {
    let layers = Layers::new(CSS, "");
    for light in [false, true] {
        let r = Reader::new(&layers, light, Colour::rgb(0.98, 0.78, 0.07));
        Sheet::read(&r);
        let _ = Theme::from(&layers, light, "#fbc711");
        assert!(r.missing.borrow().is_empty(), "{:?}", r.missing.borrow());
        for (name, min, max) in r.limits.borrow().iter() {
            let line = CSS.lines().find(|l| l.trim_start().starts_with(&format!("--{name}:"))).unwrap_or_else(|| panic!("{name}"));
            let shown = |v: f32| if v.fract() == 0.0 { format!("{v:.0}") } else { format!("{v}") };
            assert!(line.contains(&format!("({} to {})", shown(*min), shown(*max))), "{name}: {line}");
        }
    }
    for name in ["page", "card", "box", "text", "text-quiet", "highlight", "highlight-hover", "shade-red",
                 "card-corner", "box-corner", "card-gap", "page-padding", "button-height", "icon-size"] {
        let at = CSS.find(&format!("--trindle-{name}:")).unwrap_or_else(|| panic!("{name}"));
        assert!(at < CSS.find("   2 COLOURS\n   ===").expect("the COLOURS heading is part 2"), "{name} is in part 1");
    }
}

pub fn floors_hold(t: &Theme, what: &str) {
    let area = |c: Colour| if c.a > 0.0 { c } else { t.bg };
    let layers = [t.bg, t.panel, t.raised, t.raised_more, t.field, t.control, t.hover, t.well, area(t.top_bar), area(t.side_bar), area(t.foot),
                  t.trial_foot];
    let dark = luminance(t.bg) < 0.18;
    for (i, l) in layers.iter().enumerate() {
        assert!(l.a == 1.0, "{what}: layer {i} is solid");
        assert!(if dark { luminance(*l) <= DARK_BELOW + 0.001 } else { luminance(*l) >= LIGHT_ABOVE - 0.001 }, "{what}: layer {i} keeps the page's side");
        for (name, ink) in [("words", t.fg), ("quiet words", t.muted), ("red words", t.danger_text), ("highlighted words", t.accent_text)] {
            assert!(contrast(ink, *l) >= WORDS, "{what}: {name} on layer {i}: {}", contrast(ink, *l));
        }
    }
    assert!(contrast(t.on_picked, t.picked) >= WORDS, "{what}: a picked row");
    assert!(contrast(t.accent, t.panel) >= EDGES, "{what}: the highlight on a card");
    assert!(contrast(t.on_accent, t.accent) >= WORDS, "{what}: words on the highlight");
    assert!(contrast(t.field_edge, t.panel) >= EDGES && contrast(t.field_edge, t.field) >= EDGES, "{what}: a typing box's edge");
    assert!(contrast(t.danger, t.panel) >= EDGES, "{what}: the red mark");
    let red = |c: Colour| c.r > c.g && c.r > c.b;
    assert!(red(t.danger_text) || contrast(t.danger_text, BLACK) > 20.0 || contrast(t.danger_text, WHITE) > 20.0, "{what}: red words are red");
    assert!(numbers_are_real(&format!("{t:?}")), "{what}: a number is not one");
}

fn numbers_are_real(said: &str) -> bool {
    !said.contains("NaN") && !said.contains(": inf") && !said.contains("-inf") && !said.contains("(inf") && !said.contains(" inf,")
}

#[test]
fn a_theme_is_laid_between_trindles_own_and_the_persons() {
    let theme = ":root { --trindle-theme-name: \"Harbour\"; --trindle-card: #182432; --trindle-page: #0e1620; --trindle-gap: 16;
                         --trindle-highlight: #ff00ff; --trindle-made-up: 3; }";
    let layers = Layers::themed(CSS, theme, ":root { --trindle-page: #000000; }");
    let t = Theme::from(&layers, false, "#fbc711");
    assert_eq!((hex(t.panel), hex(t.bg), hex(t.raised)), ((0x18, 0x24, 0x32), (0, 0, 0), (0x23, 0x36, 0x55)));
    assert_eq!(Sheet::from(&layers).gap, 16.0);
    assert_eq!(hex(t.accent), (0xfb, 0xc7, 0x11));
    assert!(!layers.theme.has("trindle-highlight") && !layers.theme.has("trindle-made-up"));
    assert_eq!(layers.only, Some(false));
    assert!(!Theme::from(&layers, true, "#fbc711").light);
    let chalk = Layers::themed(CSS, ":root { --trindle-theme-look: light; --trindle-page: #ecebe6; --trindle-card: #fbfaf6; }", "");
    let t = Theme::from(&chalk, false, "#2295f1");
    assert!(t.light && hex(t.bg) == (0xec, 0xeb, 0xe6) && hex(t.raised) == (0xf2, 0xf2, 0xf2) && hex(t.fg) == (0x11, 0x11, 0x11));
    let both = Layers::themed(CSS, ":root { --trindle-page: #101010; } @media (prefers-color-scheme: light) { :root { --trindle-page: #fafafa; } }", "");
    assert_eq!(both.only, None);
    assert_eq!((hex(Theme::from(&both, false, "#fbc711").bg), hex(Theme::from(&both, true, "#fbc711").bg)), ((0x10, 0x10, 0x10), (0xfa, 0xfa, 0xfa)));
    assert_eq!(check(CSS), Ok(()));
    assert_eq!(Theme::from(&Layers::themed(CSS, CSS, ""), false, "#fbc711"), Theme::from(&Layers::new(CSS, ""), false, "#fbc711"));
    assert_eq!(Sheet::from(&Layers::themed(CSS, CSS, "")), Sheet::from(&Layers::new(CSS, "")));
}

#[test]
fn every_kind_of_value_in_a_theme_is_held_or_gives_way() {
    let wild = ":root { --trindle-card-corner: 1e30; --trindle-gap: -1e30; --trindle-text-size: NaN; --trindle-button-height: inf;
        --trindle-row-height: 99999999999999999999999999999999999999999999; --trindle-icon-size: 0x40; --trindle-page-padding: var(--trindle-page-padding);
        --trindle-card-padding: var(--trindle-nothing); --trindle-switch-width: 1e-40; --trindle-label-rest: 99999999s; --trindle-glow-time: -5ms;
        --trindle-focus-width: 4.0000001; --trindle-dropdown-rows: 1e9; --trindle-menu-width: url(http://example.com/a); }";
    let layers = Layers::themed(CSS, wild, "");
    let s = Sheet::from(&layers);
    let r = Reader::new(&layers, false, WHITE);
    Sheet::read(&r);
    for (name, min, max) in r.limits.borrow().iter() {
        let v = layers.number(name, false, *min, *max).unwrap_or(f32::NAN);
        assert!(v.is_finite() && v >= *min && v <= *max, "{name}: {v}");
    }
    assert_eq!((s.corner_of(Part::Card), s.gap, s.text.0, s.button, s.row, s.icon), (FULL_ROUND, 6.0, 14.0, 34.0, 40.0, 16.0));
    assert_eq!((s.page_padding, s.card_padding, s.switch.0, s.focus_width, s.dropdown_rows, s.menu_width), (24.0, 18.0, 32.0, 4.0, 16.0, 272.0));
    assert_eq!((s.label_rest, s.glow), (Duration::from_millis(2000), Duration::from_millis(300)));
    let wild = ":root { --trindle-card: porridge; --trindle-box: rgb(300, 0, 0); --trindle-text: var(--trindle-text);
        --trindle-text-quiet: #b3b3b3ff00; --trindle-typing-box: var(--trindle-nothing); --trindle-side-line: url(file:///etc/passwd);
        --trindle-well: #12; --trindle-hover: mix(#000, #fff); --trindle-control: shade(#808080, lots); --trindle-band: rgba(0, 0, 0, 2); }";
    let layers = Layers::themed(CSS, wild, "");
    let t = Theme::from(&layers, false, "#fbc711");
    assert_eq!(t, Theme::from(&Layers::new(CSS, ""), false, "#fbc711"));
    let ring = Layers::themed(CSS, ":root { --trindle-text: var(--trindle-text-quiet); --trindle-text-quiet: var(--trindle-text); }", "");
    let t = Theme::from(&ring, false, "#fbc711");
    assert!([(0xec, 0xf2, 0xfb), (0xae, 0xbd, 0xd4)].contains(&hex(t.fg)) && [(0xec, 0xf2, 0xfb), (0xae, 0xbd, 0xd4)].contains(&hex(t.muted)));
    let wild = ":root { --trindle-card-shadow: 0 9e9px 9e9px 9e9px rgba(0, 0, 0, 0.5); --trindle-box-shadow: 0 1px #000, 0 2px #000, 0 3px #000, 0 4px #000, 0 5px #000;
        --trindle-float-shadow: enormous; --trindle-typing-box-shadow: 0 -90px 4px 0 #000; }";
    let t = Theme::from(&Layers::themed(CSS, wild, ""), false, "#fbc711");
    assert_eq!(&*t.panel_falls, &[fall(32.0, 64.0, 16.0, 0.5)]);
    assert_eq!(t.raised_falls.len(), MOST_FALLS);
    assert_eq!(&*t.float, &[fall(22.0, 54.0, 0.0, 0.55), fall(3.0, 10.0, 0.0, 0.40)]);
    assert_eq!(t.field_inset, Some(fall(0.0, 4.0, 0.0, 1.0)));
}

#[test]
fn a_theme_cannot_reach_for_a_file_run_anything_or_be_more_than_values() {
    let line = |css: &str| check(css).map_err(|f| (f.line, f.why));
    let refused = |css: &str, at: usize, has: &str| match line(css) {
        Err((l, why)) => assert!(l == at && why.contains(has), "{css:?}: line {l}: {why}"),
        Ok(()) => panic!("{css:?} was let in"),
    };
    refused("@import \"other.css\";\n:root { --trindle-page: #000; }", 1, "another file");
    refused(":root {\n --trindle-page: #000;\n --trindle-card: url(file:///etc/passwd);\n}", 3, "another file");
    refused(":root { --trindle-page: URL ( 'x' ); }", 1, "another file");
    refused(":root { --trindle-card: #000; }\n@font-face { font-family: x; src: local(x); }", 2, "another file");
    refused(":root { --trindle-theme-about: \"see https://example.com\"; --trindle-page: #000; }", 1, "web address");
    refused(":root {\n\n  --trindle-page: #000;\n  --trindle-card: javascript:alert(1);\n}", 4, "not a colour");
    refused(":root { --trindle-gap: calc(1 + 1); }", 1, "not a number");
    refused(":root {\n --trindle-gap: 12;\n --trindle-card-shadow: lots;\n}", 3, "not a shadow");
    refused(":root { --trindle-page: #000; }\n@media (prefers-color-scheme: light) {\n :root {\n  --trindle-page: pale;\n }\n}", 4, "not a colour");
    refused("body { background: red; } .x { --trindle-page: #000; }", 0, "Nothing in the file");
    refused("", 0, "Nothing in the file");
    refused(&format!(":root {{ --trindle-page: #000; }} /*{}*/", "x".repeat(70_000)), 0, "too big");
    refused(":root { --trindle-page: #000; }\u{0}", 0, "more than words");
    refused(&format!(":root {{ --trindle-page: mix(#000, #fff, {}0.5); }}", " ".repeat(300)), 1, "too long");
    assert_eq!(line("/* @import url(x) */ :root { --trindle-page: #000; }"), Ok(()));
    assert_eq!(line(":root { --trindle-page: #101820; --trindle-from-the-future: 7; --trindle-card-corner: 900; --trindle-gap: var(--trindle-card-gap); }"), Ok(()));
    let layers = Layers::themed(CSS, ":root { --trindle-card: url(/dev/zero); --trindle-box: file:///etc/shadow; }", "");
    let r = Reader::new(&layers, false, WHITE);
    assert_eq!((r.colour("url(/dev/zero)", 0), r.colour("@import x", 0), r.colour("var(--../../etc/passwd)", 0)), (None, None, None));
}

#[test]
fn colours_of_your_own_sheet_that_fall_short_are_said_and_a_readable_one_is_not() {
    let hard = ":root { --trindle-page: #101010; --trindle-text: #121212; }";
    assert!(own_colours_fall_short(&Layers::themed(CSS, "", hard), "#fbc711"));
    assert!(!own_colours_fall_short(&Layers::themed(CSS, "", ":root { --trindle-page: #101820; }"), "#fbc711"));
    assert!(!own_colours_fall_short(&Layers::themed(CSS, "", ""), "#fbc711"));
    let bad = ":root { --trindle-page: #101010; --trindle-text: #121212; }";
    assert!(!own_colours_fall_short(&Layers::themed(CSS, bad, ":root { --trindle-gap: 12; }"), "#fbc711"));
}

#[test]
fn a_themes_words_are_held_to_what_can_be_read() {
    let bad = ":root { --trindle-page: rgba(16, 16, 16, 0.2); --trindle-card: #f0f0f0; --trindle-box: #202020; --trindle-text: #101010;
        --trindle-text-quiet: #1a1a1a; --trindle-shade-red: #2e8bc0; --trindle-bad: #121212; --trindle-typing-box-edge: #141414;
        --trindle-on-picked: var(--trindle-picked); --trindle-highlight-text: #121212; --trindle-label-ink: var(--trindle-label-background);
        --trindle-good: #f34236; --trindle-calm: #808080; }";
    let layers = Layers::themed(CSS, bad, "");
    for accent in ["#fbc711", "#3d51b4", "#000000", "#ffffff", "#795649"] {
        let t = Theme::from(&layers, false, accent);
        assert!(t.corrected);
        floors_hold(&t, accent);
    }
    let blue = Layers::themed(CSS, ":root { --trindle-shade-red: #2e8bc0; --trindle-bad: #2295f1; }", "");
    let t = Theme::from(&blue, false, "#fbc711");
    assert_eq!((hex(t.danger_text), hex(t.danger), t.corrected), ((0xff, 0xa5, 0x9c), (0xf3, 0x42, 0x36), true));
    let s = Sheet::from(&layers);
    assert!(contrast(s.label_ink, s.label_back) >= WORDS);
    assert_eq!((hex(s.good), hex(s.calm)), ((0x01, 0x95, 0x87), (0x22, 0x95, 0xf1)));
    let pink = Layers::themed(CSS, ":root { --trindle-shade-red: #ffb4ab; --trindle-bad: #ff5449; }", "");
    let t = Theme::from(&pink, false, "#fbc711");
    assert_eq!((hex(t.danger_text), hex(t.danger), t.corrected), ((0xff, 0xb4, 0xab), (0xff, 0x54, 0x49), false));
    let chalk = Layers::themed(CSS, ":root { --trindle-theme-look: light; --trindle-page: #ecebe6; --trindle-card: #fbfaf6; }", "");
    let t = Theme::from(&chalk, false, "#fbc711");
    assert!(contrast(t.accent, t.panel) >= EDGES && t.accent.r > t.accent.b && hex(t.accent) != (0xfb, 0xc7, 0x11));
    floors_hold(&t, "chalk");
    assert_eq!(hex(Theme::from(&chalk, false, "#3d51b4").accent), (0x3d, 0x51, 0xb4));
    let own = Layers::themed(CSS, bad, ":root { --trindle-text: #131313; }");
    assert_eq!(hex(Theme::from(&own, false, "#fbc711").fg), (0x13, 0x13, 0x13));
    let good = Layers::themed(CSS, ":root { --trindle-page: #0e1620; --trindle-card: #182432; --trindle-box: #223244; }", "");
    let t = Theme::from(&good, false, "#fbc711");
    assert!(!t.corrected && hex(t.panel) == (0x18, 0x24, 0x32) && hex(t.fg) == (0xec, 0xf2, 0xfb));
    let titles = Layers::themed(CSS, ":root { --trindle-title-background: #101010; --trindle-title-ink: #151515; --trindle-title-away-ink: #f0f0f0;
                                       --trindle-title-away-background: #ffffff; }", "");
    assert_eq!(titles.get("trindle-title-ink", false), Some("#ffffff"));
    assert_eq!(titles.get("trindle-title-away-ink", false), Some("#000000"));
}

pub fn as_shown(a: Colour, b: Colour) -> f32 {
    let k = |c: Colour| Colour::rgb(f32::from(byte(c.r)) / 255.0, f32::from(byte(c.g)) / 255.0, f32::from(byte(c.b)) / 255.0);
    let (x, y) = (luminance(k(a)), luminance(k(b)));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

#[test]
fn an_ink_a_rule_moved_stands_its_floor_as_a_screen_shows_it() {
    let mut dice = Dice(299);
    let mut any = || {
        let mut part = || dice.below(100_000) as f32 / 99_999.0;
        Colour::rgb(part(), part(), part())
    };
    let t = Theme::new(false, "#fbc711");
    let mut short = Vec::new();
    for i in 0..6000 {
        let (ink, ground, share) = (any(), any(), 0.2 + any().r * 0.6);
        for floor in [WORDS, EDGES] {
            let moved = clear(ink, &[ground], floor);
            if as_shown(moved, ground) < floor {
                short.push(format!("clear: {} on {}: {:.3} of {floor}", moved.css(), ground.css(), as_shown(moved, ground)));
            }
            let faint = dimmed(moved, share, &[ground], floor);
            if as_shown(over(faint, kept(ground)), ground) < floor {
                short.push(format!("dimmed: {} at {} on {}: {:.3} of {floor}", moved.css(), faint.a, ground.css(), as_shown(over(faint, kept(ground)), ground)));
            }
        }
        if as_shown(on(ground), ground) < WORDS {
            short.push(format!("on: {}: {:.3}", ground.css(), as_shown(on(ground), ground)));
        }
        if i % 4 == 0 && as_shown(t.fg, ground) >= WORDS {
            let wash = t.wash(alpha(ink, share), &[ground]);
            if as_shown(t.fg, over(wash, kept(ground))) < WORDS {
                short.push(format!("wash: {} at {} on {}: {:.3}", ink.css(), wash.a, ground.css(), as_shown(t.fg, over(wash, kept(ground)))));
            }
        }
    }
    assert!(short.is_empty(), "{} inks a rule made stand under their floor as shown:\n{}", short.len(), short.iter().take(40).cloned().collect::<Vec<_>>().join("\n"));
}

#[test]
fn the_compositors_measure_and_the_toolkits_are_one() {
    assert_eq!(crate::common::sheet::TITLE_CONTRAST, WORDS, "a title's floor is the words' floor");
    let mut dice = Dice(0x299);
    for _ in 0..4000 {
        let mut any = || {
            let mut part = || dice.below(100_000) as f32 / 99_999.0;
            Colour::rgb(part(), part(), part())
        };
        let (a, b) = (any(), any());
        let theirs = crate::common::sheet::contrast([a.r, a.g, a.b, 1.0], [b.r, b.g, b.b, 1.0]);
        assert_eq!((theirs, contrast(a, b)), (as_shown(a, b), as_shown(a, b)), "{} and {}", a.css(), b.css());
    }
}

pub struct Dice(pub u64);

impl Dice {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn spoiled(dice: &mut Dice, nasty: &[&str]) -> String {
    let mut bytes = CSS.as_bytes().to_vec();
    for _ in 0..(1 + dice.below(40)) {
        let at = dice.below(bytes.len());
        match dice.below(5) {
            0 => { bytes.truncate(at.max(1)); }
            1 => { let end = (at + dice.below(400)).min(bytes.len()); bytes.drain(at..end); }
            2 => { let piece = bytes[at..(at + dice.below(200)).min(bytes.len())].to_vec(); bytes.splice(at..at, piece); }
            3 => { bytes[at] = b"{};:/*()-#,9e \n\"'@"[dice.below(18)]; }
            _ => { let junk = nasty[dice.below(nasty.len())].as_bytes().to_vec(); bytes.splice(at..at, junk); }
        }
        if bytes.is_empty() {
            bytes.push(b' ');
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

pub fn hostile_sheets(count: usize, seed: u64) -> Vec<String> {
    let names: Vec<String> = style::declarations(CSS).into_iter().map(|d| d.name).collect();
    let deep = format!("{}#fff{}", "mix(#000, ".repeat(300), ", 0.5)".repeat(300));
    let long = "9".repeat(4000);
    let nasty = ["1e39", "-1e39", "NaN", "inf", "-inf", "0", "-0", "0.0000001", "99999999", "-99999999", "1e-45", "", " ", "#000", "#fff",
                 "#ffffff00", "#12", "#zzzzzz", "rgb(1,2)", "rgba(999, 0, 0, 9)", "rgba(0,0,0,0)", "transparent", "white", "black", "red",
                 "var(--trindle-page)", "var(--trindle-text)", "var(--trindle-nothing)", "var(--)", "var(", "var(--trindle-card", "))))", "((((",
                 "mix(#000, #fff, 0.5)", "mix(#000, #fff, 1e9)", "mix(var(--trindle-text), var(--trindle-page), -4)", "shade(#808080, 1e30)",
                 "shade(var(--trindle-highlight), 0)", "alpha(#fff, -3)", "alpha(var(--trindle-card), NaN)", "url(http://example.com/x.css)",
                 "url(file:///etc/passwd)", "@import 'x'", "expression(alert(1))", "javascript:1", "$(rm -rf ~)", "`id`", "../../etc/passwd",
                 "0 9e9px 9e9px 9e9px #000", "0 -9e9px", "none", "inset", "0 1px 2px 3px 4px 5px #000", "0 1px #000, 0 1px #000, 0 1px #000, 0 1px #000",
                 "400ms", "-400ms", "1e30s", "3s", "\"quoted\"", "'", "\"", "\\", "/*", "*/", "{", "}", ";;", "light", "dark", "both", "\u{202e}txet",
                 "\u{1f600}", "100%", "12pt", "12px", "12em", "calc(1/0)", "1/0", "0x7fffffff", "١٢", &deep, &long];
    let mut dice = Dice(seed | 1);
    let mut out = Vec::with_capacity(count);
    for n in 0..count {
        let mut css = String::new();
        match n % 4 {
            0 | 1 => {
                let light = n % 8 >= 4;
                css.push_str(if light { "@media (prefers-color-scheme: light) { :root {\n" } else { ":root {\n" });
                for _ in 0..(1 + dice.below(if n % 16 == 0 { 3000 } else { 60 })) {
                    css.push_str(&format!("  --{}: {};\n", names[dice.below(names.len())], nasty[dice.below(nasty.len())]));
                }
                css.push_str(if light { "} }\n" } else { "}\n" });
            }
            2 => {
                css.push_str(":root {\n");
                for _ in 0..40 {
                    css.push_str(&format!("  --{}: var(--{});\n", names[dice.below(names.len())], names[dice.below(names.len())]));
                }
                css.push_str("}\n");
            }
            _ => css = spoiled(&mut dice, &nasty),
        }
        out.push(css);
    }
    out.push("{".repeat(60_000));
    out.push(" ".repeat(64_000));
    out.push(format!(":root {{ --trindle-page: {}; }}", "(".repeat(60_000)));
    out.push(format!(":root {{ {} }}", "--trindle-gap: 7;".repeat(3500)));
    out.push(format!(":root {{ --trindle-gap: {} }}", "1".repeat(60_000)));
    out.push(format!("/*{}", "*".repeat(60_000)));
    out
}

fn cpu() -> Duration {
    crate::sys::os::thread_time()
}

#[test]
fn thousands_of_hostile_sheets_break_nothing() {
    let accents = ["#fbc711", "#000000", "#ffffff", "#3d51b4", "#f34236", "nonsense"];
    let mut slowest = Duration::ZERO;
    let sheets: Vec<String> = [0x7412_d1e5, 0x1234_5678, 0x0bad_5eed].iter().flat_map(|seed| hostile_sheets(1500, *seed)).collect();
    for (n, css) in sheets.iter().enumerate() {
        let one = cpu();
        let _ = check(css);
        let _ = style::declarations(css);
        let layers = Layers::themed(CSS, css, "");
        let sheet = Sheet::from(&layers);
        assert!(numbers_are_real(&format!("{sheet:?}")), "sheet {n}: a number is not one");
        for light in [false, true] {
            let r = Reader::new(&layers, light, WHITE);
            Sheet::read(&r);
            for (name, min, max) in r.limits.borrow().iter() {
                let v = layers.number(name, light, *min, *max).unwrap_or(f32::NAN);
                assert!(v.is_finite() && v >= *min && v <= *max, "sheet {n}: {name} is {v}");
            }
            let t = Theme::from(&layers, light, accents[n % accents.len()]);
            if !layers.theme.is_empty() {
                floors_hold(&t, &format!("sheet {n}"));
                assert!(contrast(sheet.label_ink, sheet.label_back) >= WORDS, "sheet {n}: the label");
            }
            for fall in t.float.iter().chain(t.panel_falls.iter()).chain(t.raised_falls.iter()).chain(t.field_inset.iter()) {
                assert!((0.0..=32.0).contains(&fall.y) && (0.0..=64.0).contains(&fall.blur) && (-32.0..=16.0).contains(&fall.spread)
                        && (0.0..=1.0).contains(&fall.alpha), "sheet {n}: {fall:?}");
            }
        }
        slowest = slowest.max(cpu() - one);
    }
    assert!(slowest < Duration::from_secs(10), "the slowest sheet took {slowest:?}");
}

pub const PROPOSED: [(&str, &str); 4] = [("midnight", include_str!("../../share/themes/midnight.css")), ("paper", include_str!("../../share/themes/paper.css")),
                                     ("bold", include_str!("../../share/themes/bold.css")), ("snug", include_str!("../../share/themes/snug.css"))];

#[test]
fn the_pane_in_use_is_told_by_its_cards_edge() {
    let mut looks = vec![("Trindle, dark".to_string(), Layers::new(CSS, ""), false), ("Trindle, light".to_string(), Layers::new(CSS, ""), true)];
    for (name, css) in PROPOSED {
        let layers = Layers::themed(CSS, css, "");
        for light in [false, true] {
            if layers.only.is_none_or(|only| only == light) {
                looks.push((format!("the {name} theme, {}", if light { "light" } else { "dark" }), layers.clone(), light));
            }
        }
    }
    for (name, layers, light) in looks {
        for accent in highlights() {
            let t = Theme::from(&layers, light, &accent);
            let (edge, usual) = (over(t.current_edge, t.panel), over(t.edge, t.panel));
            assert!(contrast(edge, t.panel) >= EDGES, "{name}: {:.2} to one from its card", contrast(edge, t.panel));
            assert_eq!(luminance(edge) > luminance(t.panel), luminance(t.panel) < 0.5, "{name}: lighter on dark, darker on light");
            assert_ne!(edge, usual, "{name}: not the ordinary edge");
            assert!(contrast(edge, usual) >= 1.3, "{name}: {:.2} to one from the ordinary edge", contrast(edge, usual));
            assert_ne!(edge, t.accent, "{name}: never the highlight");
        }
    }
    let sheet = Sheet::from(&Layers::new(CSS, ""));
    assert!(sheet.current_edge >= 2.0 && sheet.current_edge <= 3.0, "thicker than the ordinary edge");
    assert!((36.0..=72.0).contains(&sheet.pane_strip));
}

#[test]
fn a_worded_buttons_words_never_leave_it() {
    use crate::widget::{action_fixed_in, action_words_in, Texts};
    let biggest = ":root { --trindle-action-size: 999; --trindle-action-padding: 999; --trindle-action-gap: 999; --trindle-icon-size: 999; --trindle-button-height: 999; }";
    let mut sheets = vec![("Trindle".to_string(), Sheet::from(&Layers::new(CSS, ""))), ("the biggest a sheet allows".to_string(), Sheet::from(&Layers::new(CSS, biggest)))];
    sheets.extend(PROPOSED.iter().map(|(name, css)| (format!("the {name} theme"), Sheet::from(&Layers::themed(CSS, css, "")))));
    assert_eq!((sheets[1].1.action_text, sheets[1].1.action_padding, sheets[1].1.action_gap, sheets[1].1.icon), (20.0, 32.0, 16.0, 32.0));
    let (fonts, texts) = (crate::text::Fonts::new(), Texts::default());
    let said = ["Open", "Connect to a server", "Trust it and connect", "Delete permanently", "Empty the Trash", "Move to Trash", "Open in its app",
                "Show everything", "Clear the search", "Go up one folder", "Incomprehensibilities", "A much longer name than any button should ever be given"];
    for (name, s) in &sheets {
        for scale in [1.0, 1.25, 1.5, 2.0, 3.0] {
            let fixed = action_fixed_in(s, scale);
            for words in said {
                let whole = action_words_in(s, &fonts, &texts, words, 0.0, scale);
                assert_eq!(whole.lines.len(), 1, "{name}: given all the room it wants, {words:?} is one line");
                let mut across = fixed + (2.0 * s.action_text * scale).ceil();
                while across < fixed + whole.width + 40.0 {
                    let l = action_words_in(s, &fonts, &texts, words, across, scale);
                    let wide = l.widest();
                    assert!(wide <= across - fixed + 0.5, "{name}, scale {scale}: {words:?} is {wide} wide in a pill with {} for words", across - fixed);
                    assert!((1..=2).contains(&l.lines.len()), "{name}, scale {scale}: {words:?} takes {} lines at {across}", l.lines.len());
                    if whole.width.ceil() <= across - fixed {
                        assert_eq!(l.lines.len(), 1, "{name}: {words:?} stays on one line when it fits");
                    }
                    across += 7.0;
                }
            }
        }
    }
}

#[test]
fn the_rings_edge_stands_out_from_light_and_dark_windows_in_every_look() {
    let (white, black) = (Colour::rgb(1.0, 1.0, 1.0), Colour::rgb(0.0, 0.0, 0.0));
    for (look, t) in every_look() {
        assert!(t.ring_edge.a >= 1.0, "{look}: the edge is solid");
        assert!(contrast(t.ring_edge, white) >= EDGES, "{look}: edge on a white window {}", contrast(t.ring_edge, white));
        assert!(contrast(t.ring_edge, black) >= EDGES, "{look}: edge on a black window {}", contrast(t.ring_edge, black));
        assert!(contrast(t.ring_edge, t.ring) >= EDGES, "{look}: edge on the ring's grey {}", contrast(t.ring_edge, t.ring));
    }
    for bad in ["#ffffff", "#000000", "rgba(255, 255, 255, 0.1)", "#fefefe", "#010101"] {
        for light in [false, true] {
            let layers = Layers::themed(CSS, &format!(":root {{ --trindle-ring-edge: {bad}; }}"), "");
            let e = Theme::from(&layers, light, style::DEFAULT_ACCENT).ring_edge;
            assert!(e.a >= 1.0 && contrast(e, white) >= EDGES && contrast(e, black) >= EDGES, "{bad}: moved to {}", e.css());
        }
    }
    let wide = Layers::themed(CSS, ":root { --trindle-ring-edge-width: 40; }", "");
    let thin = Layers::themed(CSS, ":root { --trindle-ring-edge-width: 0; }", "");
    assert_eq!(Sheet::read(&Reader::new(&wide, false, Colour::rgb(1.0, 0.8, 0.0))).ring_edge_width, 3.0);
    assert_eq!(Sheet::read(&Reader::new(&thin, false, Colour::rgb(1.0, 0.8, 0.0))).ring_edge_width, 1.0);
}

pub fn every_look() -> Vec<(String, Theme)> {
    every_look_with("", "")
}

pub fn every_look_high_contrast() -> Vec<(String, Theme)> {
    every_look_with(":root { --trindle-high-contrast: 1; }", ", high contrast")
}

fn every_look_with(own: &str, suffix: &str) -> Vec<(String, Theme)> {
    let mut looks = vec![("Trindle, dark".to_string(), Layers::new(CSS, own), false), ("Trindle, light".to_string(), Layers::new(CSS, own), true)];
    for (name, css) in PROPOSED {
        let layers = Layers::themed(CSS, css, own);
        for light in [false, true] {
            if layers.only.is_none_or(|only| only == light) {
                looks.push((format!("the {name} theme, {}", if light { "light" } else { "dark" }), layers.clone(), light));
            }
        }
    }
    looks.into_iter().map(|(name, layers, light)| (format!("{name}{suffix}"), Theme::from(&layers, light, style::DEFAULT_ACCENT))).collect()
}

#[test]
fn every_look_parts_the_areas_by_steps_of_grey_and_only_bold_draws_a_line() {
    for (name, t) in every_look() {
        assert!(t.top_bar.a == 1.0 && t.side_bar.a == 1.0 && t.foot.a == 1.0, "{name}: each area has a grey of its own");
        assert!(t.top_bar != t.side_bar && t.top_bar != t.bg && t.side_bar != t.bg, "{name}: the steps differ");
        assert_eq!(t.area_line.a > 0.0, name.contains("bold"), "{name}: a line only in Bold");
    }
}

pub const SECTION_LINE_LEAST: f32 = 1.15;

pub fn section_lines_hold<M: Clone + 'static>(what: &str, view: crate::widget::Node<M>, look: &Theme, width: u32, lines: usize, below: u32, folder: Option<(&std::path::Path, &str)>) -> f32 {
    let mut ui = crate::ui::Ui::new(view, look.clone());
    let height = ui.natural_height(width as f32).ceil() as u32;
    let mut pix = tiny_skia::Pixmap::new(width, height).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    if let Some((dir, name)) = folder {
        let mut big = tiny_skia::Pixmap::new(width * 2, height * 2).unwrap();
        ui.paint(&mut big.as_mut(), 2.0);
        big.save_png(dir.join(name)).unwrap();
    }
    let thick = sheet().section_line_width.round() as u32;
    let (from, to) = (width * 3 / 10, width * 7 / 10);
    let at = |x: u32, y: u32| {
        let p = pix.pixel(x, y).unwrap();
        [p.red(), p.green(), p.blue()]
    };
    let flat = |y: u32| -> Option<[u8; 3]> {
        let first = at(from, y);
        (from..to).all(|x| at(x, y) == first).then_some(first)
    };
    let rgb = |c: [u8; 3]| Colour { r: c[0] as f32 / 255.0, g: c[1] as f32 / 255.0, b: c[2] as f32 / 255.0, a: 1.0 };
    let mut found = 0;
    let mut at_rows = Vec::new();
    let mut least = f32::MAX;
    let mut y = below.max(5);
    while y + thick + 5 < height {
        let (above, line, below) = (flat(y - 1), flat(y), flat(y + thick));
        if let (Some(a), Some(l), Some(b)) = (above, line, below) {
            if a == b && l != a && (y..y + thick).all(|r| flat(r) == Some(l)) {
                for r in y - 4..y {
                    assert_eq!(flat(r), Some(a), "{what}: the line at {y} touches what is above it");
                }
                for r in y + thick..y + thick + 4 {
                    assert_eq!(flat(r), Some(a), "{what}: the line at {y} touches what is below it");
                }
                let c = contrast(rgb(l), rgb(a));
                assert!(c >= SECTION_LINE_LEAST, "{what}: the line at {y} stands only {c:.2} to one from its side bar");
                least = least.min(c);
                found += 1;
                at_rows.push(y);
                y += thick + 4;
                continue;
            }
        }
        y += 1;
    }
    assert_eq!(found, lines, "{what}: {found} lines between sections, at {at_rows:?}, not {lines}");
    least
}

pub fn highlights() -> Vec<String> {
    let mut all = vec![style::DEFAULT_ACCENT.to_string()];
    all.extend(Palette::ALL.iter().map(|p| p.swatch().css()));
    all
}

fn folder_grounds(t: &Theme, pill: Colour) -> Vec<(String, Colour)> {
    let mut all = vec![("the drag picture".to_string(), pill)];
    let side = over(t.side_bar, t.bg);
    for (depth, ground) in [(0, t.bg), (0, side), (1, t.panel), (2, t.raised)] {
        let lighter = t.card(depth.max(1));
        let band = over(t.band, ground);
        for (name, c) in [("a row", ground), ("a banded row", band), ("a row pointed at", lighter), ("a picked row", t.picked),
                          ("a side bar's row", ground), ("where you are in a side bar", lighter), ("a side bar's row pointed at", over(alpha(t.fg, 0.06), ground))] {
            all.push((format!("{name}, {depth} cards deep"), c));
        }
    }
    for (on, ground) in [("the page", t.bg), ("a card", t.panel)] {
        for (banded, ground) in [("", ground), (" on a band", over(t.band, ground))] {
            let wash = t.picked;
            for (name, c) in [("an item", ground), ("an item pointed at", over(t.hover, ground)), ("a picked item", over(wash, ground))] {
                all.push((format!("Folders, {name}{banded} on {on}"), c));
            }
        }
    }
    all
}

#[test]
fn a_folder_icon_can_be_found_wherever_it_is_drawn() {
    let dark = Theme::from(&Layers::new(CSS, ""), false, style::DEFAULT_ACCENT);
    assert_eq!(dark.folder, mix(dark.accent, WHITE, 0.6), "a pale tint of the highlight");
    assert_eq!(dark.icon, mix(dark.fg, dark.panel, 0.1), "other icons a little softer than words");
    let teal = Colour::parse("#009688").unwrap();
    assert_eq!(dark.folder_ink(Some(teal), &[teal]), teal);
    assert_eq!(dark.faded().folder, dark.muted);
    let pink = Layers::new(CSS, ":root { --trindle-folder-icon: #ff80ab; }");
    assert_eq!(hex(Theme::from(&pink, false, style::DEFAULT_ACCENT).folder), (0xff, 0x80, 0xab));
    let mut looks = vec![("Trindle, dark".to_string(), Layers::new(CSS, ""), false), ("Trindle, light".to_string(), Layers::new(CSS, ""), true)];
    for (name, css) in PROPOSED {
        let layers = Layers::themed(CSS, css, "");
        for light in [false, true] {
            if layers.only.is_none_or(|only| only == light) {
                looks.push((format!("the {name} theme, {}", if light { "light" } else { "dark" }), layers.clone(), light));
            }
        }
    }
    for (look, layers, light) in looks {
        let pill = Sheet::from(&layers).label_back;
        let (mut least, mut least_plain, mut pale) = (f32::INFINITY, f32::INFINITY, Vec::<(String, usize)>::new());
        for accent in highlights() {
            let t = Theme::from(&layers, light, &accent);
            for (name, under) in folder_grounds(&t, pill) {
                let ink = t.folder_ink(None, &[under]);
                let stands = contrast(over(ink, under), under);
                assert!(stands >= EDGES, "{look}, {accent}: a folder on {name}: {stands}");
                least = least.min(stands);
                if ink == t.folder {
                    least_plain = least_plain.min(stands);
                } else if let Some(seen) = pale.iter_mut().find(|(n, _)| *n == name) {
                    seen.1 += 1;
                } else {
                    pale.push((name, 1));
                }
            }
            for ground in [t.bg, t.panel] {
                assert_eq!(t.folder_ink(None, &[ground]), t.folder, "{look}, {accent}: a folder on a plain row");
            }
        }
        println!("FOLDER|{look}|least {least:.2}|least in the folder colour itself {least_plain:.2}");
        for (name, highlights) in pale {
            println!("FOLDER|{look}|a paler shade with {highlights} of the 20 highlights on {name}");
        }
    }
}

