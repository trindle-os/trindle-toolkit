//! What a new description costs on long lists: many avatars, and rows inside rows.
use crate::sys::os::thread_time;
use crate::theme::Theme;
use crate::ui::Ui;
use crate::widget::*;
use std::rc::Rc;

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

fn timed(f: impl FnOnce()) -> f64 {
    let t = thread_time();
    f();
    (thread_time() - t).as_secs_f64() * 1e3
}

fn photo() -> Rc<image::DynamicImage> {
    Rc::new(image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(1024, 768, |x, y| image::Rgb([x as u8, y as u8, (x ^ y) as u8]))))
}

fn people(n: usize, pic: Option<&Rc<image::DynamicImage>>) -> Node<()> {
    scroll("s", column((0..n).map(|i| row(vec![
        avatar(pic.cloned(), format!("John Smith {i}"), 40.0),
        label(format!("John Smith {i}")).width(Length::Fill),
    ]).gap(8.0)).collect()).gap(8.0)).height(Length::Fill)
}

fn nested(depth: usize, n: usize) -> Node<()> {
    scroll("s", column((0..n).map(|_| deep(depth)).collect()).gap(8.0)).height(Length::Fill)
}

fn deep(depth: usize) -> Node<()> {
    let mut inner: Node<()> = row(vec![label("Some words that wrap when the row is narrow"), button("list-add", "Add")]).gap(8.0).wraps();
    for _ in 1..depth {
        inner = row(vec![label("More words beside it"), inner]).gap(8.0).wraps();
    }
    inner
}

fn change(make: &dyn Fn() -> Node<()>, n: usize) -> (f64, f64) {
    let (w, h, scale) = (1280u32, 800u32, 2.0f32);
    let mut pix = tiny_skia::Pixmap::new(w, h).unwrap();
    let mut ui: Ui<()> = Ui::new(make(), Theme::chosen());
    ui.paint(&mut pix.as_mut(), scale);
    ui.lay_at(w, h);
    ui.paint(&mut pix.as_mut(), scale);
    let lay = median((0..n).map(|_| { let t = make(); timed(|| { ui.set_view(t); ui.lay_at(w, h); }) }).collect());
    let mut views: Vec<_> = (0..n).map(|_| make()).collect();
    let whole = median((0..n).map(|_| {
        let t = views.pop().unwrap();
        timed(|| { ui.set_view(t); ui.paint(&mut pix.as_mut(), scale); })
    }).collect());
    (lay, whole)
}

#[test]
#[ignore = "a timing run, for a machine with nothing else busy"]
fn measure_long_lists() {
    let pic = photo();
    println!("| Page | Build | Lay out | Lay out and paint |");
    println!("|---|---|---|---|");
    for n in [50, 500] {
        for (name, p) in [("initials", None), ("1024 x 768 photo", Some(&pic))] {
            let build = median((0..9).map(|_| timed(|| { std::hint::black_box(people(n, p)); })).collect());
            let (lay, whole) = change(&|| people(n, p), 9);
            println!("| {n} avatars, {name} | {build:.2} ms | {lay:.2} ms | {whole:.2} ms |");
        }
    }
    for depth in 1..=5 {
        let (lay, whole) = change(&|| nested(depth, 200), 9);
        println!("| 200 rows {depth} deep | | {lay:.2} ms | {whole:.2} ms |");
    }
}
