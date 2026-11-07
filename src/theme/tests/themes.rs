//! Checks on the themes that ship: each look of each is sound and keeps its own colours.

use super::*;

#[test]
fn each_look_of_each_theme_is_sound_and_a_saved_other_look_is_ignored() {
    let harbour = ":root { --trindle-theme-name: \"Harbour\"; --trindle-theme-look: both; --trindle-page: #0e1620; --trindle-card: #182432; \
                   --trindle-box: #223244; }\n@media (prefers-color-scheme: light) { :root { --trindle-page: #e6ecf1; --trindle-card: #ffffff; \
                   --trindle-box: #eef2f6; } }";
    let mut all = vec![("trindle", "", None)];
    all.extend(PROPOSED.iter().map(|(name, css)| (*name, *css, match *name { "paper" => Some(true), "snug" => None, _ => Some(false) })));
    all.push(("harbour", harbour, None));
    for (name, css, only) in all {
        assert!(css.is_empty() || check(css) == Ok(()), "{name}");
        let layers = Layers::themed(CSS, css, "");
        assert_eq!(layers.only, only, "{name}: the looks it has");
        assert_eq!(sample(css, false, style::DEFAULT_ACCENT).both, only.is_none(), "{name}");
        for saved in [false, true] {
            let light = only.unwrap_or(saved);
            let look = format!("{name}, {} saved", if saved { "light" } else { "dark" });
            for accent in highlights() {
                let t = Theme::from(&layers, saved, &accent);
                assert_eq!(t.light, light, "{look} with {accent}: the look drawn");
                assert_eq!(luminance(t.bg) >= LIGHT_ABOVE, light, "{look} with {accent}: the page is on its look's side");
                if !css.is_empty() {
                    floors_hold(&t, &format!("{look} with {accent}"));
                }
                if only.is_some() {
                    assert_eq!(t, Theme::from(&layers, !saved, &accent), "{look} with {accent}: the other look saved changes nothing");
                } else {
                    assert_ne!(t.bg, Theme::from(&layers, !saved, &accent).bg, "{look} with {accent}: its two looks differ");
                }
            }
            for (ink, bar) in [("trindle-title-ink", "trindle-title-background"), ("trindle-title-away-ink", "trindle-title-away-background")] {
                let (i, b) = (layers.colour(ink, saved).unwrap(), layers.colour(bar, saved).unwrap());
                assert_eq!((i, b), (layers.colour(ink, light).unwrap(), layers.colour(bar, light).unwrap()), "{look}: its title bars");
                if !css.is_empty() {
                    let c = |v: [f32; 4]| Colour { r: v[0], g: v[1], b: v[2], a: v[3] };
                    assert!(contrast(c(i), c(b)) >= WORDS, "{look}: a window's title on its bar: {}", contrast(c(i), c(b)));
                }
            }
        }
        if only.is_some() {
            let (dark, light) = (Reader::new(&layers, false, WHITE), Reader::new(&layers, true, WHITE));
            assert!(Sheet::read(&dark) == Sheet::read(&light), "{name}: its measures are the same whichever look is saved");
        }
    }
}

#[test]
fn the_proposed_themes_are_sound_with_every_highlight() {
    let ours = Layers::new(CSS, "");
    for (name, css) in PROPOSED {
        assert_eq!(check(css), Ok(()), "{name}");
        let layers = Layers::themed(CSS, css, "");
        assert!(layers.only.is_some() || name == "snug", "{name} says which look it is");
        for d in style::declarations(css) {
            assert!(ours.default.has(&d.name), "{name}: --{} is not a name Trindle has", d.name);
            assert_ne!(d.name, "trindle-highlight", "{name} leaves the highlight to the person");
        }
        let r = Reader::new(&layers, false, WHITE);
        Sheet::read(&r);
        for (n, min, max) in r.limits.borrow().iter() {
            if let Some(v) = layers.theme.get(n, false).and_then(crate::common::sheet::number) {
                assert!(v >= *min && v <= *max, "{name}: --{n}: {v} is outside {min} to {max}");
            }
        }
        for light in [false, true] {
            if layers.only.is_some_and(|only| only != light) {
                continue;
            }
            let plain = Theme::from(&layers, light, "#3d51b4");
            let raw = Reader::new(&layers, plain.light, plain.accent);
            let moved: Vec<String> = [("page", plain.bg), ("card", plain.panel), ("box", plain.raised), ("box-more", plain.raised_more),
                                      ("typing-box", plain.field), ("control", plain.control), ("hover", plain.hover), ("well", plain.well),
                                      ("text", plain.fg), ("text-quiet", plain.muted), ("shade-red", plain.danger_text), ("bad", plain.danger),
                                      ("typing-box-edge", plain.field_edge)]
                .iter().filter(|(n, c)| raw.c(&format!("trindle-{n}")) != *c).map(|(n, c)| format!("{n} became {}", c.css())).collect();
            assert!(moved.is_empty() && !plain.corrected, "{name}, light {light}: its own colours had to be moved: {moved:?}");
            for accent in highlights() {
                let t = Theme::from(&layers, light, &accent);
                floors_hold(&t, &format!("{name}, light {light} with {accent}"));
                assert!(contrast(t.on_accent, t.accent) >= WORDS, "{name}, light {light} with {accent}: words on the highlight");
            }
            let s = Sheet::read(&Reader::new(&layers, light, plain.accent));
            assert!(contrast(s.label_ink, s.label_back) >= WORDS, "{name}, light {light}: the label on pointing");
            assert!(contrast(s.knob_off, plain.bg) >= EDGES && contrast(s.knob_off, plain.panel) >= EDGES, "{name}, light {light}: a switch's knob while off");
            for accent in highlights().into_iter().filter(|_| !light) {
                let t = Theme::from(&layers, light, &accent);
                assert!(contrast(s.knob_colour, t.accent) >= EDGES, "{name}, light {light} with {accent}: a switch's knob while on: {}", contrast(s.knob_colour, t.accent));
            }
        }
    }
}

#[test]
fn midnight_is_the_look_trindle_had_and_the_other_themes_keep_theirs() {
    let midnight = PROPOSED[0].1;
    let layers = Layers::themed(CSS, midnight, "");
    let t = Theme::from(&layers, false, "#fbc711");
    for (name, c, want) in [("page", t.bg, (0x12, 0x12, 0x12)), ("card", t.panel, (0x1f, 0x1f, 0x1f)), ("box", t.raised, (0x2c, 0x2c, 0x2c)),
                            ("control", t.control, (0x1f, 0x1f, 0x1f)), ("hover", t.hover, (0x2c, 0x2c, 0x2c)), ("typing box", t.field, (0x12, 0x12, 0x12)),
                            ("typing box edge", t.field_edge, (0x8c, 0x8c, 0x8c)), ("words", t.fg, (0xe5, 0xe5, 0xe5)),
                            ("quiet words", t.muted, (0xb3, 0xb3, 0xb3)), ("red words", t.danger_text, (0xff, 0x8a, 0x80)),
                            ("side line", t.side_line, (0x26, 0x26, 0x26)), ("shadow", t.shadow, (0, 0, 0)), ("highlight", t.accent, (0xfb, 0xc7, 0x11))] {
        assert_eq!((hex(c), c.a), (want, 1.0), "Midnight's {name}");
    }
    assert_eq!(t.raised_more, mix(t.raised, t.fg, 0.06));
    assert_eq!(t.well, shade(t.bg, 0.6));
    assert_eq!(t.band, Colour { a: 0.2, ..BLACK });
    assert!(t.edge.a == 0.0 && t.card_edge.a == 0.0 && t.control_edge.a == 0.0 && !t.corrected);
    assert!(t.panel_falls.is_empty() && t.raised_falls.is_empty() && t.field_inset.is_none());
    assert_eq!(&*t.float, &[fall(10.0, 28.0, 0.0, 0.55)]);
    let s = Sheet::from(&layers);
    assert_eq!((s.corner_of(Part::Card), s.corner_of(Part::Box), s.corner_of(Part::Row), s.corner_of(Part::Menu)), (8.0, 6.0, 6.0, 8.0));
    assert_eq!((s.panel_gap, s.card_padding, s.snug_padding, s.menu_padding), (8.0, 16.0, 3.0, 6.0));
    assert_eq!((s.switch, s.knob, s.focus_offset, s.glow_width), ((40.0, 22.0), 16.0, 2.0, 3.0));
    assert_eq!((s.page_title, s.section, s.heading), ((36.0, 500), (24.0, 500), (16.0, 600)));
    assert_eq!((hex(s.label_back), hex(s.label_ink), s.label_edge.a, s.label_padding), ((0x2c, 0x2c, 0x2c), (0xe5, 0xe5, 0xe5), 0.0, (10.0, 5.0)));
    assert_eq!((hex(s.knob_colour), s.knob_shadow.a, hex(s.knob_off)), ((0x11, 0x11, 0x11), 0.0, (0xb3, 0xb3, 0xb3)));
    let title = |name: &str| layers.colour(name, false).map(|c| (byte(c[0]), byte(c[1]), byte(c[2])));
    assert_eq!((title("trindle-title-background"), title("trindle-title-ink"), title("trindle-title-border")),
               (Some((0x12, 0x12, 0x12)), Some((0xe5, 0xe5, 0xe5)), Some((0x26, 0x26, 0x26))));
    let own = crate::common::sheet::Values::parse(CSS);
    let changed: Vec<String> = style::declarations(midnight).into_iter().filter(|d| !ABOUT.contains(&d.name.as_str())).map(|d| {
        assert_ne!(own.get(&d.name, false), Some(d.value.as_str()), "Midnight need not name --{}", d.name);
        d.name
    }).collect();
    assert!(changed.len() > 40, "{}", changed.len());
    for (id, css) in &PROPOSED[1..] {
        let theme = crate::common::sheet::Values::parse(css);
        let light = Layers::themed(CSS, css, "").only == Some(true);
        for name in &changed {
            let lights_own = light && style::declarations(CSS).iter().any(|d| d.light && d.name == *name);
            assert!(theme.has(name) || lights_own, "{id} would take the new --{name} from Trindle's own look");
        }
    }
}

#[test]
#[ignore = "prints the contrast table that the themes page reads (contrast.txt)"]
fn print_the_contrast_of_the_themes() {
    let mut all = vec![("trindle", "")];
    all.extend(PROPOSED);
    for (name, css) in all {
        let layers = Layers::themed(CSS, css, "");
        let t = Theme::from(&layers, false, "#fbc711");
        let s = Sheet::from(&layers);
        let row = |what: &str, a: Colour, b: Colour| println!("CONTRAST|{name}|{what}|{}|{}|{:.1}", a.css(), b.css(), contrast(over(a, b), b));
        row("Words on the page", t.fg, t.bg);
        row("Words on a card", t.fg, t.panel);
        row("Words on a box", t.fg, t.raised);
        row("Words in a typing box", t.fg, t.field);
        row("Quiet words on the page", t.muted, t.bg);
        row("Quiet words on a card", t.muted, t.panel);
        row("Quiet words on a box", t.muted, t.raised);
        row("Red words on a card", t.danger_text, t.panel);
        row("Red words on a box", t.danger_text, t.raised);
        row("Typing box edge on a card", t.field_edge, t.panel);
        row("Highlight (yellow) on a card", t.accent, t.panel);
        row("Words on the highlight", t.on_accent, t.accent);
        row("Words on a picked row", t.on_picked, t.picked);
        row("Switch knob, off, on a card's track", s.knob_off, t.bg);
        row("Label on pointing", s.label_ink, s.label_back);
        row("Card edge on the page", over(t.edge, t.panel), t.bg);
        row("Card against the page", t.panel, t.bg);
        row("Round button on a card", t.bg, t.panel);
        let worst = highlights().iter().map(|a| { let t = Theme::from(&layers, false, a); (contrast(t.accent, t.panel), contrast(t.on_accent, t.accent)) })
            .fold((f32::INFINITY, f32::INFINITY), |m, c| (m.0.min(c.0), m.1.min(c.1)));
        println!("CONTRAST|{name}|Any of the 20 highlights on a card, at least|||{:.1}", worst.0);
        println!("CONTRAST|{name}|Words on any of the 20 highlights, at least|||{:.1}", worst.1);
    }
}
