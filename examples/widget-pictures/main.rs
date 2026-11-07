//! Paints the picture for each guide in docs/widgets from the guide's own code, with the built-in fonts; run it with tools/widget-pictures.sh.

mod guides;

use std::path::Path;

use trindle_toolkit::input::{Keysym, Mods};
use trindle_toolkit::testing::Harness;
use trindle_toolkit::ui::App;
use trindle_toolkit::widget::Node;

struct Still<M>(fn() -> Node<M>);

impl<M> Clone for Still<M> {
    fn clone(&self) -> Self {
        Still(self.0)
    }
}

impl<M: Clone> App for Still<M> {
    type Msg = M;

    fn update(&mut self, _: M) {}

    fn view(&self) -> Node<M> {
        (self.0)()
    }
}

const MARGIN: u32 = 16;
const SCALE: f32 = 2.0;

static FAILED: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

fn snap<M: Clone + 'static>(only: &[String], name: &str, (width, height): (u32, u32), keys: &str, make: fn() -> Node<M>) {
    if !only.is_empty() && !only.iter().any(|o| o == name) {
        return;
    }
    eprintln!("{name}");
    if std::panic::catch_unwind(|| paint(name, width, height, keys, make)).is_err() {
        FAILED.lock().expect("one thread").push(name.to_string());
    }
}

fn paint<M: Clone + 'static>(name: &str, width: u32, height: u32, keys: &str, make: fn() -> Node<M>) {
    let tall = match height {
        0 => Harness::new(Still(make), (width, 100)).ui().natural_height(width as f32).ceil() as u32 + MARGIN,
        fixed => fixed,
    };
    let mut window = Harness::new(Still(make), (width, tall));
    for key in keys.split(',').filter(|k| !k.is_empty()) {
        window.key(press(key), Mods::default());
    }
    let pix = window.picture(SCALE);
    let (w, h) = (pix.width(), pix.height());
    let data = pix.data();
    let at = |x: u32, y: u32| &data[((y * w + x) * 4) as usize..][..4];
    let back = at(0, 0);
    let differs = |x: u32, y: u32| at(x, y).iter().zip(back).any(|(a, b)| a.abs_diff(*b) > 2);
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for y in 0..h {
        for x in 0..w {
            if differs(x, y) {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
        }
    }
    if x1 < x0 {
        eprintln!("{name}: nothing was drawn");
        return;
    }
    let (x0, y0) = (x0.saturating_sub(MARGIN), y0.saturating_sub(MARGIN));
    let (x1, y1) = ((x1 + MARGIN + 1).min(w), (y1 + MARGIN + 1).min(h));
    let rect = tiny_skia::IntRect::from_xywh(x0 as i32, y0 as i32, x1 - x0, y1 - y0).expect("a picture has a size");
    let cut = pix.clone_rect(rect).expect("the cut is inside the picture");
    let rgba: Vec<u8> = cut.pixels().iter().flat_map(|p| {
        let c = p.demultiply();
        [c.red(), c.green(), c.blue(), c.alpha()]
    }).collect();
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/images/widgets").join(format!("{name}.png"));
    let out = std::fs::File::create(&file).expect("the pictures folder exists");
    let png = image::codecs::png::PngEncoder::new_with_quality(std::io::BufWriter::new(out), image::codecs::png::CompressionType::Best, image::codecs::png::FilterType::Adaptive);
    image::ImageEncoder::write_image(png, &rgba, cut.width(), cut.height(), image::ExtendedColorType::Rgba8).expect("the picture is saved");
    println!("{name}: {}x{}", cut.width(), cut.height());
}

fn press(name: &str) -> Keysym {
    match name {
        "Tab" => Keysym::Tab,
        "Menu" => Keysym::Menu,
        "Down" => Keysym::Down,
        "Return" => Keysym::Return,
        other => panic!("the picture tool does not know the key {other}"),
    }
}

fn main() {
    let only: Vec<String> = std::env::args().skip(1).collect();
    if !only.iter().any(|name| name == "emoji-picker") {
        trindle_toolkit::text::built_in_fonts_only();
    }
    guides::all(&only);
    let failed = FAILED.lock().expect("one thread");
    if !failed.is_empty() {
        eprintln!("no picture for: {}", failed.join(", "));
        std::process::exit(1);
    }
}
