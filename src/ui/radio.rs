//! A radio group: Tab stops only on its chosen answer, and the arrow keys move the choice.

use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn tab_stop(&self, spot: &Spot) -> bool {
        let radio = |path: &[usize]| self.tree.at(path).and_then(|n| n.says.as_deref()).and_then(|s| s.radio);
        let Some(chosen) = radio(&spot.path) else { return true };
        let Some((&at, parent)) = spot.path.split_last() else { return true };
        let Some(group) = self.tree.at(parent) else { return true };
        let rows: Vec<(usize, bool)> = (0..group.items().len()).filter_map(|i| radio(&[parent, &[i]].concat()).map(|c| (i, c))).collect();
        self.focus.as_deref() == Some(spot.id.as_str()) || chosen || (!rows.iter().any(|r| r.1) && rows.first().is_some_and(|r| r.0 == at))
    }

    pub(super) fn radio_key(&mut self, id: &str, sym: Keysym, reply: &mut Reply<M>) -> bool {
        let Some(path) = self.spot(id).map(|s| s.path.clone()) else { return false };
        if self.tree.at(&path).is_none_or(|n| n.says.as_ref().is_none_or(|s| s.radio.is_none())) {
            return false;
        }
        let (parent, at) = (&path[..path.len() - 1], path[path.len() - 1]);
        let Some(group) = self.tree.at(parent) else { return false };
        let rows: Vec<usize> = (0..group.items().len()).filter(|i| group.items()[*i].says.as_ref().is_some_and(|s| s.radio.is_some())).collect();
        let Some(here) = rows.iter().position(|r| *r == at) else { return false };
        let to = match sym {
            Keysym::Left | Keysym::Up | Keysym::KP_Left | Keysym::KP_Up => (here + rows.len() - 1) % rows.len(),
            Keysym::Right | Keysym::Down | Keysym::KP_Right | Keysym::KP_Down => (here + 1) % rows.len(),
            Keysym::Home | Keysym::KP_Home => 0,
            Keysym::End | Keysym::KP_End => rows.len() - 1,
            _ => return false,
        };
        let mut next = parent.to_vec();
        next.push(rows[to]);
        if let Some(spot) = self.spots.iter().find(|s| s.path == next) {
            let target = spot.id.clone();
            if let Some(m) = self.tree.at(&next).and_then(|n| if let What::Card(_, Some(m)) = &n.what { Some(m.clone()) } else { None }) {
                reply.messages.push(m);
            }
            self.focus = Some(target.clone());
            self.reveal(&target, None);
            reply.redraw = true;
        }
        true
    }
}
