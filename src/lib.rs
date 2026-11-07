//! The Trindle Toolkit: how Trindle's own windows wait, paint and show pictures, written for Trindle and using no other desktop's toolkit.

#[macro_use]
mod said;
#[cfg(feature = "core")]
pub mod common;
#[cfg(feature = "core")]
pub mod host;
#[cfg(feature = "ui")]
pub mod help;
#[cfg(feature = "ui")]
pub mod hint;

#[cfg(feature = "ui")]
pub mod action;
#[cfg(feature = "ui")]
pub mod ahead;
#[cfg(feature = "ui")]
pub mod badges;
#[cfg(feature = "ui")]
pub mod chips;
#[cfg(feature = "ui")]
pub mod combo;
#[cfg(feature = "ui")]
pub mod duration;
#[cfg(feature = "ui")]
pub mod find;
#[cfg(feature = "ui")]
pub mod rating;
#[cfg(feature = "ui")]
pub mod steps;
#[cfg(feature = "ui")]
pub mod badge;
#[cfg(feature = "ui")]
pub mod colour;
#[cfg(feature = "ui")]
pub mod config;
#[cfg(feature = "ui")]
pub mod desktop;
#[cfg(feature = "ui")]
pub mod dial;
#[cfg(feature = "ui")]
pub mod dock;
#[cfg(feature = "ui")]
pub mod dnd;
#[cfg(feature = "ui")]
#[cfg(any(test, feature = "testing"))]
pub mod drag_test;
#[cfg(feature = "ui")]
pub mod edit;
#[cfg(feature = "ui")]
pub mod editing;
#[cfg(feature = "core")]
pub mod bus;
#[cfg(feature = "ui")]
pub mod buttons;
#[cfg(feature = "ui")]
pub mod cellgrid;
#[cfg(feature = "ui")]
mod draw;
#[cfg(feature = "ui")]
pub mod charts;
#[cfg(feature = "ui")]
pub mod chooser;
#[cfg(feature = "ui")]
pub mod code;
#[cfg(feature = "ui")]
pub mod complete;
#[cfg(feature = "ui")]
pub mod emoji;
#[cfg(feature = "ui")]
pub mod emoji_picker;
#[cfg(feature = "ui")]
pub mod file_page;
#[cfg(feature = "ui")]
pub mod font_page;
#[cfg(feature = "ui")]
#[cfg(test)]
mod fuzz_tests;
#[cfg(feature = "ui")]
#[cfg(test)]
mod gallery;
#[cfg(feature = "core")]
mod sys;
#[cfg(feature = "core")]
pub mod event;
#[cfg(feature = "ui")]
pub mod frames;
#[cfg(feature = "ui")]
pub mod form;
#[cfg(feature = "ui")]
pub mod generic;
#[cfg(feature = "ui")]
pub mod grid;
#[cfg(feature = "core")]
pub mod history;
#[cfg(feature = "ui")]
pub mod pinch;
#[cfg(feature = "ui")]
pub mod icon;
#[cfg(feature = "ui")]
pub mod icongrid;
#[cfg(feature = "ui")]
pub mod info;
#[cfg(feature = "ui")]
pub mod input;
#[cfg(feature = "ui")]
pub mod kanban;
#[cfg(feature = "ui")]
pub mod entry;
#[cfg(feature = "ui")]
pub mod frame;
#[cfg(feature = "ui")]
pub mod global_keys;
#[cfg(feature = "ui")]
pub mod keep;
#[cfg(feature = "core")]
pub mod lang;
#[cfg(feature = "ui")]
pub mod link;
#[cfg(feature = "core")]
pub mod model;
#[cfg(feature = "ui")]
pub mod locked;
#[cfg(feature = "ui")]
pub mod motion;
#[cfg(feature = "ui")]
pub mod nav;
#[cfg(feature = "ui")]
pub mod notices;
#[cfg(feature = "ui")]
pub mod menubar;
#[cfg(feature = "ui")]
pub mod context_menu;
#[cfg(feature = "ui")]
pub mod dialog;
#[cfg(feature = "ui")]
pub mod bars;
#[cfg(feature = "ui")]
pub mod range;
#[cfg(feature = "core")]
pub mod notify;
#[cfg(feature = "core")]
pub mod numbers;
#[cfg(feature = "ui")]
pub mod mpris;
#[cfg(feature = "ui")]
pub mod once;
#[cfg(feature = "ui")]
pub mod pages;
#[cfg(feature = "ui")]
pub mod open_with;
#[cfg(feature = "ui")]
mod paint;
#[cfg(feature = "ui")]
pub use paint::Painter;
#[cfg(feature = "ui")]
pub use paint::service::main as draw_service;
#[cfg(feature = "ui")]
pub mod palette;
#[cfg(feature = "ui")]
pub mod photos;
#[cfg(feature = "ui")]
pub mod outline;
#[cfg(feature = "ui")]
pub mod panel;
#[cfg(feature = "ui")]
pub mod picker;
#[cfg(feature = "ui")]
pub mod picture;
#[cfg(feature = "ui")]
pub mod crash;
#[cfg(feature = "ui")]
pub mod crop;
#[cfg(feature = "ui")]
pub mod pill;
#[cfg(feature = "ui")]
pub mod role;
#[cfg(feature = "ui")]
pub mod player;
#[cfg(feature = "ui")]
pub mod popup;
#[cfg(feature = "ui")]
pub mod scene;
#[cfg(feature = "ui")]
pub mod rich;
#[cfg(feature = "ui")]
pub mod rows;
#[cfg(feature = "ui")]
pub mod print;
#[cfg(feature = "ui")]
pub mod portal;
#[cfg(feature = "ui")]
pub mod segments;
#[cfg(feature = "ui")]
pub mod settings;
#[cfg(feature = "ui")]
pub mod shadow;
#[cfg(feature = "ui")]
pub mod shortcuts;
#[cfg(feature = "ui")]
mod sharp;
#[cfg(feature = "ui")]
pub mod table;
#[cfg(feature = "ui")]
pub mod spelling;
#[cfg(feature = "ui")]
pub mod tabs;
#[cfg(feature = "core")]
pub mod tasks;
#[cfg(feature = "ui")]
pub mod term;
#[cfg(feature = "ui")]
pub mod text;
#[cfg(feature = "ui")]
pub mod theme;
#[cfg(feature = "ui")]
pub mod spin;
#[cfg(feature = "ui")]
pub mod showcase;
#[cfg(feature = "ui")]
#[cfg(any(test, feature = "testing"))]
pub mod testing;
#[cfg(test)]
mod unsafe_tests;
#[cfg(all(test, feature = "ui"))]
mod big_screens_tests;
#[cfg(all(test, feature = "ui"))]
mod list_cost_tests;
#[cfg(all(test, feature = "ui"))]
mod perf_checks_tests;
#[cfg(test)]
extern crate self as trindle_toolkit;
#[cfg(feature = "ui")]
#[cfg(test)]
mod template_tests;
#[cfg(feature = "ui")]
pub mod tiles;
#[cfg(feature = "ui")]
pub mod toolbox;
#[cfg(feature = "ui")]
pub mod ui;
#[cfg(feature = "ui")]
pub mod vector;
#[cfg(feature = "ui")]
pub mod validate;
#[cfg(feature = "ui")]
pub mod widget;
#[cfg(feature = "core")]
pub mod watch;
#[cfg(feature = "ui")]
pub mod when;
#[cfg(feature = "ui")]
pub mod zoom;
#[cfg(feature = "ui")]
pub mod window;

#[cfg(all(doctest, feature = "ui"))]
mod guides {
    macro_rules! guide {
        ($name:ident, $file:literal) => {
            #[doc = include_str!($file)]
            pub struct $name;
        };
    }
    guide!(Buttons, "../docs/guides/buttons.md");
    guide!(Choices, "../docs/guides/choices.md");
    guide!(Typing, "../docs/guides/typing.md");
    guide!(Lists, "../docs/guides/lists.md");
    guide!(Layout, "../docs/guides/layout.md");
    guide!(Progress, "../docs/guides/progress.md");
    guide!(Custom, "../docs/guides/custom-widgets.md");
}

#[cfg(all(doctest, feature = "ui"))]
#[allow(non_camel_case_types)]
mod widget_guides {
    include!("../docs/widgets/doctests.rs");
}
