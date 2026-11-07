# Rating

`rating` is a row of stars the person presses to give a mark out of `max`. Pressing star `n` sends `n`, and pressing the star that is already the mark sends 0, which clears it. Your state keeps the mark.

![Two ratings, one with four of five stars and one with none](../images/widgets/rating.png)

**Keys:** Tab reaches it. Left and Down take a star away, Right and Up add one, Home clears the mark and End fills every star.

**The screen reader hears:** the tip, then the mark, such as "Service, 4 of 5 stars".

<!-- picture width 320 -->
```rust
use trindle_toolkit::rating::rating;
use trindle_toolkit::widget::{column, label, row, Node};

#[derive(Clone)]
enum Msg {
    Service(u8),
    Food(u8),
}

fn view() -> Node<Msg> {
    column(vec![
        row(vec![label("Service"), rating(4, 5, "Service", Msg::Service)]).gap(16.0),
        row(vec![label("Food"), rating(0, 5, "Food", Msg::Food)]).gap(16.0),
    ]).gap(16.0).padding(24.0)
}
# let _ = view();
```
