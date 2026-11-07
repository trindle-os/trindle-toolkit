//! Checks on the settings for people who need them: high contrast and colour-blind safe.

use super::*;

#[test]
fn high_contrast_holds_every_part_of_every_theme() {
    let mut all = vec![("trindle", "")];
    all.extend(PROPOSED);
    for (name, css) in all {
        let (plain, high) = (Layers::themed(CSS, css, ""), Layers::themed(CSS, css, ":root { --trindle-high-contrast: 1; }"));
        for light in [false, true] {
            for accent in highlights() {
                let (was, t) = (Theme::from(&plain, light, &accent), Theme::from(&high, light, &accent));
                let what = format!("{name}, {}, {accent}", if light { "light" } else { "dark" });
                let area = |c: Colour| if c.a > 0.0 { over(c, t.bg) } else { t.bg };
                let grounds = [t.bg, t.panel, t.raised, t.raised_more, t.field, t.control, t.hover, t.well, area(t.top_bar), area(t.side_bar), area(t.foot),
                               t.trial_foot, area(t.band)];
                let least = |c: Colour| grounds.iter().map(|g| contrast(c, *g)).fold(f32::INFINITY, f32::min);
                for (part, ink, before) in [("words", t.fg, was.fg), ("quiet words", t.muted, was.muted), ("icons", t.icon, was.icon),
                                            ("highlighted words", t.accent_text, was.accent_text), ("red words", t.danger_text, was.danger_text)] {
                    assert!(least(ink) >= HIGH_WORDS - 0.01, "{what}: {part} {}", least(ink));
                    assert!(least(ink) >= least(before) - 0.01, "{what}: {part} lost contrast");
                }
                for (part, line) in [("card edge", t.edge), ("box edge", t.card_edge), ("control edge", t.control_edge), ("typing box edge", t.field_edge),
                                     ("area line", t.area_line), ("side line", t.side_line), ("section line", t.section_line), ("current edge", t.current_edge)] {
                    assert!(least(line) >= HIGH_EDGES - 0.01, "{what}: {part} {}", least(line));
                }
                let best = |g: Colour| HIGH_WORDS.min(contrast(WHITE, g).max(contrast(BLACK, g)));
                assert!(contrast(t.on_picked, t.picked) >= best(t.picked) - 0.01, "{what}: a picked row");
                assert!(contrast(t.on_accent, t.accent) >= best(t.accent) - 0.01, "{what}: words on the highlight");
                assert_eq!((t.bg, t.panel, t.accent), (was.bg, was.panel, was.accent), "{what}: the layers and the highlight stay");
            }
        }
    }
}

const SEEN_AS: [[f32; 9]; 4] = [
    [0.152286, 1.052583, -0.204868, 0.114503, 0.786281, 0.099216, -0.003882, -0.048116, 1.051998],
    [0.367322, 0.860646, -0.227968, 0.280085, 0.672501, 0.047413, -0.011820, 0.042940, 0.968881],
    [1.255528, -0.076749, -0.178779, -0.078411, 0.930809, 0.147602, 0.004733, 0.691367, 0.303900],
    [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
];

fn seen(c: Colour, kind: usize) -> [f32; 3] {
    let lin = |v: f32| if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
    let (rgb, m) = ([lin(c.r), lin(c.g), lin(c.b)], &SEEN_AS[kind]);
    let [r, g, b] = [0, 3, 6].map(|i| (m[i] * rgb[0] + m[i + 1] * rgb[1] + m[i + 2] * rgb[2]).clamp(0.0, 1.0));
    let (x, y, z) = (0.4124 * r + 0.3576 * g + 0.1805 * b, 0.2126 * r + 0.7152 * g + 0.0722 * b, 0.0193 * r + 0.1192 * g + 0.9505 * b);
    let f = |t: f32| if t > 0.008856 { t.cbrt() } else { 7.787 * t + 16.0 / 116.0 };
    let (fx, fy, fz) = (f(x / 0.95047), f(y), f(z / 1.08883));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

fn least_apart(colours: &[Colour]) -> f32 {
    let mut least = f32::MAX;
    for kind in 0..SEEN_AS.len() {
        for (i, a) in colours.iter().enumerate() {
            for b in &colours[i + 1..] {
                let (p, q) = (seen(*a, kind), seen(*b, kind));
                least = least.min(((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt());
            }
        }
    }
    least
}

const TOLD_APART: f32 = 15.0;

#[test]
fn colour_blind_safe_tells_states_and_pieces_apart_in_every_kind_of_colour_blindness() {
    let own = Layers::new(CSS, ":root { --trindle-colour-blind-safe: 1; }");
    let (plain, safe) = (Sheet::read(&Reader::new(&Layers::new(CSS, ""), false, WHITE)), Sheet::read(&Reader::new(&own, false, WHITE)));
    assert!(!plain.colour_blind_safe && safe.colour_blind_safe);
    let states = |s: &Sheet, light: bool| if light { [s.calm_light, s.good_light, s.warn_light, s.problem] } else { [s.calm, s.good, s.warn, s.problem] };
    let before = [false, true].map(|light| least_apart(&states(&plain, light)));
    for (light, floor) in [(false, 0.0), (true, EDGES)] {
        let after = safe.safe[usize::from(light)];
        assert_eq!(states(&safe, light)[..3], after.states[..3], "the sheet's states are the safe ones when asked");
        for (what, colours) in [("states", after.states.to_vec()), ("pieces", after.series.to_vec())] {
            let apart = least_apart(&colours);
            assert!(apart >= TOLD_APART, "light {light}: the {what} are only {apart:.1} apart");
            let look = Theme::from(&own, light, style::DEFAULT_ACCENT);
            for c in &colours {
                assert!(contrast(*c, look.bg) >= floor && contrast(*c, look.panel) >= floor, "light {light}: {} is not {floor} to one from the page and the card", c.css());
            }
        }
        assert!(least_apart(&states(&plain, light)) < TOLD_APART, "Trindle's own states are told apart, so the switch would change nothing");
        assert!(least_apart(&after.states) > before[usize::from(light)], "light {light}: the safe states are no better told apart");
    }
    assert_eq!((plain.problem, plain.calm), (Reader::new(&Layers::new(CSS, ""), false, WHITE).c("trindle-bad"), Reader::new(&Layers::new(CSS, ""), false, WHITE).c("trindle-calm")));
    assert_eq!(plain.button, safe.button);
    assert_ne!(plain.problem, safe.problem);
}
