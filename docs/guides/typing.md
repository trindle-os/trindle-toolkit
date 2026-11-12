# Typing: entries and text areas

`entry(text, tip, change)` is a one-line box, or one of many lines with `.lines(N)`. Your state holds the text; each edit sends the whole new text through `change`. `tip` is the hint shown while it is empty and the name a screen reader says.

- `.on_submit(msg)` sends `msg` on Enter (Ctrl+Enter in a box of many lines).
- `.lines(n)` makes the box at least `n` lines tall: words wrap at its width, Enter starts a new line, Up and Down move by line, and the text scrolls inside the box once it is taller than the room.
- `.secret()` is a password box: dots on screen, never published to a screen reader, with an eye to show it.
- `.glow(n)` makes the box glow once in the highlight when `n` changes, which is how a saved value is shown. No toast.
- A refused value is shown by a `note(..).tone(Tone::Problem)` under the box.
- `.off(true)` greys it.

For a whole document use `editing::area::TextArea` (undo, wrapping, scroll bar, clipboard, right-click menu).

```rust
use trindle_toolkit::widget::{column, entry, note, Length, Node, Tone};

#[derive(Clone)]
enum Msg {
    Name(String),
    Port(String),
    Send,
}

fn view(name: &str, port: &str) -> Node<Msg> {
    let bad = port.parse::<u16>().map_or(true, |p| p < 1024);
    let mut items = vec![
        entry(name, "Name", Msg::Name).on_submit(Msg::Send).width(Length::Fixed(240.0)),
        entry(port, "Port", Msg::Port).width(Length::Fixed(240.0)),
    ];
    if bad {
        items.push(note("Choose a port from 1024 to 65535.").tone(Tone::Problem));
    }
    column(items).gap(8.0).padding(24.0)
}
# let _ = view("John Smith", "99999");
```
