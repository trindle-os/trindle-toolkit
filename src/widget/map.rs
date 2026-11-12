//! A description built for one program's messages, carried into another's:

use std::mem::replace;
use std::rc::Rc;

use super::{Branches, Many, MenuDo, MenuRow, Node, What};

fn never<A, M>() -> Box<dyn Fn(A) -> M> {
    Box::new(|_| unreachable!("a stand-in that is dropped at once"))
}

impl<M: 'static> Node<M> {
    pub fn map<N: 'static>(mut self, f: Rc<dyn Fn(M) -> N>) -> Node<N> {
        let what = map_what(&mut self.what, &f);
        Node {
            what,
            width: self.width,
            height: self.height,
            key: self.key.take(),
            escape: self.escape.take().map(|m| f(m)),
            first: self.first,
            grab: self.grab,
            off: self.off,
            flat: self.flat,
            chip: self.chip,
            snug: self.snug,
            opacity: self.opacity,
            lit: self.lit,
            cut: self.cut,
            keep_lines: self.keep_lines,
            seek: self.seek,
            nudge: self.nudge,
            glow: self.glow,
            sunk: self.sunk,
            filled: self.filled,
            tint: self.tint,
            beside: self.beside,
            stacks: self.stacks,
            wraps: self.wraps,
            twice: self.twice,
            own_act: self.own_act,
            side: self.side,
            area: self.area,
            floats: self.floats,
            span: self.span,
            band: self.band,
            current: self.current,
            claims: self.claims.take().map(|m| f(m)),
            context: self.context.take().map(|g| {
                let f = f.clone();
                Box::new(move || g().into_iter().map(|r| map_row(r, &f)).collect()) as Box<dyn Fn() -> Vec<MenuRow<N>>>
            }),
            round: self.round,
            leads: self.leads,
            why: self.why,
            whole: self.whole,
            own_labels: self.own_labels,
            quiet: self.quiet,
            says: self.says,
            when_armed: self.when_armed,
            kept: self.kept,
            outside: self.outside.take().map(|o| {
                let (f, take) = (f.clone(), o.take);
                super::Outside { kinds: o.kinds, would: o.would, take: Box::new(move |held, given, way| f(take(held, given, way))) }
            }),
        }
    }
}

fn all<M: 'static, N: 'static>(items: &mut Vec<Node<M>>, f: &Rc<dyn Fn(M) -> N>) -> Vec<Node<N>> {
    std::mem::take(items).into_iter().map(|n| n.map(f.clone())).collect()
}

fn boxed<M: 'static, N: 'static>(inner: &mut Box<Node<M>>, f: &Rc<dyn Fn(M) -> N>) -> Box<Node<N>> {
    let n = replace(&mut **inner, super::space());
    Box::new(n.map(f.clone()))
}

fn via<A: 'static, M: 'static, N: 'static>(g: Box<dyn Fn(A) -> M>, f: &Rc<dyn Fn(M) -> N>) -> Box<dyn Fn(A) -> N> {
    let f = f.clone();
    Box::new(move |a| f(g(a)))
}

fn via2<A: 'static, B: 'static, M: 'static, N: 'static>(g: Box<dyn Fn(A, B) -> Option<M>>, f: &Rc<dyn Fn(M) -> N>) -> Box<dyn Fn(A, B) -> Option<N>> {
    let f = f.clone();
    Box::new(move |a, b| g(a, b).map(&*f))
}

fn map_what<M: 'static, N: 'static>(what: &mut What<M>, f: &Rc<dyn Fn(M) -> N>) -> What<N> {
    match what {
        What::Column { items, gap, padding, centre } => What::Column { items: all(items, f), gap: *gap, padding: *padding, centre: *centre },
        What::Row { items, gap, padding } => What::Row { items: all(items, f), gap: *gap, padding: *padding },
        What::Grid { items, min, gap, even, puzzle, grow, flow } => What::Grid { items: all(items, f), min: *min, gap: *gap, even: *even, puzzle: *puzzle, grow: *grow, flow: *flow },
        What::Card(inner, press) => What::Card(boxed(inner, f), press.take().map(|m| f(m))),
        What::Space => What::Space,
        What::Label { text, words, tone, marks } => What::Label { text: std::mem::take(text), words: *words, tone: *tone, marks: std::mem::take(marks) },
        What::Ring { share, size, tone } => What::Ring { share: *share, size: *size, tone: *tone },
        What::Button { icon, tip, press, on, face, spoken, size, working, words, for_good } => What::Button {
            icon: std::mem::take(icon), tip: std::mem::take(tip), press: press.take().map(|m| f(m)), on: *on, face: face.take(),
            spoken: spoken.take(), size: *size, working: *working, words: words.take(), for_good: *for_good,
        },
        What::Rich { spans, words, follow } => What::Rich { spans: std::mem::take(spans), words: *words, follow: follow.take().map(|g| via(g, f)) },
        What::Switch { on, tip, change } => What::Switch { on: *on, tip: std::mem::take(tip), change: via(replace(change, never()), f) },
        What::Tick { on, tip, change } => What::Tick { on: *on, tip: std::mem::take(tip), change: via(replace(change, never()), f) },
        What::Slider { value, min, max, step, tip, change } => What::Slider {
            value: *value, min: *min, max: *max, step: *step, tip: std::mem::take(tip), change: via(replace(change, never()), f),
        },
        What::Entry { text, tip, change, submit, secret, kind, guarded, show, spoken, lines, select, leave, tab, updown, centre, weight, misspelt, back, steps } => What::Entry {
            text: std::mem::take(text), tip: std::mem::take(tip), change: via(replace(change, never()), f),
            submit: submit.take().map(|m| f(m)), secret: *secret, kind: *kind, guarded: *guarded, show: *show, spoken: spoken.take(), lines: *lines,
            select: select.take(), leave: leave.take().map(|m| f(m)), tab: tab.take().map(|t| via(t, f)),
            updown: updown.take().map(|t| via(t, f)), centre: *centre, weight: *weight, misspelt: std::mem::take(misspelt), back: back.take().map(|m| f(m)), steps: steps.take().map(|s| via(s, f)),
        },
        What::Choice { options, chosen, tip, choose, landed, drop_on, middle, arrows } => What::Choice {
            options: std::mem::take(options), chosen: *chosen, tip: std::mem::take(tip), choose: via(replace(choose, never()), f),
            landed: landed.take().map(|l| via(l, f)), drop_on: drop_on.take(), middle: middle.take().map(|l| via(l, f)),
            arrows: arrows.take().map(|a| {
                let super::Arrows { subs, opening, pick } = a;
                let f2 = f.clone();
                super::Arrows { subs, opening: via(opening, f), pick: Box::new(move |i, row| f2(pick(i, row))) as Box<dyn Fn(usize, usize) -> N> }
            }),
        },
        What::Dropdown { options, chosen, tip, choose } => What::Dropdown {
            options: std::mem::take(options), chosen: *chosen, tip: std::mem::take(tip), choose: via(replace(choose, never()), f),
        },
        What::Scroll(inner) => What::Scroll(boxed(inner, f)),
        What::ScrollX(inner) => What::ScrollX(boxed(inner, f)),
        What::List { .. } => map_list(what, f),
        What::Spinner { size } => What::Spinner { size: *size },
        What::Drawing { .. } => map_drawing(what, f),
        What::Layers(items) => What::Layers(all(items, f)),
        What::Split { items, down, start, least } => What::Split { items: all(items, f), down: *down, start: *start, least: *least },
        What::Divider { down } => What::Divider { down: *down },
        What::Placed { inner, at, stays, within } => What::Placed { inner: boxed(inner, f), at: replace(at, Box::new(|_, _| (0.0, 0.0))), stays: *stays, within: *within },
        What::Slide { inner, by, wheel } => What::Slide { inner: boxed(inner, f), by: *by, wheel: wheel.take().map(|w| via(w, f)) },
        What::Aspect { inner, ratio } => What::Aspect { inner: boxed(inner, f), ratio: *ratio },
    }
}

fn map_list<M: 'static, N: 'static>(what: &mut What<M>, f: &Rc<dyn Fn(M) -> N>) -> What<N> {
    let What::List { rows, chosen, choose, open, row, many, menu, landed, drop_on, middle, between, reorder, move_in, ahead, here, tree } = what else { unreachable!() };
    What::List {
        landed: landed.take().map(|l| via(l, f)), drop_on: drop_on.take(), middle: middle.take().map(|l| via(l, f)),
        between: between.take().map(|l| via(l, f)),
        reorder: reorder.take().map(|r| {
            let f = f.clone();
            Box::new(move |from, gap| f(r(from, gap))) as Box<dyn Fn(usize, usize) -> N>
        }),
        move_in: move_in.take().map(|m| {
            let f = f.clone();
            Box::new(move |list, from, gap| f(m(list, from, gap))) as Box<dyn Fn(String, usize, usize) -> N>
        }),
        rows: std::mem::take(rows), chosen: *chosen, choose: via(replace(choose, never()), f),
        open: open.take().map(|o| via(o, f)), row: *row,
        many: many.take().map(|m| Many { picked: m.picked, pick: via(m.pick, f) }),
        menu: menu.take().map(|m| {
            let f = f.clone();
            Box::new(move |i, x, y| f(m(i, x, y))) as Box<dyn Fn(usize, f32, f32) -> N>
        }),
        ahead: *ahead, here: *here,
        tree: tree.take().map(|t| Branches { rows: t.rows, toggle: via(t.toggle, f) }),
    }
}

fn map_drawing<M: 'static, N: 'static>(what: &mut What<M>, f: &Rc<dyn Fn(M) -> N>) -> What<N> {
    let What::Drawing { size, tip, paint, pointer, cursor, hovers, right_drags, keys, scroll, click, twice, seen_only, named, cut, live, alive, menu, passes } = what else { unreachable!() };
    What::Drawing {
        size: *size, tip: std::mem::take(tip), paint: replace(paint, Box::new(|_, _| {})),
        pointer: pointer.take().map(|p| via2(p, f)), cursor: cursor.take(), hovers: *hovers, passes: *passes, right_drags: *right_drags,
        keys: keys.take().map(|k| {
            let f = f.clone();
            Box::new(move |i: &super::Input| k(i).map(|m| f(m))) as Box<dyn Fn(&super::Input) -> Option<N>>
        }),
        scroll: scroll.take().map(|s| via(s, f)),
        click: click.take().map(|m| f(m)), twice: twice.take().map(|m| f(m)), seen_only: *seen_only, named: *named, cut: cut.take(), live: live.take(), alive: *alive,
        menu: menu.take().map(|g| {
            let f = f.clone();
            Box::new(move |x, y| g(x, y).into_iter().map(|r| map_row(r, &f)).collect()) as Box<dyn Fn(f32, f32) -> Vec<MenuRow<N>>>
        }),
    }
}

fn map_row<M, N>(r: MenuRow<M>, f: &Rc<dyn Fn(M) -> N>) -> MenuRow<N> {
    MenuRow {
        icon: r.icon, words: r.words, keys: r.keys, rule: r.rule, check: r.check, radio: r.radio,
        sub: r.sub.into_iter().map(|s| map_row(s, f)).collect(),
        does: match r.does {
            MenuDo::Copy(t) => MenuDo::Copy(t),
            MenuDo::Send(m) => MenuDo::Send(f(m)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{button, column, entry, switch};

    #[test]
    fn every_message_a_description_sends_goes_through_the_function() {
        let tree = column(vec![
            button("x", "Go").on_press(1u32),
            entry("abc", "Name", |s| s.len() as u32),
            switch(false, "Wi-Fi", |on| u32::from(on) + 10),
        ]);
        let mapped = tree.map(Rc::new(|n: u32| format!("m{n}")));
        let items = mapped.items();
        assert!(matches!(&items[0].what, What::Button { press: Some(m), .. } if m == "m1"));
        assert!(matches!(&items[1].what, What::Entry { change, .. } if change("xy".into()) == "m2"));
        assert!(matches!(&items[2].what, What::Switch { change, .. } if change(true) == "m11"));
    }
}
