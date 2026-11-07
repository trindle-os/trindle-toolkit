//! Rich words in a window: painting them, the link the keyboard is on, and pressing a link.

use super::*;
use crate::rich::{self, Flow};

impl<M: Clone> Ui<M> {
    pub(super) fn rich_flow(&self, id: &str) -> Option<Flow> {
        let (Some(s), Some(What::Rich { spans, words, .. })) = (self.spot(id), self.node(id).map(|n| &n.what)) else { return None };
        Some(rich::flow(&|t, st| self.texts.get(&self.fonts, t, st, None), spans, &words.style(self.scale), Some(s.rect.w.max(1.0))))
    }

    pub(super) fn rich_links(&self, id: &str) -> Vec<usize> {
        match self.node(id).map(|n| &n.what) {
            Some(What::Rich { spans, .. }) => spans.iter().enumerate().filter(|(_, s)| s.link.is_some()).map(|(i, _)| i).collect(),
            _ => Vec::new(),
        }
    }

    pub(super) fn rich_lit(&self, id: &str) -> Option<usize> {
        let links = self.rich_links(id);
        self.along.as_ref().filter(|(a, _)| a == id).and_then(|(_, i)| links.iter().find(|&&l| l == *i).copied()).or(links.first().copied())
    }

    pub(super) fn paint_rich(&self, canvas: &mut PixmapMut, node: &Node<M>, spot: &Spot, t: &Theme) {
        let What::Rich { spans, .. } = &node.what else { return };
        let Some(flow) = self.rich_flow(&spot.id) else { return };
        let under = [self.lies_on().unwrap_or(t.panel)];
        let (plain, link) = (t.words_on(&under), t.link_on(&under));
        flow.draw(&self.fonts, canvas, spot.rect.x, spot.rect.y, &|i| if spans[i].link.is_some() { link } else { plain });
    }

    pub(super) fn rich_ring(&self, spot: &Spot) -> Option<Rect> {
        let lit = self.rich_lit(&spot.id)?;
        let (x, y, w, h) = *self.rich_flow(&spot.id)?.bands(lit).first()?;
        Some(Rect { x: spot.rect.x + x, y: spot.rect.y + y, w, h })
    }

    pub(super) fn rich_tab(&mut self, forward: bool) -> bool {
        let Some(id) = self.focus.clone() else { return false };
        let links = self.rich_links(&id);
        let Some(at) = self.rich_lit(&id).and_then(|l| links.iter().position(|&k| k == l)) else { return false };
        let to = if forward { at + 1 } else { at.wrapping_sub(1) };
        let Some(&span) = links.get(to) else { return false };
        self.along = Some((id, span));
        true
    }

    pub(super) fn rich_arrive(&mut self, forward: bool) {
        let Some(id) = self.focus.clone() else { return };
        let links = self.rich_links(&id);
        if let Some(&span) = if forward { links.first() } else { links.last() } {
            self.along = Some((id, span));
        }
    }

    pub(super) fn rich_press(&mut self, id: &str, x: f32, y: f32) -> bool {
        let (Some(s), Some(flow)) = (self.spot(id), self.rich_flow(id)) else { return false };
        let span = flow.span_at(x - s.rect.x, y - s.rect.y).filter(|i| self.rich_links(id).contains(i));
        if let Some(span) = span {
            self.along = Some((id.to_string(), span));
        }
        span.is_some()
    }

    pub(super) fn rich_follow(&self, id: &str) -> Option<M> {
        let lit = self.rich_lit(id)?;
        let Some(What::Rich { spans, follow: Some(f), .. }) = self.node(id).map(|n| &n.what) else { return None };
        spans[lit].link.clone().map(f)
    }
}
