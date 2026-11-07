//! A page of emoji to pick from, in groups, found by their names in the person's language.

use super::widget::{column, entry, even_grid, note, page_title, quiet_action, section, Node};
use crate::t;

type Group = (&'static str, Vec<(&'static str, &'static str)>);

fn groups() -> Vec<Group> {
    living().into_iter().chain(things()).collect()
}

fn living() -> [Group; 4] {
    [
        (t!("kit-emoji-faces"), vec![
            ("😀", t!("kit-emoji-grin")), ("😂", t!("kit-emoji-tears-of-joy")), ("🙂", t!("kit-emoji-smile")), ("😉", t!("kit-emoji-wink")),
            ("😍", t!("kit-emoji-heart-eyes")), ("🤔", t!("kit-emoji-thinking")), ("😢", t!("kit-emoji-crying")), ("😮", t!("kit-emoji-surprised")),
            ("😊", t!("kit-emoji-blush")), ("😇", t!("kit-emoji-halo")), ("😎", t!("kit-emoji-cool")), ("😴", t!("kit-emoji-sleeping")),
            ("😭", t!("kit-emoji-sobbing")), ("😡", t!("kit-emoji-angry")), ("😱", t!("kit-emoji-scream")), ("🙄", t!("kit-emoji-eye-roll")),
            ("😅", t!("kit-emoji-relieved")), ("🤣", t!("kit-emoji-rolling-laugh")), ("😘", t!("kit-emoji-blowing-a-kiss")),
            ("😐", t!("kit-emoji-straight-face")), ("😳", t!("kit-emoji-flushed")), ("🤗", t!("kit-emoji-hug")), ("🤒", t!("kit-emoji-unwell")),
            ("🥳", t!("kit-emoji-celebrating")),
        ]),
        (t!("kit-emoji-hands"), vec![
            ("👍", t!("kit-emoji-thumbs-up")), ("👎", t!("kit-emoji-thumbs-down")), ("👏", t!("kit-emoji-clapping")), ("🙏", t!("kit-emoji-thank-you")),
            ("👋", t!("kit-emoji-waving")), ("✌️", t!("kit-emoji-victory")), ("👌", t!("kit-emoji-ok")), ("🤞", t!("kit-emoji-crossed-fingers")),
            ("💪", t!("kit-emoji-strong")), ("👆", t!("kit-emoji-pointing-up")), ("👇", t!("kit-emoji-pointing-down")),
            ("👈", t!("kit-emoji-pointing-left")), ("👉", t!("kit-emoji-pointing-right")), ("✋", t!("kit-emoji-raised-hand")),
            ("🤝", t!("kit-emoji-handshake")), ("✊", t!("kit-emoji-fist")), ("🙌", t!("kit-emoji-raised-hands")), ("🤙", t!("kit-emoji-call-me")),
            ("✍️", t!("kit-emoji-writing")), ("🤷", t!("kit-emoji-shrug")),
        ]),
        (t!("kit-emoji-animals"), vec![
            ("🐶", t!("kit-emoji-dog")), ("🐱", t!("kit-emoji-cat")), ("🐭", t!("kit-emoji-mouse")), ("🐰", t!("kit-emoji-rabbit")),
            ("🦊", t!("kit-emoji-fox")), ("🐻", t!("kit-emoji-bear")), ("🐼", t!("kit-emoji-panda")), ("🐯", t!("kit-emoji-tiger")),
            ("🦁", t!("kit-emoji-lion")), ("🐮", t!("kit-emoji-cow")), ("🐷", t!("kit-emoji-pig")), ("🐸", t!("kit-emoji-frog")),
            ("🐵", t!("kit-emoji-monkey")), ("🐔", t!("kit-emoji-chicken")), ("🐧", t!("kit-emoji-penguin")), ("🐦", t!("kit-emoji-bird")),
            ("🦆", t!("kit-emoji-duck")), ("🦉", t!("kit-emoji-owl")), ("🐴", t!("kit-emoji-horse")), ("🐝", t!("kit-emoji-bee")),
            ("🦋", t!("kit-emoji-butterfly")), ("🐢", t!("kit-emoji-turtle")), ("🐟", t!("kit-emoji-fish")), ("🐬", t!("kit-emoji-dolphin")),
        ]),
        (t!("kit-emoji-nature"), vec![
            ("🌸", t!("kit-emoji-blossom")), ("🌳", t!("kit-emoji-tree")), ("☀️", t!("kit-emoji-sun")), ("🌧️", t!("kit-emoji-rain")),
            ("🌹", t!("kit-emoji-rose")), ("🌻", t!("kit-emoji-sunflower")), ("🌷", t!("kit-emoji-tulip")), ("🌲", t!("kit-emoji-evergreen")),
            ("🍀", t!("kit-emoji-clover")), ("🍁", t!("kit-emoji-maple-leaf")), ("🌵", t!("kit-emoji-cactus")), ("🌙", t!("kit-emoji-moon")),
            ("🌈", t!("kit-emoji-rainbow")), ("☁️", t!("kit-emoji-cloud")), ("❄️", t!("kit-emoji-snowflake")), ("⚡", t!("kit-emoji-lightning")),
            ("🌊", t!("kit-emoji-wave")), ("🌍", t!("kit-emoji-earth")), ("⛰️", t!("kit-emoji-mountain")), ("🌴", t!("kit-emoji-palm-tree")),
        ]),
    ]
}

fn things() -> [Group; 4] {
    [
        (t!("kit-emoji-food"), vec![
            ("🍎", t!("kit-emoji-apple")), ("🍞", t!("kit-emoji-bread")), ("🍰", t!("kit-emoji-cake")), ("☕", t!("kit-emoji-tea")),
            ("🍌", t!("kit-emoji-banana")), ("🍊", t!("kit-emoji-orange")), ("🍓", t!("kit-emoji-strawberry")), ("🍇", t!("kit-emoji-grapes")),
            ("🍉", t!("kit-emoji-watermelon")), ("🍋", t!("kit-emoji-lemon")), ("🥕", t!("kit-emoji-carrot")), ("🌽", t!("kit-emoji-corn")),
            ("🍕", t!("kit-emoji-pizza")), ("🍔", t!("kit-emoji-burger")), ("🍟", t!("kit-emoji-chips")), ("🍝", t!("kit-emoji-pasta")),
            ("🍳", t!("kit-emoji-fried-egg")), ("🧀", t!("kit-emoji-cheese")), ("🍪", t!("kit-emoji-biscuit")), ("🍫", t!("kit-emoji-chocolate")),
            ("🍦", t!("kit-emoji-ice-cream")), ("🍺", t!("kit-emoji-beer")), ("🍷", t!("kit-emoji-wine")), ("🥛", t!("kit-emoji-milk")),
        ]),
        (t!("kit-emoji-travel"), vec![
            ("🚗", t!("kit-emoji-car")), ("🚌", t!("kit-emoji-bus")), ("🚲", t!("kit-emoji-bicycle")), ("🚂", t!("kit-emoji-train")),
            ("✈️", t!("kit-emoji-aeroplane")), ("🚀", t!("kit-emoji-rocket")), ("🚢", t!("kit-emoji-ship")), ("⛵", t!("kit-emoji-sailing-boat")),
            ("🚁", t!("kit-emoji-helicopter")), ("🏠", t!("kit-emoji-house")), ("🏢", t!("kit-emoji-office")), ("🏥", t!("kit-emoji-hospital")),
            ("🏫", t!("kit-emoji-school")), ("⛪", t!("kit-emoji-church")), ("🏖️", t!("kit-emoji-beach")), ("🗼", t!("kit-emoji-tower")),
            ("🌉", t!("kit-emoji-bridge")), ("⛽", t!("kit-emoji-petrol-pump")), ("🚦", t!("kit-emoji-traffic-light")), ("🗺️", t!("kit-emoji-map")),
        ]),
        (t!("kit-emoji-objects"), vec![
            ("📱", t!("kit-emoji-phone")), ("💻", t!("kit-emoji-laptop")), ("⌨️", t!("kit-emoji-keyboard")), ("📷", t!("kit-emoji-camera")),
            ("💡", t!("kit-emoji-light-bulb")), ("🔑", t!("kit-emoji-key")), ("🔒", t!("kit-emoji-lock")), ("📚", t!("kit-emoji-books")),
            ("📝", t!("kit-emoji-memo")), ("✏️", t!("kit-emoji-pencil")), ("📎", t!("kit-emoji-paperclip")), ("📅", t!("kit-emoji-calendar")),
            ("📧", t!("kit-emoji-email")), ("🎁", t!("kit-emoji-gift")), ("🎵", t!("kit-emoji-music-note")), ("🎧", t!("kit-emoji-headphones")),
            ("⏰", t!("kit-emoji-alarm-clock")), ("🔔", t!("kit-emoji-bell")), ("🔧", t!("kit-emoji-spanner")), ("💰", t!("kit-emoji-money-bag")),
            ("🛒", t!("kit-emoji-shopping-trolley")), ("⚽", t!("kit-emoji-football")),
        ]),
        (t!("kit-emoji-symbols"), vec![
            ("❤️", t!("kit-emoji-heart")), ("⭐", t!("kit-emoji-star")), ("✅", t!("kit-emoji-done")), ("❓", t!("kit-emoji-question")),
            ("🎉", t!("kit-emoji-party")), ("🔥", t!("kit-emoji-fire")), ("❌", t!("kit-emoji-cross")), ("❗", t!("kit-emoji-exclamation")),
            ("⚠️", t!("kit-emoji-warning")), ("💯", t!("kit-emoji-hundred")), ("💔", t!("kit-emoji-broken-heart")), ("💙", t!("kit-emoji-blue-heart")),
            ("💚", t!("kit-emoji-green-heart")), ("✨", t!("kit-emoji-sparkles")), ("➕", t!("kit-emoji-plus")), ("➖", t!("kit-emoji-minus")),
            ("➡️", t!("kit-emoji-right-arrow")), ("⬅️", t!("kit-emoji-left-arrow")), ("🔄", t!("kit-emoji-refresh")), ("🚫", t!("kit-emoji-forbidden")),
            ("♻️", t!("kit-emoji-recycling")), ("💤", t!("kit-emoji-sleep")),
        ]),
    ]
}

pub fn emoji_page<M: Clone + 'static>(search: &str, typed: impl Fn(String) -> M + 'static, pick: impl Fn(String) -> M) -> Node<M> {
    let wanted = search.trim().to_lowercase();
    let mut items = vec![page_title(t!("kit-emoji")), entry(search, t!("kit-emoji-search"), typed).key("emoji-search")];
    for (group, all) in groups() {
        let shown: Vec<Node<M>> = all.into_iter().filter(|(_, name)| name.to_lowercase().contains(&wanted))
            .map(|(e, name)| quiet_action("", e).spoken(name).on_press(pick(e.to_string())).key(format!("emoji-{name}"))).collect();
        if !shown.is_empty() {
            items.push(section(group));
            items.push(even_grid(48.0, shown));
        }
    }
    if items.len() == 2 {
        items.push(note(t!("kit-emoji-none")));
    }
    column(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Button, Input};
    use crate::theme::Theme;
    use crate::ui::Ui;
    use std::time::Instant;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Search(String),
        Pick(String),
    }

    fn page(search: &str) -> Ui<Msg> {
        let mut ui = Ui::new(emoji_page(search, Msg::Search, Msg::Pick), Theme::new(false, "#fbc711"));
        ui.lay_at(600, 900);
        ui
    }

    #[test]
    fn pressing_an_emoji_sends_it_and_each_has_a_name() {
        let mut ui = page("");
        let (_, r, _) = ui.boxes().into_iter().find(|(id, ..)| id == "emoji-Cat").unwrap();
        let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
        let now = Instant::now();
        ui.input(&Input::Press { button: Button::Left, x, y }, now);
        assert_eq!(ui.input(&Input::Release { button: Button::Left, x, y }, now).messages, vec![Msg::Pick("🐱".into())]);
        assert!(ui.unnamed_controls().is_empty());
    }

    #[test]
    fn typing_a_name_keeps_only_what_matches_and_says_when_nothing_does() {
        let mut ui = page("");
        ui.keyboard_to("emoji-search");
        let sent = ui.input(&Input::Key { sym: crate::input::Keysym::t, text: Some("t".into()), mods: Default::default(), repeat: false }, Instant::now()).messages;
        assert_eq!(sent, vec![Msg::Search("t".into())]);
        let keys = |ui: &Ui<Msg>| ui.boxes().into_iter().filter(|(id, ..)| id.starts_with("emoji-") && id != "emoji-search").count();
        let all = keys(&ui);
        let cat = page("CAT");
        assert_eq!(keys(&cat), 1);
        assert!(all >= 150);
        assert!(cat.boxes().iter().any(|(id, ..)| id == "emoji-Cat"));
        let mut none = page("zzz");
        assert_eq!(keys(&none), 0);
        let words: Vec<String> = none.access("Emoji").nodes.iter().filter_map(|(_, p)| p.label().or(p.value()).map(str::to_string)).collect();
        assert!(words.iter().any(|w| w == "No emoji has that name."));
    }

    #[test]
    fn there_are_eight_groups_and_a_hundred_and_fifty_emoji_each_with_its_own_name() {
        let all = groups();
        assert_eq!(all.len(), 8);
        let mut names: Vec<&str> = all.iter().flat_map(|(_, es)| es.iter().map(|e| e.1)).collect();
        assert!(names.len() >= 150, "{}", names.len());
        names.sort_unstable();
        names.dedup();
        assert!(names.len() >= 150, "two emoji share a name");
        assert!(all.iter().all(|(group, es)| !group.starts_with("kit-") && es.len() >= 16));
    }
}
