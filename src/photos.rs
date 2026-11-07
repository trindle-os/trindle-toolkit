//! A gallery of pictures: a large one, arrow buttons and a strip of thumbnails, with arrow keys and "n of total" for the screen reader.

use std::rc::Rc;

use image::DynamicImage;

use super::input::{Input, Keysym};
use super::widget::{button, card, column, image_view, note, row, scroll_x, space, Length, Node};

fn said(name: &str, i: usize, total: usize) -> String {
    crate::t!("kit-photos-place", name = name, n = i + 1, total = total)
}

fn target(key: &Keysym, current: usize, total: usize) -> Option<usize> {
    let last = total.checked_sub(1)?;
    let to = match *key {
        Keysym::Left | Keysym::Up | Keysym::Page_Up => current.saturating_sub(1),
        Keysym::Right | Keysym::Down | Keysym::Page_Down => (current + 1).min(last),
        Keysym::Home => 0,
        Keysym::End => last,
        _ => return None,
    };
    (to != current).then_some(to)
}

pub fn gallery<M: Clone + 'static>(key: &str, slides: &[(Rc<DynamicImage>, String)], current: usize, stage: (f32, f32),
                                   choose: impl Fn(usize) -> M + 'static) -> Node<M> {
    let total = slides.len();
    let (thumb, room) = (crate::theme::sheet().thumb, crate::theme::gaps().medium);
    let Some(last) = total.checked_sub(1) else { return space() };
    let current = current.min(last);
    let choose = Rc::new(choose);
    let (picture, name) = &slides[current];
    let c = choose.clone();
    let big = image_view(picture.clone(), "", stage).called(said(name, current, total)).on_keys(move |input| match input {
        Input::Key { sym, mods, repeat: false, .. } if !mods.ctrl && !mods.alt && !mods.logo => target(sym, current, total).map(|to| c(to)),
        _ => None,
    });
    let step = |icon: &str, tip: String, to: Option<usize>| {
        let b = button(icon, tip);
        match to {
            Some(to) => b.on_press(choose(to)),
            None => b.off(true),
        }
    };
    let bar = row(vec![
        step("go-previous-symbolic", crate::t!("kit-photos-previous").to_string(), current.checked_sub(1)),
        space().width(Length::Fill),
        note(crate::t!("kit-photos-count", n = current + 1, total = total)),
        space().width(Length::Fill),
        step("go-next-symbolic", crate::t!("kit-photos-next").to_string(), (current < last).then_some(current + 1)),
    ]).gap(room).centred();
    let thumbs = slides.iter().enumerate().map(|(i, (picture, name))| {
        let t = card(image_view(picture.clone(), name, (thumb, thumb))).on_press(choose(i)).called(said(name, i, total)).radio(i == current);
        if i == current { t } else { t.flat() }
    }).collect();
    let strip = scroll_x(format!("{key}:thumbs"), row(thumbs).gap(room).padding(room)).height(Length::Fixed(thumb + 4.0 * room));
    column(vec![big, bar, strip]).gap(room)
}
