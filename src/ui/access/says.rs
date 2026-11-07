//! What a drawing says of itself, told to the screen reader: its role, its lines, menus, tabs and tables.

use super::*;

pub(super) fn tell_says(id: &str, part: &mut Part, says: &Says, b: Rect, scale: f32, own: &mut Vec<NodeId>, extra: &mut Vec<(NodeId, Part)>) {
    if let Some(name) = &says.name {
        part.set_label(name.as_str());
    }
    if let Some(urgency) = says.live {
        part.set_live(if urgency == Urgency::Assertive { Live::Assertive } else { Live::Polite });
        part.set_live_atomic();
    }
    if says.alert {
        part.set_role(Role::Alert);
        part.set_live(Live::Assertive);
        part.set_live_atomic();
    }
    if says.busy {
        part.set_busy();
    }
    if let Some(kind) = says.landmark {
        part.set_role(landmark_role(kind));
    }
    if let Some(share) = says.progress {
        tell_progress(part, share);
    }
    if let Some(words) = &says.words {
        part.set_value(words.as_str());
    }
    if says.misspelt {
        part.set_invalid(Invalid::Spelling);
    }
    if let Some(reading) = &says.reading {
        tell_reading(id, part, reading, b, own, extra);
    }
    if let Some(table) = &says.table {
        tell_table(id, part, table, b, scale, own, extra);
    }
    if let Some(tabs) = &says.tabs {
        tell_tabs(id, part, tabs, b, scale, own, extra);
    }
    if let Some(rows) = &says.menu {
        tell_menu(id, part, rows, b, scale, own, extra);
    }
    if let Some(role) = &says.role {
        tell_role(id, part, role, b, scale, own, extra);
    }
}

fn landmark_role(kind: Landmark) -> Role {
    match kind {
        Landmark::Main => Role::Main,
        Landmark::Navigation => Role::Navigation,
        Landmark::Beside => Role::Complementary,
        Landmark::Search => Role::Search,
        Landmark::Form => Role::Form,
        Landmark::Top => Role::Banner,
        Landmark::Foot => Role::ContentInfo,
        Landmark::Region => Role::Region,
    }
}

fn tell_progress(part: &mut Part, share: Option<f32>) {
    part.set_role(Role::ProgressIndicator);
    if part.label().is_none_or(|l| l.trim().is_empty()) {
        part.set_label(crate::t!("kit-progress"));
    }
    part.set_min_numeric_value(0.0);
    part.set_max_numeric_value(1.0);
    match share {
        Some(v) => {
            part.set_numeric_value(f64::from(v));
            part.set_value(crate::numbers::percent(f64::from(v), 0));
        }
        None => part.set_busy(),
    }
}

struct Pieces<'a> {
    id: &'a str,
    b: Rect,
    scale: f32,
    own: &'a mut Vec<NodeId>,
    extra: &'a mut Vec<(NodeId, Part)>,
}

impl Pieces<'_> {
    fn add(&mut self, i: usize, part: Part) {
        let oid = part_id(self.id, Piece::Item(i));
        self.own.push(oid);
        self.extra.push((oid, part));
    }

    fn along(&self, x: f32, w: f32) -> Bounds {
        bounds(Rect { x: self.b.x + x * self.scale, w: w * self.scale, ..self.b })
    }
}

fn tell_role(id: &str, part: &mut Part, role: &crate::role::Role, b: Rect, scale: f32, own: &mut Vec<NodeId>, extra: &mut Vec<(NodeId, Part)>) {
    use crate::role::Role as K;
    let mut pieces = Pieces { id, b, scale, own, extra };
    let (role, value) = match *role {
        K::Button => (Role::Button, None),
        K::CheckBox(on) => {
            part.set_toggled(if on { Toggled::True } else { Toggled::False });
            (Role::CheckBox, None)
        }
        K::Switch(on) => {
            part.set_toggled(if on { Toggled::True } else { Toggled::False });
            (Role::Switch, None)
        }
        K::Slider { value, min, max } => (Role::Slider, Some((value, min, max))),
        K::SpinButton { value, min, max } => (Role::SpinButton, Some((value, min, max))),
        K::Meter { value, min, max } => (Role::Meter, Some((value, min, max))),
        K::Image => (Role::Image, None),
        K::Group => (Role::Group, None),
        K::Dialog => (Role::Dialog, None),
        K::MenuBar { ref titles, lit, open } => {
            tell_menu_bar(part, titles, lit, open, &mut pieces);
            (Role::MenuBar, None)
        }
        K::Toolbar { ref tools, lit, ref on } => {
            tell_toolbar(tools, lit, on, &mut pieces);
            (Role::Toolbar, None)
        }
        K::Status => (Role::Status, None),
        K::Path => (Role::Navigation, None),
        K::Range { low, high, min, max, lit } => {
            tell_range(part, (low, high), (min, max), lit, &mut pieces);
            (Role::Group, None)
        }
        K::Segments { ref parts, lit } => {
            tell_segments(part, parts, lit, &mut pieces);
            (Role::Group, None)
        }
    };
    part.set_role(role);
    if let Some((value, min, max)) = value {
        part.set_numeric_value(value);
        part.set_min_numeric_value(min);
        part.set_max_numeric_value(max);
    }
}

fn tell_menu_bar(part: &mut Part, titles: &[(String, f32, f32)], lit: Option<usize>, open: bool, pieces: &mut Pieces) {
    part.set_size_of_set(titles.len());
    for (i, (words, x, w)) in titles.iter().enumerate() {
        let mut item = Part::new(Role::MenuItem);
        item.set_label(words.as_str());
        item.set_position_in_set(i);
        item.set_selected(lit == Some(i));
        item.set_has_popup(accesskit::HasPopup::Menu);
        item.set_expanded(open && lit == Some(i));
        item.set_bounds(pieces.along(*x, *w));
        pieces.add(i, item);
    }
}

fn tell_toolbar(tools: &[(String, f32, f32)], lit: Option<usize>, on: &[Option<bool>], pieces: &mut Pieces) {
    for (i, (words, x, w)) in tools.iter().enumerate() {
        let mut item = Part::new(Role::Button);
        if let Some(&Some(on)) = on.get(i) {
            item.set_toggled(if on { Toggled::True } else { Toggled::False });
        }
        item.set_label(words.as_str());
        item.set_selected(lit == Some(i));
        item.add_action(Action::Click);
        item.set_bounds(pieces.along(*x, *w));
        pieces.add(i, item);
    }
}

fn tell_range(part: &Part, (low, high): (f64, f64), (min, max): (f64, f64), lit: usize, pieces: &mut Pieces) {
    let (b, scale) = (pieces.b, pieces.scale);
    let knob = (crate::theme::sheet().knob * scale).round();
    let side = (crate::theme::TARGET_LEAST * scale).max(knob).min(b.h).min(b.w);
    let along = |v: f64| if max > min { ((v - min) / (max - min)).clamp(0.0, 1.0) as f32 } else { 0.0 };
    for (i, (value, end)) in [(low, crate::t!("kit-range-low")), (high, crate::t!("kit-range-high"))].into_iter().enumerate() {
        let mut handle = Part::new(Role::Slider);
        handle.set_label(format!("{} {end}", part.label().unwrap_or_default()));
        handle.set_numeric_value(value);
        handle.set_min_numeric_value(if i == 0 { min } else { low });
        handle.set_max_numeric_value(if i == 0 { high } else { max });
        handle.set_selected(lit == i);
        handle.add_action(Action::Increment);
        handle.add_action(Action::Decrement);
        handle.add_action(Action::Focus);
        let x = b.x + along(value) * (b.w - knob) + knob / 2.0 - side / 2.0;
        handle.set_bounds(bounds(Rect { x: x.clamp(b.x, b.x + b.w - side), y: b.y + (b.h - side) / 2.0, w: side, h: side }));
        pieces.add(i, handle);
    }
}

fn tell_segments(part: &Part, parts: &[(String, f64, f64, f32, f32)], lit: usize, pieces: &mut Pieces) {
    for (i, (name, value, max, x, w)) in parts.iter().enumerate() {
        let mut seg = Part::new(Role::SpinButton);
        seg.set_label(format!("{} {name}", part.label().unwrap_or_default()));
        seg.set_numeric_value(*value);
        seg.set_min_numeric_value(0.0);
        seg.set_max_numeric_value(*max);
        seg.set_selected(lit == i);
        seg.add_action(Action::Increment);
        seg.add_action(Action::Decrement);
        seg.add_action(Action::Focus);
        seg.set_bounds(pieces.along(*x, *w));
        pieces.add(i, seg);
    }
}

fn tell_reading(id: &str, part: &mut Part, reading: &Reading, b: Rect, own: &mut Vec<NodeId>, extra: &mut Vec<(NodeId, Part)>) {
    let Reading { lines, caret, selection, editable } = reading;
    part.set_role(if *editable { Role::MultilineTextInput } else { Role::Terminal });
    let span = near(lines.len(), caret.map_or(lines.len(), |c| c.0));
    let shown = &lines[span.clone()];
    let tall = (b.h / shown.len().max(1) as f32).max(1.0);
    let counts: Vec<usize> = shown.iter().map(|l| letters(l).len()).collect();
    for (i, line) in shown.iter().enumerate() {
        let all = letters(line);
        let pieces = pieces_of(&all);
        let n = span.start + i;
        for (k, piece) in pieces.iter().enumerate() {
            let top = b.y + i as f32 * tall;
            let mut run = text_run(piece, bounds(Rect { y: top, h: tall, ..b }));
            if k + 1 < pieces.len() {
                run.set_next_on_line(part_id(id, Piece::Line(n, k + 1)));
            }
            if k > 0 {
                run.set_previous_on_line(part_id(id, Piece::Line(n, k - 1)));
            }
            let rid = part_id(id, Piece::Line(n, k));
            own.push(rid);
            extra.push((rid, run));
        }
    }
    if shown.is_empty() {
        let run = text_run(&[], bounds(b));
        let rid = part_id(id, Piece::Line(0, 0));
        own.push(rid);
        extra.push((rid, run));
    }
    let at = |(line, col): (usize, usize)| {
        let line = line.clamp(span.start, span.end.saturating_sub(1).max(span.start));
        let n = counts.get(line - span.start).copied().unwrap_or(0);
        let (k, i) = place(col.min(n), n);
        TextPosition { node: part_id(id, Piece::Line(line, k)), character_index: i }
    };
    if let Some(c) = caret {
        let (anchor, focus) = selection.unwrap_or((*c, *c));
        part.set_text_selection(TextSelection { anchor: at(anchor), focus: at(focus) });
    }
}

fn tell_menu(id: &str, part: &mut Part, rows: &[w::SaidRow], b: Rect, scale: f32, own: &mut Vec<NodeId>, extra: &mut Vec<(NodeId, Part)>) {
    part.set_role(Role::Menu);
    part.set_size_of_set(rows.len());
    for (i, row) in rows.iter().enumerate() {
        let mut item = menu_item(row.check, row.radio);
        item.set_label(row.words.as_str());
        item.set_position_in_set(i);
        item.set_selected(row.lit);
        item.set_bounds(bounds(Rect { y: b.y + row.top * scale, h: row.height * scale, ..b }));
        let oid = part_id(id, Piece::Item(i));
        own.push(oid);
        extra.push((oid, item));
    }
}

fn menu_item(check: Option<bool>, radio: bool) -> Part {
    let Some(on) = check else { return Part::new(Role::MenuItem) };
    let mut item = Part::new(if radio { Role::MenuItemRadio } else { Role::MenuItemCheckBox });
    item.set_toggled(if on { Toggled::True } else { Toggled::False });
    item
}

pub(super) fn tell_copy_menu(of: &str, rows: &[Row], lit: Option<usize>, r: Rect) -> Vec<(NodeId, Part)> {
    let tall = r.h / rows.len().max(1) as f32;
    let mut menu = Part::new(Role::Menu);
    menu.set_label(crate::t!("kit-menu"));
    menu.set_bounds(bounds(r));
    menu.set_size_of_set(rows.len());
    let mut parts = Vec::new();
    let mut kids = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let mut item = menu_item(row.check, row.radio);
        item.set_label(row.said());
        item.set_position_in_set(i);
        item.set_selected(lit == Some(i));
        item.add_action(Action::Click);
        item.set_bounds(bounds(Rect { y: r.y + i as f32 * tall, h: tall, ..r }));
        let id = part_id(of, Piece::Item(i));
        kids.push(id);
        parts.push((id, item));
    }
    menu.set_children(kids);
    parts.insert(0, (part_id(of, Piece::Widget), menu));
    parts
}

fn tell_tabs(id: &str, part: &mut Part, tabs: &w::SaidTabs, b: Rect, scale: f32, own: &mut Vec<NodeId>, extra: &mut Vec<(NodeId, Part)>) {
    part.set_role(Role::TabList);
    part.set_size_of_set(tabs.total);
    let at = |x: f32, w: f32| bounds(Rect { x: b.x + x * scale, y: b.y, w: w * scale, h: tabs.height * scale });
    let mut kids = Vec::new();
    for tab in &tabs.tabs {
        let tid = part_id(id, Piece::Tab(tab.index));
        let mut t = Part::new(Role::Tab);
        t.set_label(tab.words.as_str());
        t.set_position_in_set(tab.index);
        t.set_size_of_set(tabs.total);
        t.set_selected(tab.chosen);
        t.add_action(Action::Click);
        t.add_action(Action::Focus);
        t.set_bounds(at(tab.x, tab.width));
        if tab.closable {
            let cid = part_id(id, Piece::Close(tab.index));
            let mut c = Part::new(Role::Button);
            c.set_label(crate::t!("kit-tab-close"));
            c.add_action(Action::Click);
            c.add_action(Action::Focus);
            let cb = tabs::close_box(Rect { x: tab.x, y: 0.0, w: tab.width, h: tabs.height });
            c.set_bounds(bounds(Rect { x: b.x + cb.x * scale, y: b.y + cb.y * scale, w: cb.w * scale, h: cb.h * scale }));
            t.set_children(vec![cid]);
            own.push(cid);
            extra.push((cid, c));
        }
        kids.push(tid);
        own.push(tid);
        extra.push((tid, t));
    }
    if tabs.more {
        let mid = part_id(id, Piece::More);
        let mut m = Part::new(Role::Button);
        m.set_label(crate::t!("kit-tab-more"));
        m.add_action(Action::Click);
        let right = tabs.tabs.last().map_or(0.0, |t| t.x + t.width);
        m.set_bounds(at(right, tabs::more()));
        kids.push(mid);
        own.push(mid);
        extra.push((mid, m));
    }
    part.set_children(kids);
}

fn cell_bounds(table: &w::SaidTable, b: Rect, scale: f32, col: usize, top: f32, height: f32) -> Bounds {
    let (x, w) = table.columns.get(col).copied().unwrap_or((0.0, 0.0));
    bounds(Rect { x: b.x + x * scale, y: b.y + top * scale, w: w * scale, h: height * scale })
}

fn tell_table_head(id: &str, table: &w::SaidTable, b: Rect, scale: f32, own: &mut Vec<NodeId>, extra: &mut Vec<(NodeId, Part)>) {
    let head = part_id(id, Piece::TableHead);
    let mut head_row = Part::new(Role::Row);
    head_row.set_row_index(0);
    head_row.set_bounds(bounds(Rect { y: b.y, h: table.head_height * scale, ..b }));
    let mut heads = Vec::new();
    for (c, title) in table.headings.iter().enumerate() {
        let mut h = Part::new(Role::ColumnHeader);
        h.set_label(title.as_str());
        h.set_column_index(c);
        h.set_bounds(cell_bounds(table, b, scale, c, 0.0, table.head_height));
        if let Some((_, up)) = table.sort.filter(|&(s, _)| s == c) {
            h.set_sort_direction(if up { accesskit::SortDirection::Ascending } else { accesskit::SortDirection::Descending });
        }
        let hid = part_id(id, Piece::Heading(c));
        heads.push(hid);
        extra.push((hid, h));
    }
    head_row.set_children(heads);
    own.push(head);
    extra.push((head, head_row));
}

fn tell_table_row(id: &str, table: &w::SaidTable, row: &w::SaidCells, b: Rect, scale: f32, own: &mut Vec<NodeId>, extra: &mut Vec<(NodeId, Part)>) {
    let rid = part_id(id, Piece::TableRow(row.index));
    let mut r = Part::new(Role::Row);
    r.set_row_index(row.index + 1);
    r.set_selected(row.chosen);
    if let Some(b) = row.branch {
        r.set_level(b.level);
        r.set_position_in_set(b.place);
        r.set_size_of_set(b.of);
        if let Some(open) = b.open {
            r.set_expanded(open);
            r.add_action(if open { Action::Collapse } else { Action::Expand });
        }
    }
    if let Some(group) = &row.group {
        r.set_description(group.as_str());
    }
    r.add_action(Action::Click);
    r.set_bounds(bounds(Rect { y: b.y + row.top * scale, h: row.height * scale, ..b }));
    let mut cells = Vec::new();
    for (c, words) in row.cells.iter().enumerate() {
        let mut cell = Part::new(Role::Cell);
        cell.set_label(words.as_str());
        cell.set_column_index(c);
        cell.set_row_index(row.index + 1);
        cell.set_bounds(cell_bounds(table, b, scale, c, row.top, row.height));
        let cid = part_id(id, Piece::Cell(row.index, c));
        cells.push(cid);
        extra.push((cid, cell));
    }
    r.set_children(cells);
    own.push(rid);
    extra.push((rid, r));
}

fn tell_table(id: &str, part: &mut Part, table: &w::SaidTable, b: Rect, scale: f32, own: &mut Vec<NodeId>, extra: &mut Vec<(NodeId, Part)>) {
    part.set_role(if table.tree { Role::TreeGrid } else { Role::Table });
    if table.many {
        part.set_multiselectable();
    }
    part.set_row_count(table.total + 1);
    part.set_column_count(table.headings.len());
    tell_table_head(id, table, b, scale, own, extra);
    for row in table.rows.iter().take(MOST_LINES) {
        tell_table_row(id, table, row, b, scale, own, extra);
    }
    if let Some(c) = table.column.filter(|&c| c < table.headings.len()) {
        let chosen = table.rows.iter().take(MOST_LINES).find(|r| table.cursor.map_or(r.chosen, |at| r.index == at));
        part.set_active_descendant(chosen.map_or(part_id(id, Piece::Heading(c)), |r| part_id(id, Piece::Cell(r.index, c))));
    }
}
