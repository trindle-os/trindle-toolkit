//! What a layout works from, and the name each widget is known by from one layout to the next.

use std::collections::HashMap;

use crate::text::Fonts;

use super::{bento, Node, Stays, Texts, What};

pub(crate) type Anchor<'a> = Option<(&'a str, usize)>;

pub(crate) fn id_of(key: Option<&String>, anchor: Anchor, path: &[usize]) -> String {
    if let Some(k) = key {
        return k.clone();
    }
    let place = |p: &[usize]| p.iter().map(usize::to_string).collect::<Vec<_>>().join(".");
    match anchor {
        Some((name, deep)) => format!("{name}/{}", place(&path[deep.min(path.len())..])),
        None => place(path),
    }
}

pub(crate) fn puzzle_names<M>(tree: &Node<M>) -> HashMap<usize, String> {
    fn walk<M>(n: &Node<M>, above: &str, count: &mut HashMap<String, usize>, out: &mut HashMap<usize, String>) {
        if let What::Grid { items, puzzle: true, even: false, .. } = &n.what {
            if !items.is_empty() {
                let name = n.key.clone().unwrap_or_else(|| {
                    let nth = count.entry(above.to_string()).or_default();
                    *nth += 1;
                    format!("{above}#{nth}")
                });
                out.insert(items.as_ptr() as usize, name);
            }
        }
        let above = n.key.as_deref().unwrap_or(above);
        for item in n.items() {
            walk(item, above, count, out);
        }
    }
    let mut out = HashMap::new();
    walk(tree, "", &mut HashMap::new(), &mut out);
    out
}

pub(crate) struct Lay<'a> {
    pub fonts: &'a Fonts,
    pub texts: &'a Texts,
    pub scale: f32,
    pub scrolled: &'a HashMap<String, f32>,
    pub(crate) splits: Option<&'a HashMap<String, f32>>,
    pub natural: std::cell::Cell<bool>,
    pub named: bool,
    pub whole: std::cell::Cell<bool>,
    pub(crate) bento: bento::Memo,
    pub(crate) puzzles: HashMap<usize, String>,
    pub(crate) held: Option<&'a bento::Kept>,
    pub(crate) stays: Option<&'a Stays>,
}

#[cfg_attr(not(any(test, feature = "testing")), allow(clippy::needless_lifetimes))]
impl<'a> Lay<'a> {
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn bare(fonts: &'a Fonts, texts: &'a Texts, scrolled: &'a HashMap<String, f32>) -> Lay<'a> {
        Lay { fonts, texts, scale: 1.0, scrolled, splits: None, natural: Default::default(), bento: Default::default(), named: false, whole: Default::default(),
              puzzles: Default::default(), held: None, stays: None }
    }
}

impl Lay<'_> {
    pub(super) fn px(&self, points: f32) -> f32 {
        (points * self.scale).round()
    }

    pub(super) fn rounding(&self, parts: usize) -> f32 {
        if self.scale.fract() == 0.0 {
            return 0.5;
        }
        0.5 * (2 * parts + 2) as f32
    }
}
