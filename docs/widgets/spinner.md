# Spinner

`spinner()` is a ring turning round while something loads and its end cannot be guessed. Its size comes from the style sheet (`--trindle-spinner`). It turns on screen; a picture can only show one frame of it.

![A spinner beside the words Loading your files, caught at one moment of its turn](../images/widgets/spinner.png)

**The screen reader hears:** a progress indicator called "Loading", marked busy.

When the end can be guessed, use [ring](ring.md) or a progress bar instead. See [progress](../guides/progress.md).

<!-- picture width 280 -->
```rust
use trindle_toolkit::widget::{label, row, spinner, Node};

fn view() -> Node<()> {
    row(vec![spinner(), label("Loading your files")]).gap(16.0).padding(24.0)
}
# let _ = view();
```
