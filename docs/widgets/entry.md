# Entry

`entry` is a one-line text box. Your state holds the text and every edit sends the whole new text. The tip shows faintly while the box is empty and is its name. `.secret()` makes it a password box (give it a `.key(..)` too, which the eye is kept by): dots in place of letters, nothing in it can be copied, and an eye at its right end that shows what is typed while it is on.

![An empty entry showing its tip, a filled one, a password box and a switched-off one](../images/widgets/entry.png)

**Keys:** Left, Right, Home and End move the caret, Backspace and Delete remove, Enter sends `.on_submit`, Ctrl+A selects all, Ctrl+C, Ctrl+X and Ctrl+V copy, cut and paste, Ctrl+Z undoes and Ctrl+Y or Ctrl+Shift+Z redoes.

**The screen reader hears:** a text box called by the tip, holding the text. A password box is told as a password box with dots, never the letters, even while the eye shows them, and the eye is a button of its own.

The `.on_submit`, `.glow` and refused-value options are in [typing](../guides/typing.md). For many lines see `editing::area::TextArea`.

<!-- picture width 320 -->
```rust
use trindle_toolkit::widget::{column, entry, form_label, Node};

#[derive(Clone)]
enum Msg {
    Name(String),
    Town(String),
    Password(String),
}

fn view() -> Node<Msg> {
    column(vec![
        form_label("Empty"),
        entry("", "Your name", Msg::Name),
        form_label("Filled"),
        entry("John Smith", "Your name", Msg::Name),
        form_label("Password"),
        entry("sunflower", "Password", Msg::Password).secret().key("password"),
        form_label("Switched off"),
        entry("Mildenhall", "Town", Msg::Town).off(true),
    ]).gap(8.0).padding(24.0)
}
# let _ = view();
```
