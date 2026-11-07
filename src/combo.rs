//! A combo box: a typing box with the answers that match under it; type a new value or press one.

use super::widget::{card, column, entry, label, Length, Node};

const SHOWN: usize = 6;

pub fn matches<'a>(typed: &str, options: &'a [String]) -> Vec<&'a str> {
    let lower = typed.to_lowercase();
    options.iter().map(String::as_str)
        .filter(|o| !lower.is_empty() && o.to_lowercase().contains(&lower) && *o != typed)
        .take(SHOWN).collect()
}

pub fn combo<M: 'static>(text: &str, options: &[String], tip: impl Into<String>, change: impl Fn(String) -> M + Clone + 'static) -> Node<M> {
    let typed = change.clone();
    let mut all = vec![entry(text, tip, typed)];
    all.extend(matches(text, options).into_iter().map(|o| {
        card(label(o)).flat().width(Length::Fill).on_press(change(o.to_string())).called(o)
    }));
    column(all).gap(crate::theme::gaps().small)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_ignore_case_hide_an_exact_answer_and_are_few() {
        let o: Vec<String> = ["Paris", "Parma", "Rome", "Pari"].iter().map(|s| s.to_string()).collect();
        assert_eq!(matches("par", &o), ["Paris", "Parma", "Pari"]);
        assert_eq!(matches("Pari", &o), ["Paris"]);
        assert!(matches("", &o).is_empty() && matches("x", &o).is_empty());
        let many: Vec<String> = (0..20).map(|i| format!("a{i}")).collect();
        assert_eq!(matches("a", &many).len(), SHOWN);
    }
}
