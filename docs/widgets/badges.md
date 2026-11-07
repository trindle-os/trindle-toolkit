# Badges and avatar groups

`badge_count` is a small box in the highlight that holds a count, and shows `99+` from 100. `avatar_group` puts up to a set number of avatars in a row, then a `+N` box for the rest. Both are drawn in the sheet's colours.

![Count badges for 3, 42 and 250, and a group of three avatars with a plus two box](../images/widgets/badges.png)

**The screen reader hears:** the name you give a badge, such as "3 unread messages"; for a group, each shown person's name, then "2 more people" for the box.

For the avatars themselves, see [image and avatar](image-and-avatar.md).

<!-- picture width 360 -->
```rust
use trindle_toolkit::badges::{avatar_group, badge_count};
use trindle_toolkit::widget::{column, label, row, Node};

fn view() -> Node<()> {
    let people = ["Ann Lee", "John Smith", "Mary Jones", "Sam Taylor", "Priya Shah"]
        .iter()
        .map(|name| (None, name.to_string()))
        .collect();
    column(vec![
        row(vec![
            label("Inbox"),
            badge_count(3, "3 unread messages"),
            label("Drafts"),
            badge_count(42, "42 drafts"),
            label("Spam"),
            badge_count(250, "More than 99 spam messages"),
        ]).gap(12.0),
        row(vec![
            avatar_group(people, 36.0, 3),
            label("Five people, three shown"),
        ]).gap(16.0),
    ]).gap(20.0).padding(24.0)
}
# let _ = view();
```
