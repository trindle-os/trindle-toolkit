//! Words of any length, in any script, stay inside the room of the part that shows them.

use super::*;
use super::super::theme::{drawn, Theme};
use super::super::widget::{self as ww, Length};

const WORDS: [(&str, &str); 7] = [
    ("english", "Supercalifragilisticexpialidocious configuration"),
    ("german", "Donaudampfschifffahrtsgesellschaftskapitaensmuetze"),
    ("arabic", "الاستعلاماتالمتعلقةبالإعداداتالمتقدمةللنظام الكاملة"),
    ("hebrew", "ההגדרותהמתקדמותשלמערכתההפעלההמלאה לשימוש"),
    ("thai", "การตั้งค่าขั้นสูงสำหรับระบบปฏิบัติการทั้งหมดของคุณ"),
    ("chinese", "系统的所有高级设置和配置选项都在这里管理和保存"),
    ("hindi", "सिस्टमकीसभीउन्नतसेटिंग्सऔरकॉन्फ़िगरेशनविकल्प"),
];

const ACROSS: f32 = 90.0;

type Make = fn(String) -> Node<()>;

fn kinds() -> Vec<(&'static str, Make)> {
    vec![
        ("label", |w| ww::label(w)),
        ("label cut", |w| ww::label(w).cut()),
        ("label cut_after", |w| ww::label(w).cut_after(2)),
        ("title", |w| ww::title(w)),
        ("heading", |w| ww::heading(w)),
        ("note", |w| ww::note(w)),
        ("section", |w| ww::section(w)),
        ("form_label", |w| ww::form_label(w)),
        ("column_heading", |w| ww::column_heading(w)),
        ("side_heading", |w| ww::side_heading(w)),
        ("main_word", |w| ww::main_word(w).on_press(())),
        ("main_action", |w| ww::main_action("document-open-symbolic", w).on_press(())),
        ("word_button", |w| ww::word_button(w).on_press(())),
        ("choice", |w| ww::choice(vec![w, "Two".into()], 0, "Which", |_| ())),
        ("dropdown", |w| ww::dropdown(vec![w, "Two".into()], Some(0), "Which", |_| ())),
        ("list", |w| ww::list(vec![ww::item(w.clone()), ww::item("Short").under(w)], None, |_| ())),
        ("chip", |w| ww::card(ww::label(w)).chip()),
        ("setting", |w| crate::settings::setting(w.clone(), w, ww::switch(true, "On", |_| ()))),
    ]
}

fn faults(kind: &str, make: Make, words: &str) -> Vec<String> {
    let node = ww::column(vec![make(words.to_string()).key("it").width(Length::Fixed(ACROSS))]).padding(10.0);
    let mut ui = Ui::new(node, Theme::new(false, "#fbc711"));
    let mut pix = tiny_skia::Pixmap::new(300, 600).unwrap();
    drawn::begin();
    ui.paint(&mut pix.as_mut(), 1.0);
    let noted = drawn::end();
    let mut wrong = Vec::new();
    let mut mine = ui.boxes().into_iter().filter(|b| b.0.starts_with("it")).map(|b| b.1);
    let first = mine.next().unwrap();
    let whole = mine.fold(first, |a, r| {
        let (x, y) = (a.x.min(r.x), a.y.min(r.y));
        Rect { x, y, w: (a.x + a.w).max(r.x + r.w) - x, h: (a.y + a.h).max(r.y + r.h) - y }
    });
    for d in noted.drawn.iter().filter(|d| d.kind == drawn::Kind::Words) {
        let (x, w) = (d.at.0 as f32, d.at.2 as f32);
        if x < whole.x - 1.0 || x + w > whole.x + whole.w + 1.0 {
            wrong.push(format!("{kind}: words at {:?} leave the part at {:?}", d.at, (whole.x, whole.w)));
        }
    }
    let bg = ui.theme.bg.to_skia().to_color_u8();
    let (w, h) = (pix.width() as i32, pix.height() as i32);
    let mut stray = 0;
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (x as f32, y as f32);
            if fx >= whole.x - 3.0 && fx <= whole.x + whole.w + 3.0 && fy >= whole.y - 3.0 && fy <= whole.y + whole.h + 3.0 {
                continue;
            }
            let p = pix.pixel(x as u32, y as u32).unwrap();
            if (p.red(), p.green(), p.blue()) != (bg.red(), bg.green(), bg.blue()) {
                stray += 1;
            }
        }
    }
    if stray > 0 {
        wrong.push(format!("{kind}: {stray} pixels painted outside the part at {:?}", (whole.x, whole.y, whole.w, whole.h)));
    }
    wrong
}

#[test]
fn words_of_any_length_and_script_stay_in_their_room() {
    let mut wrong = Vec::new();
    for (name, words) in WORDS {
        for (kind, make) in kinds() {
            for f in faults(&format!("{kind} ({name})"), make, words) {
                wrong.push(f);
            }
        }
    }
    assert!(wrong.is_empty(), "{}\n{}", wrong.len(), wrong.join("\n"));
}

mod drawn_by_hand {
    use super::*;
    use crate::icon::Icons;
    use crate::table::{paint_table, Column, Table};
    use crate::tabs::{paint_tabs, Tab, Tabs};
    use crate::text::Fonts;
    use crate::widget::Pen;

    fn inks(paint: impl FnOnce(&Pen, &mut tiny_skia::PixmapMut)) -> Vec<(f32, f32)> {
        let (theme, fonts, icons) = (Theme::new(false, "#fbc711"), Fonts::new(), Icons::new());
        let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
        let mut pix = tiny_skia::Pixmap::new(400, 120).unwrap();
        drawn::begin();
        paint(&pen, &mut pix.as_mut());
        drawn::end().drawn.iter().filter(|d| d.kind == drawn::Kind::Words).map(|d| (d.at.0 as f32, (d.at.0 + d.at.2 as i32) as f32)).collect()
    }

    #[test]
    fn a_tab_keeps_its_title_between_its_edges_and_its_close_box() {
        for (name, words) in WORDS {
            for count in [None, Some(12)] {
                let mut tab = Tab::new(words);
                tab.count = count;
                let mut tabs = Tabs::new(vec![tab]);
                tabs.least = 60.0;
                tabs.most = 120.0;
                let laid = tabs.layout(400.0);
                let r = laid.shown[0].1;
                let right = tabs.close_rect(0, r).map_or(r.x + r.w, |c| c.x);
                for (from, to) in inks(|pen, canvas| paint_tabs(canvas, pen, &tabs, None, None)){
                    assert!(from >= r.x - 1.0 && to <= right + 1.0, "{name} {count:?}: {from}..{to} leaves {}..{right}", r.x);
                }
            }
        }
    }

    #[test]
    fn a_cell_and_a_heading_keep_to_their_column() {
        for (name, words) in WORDS {
            let mut table = Table::new(vec![Column::new(words, 90.0), Column::new(words, 90.0).end(), Column::new("Kind", 80.0)]);
            table.set_rows(2);
            let cell = |_: usize, c: usize| if c == 2 { "Kind".to_string() } else { words.to_string() };
            for (from, to) in inks(|pen, canvas| paint_table(canvas, pen, &table, None, cell)) {
                let column = (0..3).find(|&i| from >= table.column_x(i) - 1.0 && from < table.column_x(i) + table.columns[i].width).unwrap_or(9);
                assert!(column < 3, "{name}: {from}..{to} is in no column");
                let (left, right) = (table.column_x(column), table.column_x(column) + table.columns[column].width);
                assert!(to <= right + 1.0 && from >= left - 1.0, "{name}: {from}..{to} leaves {left}..{right}");
            }
        }
    }

    #[test]
    fn a_pill_keeps_its_words_inside_the_canvas() {
        for (name, words) in WORDS {
            for (from, to) in inks(|pen, canvas| crate::pill::paint_pill(canvas, pen, words)) {
                assert!(from >= 0.0 && to <= 400.0, "{name}: {from}..{to}");
            }
        }
    }
}

#[test]
fn words_cut_short_in_any_script_are_given_whole_on_pointing() {
    let cut: [(&str, Make); 6] = [
        ("label cut", |w| ww::label(w).cut()),
        ("main_word", |w| ww::main_word(w).on_press(())),
        ("word_button", |w| ww::word_button(w).on_press(())),
        ("choice", |w| ww::choice(vec![w, "Two".into()], 0, "Which", |_| ())),
        ("dropdown", |w| ww::dropdown(vec![w, "Two".into()], Some(0), "Which", |_| ())),
        ("list", |w| ww::list(vec![ww::item(w)], None, |_| ())),
    ];
    for (name, words) in WORDS {
        for (kind, make) in cut {
            let node = ww::column(vec![make(words.to_string()).key("it").width(Length::Fixed(ACROSS))]).padding(10.0);
            let mut ui = Ui::new(node, Theme::new(false, "#fbc711"));
            let mut pix = tiny_skia::Pixmap::new(300, 600).unwrap();
            ui.paint(&mut pix.as_mut(), 1.0);
            let r = ui.boxes().into_iter().find(|b| b.0.starts_with("it")).map(|b| b.1).unwrap();
            ui.pointer = Some((r.x + 12.0, r.y + r.h / 2.0));
            let said = ui.label_of("it").unwrap_or_default();
            assert!(said.contains(words), "{kind} ({name}) says {said:?}");
        }
    }
}
