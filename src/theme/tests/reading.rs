//! Checks that the sheet's values are read as written: layers, limits, shadows, mixes, corners, large text and motion.

use super::*;

#[test]
fn a_persons_sheet_is_laid_over_trindles_own() {
    let own = "/* cards with depth again, blue buttons */
        :root { --trindle-box-shadow: 0 12px 40px 4px rgba(0, 0, 0, 0.8);
                --trindle-gap: 16px; --trindle-highlight: #2e8bc0; --trindle-made-up: 3; }
        @media (prefers-color-scheme: light) { :root { --trindle-page: #ffeedd; } }
        .something-else { --trindle-card: red; }";
    let layers = Layers::new(CSS, own);
    assert_eq!(layers.own.get("trindle-made-up", false), None);
    assert_eq!(layers.get("trindle-card", false), Some("#172640"));
    assert_eq!(Sheet::from(&layers).gap, 16.0);
    let dark = Theme::from(&layers, false, "#fbc711");
    assert_eq!(&*dark.raised_falls, &[fall(12.0, 40.0, 4.0, 0.8)]);
    assert_eq!(&*dark.panel_falls, &[fall(8.0, 22.0, -6.0, 0.42)]);
    assert_eq!(&*dark.float, &[fall(22.0, 54.0, 0.0, 0.55), fall(3.0, 10.0, 0.0, 0.40)]);
    assert_eq!(hex(dark.bg), (0x0a, 0x11, 0x1e));
    let light = Theme::from(&layers, true, "#fbc711");
    assert_eq!(hex(light.bg), (0xff, 0xee, 0xdd));
    assert_eq!(hex(light.panel), (0xff, 0xff, 0xff));
    assert_eq!(layers.own.get("trindle-highlight", false), Some("#2e8bc0"));
}

#[test]
fn a_bad_value_gives_way_and_a_wild_one_is_held_within_limits() {
    let own = ":root { --trindle-gap: lots; --trindle-card-corner: 5000; --trindle-text-size: 2px;
        --trindle-button-height: -5; --trindle-card: not-a-colour; --trindle-box: var(--trindle-box);
        --trindle-card-shadow: 0 900px 900px 900px rgba(0, 0, 0, 7); --trindle-box-shadow: big;
        --trindle-switch-knob: 30; --trindle-switch-height: 16; --trindle-label-weight: 1234; }";
    let layers = Layers::new(CSS, own);
    let s = Sheet::from(&layers);
    assert_eq!(s.gap, 12.0);
    assert_eq!(s.corner_of(Part::Card), FULL_ROUND);
    assert_eq!(s.text.0, 12.0);
    assert_eq!(s.button, 28.0);
    assert_eq!(s.knob, 14.0);
    assert_eq!(s.label_weight, 900);
    let t = Theme::from(&layers, false, "#fbc711");
    assert_eq!(hex(t.panel), (0x17, 0x26, 0x40));
    assert_eq!(hex(t.raised), (0x23, 0x36, 0x55));
    assert!(t.raised_falls.is_empty());
    assert_eq!(&*t.panel_falls, &[fall(8.0, 22.0, -6.0, 0.42)]);
    let held = Falls::parse("0 900px 900px 900px rgba(0, 0, 0, 0.9)").unwrap();
    assert_eq!(&*held, &[fall(32.0, 64.0, 16.0, 0.9)]);
}

#[test]
fn the_picture_colours_are_complements_of_the_highlight() {
    let yellow = pair_for(Colour::parse("#fbc711").unwrap());
    assert_eq!((hex(yellow[0]), hex(yellow[1])), ((0x22, 0x95, 0xf1), (0x00, 0xbc, 0xd3)));
    let blue = pair_for(Colour::parse("#2196f3").unwrap());
    assert_eq!(hex(blue[0]), (0xfe, 0x97, 0x00));
    let pink = pair_for(Colour::parse("#e91e63").unwrap());
    assert_eq!(hex(pink[0]), (0x01, 0x95, 0x87));
    assert!(yellow.iter().all(|c| hex(*c) != (0xfe, 0x97, 0x00) && hex(*c) != (0x4b, 0xae, 0x4f)));
}

#[test]
fn shadows_read_as_css_writes_them() {
    assert_eq!(&*Falls::parse("none").unwrap(), &[]);
    assert_eq!(&*Falls::parse("inset 0 2px 5px 0 rgba(0, 0, 0, 0.5)").unwrap(), &[fall(2.0, 5.0, 0.0, 0.5)]);
    assert_eq!(&*Falls::parse("0 3px").unwrap(), &[fall(3.0, 0.0, 0.0, 1.0)]);
    assert_eq!(Falls::parse("0 1px #000, 0 2px #000, 0 3px #000, 0 4px #000").unwrap().len(), 3);
    for bad in ["", "big", "0", "0 1px 2px 3px 4px #000", "0 1px red green"] {
        assert_eq!(Falls::parse(bad), None, "{bad}");
    }
}

#[test]
fn shade_mix_and_alpha_work_as_the_style_list_means() {
    let grey = Colour::rgb(0.5, 0.5, 0.5);
    assert!((shade(grey, 1.2).r - 0.6).abs() < 0.001);
    let c = shade(Colour::rgb(0.8, 0.4, 0.2), 1.0);
    assert!((c.r - 0.8).abs() < 0.001 && (c.g - 0.4).abs() < 0.001 && (c.b - 0.2).abs() < 0.001);
    assert_eq!(mix(Colour::rgb(0.0, 0.0, 0.0), Colour::rgb(1.0, 1.0, 1.0), 0.25).g, 0.25);
    assert_eq!(alpha(grey, 0.5).a, 0.5);
    let layers = Layers::new(CSS, "");
    let r = Reader::new(&layers, false, Colour::rgb(0.98, 0.78, 0.07));
    assert_eq!(r.colour("mix(#000000, #ffffff, 0.5)", 0).map(|c| c.r), Some(0.5));
    assert_eq!(r.colour("alpha(var(--trindle-text), 0.5)", 0).map(|c| c.a), Some(0.5));
    assert_eq!(r.colour("mix(rgba(0, 0, 0, 1), #ffffff, 0.5)", 0).map(|c| c.g), Some(0.5));
    assert_eq!(r.colour("var(--trindle-nothing)", 0), None);
}

#[test]
fn every_part_takes_its_corner_from_the_sheet() {
    let set = |v: &str| {
        let css = Part::ALL.iter().fold(String::new(), |mut all, p| { all.push_str(&format!("--{}: {v}; ", p.name())); all });
        Sheet::from(&Layers::new(CSS, &format!(":root {{ {css} }}")))
    };
    let (square, round) = (set("0"), set("999"));
    for p in Part::ALL {
        assert_eq!(square.corner_of(p), 0.0, "{p:?} square");
        assert_eq!(round.corner_of(p), FULL_ROUND, "{p:?} round");
        assert!(CSS.contains(&format!("{}:", p.name())), "{p:?} is documented in the sheet");
    }
    assert_eq!(((round.corner_of(Part::Button) * 2.0).min(34.0 / 2.0)), 17.0);
    let base = Sheet::from(&Layers::new(CSS, ""));
    for p in Part::ALL.into_iter().filter(|p| *p != Part::Tick) {
        assert_eq!(base.corner_of(p), base.corner, "{p:?} default");
    }
    assert_eq!(base.corner_of(Part::Tick), 4.0);
    let mut code = String::new();
    for dir in ["src", "src/ui", "src/widget"] {
        for e in std::fs::read_dir(format!("{}/{dir}", env!("CARGO_MANIFEST_DIR"))).unwrap().flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with(".rs") && name != "theme.rs" && !name.contains("test") {
                code.push_str(&std::fs::read_to_string(e.path()).unwrap());
            }
        }
    }
    code.push_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/widget/map.rs")).unwrap_or_default());
    for p in Part::ALL {
        assert!(code.contains(&format!("Part::{p:?}")), "{p:?} is used by no drawing code");
    }
    assert!(!code.contains("sheet().corner)") && !code.contains("sheet().corner *"), "a part reads the base corner instead of its own");
}

#[test]
fn large_text_makes_the_words_and_what_holds_them_larger() {
    let read = |own: &str| Sheet::read(&Reader::new(&Layers::new(CSS, own), false, WHITE));
    let (plain, big) = (read(""), read(":root { --trindle-large-text: 2; }"));
    for (name, a, b) in [("text", plain.text.0, big.text.0), ("heading", plain.heading.0, big.heading.0), ("column", plain.column_heading.0, big.column_heading.0),
                         ("button", plain.button, big.button), ("row", plain.row, big.row), ("entry", plain.entry_height, big.entry_height),
                         ("menu row", plain.menu_row, big.menu_row)] {
        assert!((b - a * 2.0).abs() < 0.01, "{name}: {a} and {b}");
    }
    assert_eq!(read(":root { --trindle-large-text: 9; }").text.0, big.text.0, "held to its most");
}

#[test]
fn a_tables_row_and_heading_come_from_the_sheet() {
    let read = |own: &str| Sheet::read(&Reader::new(&Layers::new(CSS, own), false, WHITE));
    assert_eq!(read("").table, (36.0, 32.0));
    let roomy = read(":root { --trindle-table-row: larger; --trindle-table-head: smaller; }").table;
    assert!(roomy.0 > 36.0 && roomy.1 < 32.0, "{roomy:?}");
}

fn sheet_fields(sheet: &Sheet) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in format!("{sheet:#?}").lines().skip(1) {
        let named = line.strip_prefix("    ").filter(|l| !l.starts_with(' ')).and_then(|l| l.split_once(": "));
        match (named, out.last_mut()) {
            (Some((name, rest)), _) => out.push((name.to_string(), rest.to_string())),
            (None, Some(last)) => last.1.push_str(line.trim()),
            _ => {}
        }
    }
    out
}

fn first_number(value: &str) -> Option<f32> {
    let start = value.find(|c: char| c.is_ascii_digit() || c == '-')?;
    let end = value[start..].find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == 'e')).map_or(value.len(), |n| start + n);
    value[start..end].parse().ok()
}

const GROWS: &[&str] = &[
    "button", "icon", "ring_windows_title", "label_text", "info", "switch", "knob", "entry_height", "entry_eye", "row", "rich_row", "rich_icon",
    "tree_indent", "spinner", "action_height", "action_text", "word_button", "word_button_text", "page_title", "section", "lead", "heading",
    "headline", "figure", "big_figure", "text", "form_label", "note", "column_heading", "side_heading", "menu_width", "menu_row", "menu_icon",
    "menu_row_least", "tab_height", "tab_least", "tab_most", "tab_close", "tab_more", "tab_count", "pill_height", "pill_words",
    "dropdown_height", "dropdown_row", "dropdown_width", "badge", "badge_icon", "trial_foot", "folders",
    "sidebar_width", "ring_windows_name_width", "drawer_width", "icon_tile", "icon_tile_icon", "status_icon", "radio", "tick",
];

const STAYS: &[&str] = &[
    "ring_button", "ring_app_edge", "ring_button_gap", "ring_band_room", "ring_band_gap", "ring_job_programs", "ring_job_computer", "ring_job_tools",
    "ring_job_tray", "ring_windows_width", "ring_windows_width_least", "ring_windows_gap", "ring_windows_part_gap", "ring_windows_icon_alone",
    "ring_windows_track", "ring_edge_width", "ring_edge_strength", "ring_windows_tab_firm", "ring_windows_tab_front", "ring_windows_tab_pad",
    "ring_windows_title_gap", "ring_windows_dot", "ring_windows_picture", "ring_windows_picture_margin", "ring_windows_picture_corner",
    "ring_windows_picture_icon", "label_back", "label_ink", "label_edge", "label_outline", "label_shadow", "label_weight", "label_padding",
    "label_gap", "label_width_most", "label_lines_most", "label_rest", "label_stays", "slide_time", "fade_time", "reduce_motion", "notice_stays",
    "hover_edge", "armed", "work_done", "work_heavy", "bento", "welcome", "app_page", "apps_list", "palette", "term", "pairs", "calm", "good", "warn",
    "calm_light", "good_light", "warn_light", "problem", "quiet", "colour_blind_safe", "safe", "corner", "corners", "panel_gap", "page_padding", "gap", "card_padding",
    "snug_padding", "sidebar_room", "line_padding", "chip_padding", "tabs_padding", "focus_width", "focus_offset", "knob_colour", "knob_shadow",
    "knob_off", "track", "entry_padding", "caret", "selection", "mark", "rich_gap", "rich_weight", "scrollbar", "scrollbar_hover", "scrollbar_rest",
    "scrollbar_lit", "scrollbar_inset", "spinner_line", "preview_panel", "preview_picture", "drive_card", "fill_bar", "hidden_drive", "glow",
    "glow_width", "bell", "glow_strength", "lit_width", "working_line", "action_padding", "action_gap", "action_weight", "word_button_padding",
    "tighten", "menu_padding", "menu_sub_pause", "menu_sub_overlap", "menu_sub_arrow", "menu_title_pad", "menu_group_gap", "menu_edge",
    "tab_pad", "tab_slop", "tab_tear", "pill_side", "pill_widest", "pill_room", "pill_foot", "dropdown_rows", "state_dot", "area_line",
    "section_line_width", "trial_foot_padding", "current_edge", "pane_strip", "note_title_weight", "note_slide", "circle", "circle_rest_least",
    "circle_gap", "circle_margin", "circle_button", "circle_most", "circle_pointed_most", "circle_hold", "circle_step",
    "code", "title_bar_room", "frame_edge", "frame_shadow", "divider", "divider_lit", "split_narrow", "drawer_room", "dot", "dot_target", "fling_friction",
    "chart_line", "bars", "bars_width", "bars_least", "bars_gap", "bars_corner", "skeleton_line", "skeleton_gap",
    "skeleton_last", "gaps", "colour_square", "colour_swatch", "colour_swatch_corner", "colour_code", "gradient_bar", "volume", "thumb", "lane", "table", "info_icon",
];

#[test]
fn every_size_in_the_sheet_grows_with_large_text_or_is_listed_as_staying() {
    let read = |own: &str| Sheet::read(&Reader::new(&Layers::new(CSS, own), false, WHITE));
    let (plain, big) = (sheet_fields(&read("")), sheet_fields(&read(":root { --trindle-large-text: 2; }")));
    let mut unlisted = Vec::new();
    for ((name, a), (_, b)) in plain.iter().zip(&big) {
        match (GROWS.contains(&name.as_str()), STAYS.contains(&name.as_str())) {
            (true, false) => {
                let (x, y) = (first_number(a).unwrap_or(0.0), first_number(b).unwrap_or(0.0));
                assert!(x > 0.0 && (y - 2.0 * x).abs() < 0.01, "{name} is listed as growing: {a} and {b}");
            }
            (false, true) => assert_eq!(a, b, "{name} is listed as staying but changes"),
            (false, false) => unlisted.push(name.as_str()),
            (true, true) => panic!("{name} is in both lists"),
        }
    }
    assert!(unlisted.is_empty(), "sizes in neither list: {unlisted:?}");
    for name in GROWS.iter().chain(STAYS) {
        assert!(plain.iter().any(|(n, _)| n == name), "{name} is in a list but not in the sheet");
    }
}

#[test]
fn motion_comes_from_the_sheet() {
    let read = |own: &str| Sheet::read(&Reader::new(&Layers::new(CSS, own), false, WHITE));
    let s = read(":root { --trindle-slide-time: 300ms; --trindle-fade-time: 5000ms; --trindle-reduce-motion: 1; }");
    assert_eq!((s.slide_time, s.fade_time, s.reduce_motion), (Duration::from_millis(300), Duration::from_millis(1000), true));
    assert!(!read("").reduce_motion);
}
