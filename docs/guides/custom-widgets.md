# Your own kind of widget

A program adds a widget kind without touching the toolkit: `drawing(size, tip, paint)` is a space you paint, and methods on the node take what the person does to it. A function that returns a `Node<Msg>` is the widget; your state is the only state, as with every other widget.

| The drawing needs | Use |
|---|---|
| To paint | the closure: it gets the canvas and a `Pen` (theme, fonts, icons, scale); `tiles::Sketch::fit` gives a view box and the theme's colours |
| The pointer | `on_pointer`, `on_pointer_when`, `on_pointer_mods`; `Pointer::Down`, `Move`, `Up`, `Hover`, `Leave`, `Menu` in points from its top left |
| The keyboard | `on_keys(\|input\| Option<Msg>)`; having it makes the drawing reachable by Tab |
| To be a button for a screen reader | `.pressable(msg)`; it hears a button named by `tip` and can click it |
| The wheel | `on_scroll` |
| A pointer shape | `cursor_with` |
| A right-click menu | `menu` |
| Words for a screen reader | `tip` is its name; `.progress(..)`, `.reads(..)`, `.called(..)` and the live-region calls add more |
| A role for a screen reader | `.role(Role::..)` from `trindle_toolkit::role`: `Button`, `CheckBox(on)`, `Switch(on)`, `Slider { value, min, max }`, `SpinButton { .. }`, `Meter { .. }` (a gauge or level), `Image` or `Group` |
| A value for a screen reader | the number in the role, and `.value("3 of 5 stars")` for the words read |

A paint closure must read the theme from the `Pen` (never a literal colour or corner), so the widget follows light, dark, bold and the person's highlight.

A drawing is an image unless told otherwise: `.pressable(msg)` makes it a button, `.progress(..)` a progress indicator. `.role(..)` says what it is in place of those. A `Slider` or `SpinButton` whose drawing takes keys can be stepped by the screen reader: its Increment sends the drawing Up and its Decrement sends Down, so take those keys as well as your own. A `CheckBox` or `Switch` given `.pressable(msg)` is clicked by the reader as by a press.

```rust
use trindle_toolkit::input::{Input, Keysym};
use trindle_toolkit::role::Role;
use trindle_toolkit::tiles::Sketch;
use trindle_toolkit::widget::{drawing, Node, Pointer};

#[derive(Clone)]
enum Msg {
    Rate(u32),
}

/// Five stars; a press sets the rating, and Left and Right, or Down and
/// Up, change it. A screen reader hears a slider at "3 of 5 stars".
fn stars(value: u32) -> Node<Msg> {
    drawing((100.0, 20.0), "Rating", move |canvas, pen| {
        let mut s = Sketch::fit(canvas, pen, (100.0, 20.0));
        for i in 0..5 {
            let colour = if i < value { s.highlight() } else { s.track() };
            s.circle((10.0 + 20.0 * i as f32, 10.0), 8.0, colour);
        }
    })
    .on_pointer_when(|p| match p {
        Pointer::Down(x, _) => Some(Msg::Rate((x / 20.0) as u32 + 1)),
        _ => None,
    })
    .on_keys(move |input| match input {
        Input::Key { sym: Keysym::Right | Keysym::Up, .. } => Some(Msg::Rate((value + 1).min(5))),
        Input::Key { sym: Keysym::Left | Keysym::Down, .. } => Some(Msg::Rate(value.saturating_sub(1))),
        _ => None,
    })
    .role(Role::Slider { value: f64::from(value), min: 0.0, max: 5.0 })
    .value(format!("{value} of 5 stars"))
}
# let _ = stars(3);
```
