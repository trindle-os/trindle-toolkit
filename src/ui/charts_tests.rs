//! Charts drawn in a real window: paint, pointer and what the screen reader is told.

use super::*;
use super::super::charts::Chart;
use super::super::theme::Theme;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Over(Option<usize>),
}

fn ui_of(chart: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(chart.key("chart"), Theme::new(true, "#fbc711"));
    ui.size = (400, 200);
    ui.settle();
    ui
}

fn shot(ui: &mut Ui<Msg>) -> tiny_skia::Pixmap {
    let mut pix = tiny_skia::Pixmap::new(400, 200).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    pix
}

fn at(x: f32, y: f32) -> Input {
    Input::PointerMove { x, y }
}

fn kinds(values: &[f32]) -> Vec<Chart> {
    vec![Chart::line("Visits", values), Chart::area("Visits", values), Chart::columns("Visits", values), Chart::spark("Visits", values),
         Chart::pie("Visits", values), Chart::ring("Visits", values)]
}

fn labelled(ui: &mut Ui<Msg>) -> Vec<String> {
    ui.access("").nodes.iter().filter_map(|(_, n)| n.label().map(str::to_string)).collect()
}

#[test]
fn every_chart_paints_something_and_nothing_painted_is_the_bare_card() {
    let blank = shot(&mut ui_of(super::super::widget::space()));
    for chart in kinds(&[3.0, 8.0, 5.0, 9.0]) {
        let mut ui = ui_of(chart.node());
        assert_ne!(shot(&mut ui).data(), blank.data());
    }
}

#[test]
fn a_chart_takes_its_colour_from_the_highlight() {
    let mut a = ui_of(Chart::columns("Visits", &[1.0, 2.0, 3.0]).node());
    let mut b = ui_of(Chart::columns("Visits", &[1.0, 2.0, 3.0]).node());
    b.theme = Theme::new(true, "#2288ee");
    assert_ne!(shot(&mut a).data(), shot(&mut b).data());
}

#[test]
fn odd_numbers_never_stop_a_chart_painting() {
    for values in [vec![], vec![f32::NAN, f32::INFINITY, -4.0], vec![0.0, 0.0], vec![5.0], vec![-1.0, -2.0, -3.0], vec![1e30, -1e30]] {
        for chart in kinds(&values) {
            let mut ui = ui_of(chart.hover(Some(0)).node());
            shot(&mut ui);
        }
    }
}

#[test]
fn the_screen_reader_is_told_the_name_the_kind_and_the_figures() {
    let said = |c: Chart| labelled(&mut ui_of(c.node()));
    assert!(said(Chart::line("Visits", &[3.0, 8.0, 5.0])).contains(&"Visits: line chart, 3 values, first 3, last 5, lowest 3, highest 8.".to_string()));
    assert!(said(Chart::columns("Sales", &[2.5])).contains(&"Sales: column chart, 1 value: 2.5.".to_string()));
    assert!(said(Chart::spark("Load", &[1.0, 2.0])).iter().any(|s| s.starts_with("Load: sparkline, 2 values")));
    assert!(said(Chart::area("Load", &[1.0, 2.0])).iter().any(|s| s.starts_with("Load: area chart")));
    assert!(said(Chart::line("None", &[])).contains(&"None: this chart has no values.".to_string()));
}

#[test]
fn a_pie_and_a_ring_say_each_part_with_its_share() {
    let labels = vec!["Rent".to_string(), "Food".to_string()];
    let pie = Chart::pie("Spending", &[50.0, 30.0, 20.0]).labels(labels.clone()).node();
    assert!(labelled(&mut ui_of(pie)).contains(&"Spending: pie chart, 3 parts: Rent 50%, Food 30%, Part 3 20%.".to_string()));
    let ring = Chart::ring("Spending", &[1.0]).labels(labels).node();
    assert!(labelled(&mut ui_of(ring)).contains(&"Spending: ring chart, 1 part: Rent 100%.".to_string()));
}

#[test]
fn a_hovered_value_is_shown_and_changes_the_picture() {
    let (mut plain, mut over) = (ui_of(Chart::line("Visits", &[3.0, 8.0, 5.0]).node()), ui_of(Chart::line("Visits", &[3.0, 8.0, 5.0]).hover(Some(1)).node()));
    assert_ne!(shot(&mut plain).data(), shot(&mut over).data());
    let mut out_of_range = ui_of(Chart::line("Visits", &[3.0, 8.0, 5.0]).hover(Some(9)).node());
    assert_eq!(shot(&mut out_of_range).data(), shot(&mut plain).data());
}

#[test]
fn the_pointer_tells_which_value_it_is_on_and_only_when_that_changes() {
    let mut ui = ui_of(Chart::columns("Visits", &[3.0, 8.0, 5.0, 2.0]).hovering(Msg::Over));
    shot(&mut ui);
    let now = Instant::now();
    assert_eq!(ui.input(&at(10.0, 100.0), now).messages, vec![Msg::Over(Some(0))]);
    assert!(ui.input(&at(20.0, 100.0), now).messages.is_empty(), "the same value says nothing");
    assert_eq!(ui.input(&at(390.0, 100.0), now).messages, vec![Msg::Over(Some(3))]);
    assert_eq!(ui.input(&Input::PointerOut, now).messages, vec![Msg::Over(None)]);
}

#[test]
fn a_line_picks_the_nearest_point_and_a_ring_the_part_under_the_pointer() {
    let mut line = ui_of(Chart::line("Visits", &[1.0, 2.0, 3.0]).hovering(Msg::Over));
    shot(&mut line);
    let now = Instant::now();
    assert_eq!(line.input(&at(200.0, 100.0), now).messages, vec![Msg::Over(Some(1))]);
    let mut ring = ui_of(Chart::ring("Spending", &[1.0, 1.0]).tall(160.0).hovering(Msg::Over));
    ring.size = (160, 160);
    ring.settle();
    ring.paint(&mut tiny_skia::Pixmap::new(160, 160).unwrap().as_mut(), 1.0);
    assert!(ring.input(&at(80.0, 80.0), now).messages.is_empty());
    assert_eq!(ring.input(&at(130.0, 80.0), now).messages, vec![Msg::Over(Some(0))], "the right is the first half");
    assert_eq!(ring.input(&at(30.0, 80.0), now).messages, vec![Msg::Over(Some(1))], "the left is the second");
}
