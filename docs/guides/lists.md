# Lists and scrolling

`list(rows, chosen, choose)` shows rows made with `item(text)`; add `.icon(name)`, `.detail(text)` and `.folder()`. The arrow keys, Home, End and a press choose a row.

`scroll(key, inner)` scrolls what it holds when that is taller than its room; `scroll_x(key, inner)` does the same sideways, with Shift and the wheel or a sideways wheel. `key` names the area so its place survives a new view. Give a scroll area a height (`.height(Length::Fixed(..))`, or a place in a column that takes what is left).

For thousands of rows, with sorting and resizing, use `table::Table`, which paints only the rows in sight.

```rust
use trindle_toolkit::widget::{column, item, list, scroll, Length, Node};

#[derive(Clone)]
enum Msg {
    Chosen(usize),
}

fn view(chosen: Option<usize>) -> Node<Msg> {
    let rows = vec![
        item("Documents").icon("folder-symbolic").folder().detail("12 items"),
        item("holiday.mp4").icon("video-x-generic-symbolic").detail("1.2 GB"),
        item("notes.txt").icon("text-x-generic-symbolic").detail("4 KB"),
    ];
    column(vec![scroll("files", list(rows, chosen, Msg::Chosen)).height(Length::Fixed(200.0))]).padding(24.0)
}
# let _ = view(Some(0));
```
