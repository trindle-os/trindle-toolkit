//! What a window shows, told to a screen reader: each widget as a named part with its role, value and place, and what the screen reader asks done to a part done as if the person had done it.


use accesskit::{Action, ActionData, ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Invalid, Node as Part,
    NodeId, Live, Rect as Bounds, Role, TextPosition, TextSelection, Toggled, TreeId, TreeInfo, TreeUpdate};
use accesskit_unix::Adapter;
use unicode_segmentation::UnicodeSegmentation;

use super::super::edit;
use super::super::event::Post;
use super::super::popup::Row;
use super::super::tabs;
use super::super::widget::{self as w, Landmark, Rect, Reading, Says, Tone, Urgency, What, Words};
use super::{snap, Input, Keysym, Mods, Reply, Spot, Ui};

mod acting;
#[cfg(any(test, feature = "testing"))]
mod audit;
mod says;
mod telling;
#[cfg(test)]
mod tests;

pub(crate) const WINDOW: NodeId = NodeId(0);
const COPY_MENU: &str = "\u{0}copy menu";
const COPY_SUB: &str = "\u{0}copy sub-menu";
const NEWS: NodeId = NodeId(u64::MAX);

const RUN: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Piece {
    Widget,
    Row(usize),
    Run(usize),
    Eye,
    Line(usize, usize),
    Item(usize),
    TableHead,
    TableRow(usize),
    Heading(usize),
    Cell(usize, usize),
    Listing,
    Tab(usize),
    Close(usize),
    More,
}

fn part_id(id: &str, piece: Piece) -> NodeId {
    let (tag, n) = match piece {
        Piece::Widget => (0u8, 0),
        Piece::Row(i) => (1, i),
        Piece::Run(i) => (2, i),
        Piece::Eye => (3, 0),
        Piece::Line(line, k) => (4, (line << 24) | (k & 0xff_ffff)),
        Piece::Item(i) => (5, i),
        Piece::TableHead => (6, 0),
        Piece::TableRow(i) => (7, i),
        Piece::Heading(c) => (8, c),
        Piece::Cell(r, c) => (9, (r << 20) | (c & 0xf_ffff)),
        Piece::Tab(i) => (10, i),
        Piece::Close(i) => (11, i),
        Piece::More => (12, 0),
        Piece::Listing => (13, 0),
    };
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in id.bytes().chain([0xff, tag]).chain((n as u64).to_le_bytes()) {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    NodeId(h.clamp(1, u64::MAX - 1))
}

fn bounds(r: Rect) -> Bounds {
    Bounds { x0: f64::from(r.x), y0: f64::from(r.y), x1: f64::from(r.x + r.w), y1: f64::from(r.y + r.h) }
}

fn letters(shown: &str) -> Vec<&str> {
    shown.graphemes(true).map(|g| if g.len() > 255 { &g[..g.chars().next().map_or(0, char::len_utf8)] } else { g }).collect()
}

fn place(i: usize, n: usize) -> (usize, usize) {
    if i == n && i > 0 && i % RUN == 0 {
        (i / RUN - 1, RUN)
    } else {
        (i / RUN, i % RUN)
    }
}

fn pieces_of<'a>(all: &'a [&'a str]) -> Vec<&'a [&'a str]> {
    if all.is_empty() { vec![&[]] } else { all.chunks(RUN).collect() }
}

fn text_run(piece: &[&str], b: Bounds) -> Part {
    let mut run = Part::new(Role::TextRun);
    run.set_bounds(b);
    run.set_value(piece.concat());
    run.set_character_lengths(piece.iter().map(|g| g.len() as u8).collect::<Vec<u8>>());
    let mut starts = Vec::new();
    let mut gap = true;
    for (j, g) in piece.iter().enumerate() {
        let space = g.chars().all(char::is_whitespace);
        if gap && !space {
            starts.push(j as u8);
        }
        gap = space;
    }
    run.set_word_starts(starts);
    run
}

const MOST_LINES: usize = 500;

fn near(len: usize, centre: usize) -> std::ops::Range<usize> {
    let start = centre.saturating_sub(MOST_LINES / 2).min(len.saturating_sub(MOST_LINES));
    start..len.min(start + MOST_LINES)
}

#[derive(Default)]
struct Told<'a> {
    widgets: Vec<(NodeId, Part, Vec<NodeId>)>,
    extra: Vec<(NodeId, Part)>,
    top: Vec<NodeId>,
    message: Vec<&'a str>,
    dialog: Option<bool>,
    round: Vec<(usize, &'a [usize])>,
    cards: std::collections::HashMap<usize, Vec<&'a str>>,
    last_entry: Option<(usize, Option<usize>)>,
}

struct One {
    id: NodeId,
    part: Part,
    own: Vec<NodeId>,
    eye: Option<NodeId>,
}

#[cfg(debug_assertions)]
fn no_two_share(nodes: &[(NodeId, Part)]) {
    if nodes.iter().map(|n| n.0).collect::<std::collections::HashSet<_>>().len() != nodes.len() {
        let mut seen = std::collections::HashMap::new();
        let shared: Vec<String> = nodes.iter().filter_map(|(id, part)| {
            let said = format!("{:?} {:?}", part.role(), part.label().unwrap_or_default());
            seen.insert(*id, said.clone()).map(|first| format!("{id:?}: {first} and {said}"))
        }).collect();
        panic!("two parts share a number (two widgets with one key): {}", shared.join("; "));
    }
}

impl<M: Clone> Ui<M> {
    pub fn access(&mut self, title: &str) -> TreeUpdate {
        self.settle();
        let ids: Vec<String> = self.spots.iter().filter(|s| self.is_entry(&s.id) && self.edits.contains_key(&s.id))
            .map(|s| s.id.clone()).collect();
        for id in ids {
            self.edit(&id);
        }
        let mut window = Part::new(Role::Window);
        window.set_label(title);
        window.set_bounds(bounds(Rect { x: 0.0, y: 0.0, w: self.size.0 as f32, h: self.size.1 as f32 }));
        let mut told = Told::default();
        for s in &self.spots {
            self.tell_spot(&mut told, s);
        }
        let Told { mut widgets, extra, mut top, message, dialog, cards, .. } = told;
        for (card, said) in cards {
            if !said.is_empty() {
                widgets[card].1.set_label(said.join(" "));
            }
        }
        let extra_top = self.tell_over_the_rest(&mut top);
        if let Some(warns) = dialog {
            window.set_role(if warns { Role::AlertDialog } else { Role::Dialog });
            window.set_description(message.join(" "));
        }
        window.set_children(top);
        let mut nodes = vec![(WINDOW, window)];
        nodes.extend(extra_top);
        for (id, mut part, children) in widgets {
            if !children.is_empty() {
                part.set_children(children);
            }
            nodes.push((id, part));
        }
        nodes.extend(extra);
        #[cfg(debug_assertions)]
        no_two_share(&nodes);
        let mut tree = TreeInfo::new(WINDOW);
        tree.toolkit_name = Some("Trindle Toolkit".into());
        tree.toolkit_version = Some(env!("CARGO_PKG_VERSION").into());
        TreeUpdate { nodes, tree: Some(tree), tree_id: TreeId::ROOT, focus: self.access_focus() }
    }

    pub fn act(&mut self, req: &ActionRequest) -> Reply<M> {
        self.settle();
        let mut reply = Reply::new();
        let Some((id, piece)) = self.find(req.target_node) else { return reply };
        if id == COPY_MENU || id == COPY_SUB {
            if let (Action::Click, Piece::Item(i)) = (req.action, piece) {
                reply.redraw = true;
                match self.copy_menu.as_ref().and_then(|m| m.popup.beside.open).filter(|_| id == COPY_SUB) {
                    Some(o) => self.pick_sub_row(o, i, &mut reply),
                    None => self.pick_menu_row(i, &mut reply),
                }
            }
            return reply;
        }
        if self.node(&id).is_some_and(|n| n.off) && !matches!(req.action, Action::Focus | Action::ScrollIntoView) {
            return reply;
        }
        reply.redraw = true;
        let row = self.row_of(&id);
        match (req.action, piece) {
            (Action::Click, Piece::Eye) => self.eye_press(&id, std::time::Instant::now(), &mut reply),
            (Action::Focus, Piece::Eye) => self.act_focus(&id, true),
            (_, Piece::Eye) => {}
            (Action::Focus | Action::Click, Piece::Row(r)) if self.is_choice(&id) => {
                self.act_choice(&id, r, req.action == Action::Focus, &mut reply);
            }
            (Action::Focus, Piece::Row(r)) if !self.is_dropdown(&id) => {
                self.act_list_focus(&id, r, row, &mut reply);
            }
            (Action::Focus | Action::Increment | Action::Decrement, Piece::Item(i)) if self.handle_lit(&id).is_some() => {
                self.act_handle(&id, i, req.action, &mut reply);
            }
            (Action::Click, Piece::Item(_)) if matches!(self.node(&id).map(|n| &n.what), Some(What::Drawing { .. })) => {
                self.act_tap(&id, piece, &mut reply);
            }
            (Action::Focus | Action::Click, Piece::Item(i)) => {
                self.focus = Some(id.clone());
                self.along = Some((id.clone(), i));
                if req.action == Action::Click {
                    reply.messages.extend(self.rich_follow(&id));
                }
            }
            (Action::Focus, _) => {
                if self.node(&id).is_some_and(|n| n.focusable()) {
                    self.act_focus(&id, false);
                }
            }
            (Action::Click, Piece::Tab(_) | Piece::Close(_) | Piece::More | Piece::TableRow(_)) => {
                self.act_tap(&id, piece, &mut reply);
            }
            (Action::Click, Piece::Row(r)) if self.is_dropdown(&id) => self.pick_drop(r, &mut reply),
            (Action::Click, Piece::Row(r)) => {
                self.act_row_click(&id, r, &mut reply);
            }
            _ => self.act_on_the_whole(&id, piece, row, req, &mut reply),
        }
        reply
    }

    pub(super) fn told_of(&mut self, id: &str) -> Option<(String, String)> {
        let update = self.access("");
        let want = part_id(id, Piece::Widget);
        let (_, part) = update.nodes.iter().find(|(n, _)| *n == want)?;
        Some((format!("{:?}", part.role()), part.label().or(part.value()).unwrap_or_default().to_string()))
    }
}

pub(crate) enum Ask {
    Tree,
    Act(ActionRequest),
}

struct Started(Post<Ask>);

impl ActivationHandler for Started {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.0.send(Ask::Tree);
        None
    }
}

struct Doer(Post<Ask>);

impl ActionHandler for Doer {
    fn do_action(&mut self, request: ActionRequest) {
        self.0.send(Ask::Act(request));
    }
}

struct Stopped;

impl DeactivationHandler for Stopped {
    fn deactivate_accessibility(&mut self) {}
}

pub(crate) struct Link {
    adapter: Adapter,
    pub(crate) title: String,
}

impl Link {
    pub(crate) fn new(title: &str, asks: Post<Ask>) -> Link {
        let adapter = Adapter::new(Started(asks.clone()), Doer(asks), Stopped);
        Link { adapter, title: title.into() }
    }

    pub(crate) fn tell<M: Clone>(&mut self, ui: &mut Ui<M>) {
        let title = &self.title;
        self.adapter.update_if_active(|| ui.access(title));
    }

    pub(crate) fn focused(&mut self, yes: bool) {
        self.adapter.update_window_focus_state(yes);
    }
}
