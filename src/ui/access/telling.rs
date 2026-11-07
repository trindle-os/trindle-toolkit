//! The tree of parts told to the screen reader: one for each widget, with the pieces it holds.

use super::says::{tell_copy_menu, tell_says};
use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn tell_over_the_rest(&self, top: &mut Vec<NodeId>) -> Vec<(NodeId, Part)> {
        let mut extra_top = Vec::new();
        if !self.news.is_empty() {
            let mut news = Part::new(Role::Status);
            news.set_live(Live::Polite);
            news.set_live_atomic();
            news.set_label(self.news.as_str());
            news.set_bounds(bounds(Rect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }));
            top.push(NEWS);
            extra_top.push((NEWS, news));
        }
        if let (Some(m), Some((r, sub))) = (&self.copy_menu, self.menu_rects()) {
            let lit = if m.popup.beside.hover.is_some() { None } else { m.popup.hover };
            let mut parts = tell_copy_menu(COPY_MENU, &m.popup.rows, lit, r);
            if let Some(sub) = sub {
                parts.extend(tell_copy_menu(COPY_SUB, m.popup.sub_rows(), m.popup.beside.hover, sub));
            }
            top.extend(parts.iter().map(|p| p.0).filter(|&id| id == part_id(COPY_MENU, Piece::Widget) || id == part_id(COPY_SUB, Piece::Widget)));
            extra_top.extend(parts);
        }
        extra_top
    }

    pub(super) fn tell_spot<'a>(&'a self, t: &mut Told<'a>, s: &'a Spot) {
        while t.round.last().is_some_and(|&(_, p)| !s.path.starts_with(p) || p.len() >= s.path.len()) {
            t.round.pop();
        }
        let Some(n) = self.tree.at(&s.path) else { return };
        let mut o = One { id: part_id(&s.id, Piece::Widget), part: Part::new(Role::GenericContainer), own: Vec::new(), eye: None };
        o.part.set_bounds(bounds(s.rect));
        self.tell_what(t, s, n, &mut o);
        if let Some(says) = n.says.as_deref() {
            tell_says(&s.id, &mut o.part, says, s.rect, self.scale, &mut o.own, &mut t.extra);
            if says.role.as_ref().is_some_and(crate::role::Role::steps) && self.keys_of(&s.id).is_some() {
                o.part.add_action(Action::Increment);
                o.part.add_action(Action::Decrement);
            }
            if let Some(alert) = says.dialog {
                t.dialog = Some(t.dialog.unwrap_or(false) || alert);
                if let What::Label { text, .. } = &n.what {
                    t.message.push(text.as_str());
                }
            }
        }
        if n.off {
            o.part.set_disabled();
        }
        if self.outside.is_some() && (self.hover.as_deref() == Some(s.id.as_str()) || self.gap.as_ref().is_some_and(|(g, _)| *g == s.id)) {
            o.part.set_description(crate::t!("kit-drop-target"));
        }
        if n.focusable() {
            o.part.add_action(Action::Focus);
            o.part.add_action(Action::ScrollIntoView);
        }
        if (n.context.is_some() || n.focusable()) && self.context_rows(&s.path).is_some() {
            o.part.add_action(Action::ShowContextMenu);
        }
        if !matches!(n.what, What::Entry { .. }) {
            t.last_entry = None;
        }
        let words = match &n.what {
            What::Label { text, .. } => Some(text.as_str()),
            What::Drawing { tip, .. } if !tip.is_empty() => Some(tip.as_str()),
            _ => None,
        };
        if let Some(w) = words {
            for (above, _) in &t.round {
                if let Some(said) = t.cards.get_mut(above) {
                    said.push(w);
                }
            }
        }
        let kids = [Some(o.id), o.eye].into_iter().flatten();
        match t.round.last() {
            Some(&(p, _)) => t.widgets[p].2.extend(kids),
            None => t.top.extend(kids),
        }
        t.round.push((t.widgets.len(), &s.path));
        t.widgets.push((o.id, o.part, o.own));
    }

    fn tell_what(&self, t: &mut Told, s: &Spot, n: &w::Node<M>, o: &mut One) {
        let part = &mut o.part;
        match &n.what {
            What::Card(_, None) => part.set_role(Role::Group),
            What::Card(_, Some(_)) => {
                part.set_role(Role::Button);
                part.add_action(Action::Click);
                if let Some(chosen) = n.says.as_ref().and_then(|k| k.radio) {
                    part.set_role(Role::RadioButton);
                    part.set_toggled(if chosen { Toggled::True } else { Toggled::False });
                }
                t.cards.insert(t.widgets.len(), Vec::new());
            }
            What::Scroll(_) => self.tell_scroll(s, part),
            What::ScrollX(_) => self.tell_scroll_x(s, part),
            What::Slide { .. } => {}
            What::Spinner { .. } => {
                part.set_role(Role::ProgressIndicator);
                part.set_label(crate::t!("kit-loading"));
                part.set_busy();
            }
            What::Drawing { .. } if n.says.as_deref().is_some_and(|k| k.unseen) => part.set_hidden(),
            What::Drawing { tip, click: Some(_), .. } => {
                part.set_role(Role::Button);
                part.set_label(tip.as_str());
                part.add_action(Action::Click);
            }
            What::Drawing { tip, keys: None, .. } if tip.is_empty() && n.says.is_none() => part.set_hidden(),
            What::Drawing { tip, .. } => {
                part.set_role(Role::Image);
                part.set_label(tip.as_str());
            }
            What::Rich { spans, .. } => self.tell_rich(t, s, spans, o),
            What::Ring { share, .. } => {
                part.set_role(Role::ProgressIndicator);
                part.set_label(crate::t!("kit-progress"));
                part.set_numeric_value(f64::from(*share));
                part.set_min_numeric_value(0.0);
                part.set_max_numeric_value(1.0);
            }
            _ => self.tell_label(t, s, n, o),
        }
    }

    fn tell_scroll(&self, s: &Spot, part: &mut Part) {
        part.set_role(Role::ScrollView);
        part.set_scroll_y(f64::from(self.offset(s)));
        part.set_scroll_y_min(0.0);
        part.set_scroll_y_max(f64::from((s.full - s.rect.h).max(0.0)));
        if s.full > s.rect.h + 0.5 {
            part.add_action(Action::ScrollUp);
            part.add_action(Action::ScrollDown);
        }
    }

    fn tell_scroll_x(&self, s: &Spot, part: &mut Part) {
        part.set_role(Role::ScrollView);
        part.set_scroll_x(f64::from(self.offset(s)));
        part.set_scroll_x_min(0.0);
        part.set_scroll_x_max(f64::from(self.span(s)));
        if s.full > s.rect.w + 0.5 {
            part.add_action(Action::ScrollLeft);
            part.add_action(Action::ScrollRight);
        }
    }

    fn tell_rich(&self, t: &mut Told, s: &Spot, spans: &[crate::rich::Span], o: &mut One) {
        let part = &mut o.part;
        part.set_role(Role::Paragraph);
        part.set_label(spans.iter().map(|s| s.text.as_str()).collect::<String>());
        let flow = self.rich_flow(&s.id);
        for (i, sp) in spans.iter().enumerate() {
            let Some(address) = &sp.link else { continue };
            let mut link = Part::new(Role::Link);
            link.set_label(sp.text.as_str());
            link.set_url(address.as_str());
            if let Some(&(x, y, w, h)) = flow.as_ref().and_then(|f| f.bands(i).first().copied()).as_ref() {
                link.set_bounds(bounds(Rect { x: s.rect.x + x, y: s.rect.y + y, w, h }));
            }
            link.add_action(Action::Click);
            link.add_action(Action::Focus);
            let lid = part_id(&s.id, Piece::Item(i));
            o.own.push(lid);
            t.extra.push((lid, link));
        }
    }

    fn tell_label(&self, t: &mut Told, s: &Spot, n: &w::Node<M>, o: &mut One) {
        let part = &mut o.part;
        match &n.what {
            What::Label { text, tone: Tone::Problem, .. } => {
                if let Some((entry, _)) = t.last_entry.filter(|&(_, up)| up == t.round.last().map(|r| r.0)) {
                    t.widgets[entry].1.set_invalid(Invalid::True);
                    t.widgets[entry].1.set_error_message(o.id);
                }
                part.set_role(Role::Alert);
                part.set_label(text.as_str());
                part.set_live(Live::Assertive);
                part.set_live_atomic();
            }
            What::Label { text, words: Words::Page, .. } => {
                part.set_role(Role::Heading);
                part.set_level(1);
                part.set_label(text.as_str());
            }
            What::Label { text, words: words @ (Words::Title | Words::Heading | Words::Section), .. } => {
                part.set_role(Role::Heading);
                part.set_level(match words {
                    Words::Title => 2,
                    Words::Heading => 3,
                    _ => 4,
                });
                part.set_label(text.as_str());
            }
            What::Label { text, .. } => {
                part.set_role(Role::Label);
                part.set_value(text.as_str());
            }
            _ => self.tell_control(t, s, n, o),
        }
    }

    fn tell_control(&self, t: &mut Told, s: &Spot, n: &w::Node<M>, o: &mut One) {
        let part = &mut o.part;
        match &n.what {
            What::Button { tip, press, on, spoken, .. } => {
                part.set_role(Role::Button);
                part.set_label(spoken.as_deref().unwrap_or(tip));
                if press.is_none() {
                    part.set_disabled();
                } else {
                    part.add_action(Action::Click);
                }
                if let Some(on) = on {
                    part.set_toggled(if *on { Toggled::True } else { Toggled::False });
                }
            }
            What::Switch { on, tip, .. } => {
                part.set_role(Role::Switch);
                part.set_label(tip.as_str());
                part.set_toggled(if *on { Toggled::True } else { Toggled::False });
                part.add_action(Action::Click);
            }
            What::Divider { down } => self.tell_divider(s, *down, part),
            What::Tick { on, tip, .. } => {
                part.set_role(Role::CheckBox);
                part.set_label(tip.as_str());
                part.set_toggled(if *on { Toggled::True } else { Toggled::False });
                part.add_action(Action::Click);
            }
            What::Slider { value, min, max, step, tip, .. } => {
                part.set_role(Role::Slider);
                part.set_label(tip.as_str());
                part.set_numeric_value(f64::from(*value));
                part.set_min_numeric_value(f64::from(*min));
                part.set_max_numeric_value(f64::from(*max));
                part.set_numeric_value_step(f64::from(*step));
                for a in [Action::Increment, Action::Decrement, Action::SetValue] {
                    part.add_action(a);
                }
            }
            What::Entry { .. } => {
                t.last_entry = Some((t.widgets.len(), t.round.last().map(|r| r.0)));
                self.tell_entry(s, n, &mut t.extra, o);
            }
            What::List { .. } => self.tell_list(s, n, &mut t.extra, o),
            What::Choice { .. } => self.tell_choice(s, n, &mut t.extra, o),
            What::Dropdown { .. } => self.tell_dropdown(s, n, &mut t.extra, o),
            _ => {}
        }
    }

    fn tell_entry(&self, s: &Spot, n: &w::Node<M>, extra: &mut Vec<(NodeId, Part)>, o: &mut One) {
        let What::Entry { text, tip, secret, spoken, misspelt, .. } = &n.what else { return };
        let part = &mut o.part;
        let dots = *secret;
        part.set_role(if dots { Role::PasswordInput } else { Role::TextInput });
        if matches!(n.what, What::Entry { updown: Some(_), .. }) && !dots {
            part.set_role(Role::ComboBox);
            part.set_auto_complete(accesskit::AutoComplete::List);
        }
        part.set_label(spoken.as_deref().unwrap_or(tip));
        if !misspelt.is_empty() {
            part.set_invalid(Invalid::Spelling);
        }
        let shown = if dots { edit::dots(text) } else { text.clone() };
        if let Some(name) = self.eye_name(&s.id) {
            let mut button = Part::new(Role::Button);
            button.set_label(name);
            button.set_bounds(bounds(self.eye_rect(s.rect)));
            button.set_toggled(if self.eye_open(&s.id) { Toggled::True } else { Toggled::False });
            button.add_action(Action::Click);
            button.add_action(Action::Focus);
            let bid = part_id(&s.id, Piece::Eye);
            extra.push((bid, button));
            o.eye = Some(bid);
        }
        let all = letters(&shown);
        part.set_value(all.concat());
        for a in [Action::SetValue, Action::SetTextSelection, Action::ReplaceSelectedText] {
            part.add_action(a);
        }
        let pieces = pieces_of(&all);
        for (k, piece) in pieces.iter().enumerate() {
            let mut run = text_run(piece, bounds(s.rect));
            if k + 1 < pieces.len() {
                run.set_next_on_line(part_id(&s.id, Piece::Run(k + 1)));
            }
            if k > 0 {
                run.set_previous_on_line(part_id(&s.id, Piece::Run(k - 1)));
            }
            let rid = part_id(&s.id, Piece::Run(k));
            o.own.push(rid);
            extra.push((rid, run));
        }
        if let Some(e) = self.edits.get(&s.id) {
            let n = all.len();
            let at = |byte: usize| {
                let (k, i) = place(text[..byte.min(text.len())].graphemes(true).count().min(n), n);
                TextPosition { node: part_id(&s.id, Piece::Run(k)), character_index: i }
            };
            o.part.set_text_selection(TextSelection { anchor: at(e.anchor), focus: at(e.caret) });
        }
    }

    fn tell_list(&self, s: &Spot, n: &w::Node<M>, extra: &mut Vec<(NodeId, Part)>, o: &mut One) {
        let What::List { rows, chosen, row: h, many, tree, .. } = &n.what else { return };
        o.part.set_role(if tree.is_some() { Role::Tree } else { Role::ListBox });
        o.part.set_size_of_set(rows.len());
        if many.is_some() {
            o.part.set_multiselectable();
        }
        let row = self.px(*h);
        for r in self.list_span(s, rows.len(), *chosen, row) {
            let item = &rows[r];
            let mut option = Part::new(Role::ListBoxOption);
            option.set_label(item.text.as_str());
            let told: Vec<&str> = [&item.under, &item.detail].into_iter().flatten().map(String::as_str).collect();
            if !told.is_empty() {
                option.set_description(told.join(", "));
            }
            option.set_position_in_set(r);
            if let Some(&(depth, open)) = tree.as_ref().and_then(|t| t.rows.get(r)) {
                option.set_role(Role::TreeItem);
                option.set_level(depth + 1);
                if let Some(open) = open {
                    option.set_expanded(open);
                    option.add_action(if open { Action::Collapse } else { Action::Expand });
                }
            }
            option.set_selected(many.as_ref().map_or(*chosen == Some(r), |m| m.picked.contains(&r)));
            option.set_bounds(bounds(Rect { y: s.rect.y + r as f32 * row, h: row, ..s.rect }));
            option.add_action(Action::Click);
            option.add_action(Action::Focus);
            option.add_action(Action::ScrollIntoView);
            let oid = part_id(&s.id, Piece::Row(r));
            o.own.push(oid);
            extra.push((oid, option));
        }
    }

    fn tell_choice(&self, s: &Spot, n: &w::Node<M>, extra: &mut Vec<(NodeId, Part)>, o: &mut One) {
        let What::Choice { options, chosen, tip, arrows, .. } = &n.what else { return };
        o.part.set_role(Role::RadioGroup);
        o.part.set_size_of_set(options.len());
        o.part.set_label(tip.as_str());
        let pad = (crate::theme::sheet().tabs_padding * self.scale).round();
        let widths = w::pills_in(w::pill_widths(&self.fonts, &self.texts, options, arrows.as_ref().map(|a| a.subs.as_slice()), self.scale), s.rect.w - 2.0 * pad);
        let mut x = s.rect.x + pad;
        for (r, (option, width)) in options.iter().zip(&widths).enumerate() {
            let mut pill = Part::new(Role::RadioButton);
            pill.set_label(option.as_str());
            pill.set_position_in_set(r);
            pill.set_toggled(if r == *chosen { Toggled::True } else { Toggled::False });
            pill.set_bounds(bounds(Rect { x, w: *width, ..s.rect }));
            pill.add_action(Action::Click);
            pill.add_action(Action::Focus);
            x += width;
            let oid = part_id(&s.id, Piece::Row(r));
            o.own.push(oid);
            extra.push((oid, pill));
        }
    }

    fn tell_dropdown(&self, s: &Spot, n: &w::Node<M>, extra: &mut Vec<(NodeId, Part)>, o: &mut One) {
        let What::Dropdown { options, chosen, tip, .. } = &n.what else { return };
        let part = &mut o.part;
        part.set_role(Role::ComboBox);
        part.set_label(tip.as_str());
        part.set_value(chosen.and_then(|i| options.get(i)).map_or(w::nothing_chosen(), String::as_str));
        part.set_expanded(self.drop.as_ref().is_some_and(|d| d.id == s.id));
        part.add_action(Action::Click);
        let told = self.told_options(&s.id);
        if let (false, Some(l)) = (told.is_empty(), self.drop_laid()) {
            let mut list = Part::new(Role::ListBox);
            list.set_label(tip.as_str());
            list.set_size_of_set(options.len());
            list.set_bounds(bounds(l.panel));
            let mut kids = Vec::new();
            for r in told {
                let mut option = Part::new(Role::ListBoxOption);
                option.set_label(options[r].as_str());
                option.set_position_in_set(r);
                option.set_selected(*chosen == Some(r));
                option.add_action(Action::Click);
                let oid = part_id(&s.id, Piece::Row(r));
                kids.push(oid);
                extra.push((oid, option));
            }
            list.set_children(kids);
            let lid = part_id(&s.id, Piece::Listing);
            o.own.push(lid);
            extra.push((lid, list));
        }
    }

    pub(super) fn access_focus(&self) -> NodeId {
        if let Some(j) = self.copy_menu.as_ref().and_then(|m| m.popup.beside.hover) {
            return part_id(COPY_SUB, Piece::Item(j));
        }
        if let Some(i) = self.copy_menu.as_ref().and_then(|m| m.popup.hover) {
            return part_id(COPY_MENU, Piece::Item(i));
        }
        if let Some(i) = self.drop.as_ref().and_then(|d| d.lit.filter(|i| self.told_options(&d.id).contains(i))) {
            let id = self.drop.as_ref().map_or("", |d| d.id.as_str());
            return part_id(id, Piece::Row(i));
        }
        for s in &self.spots {
            let lit = self.tree.at(&s.path).and_then(|n| n.says.as_deref()).and_then(|k| k.menu.as_ref()).and_then(|m| m.iter().position(|r| r.lit));
            if let Some(i) = lit {
                return part_id(&s.id, Piece::Item(i));
            }
        }
        let Some(id) = &self.focus else { return WINDOW };
        if self.on_eye(id) {
            return part_id(id, Piece::Eye);
        }
        match self.node(id).map(|n| &n.what) {
            Some(What::List { rows, chosen: Some(r), .. }) if *r < rows.len() => part_id(id, Piece::Row(*r)),
            Some(What::Choice { options, chosen, .. }) if *chosen < options.len() => part_id(id, Piece::Row(*chosen)),
            _ => part_id(id, Piece::Widget),
        }
    }

    pub(super) fn list_span(&self, s: &w::Spot, len: usize, chosen: Option<usize>, row: f32) -> std::ops::Range<usize> {
        let seen = s.clip.map_or(0.0, |c| (c.y - s.rect.y).max(0.0));
        near(len, chosen.unwrap_or((seen / row.max(1.0)) as usize))
    }

    pub(super) fn told_options(&self, id: &str) -> Vec<usize> {
        let (Some(d), Some(l)) = (self.drop.as_ref().filter(|d| d.id == id && d.part.is_none()), self.drop_laid()) else { return Vec::new() };
        let at = d.lit.and_then(|i| l.shown.iter().position(|&x| x == i)).unwrap_or(0);
        l.shown[near(l.shown.len(), at)].to_vec()
    }
}
