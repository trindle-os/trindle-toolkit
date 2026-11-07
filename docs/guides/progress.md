# Progress and pictures

- `ring(share, size)`: fills clockwise for a job whose end can be guessed; `.tone(Tone::Accent)` for the highlight, `Tone::Problem` for a failure. A screen reader hears a progress indicator.
- `spinner()`: a turning ring while something loads.
- `avatar(picture, name, size)`: a picture cropped from its middle, or the first letters of the name.
- `image_view(picture, name, size)`: a picture kept in proportion.
- `icon_image(icon, size, tone)`: an icon in the text colour; name it with `.called(..)` if it means something.

Say what is happening in words beside the picture, and use `.polite()` on the words so a screen reader tells each change.

```rust
use trindle_toolkit::widget::{column, label, note, ring, row, spinner, Node, Tone};

fn view(share: f32, failed: bool) -> Node<()> {
    let tone = if failed { Tone::Problem } else { Tone::Accent };
    row(vec![
        ring(share, 56.0).tone(tone),
        column(vec![label("Downloading").polite(), note("A minute left")]).gap(2.0),
        spinner(),
    ]).gap(16.0).padding(24.0)
}
# let _ = view(0.64, false);
```
