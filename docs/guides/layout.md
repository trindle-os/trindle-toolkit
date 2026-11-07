# Layout and containers

| You want | Use |
|---|---|
| Items one above another, or side by side | `column(items)`, `row(items)` with `.gap(..)` and `.padding(..)` |
| A raised panel | `card(inner)` |
| Empty room that takes what is left | `space()` |
| Tiles that fill the width in bands | `grid(min, items)`, `even_grid(min, items)`, `bento(min, gap, items)` |
| Items over one another | `layers(items)` |
| A page and its words | `page_title`, `lead`, `heading`, `section`, `form_label`, `note` |
| A settings page | `settings::setting_page(key, title, lead, groups)` |

Width and height are `Length::Fill`, `Length::Shrink` or `Length::Fixed(points)`. Units are points; the window's scale makes them pixels. Nothing touches: keep a gap between neighbours and padding inside a box. Looks come from the style sheet, never from code.

```rust
use trindle_toolkit::widget::{card, column, even_grid, heading, label, note, page_title, space, Length, Node};

fn view() -> Node<()> {
    let tile = |name: &str| card(column(vec![heading(name), note("Quiet words.")]).gap(6.0)).width(Length::Fill);
    column(vec![
        page_title("Overview"),
        even_grid(240.0, vec![tile("One"), tile("Two"), tile("Three")]),
        label("Below the tiles."),
        space(),
    ]).gap(16.0).padding(24.0)
}
# let _ = view();
```
