//! The themes page writer and the boot menu maker, run by hand beside the gallery.

use super::*;

const ABOUT: [(&str, &str, &str); 4] = [
    ("midnight", "Trindle's own look as it was until 11 October 2026: dark and flat, in steps of grey. Black and grey with no edges and no shadow except under what floats, small corners and things a little closer together. Every value is the one Trindle's own sheet had before the blue look took its place.",
     "People who want black and grey with no colour in it but their highlight, and anyone who liked Trindle as it was."),
    ("paper", "Sheets of warm white paper on a cream desk. A light look that does not glare: hairline edges, the faintest shadow under each card, small crisp corners and warm near-black words. A pale highlight such as yellow is deepened until it can be found on white.",
     "Bright rooms, daylight and a window behind the screen, and anyone who finds light words on a dark screen hard to read."),
    ("bold", "As easy to see as Trindle can be made: black behind, white words, a clear pale edge round every card and box, bigger words, and bigger buttons, switches and rows. The keyboard's ring is as thick as it goes.",
     "People with low vision or tired eyes, a small or distant screen, or a hand that finds small buttons hard to hit."),
    ("snug", "More on the screen: smaller words, shorter rows and buttons, less room round everything, nearly square corners and a thin edge round each card. Cool slate greys.",
     "Small laptop screens, and people who keep long lists and many settings in view and would rather see more than scroll."),
];

const SCREENS: [(&str, &str); 14] = [
    ("appearance", "Settings, Appearance, with the Theme card"),
    ("trying-a-theme", "A theme on trial: the foot that keeps it or goes back, always in sight and always in Trindle's own look"),
    ("network", "Settings, Network"),
    ("power", "Settings, Power"),
    ("apps-discover", "Apps, Discover"),
    ("apps-one-app", "Apps, one app's page"),
    ("folders", "Folders: a list with three rows picked and a copy under way"),
    ("ring", "The ring, open over a desktop"),
    ("lock-screen", "The lock screen"),
    ("password-question", "A question for the password"),
    ("dropdown-open", "A dropdown, open"),
    ("every-control", "Every control at once"),
    ("every-control-blue-highlight", "Every control, with a blue highlight chosen"),
    ("login-screen", "The login screen"),
];

const PAGE_STYLE: &str = "
  body { margin: 0; padding: 32px; background: #0a111e; color: #ecf2fb; font: 17px/1.5 'Trindle Sans', system-ui, sans-serif; }
  main { max-width: 1760px; margin: 0 auto; }
  h1 { font-size: 36px; font-weight: 500; margin: 0 0 8px; }
  h2 { font-size: 30px; font-weight: 500; margin: 0 0 8px; }
  h3 { font-size: 18px; font-weight: 600; margin: 28px 0 10px; }
  p { margin: 0 0 12px; max-width: 78ch; }
  .quiet { color: #aebdd4; }
  section { background: #172640; border-radius: 8px; padding: 24px; margin: 0 0 16px; }
  nav a { display: inline-block; background: #233655; border-radius: 17px; padding: 5px 16px; margin: 0 8px 8px 0; }
  a { color: #ecf2fb; text-decoration: none; }
  a:hover, a:focus { outline: 2px solid #fbc711; outline-offset: 2px; }
  .pair { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; align-items: start; }
  figure { margin: 0; background: #233655; border-radius: 6px; padding: 10px; }
  figcaption { font-size: 14px; color: #aebdd4; margin: 0 0 8px; }
  img { display: block; width: 100%; height: auto; border-radius: 4px; }
  table { border-collapse: collapse; margin: 8px 0 0; }
  th, td { text-align: left; padding: 5px 18px 5px 0; font-size: 15px; }
  th { color: #aebdd4; font-weight: 600; }
  .chip { display: inline-block; width: 14px; height: 14px; border-radius: 7px; vertical-align: -2px; margin-right: 6px; }
  .fail { color: #ffa59c; }
  @media (max-width: 1100px) { .pair { grid-template-columns: 1fr; } }
";

fn safe(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn screen_of(file: &str) -> &str {
    file.strip_suffix(".png").and_then(|f| f.split_once('-')).map_or(file, |(_, screen)| screen)
}

#[test]
#[ignore = "writes a page when THEMES_PREVIEW names the folder of pictures"]
fn write_the_themes_page() {
    use std::fmt::Write as _;
    let Some(out) = std::env::var_os("THEMES_PREVIEW").map(std::path::PathBuf::from) else { return };
    let root = crate::common::source::root();
    let mut themes: Vec<String> = std::fs::read_dir(&out).unwrap().flatten()
        .filter(|e| e.path().is_dir()).map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n != "trindle" && !n.starts_with('.')).collect();
    themes.sort();
    themes.sort_by_key(|t| ABOUT.iter().position(|(id, _, _)| id == t).unwrap_or(usize::MAX));
    themes.insert(0, "trindle".into());
    let name_of = |theme: &str| if theme == "trindle" { "Trindle".to_string() } else { said(root, theme, "name") };
    let contrast = std::fs::read_to_string(out.join("contrast.txt")).unwrap_or_default();
    let mut p = String::new();
    let _ = write!(p, "<!doctype html>\n<html lang=\"en-GB\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
                       <title>Trindle: themes</title>\n<style>{PAGE_STYLE}</style>\n</head>\n<body>\n<main>\n<section>\n<h1>Themes</h1>\n\
                       <p>Each theme is one file, made only from the names Trindle's sheet already has. Those in share/themes come with Trindle; any in \
                       themes-proposed wait for a yes. Every picture here was drawn by Trindle's own toolkit with that one file loaded, so this is how each \
                       would really look today. Each screen sits beside the same screen in Trindle's own look. Press a picture to see it full size.</p>\n<nav>");
    for t in &themes {
        let _ = write!(p, "<a href=\"#{}\">{}</a>", safe(t), safe(&if t == "trindle" { "Trindle, as it is now".to_string() } else { name_of(t) }));
    }
    p.push_str("</nav>\n</section>\n");
    for t in &themes {
        let name = name_of(t);
        let _ = writeln!(p, "<section id=\"{}\">\n<h2>{}</h2>", safe(t), safe(&name));
        if t == "trindle" {
            own_look(&mut p, &out);
        } else {
            beside_own_look(&mut p, &out, root, t, &name);
        }
        contrast_table(&mut p, &contrast, t);
        p.push_str("</section>\n");
    }
    p.push_str("</main>\n</body>\n</html>\n");
    std::fs::write(out.join("index.html"), p).unwrap();
}

fn folder(root: &std::path::Path, t: &str) -> &'static str {
    if root.join("share/themes").join(format!("{t}.css")).exists() { "share/themes" } else { "themes-proposed" }
}

fn said(root: &std::path::Path, theme: &str, what: &str) -> String {
    let css = std::fs::read_to_string(root.join(folder(root, theme)).join(format!("{theme}.css"))).unwrap_or_default();
    let v = crate::common::sheet::Values::parse(&css);
    v.get(&format!("trindle-theme-{what}"), false).unwrap_or("").trim_matches('"').to_string()
}

fn caption(key: &str) -> String {
    SCREENS.iter().find(|(k, _)| *k == key).map_or(key.to_string(), |(_, c)| c.to_string())
}

fn pictures(out: &std::path::Path, theme: &str) -> Vec<String> {
    let mut all: Vec<String> = std::fs::read_dir(out.join(theme)).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".png")).collect();
    all.sort();
    all
}

fn own_look(p: &mut String, out: &std::path::Path) {
    use std::fmt::Write as _;
    p.push_str("<p>Trindle's own look since 11 October 2026: a deep blue night, with panes of cooler slate, fine light edges, large round corners and soft shadows. It is built in, always first in the list, and is the look everybody starts with. The flat greys it took over from are the Midnight theme.</p>\n\
                <p class=\"quiet\">The login screen is shown once, here: it always wears Trindle's own look, whatever theme anyone has chosen.</p>\n<div class=\"pair\">\n");
    for f in pictures(out, "trindle") {
        let cap = safe(&caption(screen_of(&f)));
        let f = safe(&f);
        let _ = writeln!(p, "<figure><figcaption>{cap}</figcaption><a href=\"trindle/{f}\"><img loading=\"lazy\" src=\"trindle/{f}\" alt=\"{cap} in Trindle's own look\"></a></figure>");
    }
    p.push_str("</div>\n");
}

fn beside_own_look(p: &mut String, out: &std::path::Path, root: &std::path::Path, t: &str, name: &str) {
    use std::fmt::Write as _;
    let (idea, who) = ABOUT.iter().find(|(id, _, _)| *id == t).map_or((said(root, t, "about"), String::new()), |(_, i, w)| (i.to_string(), w.to_string()));
    let _ = writeln!(p, "<p>{}</p>", safe(&idea));
    if !who.is_empty() {
        let _ = writeln!(p, "<p><strong>Who it is for.</strong> {}</p>", safe(&who));
    }
    let _ = writeln!(p, "<p class=\"quiet\">Its line in Settings: \"{}\" The file: {}/{t}.css</p>", safe(&said(root, t, "about")), folder(root, t));
    let (tt, n) = (safe(t), safe(name));
    for f in pictures(out, t) {
        let cap = safe(&caption(screen_of(&f)));
        let f = safe(&f);
        let _ = writeln!(p, "<h3>{cap}</h3>\n<div class=\"pair\">\n<figure><figcaption>Trindle now</figcaption><a href=\"trindle/{f}\"><img loading=\"lazy\" \
                             src=\"trindle/{f}\" alt=\"{cap} in Trindle's own look\"></a></figure>\n<figure><figcaption>{n}</figcaption><a href=\"{tt}/{f}\">\
                             <img loading=\"lazy\" src=\"{tt}/{f}\" alt=\"{cap} in the {n} theme\"></a></figure>\n</div>");
    }
}

fn contrast_table(p: &mut String, contrast: &str, t: &str) {
    use std::fmt::Write as _;
    let rows: Vec<Vec<&str>> = contrast.lines().map(|l| l.split('|').collect::<Vec<_>>()).filter(|c| c.len() == 6 && c[1] == t).collect();
    if !rows.is_empty() {
        p.push_str("<h3>Measured contrast</h3>\n<p class=\"quiet\">Words need 4.5 to one, and edges and the highlight 3 to one (WCAG 2.2 AA). The last three rows \
                    are not rules: they show how far a card and a round button stand from what they sit on.</p>\n<table><tr><th>What</th><th>Colours</th><th>Contrast</th></tr>\n");
        for c in rows {
            let (what, ratio) = (c[2], c[5].parse::<f32>().unwrap_or(f32::NAN));
            let edges = ["edge on a card", "Highlight", "knob", "Any of"].iter().any(|w| what.contains(w));
            let free = ["Card edge on the page", "Card against", "Round button"].iter().any(|w| what.contains(w));
            #[allow(clippy::neg_cmp_op_on_partial_ord)]
            let fails = !free && !(ratio >= if edges { crate::theme::EDGES } else { crate::theme::WORDS });
            let chips = [c[3], c[4]].iter().filter(|x| !x.is_empty()).fold(String::new(), |mut s, x| {
                let _ = std::fmt::Write::write_fmt(&mut s, format_args!("<span class=\"chip\" style=\"background:{}\"></span>", safe(x)));
                s
            });
            let colours = [c[3], c[4]].iter().filter(|x| !x.is_empty()).map(|x| safe(x)).collect::<Vec<_>>().join(" on ");
            let _ = writeln!(p, "<tr><td>{}</td><td>{chips}{colours}</td><td{}>{ratio:.1} to 1</td></tr>", safe(what), if fails { " class=\"fail\"" } else { "" });
        }
        p.push_str("</table>\n");
    }
}

const BOOT: &str = "share/grub/trindle";

fn boot_theme(text: &str, t: &Theme) -> String {
    let mut titled = false;
    let mut out = String::new();
    for line in text.lines() {
        let colour = match line.trim_start().split(['=', ':']).next().map(str::trim) {
            Some("desktop-color") => Some(t.bg),
            Some("selected_item_color") => Some(t.fg),
            Some("item_color") => Some(t.muted),
            Some("color") => Some(if std::mem::replace(&mut titled, true) { t.muted } else { t.fg }),
            _ => None,
        };
        match (colour, line.find('"')) {
            (Some(c), Some(at)) => out.push_str(&format!("{}\"{}\"", &line[..at], c.css())),
            _ => out.push_str(line),
        }
        out.push('\n');
    }
    out
}

fn boot_pictures(t: &Theme) -> Vec<(&'static str, tiny_skia::Pixmap)> {
    use crate::theme::over;
    use crate::widget::Rect;
    use tiny_skia::{FillRule, Paint, Pixmap, Transform};
    const END: u32 = 22;
    const STRIP: u32 = 2;
    const TALL: u32 = 44;
    let mut page = Pixmap::new(1920, 1080).unwrap();
    page.fill(t.bg.to_skia());
    let mut pill = Pixmap::new(END + STRIP + END, TALL).unwrap();
    let (w, h) = (pill.width() as f32, TALL as f32);
    for (r, colour) in [(Rect { x: 0.0, y: 0.0, w, h }, over(t.card_edge, t.raised)), (Rect { x: 1.0, y: 1.0, w: w - 2.0, h: h - 2.0 }, t.raised)] {
        let mut paint = Paint::default();
        paint.set_color(colour.to_skia());
        paint.anti_alias = true;
        let path = crate::ui::rounded(r, r.h / 2.0).unwrap();
        pill.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
    }
    let cut = |from: u32, wide: u32| {
        let mut part = Pixmap::new(wide, TALL).unwrap();
        for y in 0..TALL {
            for x in 0..wide {
                let (to, at) = (((y * wide + x) * 4) as usize, ((y * pill.width() + from + x) * 4) as usize);
                part.data_mut()[to..to + 4].copy_from_slice(&pill.data()[at..at + 4]);
            }
        }
        part
    };
    vec![("background.png", page), ("select_w.png", cut(0, END)), ("select_c.png", cut(END, STRIP)), ("select_e.png", cut(END + STRIP, END))]
}

#[test]
#[ignore = "needs Trindle's share/grub/trindle: set TRINDLE_SOURCE_ROOT to Trindle's folder"]
fn the_boot_menu_is_made_from_the_sheet() {
    let t = crate::theme::safe();
    let folder = crate::common::source::root().join(BOOT);
    let said = std::fs::read_to_string(folder.join("theme.txt")).unwrap();
    assert_eq!(boot_theme(&said, &t), said, "run draw_the_boot_menu: the boot menu's colours are not the sheet's");
    assert_eq!(said.matches(&format!("\"{}\"", t.fg.css())).count(), 2);
    assert_eq!(said.matches(&format!("\"{}\"", t.muted.css())).count(), 3);
    for (name, drawn) in boot_pictures(&t) {
        let kept = tiny_skia::Pixmap::load_png(folder.join(name)).unwrap();
        assert_eq!((kept.width(), kept.height()), (drawn.width(), drawn.height()), "{name}");
        let apart = kept.data().iter().zip(drawn.data()).map(|(a, b)| a.abs_diff(*b)).max().unwrap_or(0);
        assert!(apart <= 2, "run draw_the_boot_menu: {name} is not drawn from the sheet ({apart} apart)");
    }
}

#[test]
#[ignore = "writes Trindle's share/grub/trindle from the style sheet: set TRINDLE_SOURCE_ROOT to Trindle's folder"]
fn draw_the_boot_menu() {
    let t = crate::theme::safe();
    let folder = crate::common::source::root().join(BOOT);
    let said = std::fs::read_to_string(folder.join("theme.txt")).unwrap();
    std::fs::write(folder.join("theme.txt"), boot_theme(&said, &t)).unwrap();
    for (name, drawn) in boot_pictures(&t) {
        drawn.save_png(folder.join(name)).unwrap();
    }
}
