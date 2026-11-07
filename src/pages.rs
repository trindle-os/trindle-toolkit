//! Whole pages many programs show alike: preferences, about, welcome and a status page.

use super::nav::split_view;
use super::settings::setting;
use super::theme;
use super::widget::{card, column, icon_image, label, lead, link_named, list, main_word, note, page_title, scroll, section, space, Item, Length, Node, Tone};

pub fn preferences<M: Clone>(width: f32, pages: Vec<Item>, current: usize, choose: impl Fn(usize) -> M + 'static, showing: bool, back: M, page: Node<M>) -> Node<M> {
    let name = pages.get(current).map(|p| p.text.clone()).unwrap_or_default();
    split_view(width, list(pages, Some(current), choose).here().called(crate::t!("kit-pages")), page, name, showing, back)
}

pub fn status_page<M>(icon: impl Into<String>, title: impl Into<String>, words: impl Into<String>, action: Option<Node<M>>) -> Node<M> {
    let k = theme::sheet();
    let mut items = vec![icon_image(icon, k.status_icon, Tone::Calm), page_title(title), lead(words)];
    items.extend(action);
    column(vec![space().height(Length::Fill), column(items).gap(k.gap).centred(), space().height(Length::Fill)]).centred().padding(k.page_padding)
}

pub fn welcome_page<M>(icon: impl Into<String>, name: impl Into<String>, words: impl Into<String>, start_words: impl Into<String>, start: M) -> Node<M> {
    status_page(icon, name, words, Some(main_word(start_words).on_press(start)))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct About {
    pub name: String,
    pub icon: String,
    pub version: String,
    pub summary: String,
    pub website: Option<String>,
    pub credits: Vec<(String, Vec<String>)>,
    pub licence: String,
    pub copyright: String,
}

pub fn about_page<M>(key: impl Into<String>, about: &About) -> Node<M> {
    let k = theme::sheet();
    let mut head = vec![icon_image(about.icon.clone(), k.status_icon, Tone::Plain), page_title(about.name.clone()),
                        note(crate::t!("kit-version", version = &about.version)), lead(about.summary.clone())];
    if let Some(site) = &about.website {
        head.push(link_named(crate::t!("kit-website"), site));
    }
    let mut items = vec![column(head).gap(k.gap).centred()];
    if !about.credits.is_empty() {
        items.push(section(crate::t!("kit-credits")));
        items.push(card(column(about.credits.iter().map(|(what, who)| setting(what.clone(), "", label(who.join(", ")))).collect()).gap(k.gap)));
    }
    items.push(section(crate::t!("kit-licence")));
    items.push(card(column(vec![label(about.licence.clone()), note(about.copyright.clone())]).gap(k.gap)));
    scroll(key, column(items).gap(k.gap).padding(k.page_padding))
}
