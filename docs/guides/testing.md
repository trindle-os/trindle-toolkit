# Testing your program

A program on the Trindle Toolkit can test its window with no desktop running: `trindle_toolkit::testing::Harness` holds your `App` and its window, and drives it as a person would. It needs the `testing` feature in your tests only:

```toml
[dev-dependencies]
trindle-toolkit = { version = "0.9.0-beta.1", features = ["testing"] }
```

`trindle-new my-program` writes a program that already has such a test.

## What the harness does

| Call | What it does |
|---|---|
| `Harness::new(app, (w, h))` | Lays out `app`'s window `w` by `h` points, in the dark look with the usual highlight. |
| `click(name)` | Clicks the middle of the widget a screen reader knows by `name` (a button's words or label, a box's hint). False when no widget has that name. |
| `type_text(text)` | Types `text` where the keyboard is, a letter at a time. |
| `key(sym, mods)` | Presses one key, such as `Keysym::Tab` or `Keysym::Return`. |
| `send(&input)` | Gives the window any `Input`: pointer, keys, touch, scrolling. |
| `text()` | Every name and value the window tells a screen reader, in order: what a person can read. |
| `picture(scale)` | The window painted at `scale` pixels to the point, as a `tiny_skia::Pixmap`. |
| `save_picture(scale, path)` | The same, saved as a PNG file. |
| `app()` | Your program, to check what it holds. |
| `ui()` | The window itself, for the checks below. |

Each message the window sends goes to your `update`, and the window then shows your `view` again, exactly as when the program runs.

Because clicks go by name, a test finds a widget the way a screen reader does. A button with no name cannot be clicked by a test either.

## An example

```rust
use trindle_toolkit::testing::Harness;

#[test]
fn the_button_counts_its_presses() {
    let mut h = Harness::new(Window { pressed: 0 }, (320, 160));
    assert!(h.click("Press me"));
    assert_eq!(h.app().pressed, 1);
    assert!(h.text().contains(&"Pressed once".to_string()));
}
```

## Checks worth adding

- `h.ui().unnamed_controls()` is empty when every control has a name and a role a screen reader can use.
- `h.ui().squeezed_or_touching()` is empty when no button or switch is squeezed or overlaps another.
- `h.picture(2.0)` against a picture you keep shows a change of look at once. Paint at more than one scale.

## Your own words in a test

A test has no window to hand the toolkit your words, so run it inside `host::with` and `lang::with_catalogue`, as the template does:

```rust
fn with_words<R>(during: impl FnOnce() -> R) -> R {
    host::with(&Words, || lang::with_catalogue(Catalogue::new(Bundle::default(), lang::english()), during))
}
```

## Looking while it runs

In a running program, Ctrl+Shift+F12 turns on the inspector: point at a widget to see its kind and key, its size and place, its state, what the screen reader is told and the style sheet values that shape it. Ctrl+Shift+F11 shows how long each frame took to paint, the memory held and how much of the window was painted again.
