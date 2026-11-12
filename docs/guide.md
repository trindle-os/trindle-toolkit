# The Trindle Toolkit guide

A short guide for someone writing a first program. The [reference](reference.md) has the full widget table and the details of each part; this is the path through them.

## Building

On Debian 13 or another Debian-based system:

```sh
sudo apt install cargo rustc pkg-config libxkbcommon-dev libwayland-dev
```

Noto Sans, Noto Sans Mono and Noto Sans Arabic are built in; fonts for other scripts are found on the computer when the program runs (for example `fonts-noto-core`). Rust 1.85 or newer is needed.

Add the crate to your `Cargo.toml`, then build as usual:

```toml
[dependencies]
trindle-toolkit = { git = "https://github.com/trindle-os/trindle-toolkit" }
```

```sh
cargo +1.85 build --release
cargo +1.85 build --release --examples
```

A program with no window at all (a clipboard watcher, a background service) can take the plumbing alone, with `default-features = false, features = ["core"]`. See "Building without windows" in [the README](../README.md).

Tests run on a stand-in compositor, so they never open anything on your screen. Unset `DISPLAY`, `WAYLAND_DISPLAY` and `DBUS_SESSION_BUS_ADDRESS` before running them.

## A first window

A program is a type that implements `ui::App`. It has a `Msg` type for what the person can ask for, an `update` that changes the program's state, and a `view` that returns the widgets for that state. `Shown::run` opens the window and runs until it is closed.

```rust
use trindle_toolkit::t;
use trindle_toolkit::ui::{App, Shown};
use trindle_toolkit::widget::{column, label, main_word, Node};
use trindle_toolkit::window::Kind;

/// What the window holds: how many times the button was pressed.
struct Hello {
    pressed: u32,
}

/// What the person can ask for.
#[derive(Clone)]
enum Msg {
    Press,
}

impl App for Hello {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Press => self.pressed += 1,
        }
    }

    fn view(&self) -> Node<Msg> {
        let said = match self.pressed {
            0 => t!("kit-example-hello-none").to_string(),
            n => t!("kit-example-hello-pressed", n = n),
        };
        column(vec![label(said), main_word(t!("kit-example-hello-button")).on_press(Msg::Press)]).padding(24.0).gap(16.0).centred()
    }
}

fn main() {
    let kind = Kind::Window { title: t!("kit-example-hello-title").into(), app_id: "org.example.Hello".into() };
    if let Err(why) = Shown::run(kind, (320, 160), Hello { pressed: 0 }) {
        eprintln!("{why}");
        std::process::exit(1);
    }
}
```

This is `examples/hello.rs`. The window's size is in points, so it is the same size on every screen. The `app_id` is how the desktop knows your program, for its icon and its launcher entry; use a reverse-domain name you own and keep it the same between versions.

`Msg` must be `Clone`. Never run an example you have not read, and run one only on a Wayland desktop.

## The widget table

Every widget is a function in `widget` that returns a `Node<Msg>`. Settings such as `.padding(24.0)`, `.gap(16.0)` and `.on_press(msg)` are methods on the node. These are the ones most programs start with; the [reference](reference.md) lists them all.

Each widget also has a short guide with a picture, its keys and what a screen reader is told: see the [widget guides](widgets/README.md).

| Builder | What it is |
|---|---|
| `column(items)`, `row(items)` | Items one above another, or side by side |
| `card(inner)` | A raised panel holding one item |
| `scroll(key, inner)` | Scrolls what it holds when that is taller |
| `label(text)`, `note(text)`, `heading(text)`, `page_title(text)` | Words, from plain to a page's name |
| `button(icon, tip)` | A round button with an icon; the tip shows on pointing and is what the screen reader says |
| `main_word(words)`, `main_action(icon, words)` | The screen's one main action |
| `remove(icon, tip)` | A button that asks twice before it acts |
| `switch(on, tip, change)` | An on/off switch |
| `slider(value, min, max, step, tip, change)` | A slider |
| `entry(text, tip, change)` | A one-line text box |
| `choice(options, chosen, tip, choose)`, `dropdown(..)` | One answer out of a few, or out of many |
| `list(rows, chosen, choose)` | Rows to choose one of, built with `item(text)` |
| `ring(share, size)`, `spinner()` | Progress, and waiting |
| `drawing(size, tip, paint)` | A space you paint yourself |
| `settings::setting_page(..)` | A whole settings page from groups of rows |

`examples/gallery.rs` puts most of these in one window, each changing something you can see.

Buttons are plain icons, a soft square when pointed at or on, with the words shown on pointing. Use words only where an icon would leave the meaning to be guessed.

## The style sheet

The look is one style sheet, `share/style.css`, built into the crate. Every colour, corner, size and gap is a named value in it, and the widgets read those values; nothing of the look is kept in program code. Change a value and every widget follows.

On Trindle, the person's theme, light or dark, highlight colour and own sheet in `~/.config/trindle` are laid over the built-in one. Anywhere else, the built-in sheet and its yellow highlight are used. Themes live in `share/themes`.

The highlight colour is always the person's to change, so never write a colour of your own into a widget. If you need one in a `drawing`, take it from the sheet.

## The language files

Every word a person reads is written once, in English, in a `.ftl` file under `share/lang/en/`, and asked for by its key:

```
hello-title = Hello
hello-button = Press me
hello-pressed =
    { $n ->
        [one] Pressed once
       *[other] Pressed { $n } times
    }
```

```rust
t!("hello-title")            // plain words
t!("hello-pressed", n = 3)   // with a value
```

Your program builds its English files in with a small `build.rs` (the toolkit's own is the model) and gives them to the toolkit through `Host::words`. A message a language file lacks, a missing file and a broken file all show the English words. A key is never shown to the person.

A translator adds a folder named for the language (`fr`, `pt-BR`) with the same file names, and puts it in `~/.config/trindle/lang/`. The person's own folder wins, so words can be tried without building anything. `share/lang/README.md` is the guide for translators. Write words in sentence case, in UK English, with no em dashes.

The toolkit's own words (menus, the eye beside a password) have keys that start `kit-`. Keep your own keys apart from them.

## The screen reader link

Every window publishes an AccessKit tree (roles, names, values and states) while a screen reader is listening, and costs nothing while none is. Most of it comes from the widgets. A button's `tip` is its name, so every button must have one.

What you add yourself goes on the node:

- `called(name)` names a picture or ring that shows no words.
- `polite()`, `assertive()` and `alert()` make words spoken when they change. Words in the problem tone are told as alerts.
- `progress(share)` makes a `drawing` a progress indicator.
- `reads(..)` lets a `drawing` that paints text, such as a terminal, publish its lines and caret.

A secret entry is always a password box to the reader, and its characters are never published.

For your own tests, turn on the `testing` feature: `Ui::unnamed_controls()` lists every control with no role or no name, and every view a test paints is checked the first time. Run the check before you ship, and listen to the window once with a screen reader.
