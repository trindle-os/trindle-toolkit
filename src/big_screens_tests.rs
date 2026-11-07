//! Whole and partial repaint times at big screen sizes; the processor path's figures for MEASUREMENTS.md.
use crate::input::Input;
use crate::sys::os::thread_time;
use crate::theme::Theme;
use crate::ui::Ui;
use crate::widget::*;
use std::time::{Duration, Instant};

fn card_of(i: usize) -> Node<()> {
    card(column(vec![
        heading(format!("Card number {i}")),
        label("A line of ordinary words to show how text costs, and a second sentence after it."),
        row(vec![button("list-add", "Add"), button("edit-delete", "Remove"), switch(i % 2 == 0, "Notices", |_| ())]).gap(8.0),
        slider(0.4, 0.0, 1.0, 0.1, "Volume", |_| ()),
        entry("John Smith", "Name", |_| ()),
    ]).gap(8.0))
}

pub(crate) fn page(w: u32, h: u32, scale: f32, extra_rows: usize) -> Node<()> {
    let cols = ((w as f32 / scale / 320.0) as usize).max(1);
    let rows = ((h as f32 / scale / 230.0).ceil() as usize).max(1) + extra_rows;
    let mut lines = vec![page_title("Every control")];
    let mut n = 0;
    for _ in 0..rows {
        lines.push(row((0..cols).map(|_| { n += 1; card_of(n).width(Length::Fill) }).collect()).gap(16.0));
    }
    column(lines).gap(16.0).padding(24.0)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

fn timed(mut f: impl FnMut()) -> f64 {
    let t = thread_time();
    f();
    (thread_time() - t).as_secs_f64() * 1e3
}

#[test]
#[ignore = "a timing run, for a machine with nothing else busy"]
fn measure_big_screens() {
    let (n, scale) = (40, 2.0f32);
    println!("| Window | Whole paint | Clear, swap, opacity check | Scroll step and paint | Spinner part | One core at 60 a second |");
    println!("|---|---|---|---|---|---|");
    for (w, h) in [(1280u32, 800u32), (1920, 1080), (2256, 1504), (3840, 2160)] {
        let mut pix = tiny_skia::Pixmap::new(w, h).unwrap();
        let mut ui: Ui<()> = Ui::new(page(w, h, scale, 0), Theme::chosen());
        ui.paint(&mut pix.as_mut(), scale);
        ui.lay_at(w, h);
        ui.paint(&mut pix.as_mut(), scale);
        let whole = median((0..n).map(|_| timed(|| ui.paint(&mut pix.as_mut(), scale))).collect());
        let around = median((0..n).map(|_| timed(|| {
            let d = pix.data_mut();
            d.fill(0);
            d.chunks_exact_mut(4).for_each(|p| p.swap(0, 2));
            std::hint::black_box(d.chunks_exact(4).all(|p| p[3] == 255));
        })).collect());

        let mut su: Ui<()> = Ui::new(scroll("s", page(w, h, scale, 8)).height(Length::Fill), Theme::chosen());
        su.paint(&mut pix.as_mut(), scale);
        su.lay_at(w, h);
        su.paint(&mut pix.as_mut(), scale);
        su.input(&Input::PointerIn { x: 200.0, y: 200.0 }, Instant::now());
        let step = median((0..n).map(|i| timed(|| {
            let y = if (i / 20) % 2 == 0 { 12.0 } else { -12.0 };
            su.input(&Input::Scroll { x: 0.0, y, steps: false }, Instant::now());
            su.paint(&mut pix.as_mut(), scale);
        })).collect());

        let mut sp: Ui<()> = Ui::new(column(vec![spinner(), page(w, h, scale, 0)]), Theme::chosen());
        sp.paint(&mut pix.as_mut(), scale);
        sp.lay_at(w, h);
        sp.paint(&mut pix.as_mut(), scale);
        let part = median((0..n).map(|_| {
            std::thread::sleep(Duration::from_millis(17));
            timed(|| { sp.paint_live(&mut pix.as_mut(), scale); })
        }).collect());

        println!("| {w} x {h} | {whole:.1} ms | {around:.1} ms | {step:.1} ms | {part:.2} ms | {:.0} per cent |",
            100.0 * (whole + around) / 16.67);
    }
}
