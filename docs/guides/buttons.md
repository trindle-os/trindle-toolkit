# Buttons and actions

A button sends a message when pressed. Every kind is one height from the sheet, so rows of mixed buttons line up.

| You want | Use |
|---|---|
| An icon that does one thing | `button(icon, tip)`; `tip` shows on pointing and is what a screen reader says |
| The screen's one main action | `main_word("Save")` or `main_action(icon, "Install")`; one per screen |
| An ordinary worded action | `quiet_action(icon, "New")` |
| A small choice in words | `word_button("Dark")`, with `.on(true)` for the chosen one |
| Something that cannot be undone | `remove(icon, "Remove")`; the first press arms it, the second acts |
| A web address | `link_named("Trindle", "https://trindle.org")` |

`.on_press(msg)` sets what a press sends, `.on(true)` shows it chosen and `.off(true)` greys it, so it cannot be pressed or reached by Tab.

```rust
use trindle_toolkit::widget::{button, column, main_word, quiet_action, remove, row, Node};

#[derive(Clone)]
enum Msg {
    Add,
    Save,
    Delete,
}

fn view() -> Node<Msg> {
    column(vec![
        row(vec![button("list-add-symbolic", "Add").on_press(Msg::Add), quiet_action("document-new-symbolic", "New").on_press(Msg::Add)]).gap(8.0),
        remove("user-trash-symbolic", "Delete").on_press(Msg::Delete),
        main_word("Save").on_press(Msg::Save),
    ]).gap(12.0).padding(24.0)
}
# let _ = view();
```
