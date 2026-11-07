# Switches, ticks, choices and sliders

Each takes its current value from your state and sends the new one when the person changes it; nothing keeps its own copy.

| You want | Use |
|---|---|
| On or off, applied at once | `switch(on, tip, change)` |
| A box to tick, as in a form | `tick(on, tip, change)`; Space toggles, a screen reader hears a check box |
| One of two to four short answers | `choice(options, chosen, tip, choose)` |
| One of many | `dropdown(options, Some(chosen), tip, choose)`; its list floats and moves nothing |
| Either, whichever fits the answers | `one_of(options, chosen, tip, choose)` |
| A number in a range | `slider(value, min, max, step, tip, change)` |

`change` and `choose` are functions that turn the new value into your message; a tuple variant such as `Msg::Size` works as it is.

```rust
use trindle_toolkit::widget::{choice, column, dropdown, slider, switch, tick, Length, Node};

#[derive(Clone)]
enum Msg {
    Notices(bool),
    Agree(bool),
    Size(usize),
    Town(usize),
    Volume(f32),
}

fn view(notices: bool, agreed: bool, size: usize, town: Option<usize>, volume: f32) -> Node<Msg> {
    let sizes = vec!["Small".to_string(), "Medium".to_string(), "Large".to_string()];
    let towns = vec!["Ely".to_string(), "Thetford".to_string(), "Brandon".to_string()];
    column(vec![
        switch(notices, "Notices", Msg::Notices),
        tick(agreed, "I agree", Msg::Agree),
        choice(sizes, size, "Size", Msg::Size),
        dropdown(towns, town, "Town", Msg::Town),
        slider(volume, 0.0, 100.0, 1.0, "Volume", Msg::Volume).width(Length::Fixed(200.0)),
    ]).gap(12.0).padding(24.0)
}
# let _ = view(true, false, 1, None, 40.0);
```

## Dates, files and fonts

These keep a small state in your program: show it with `view`, give each step back to `update`. A date grid can show ISO week numbers down its left. The file and font choosers are whole pages shown in place of your page, never over it; `update` gives back the answer once the person has made it.

```rust
use trindle_toolkit::file_page::{FilePage, FileStep, Purpose};
use trindle_toolkit::font_page::{FontPage, FontStep};
use trindle_toolkit::text::FontFamily;
use trindle_toolkit::when::{Date, DateBox, Pick};
use trindle_toolkit::widget::Node;

#[derive(Clone)]
enum Msg {
    Day(Pick),
    Files(FileStep),
    Font(FontStep),
    Back,
}

struct App {
    dates: DateBox,
    files: Option<FilePage>,
    font: Option<FontPage>,
    families: Vec<FontFamily>,
}

fn view(app: &App) -> Node<Msg> {
    if let Some(files) = &app.files {
        return files.view("Open a report", Msg::Files, Msg::Back);
    }
    if let Some(font) = &app.font {
        return font.view(&app.families, "Choose a font", Msg::Font, Msg::Back);
    }
    app.dates.view(Msg::Day)
}

fn update(app: &mut App, msg: Msg) {
    match msg {
        Msg::Day(pick) => app.dates.update(pick),
        Msg::Files(step) => {
            if let Some(path) = app.files.as_mut().and_then(|f| f.update(step)) {
                println!("open {}", path.display());
                app.files = None;
            }
        }
        Msg::Font(step) => {
            if let Some((family, size)) = app.font.as_mut().and_then(|f| f.update(step)) {
                println!("{family} at {size}");
                app.font = None;
            }
        }
        Msg::Back => (app.files, app.font) = (None, None),
    }
}

let today = Date::new(2026, 10, 30).unwrap();
let mut app = App {
    dates: DateBox { weeks: true, ..DateBox::new(None, today) },
    files: Some(FilePage::new(Purpose::Open, std::env::temp_dir())),
    font: Some(FontPage::new(None, 12.0)),
    families: Vec::new(),
};
let _ = view(&app);
update(&mut app, Msg::Back);
update(&mut app, Msg::Day(Pick::Month(1)));
let _ = view(&app);
```
