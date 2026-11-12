# Changelog

## Unreleased

- `Node::lines(n)` makes an `entry` a box of many lines, at least `n` tall: words wrap at its width, Enter starts a line, Ctrl+Enter sends `on_submit`, Up, Down, Home and End go by line, and it grows with its words, then scrolls to keep the caret in view. Undo, the input method, spelling marks and the pointer work as in the one-line box, pasted lines are kept, and a screen reader hears a multi-line text box. A password box stays one line.
- The size budget for the smallest program counts only the part loaded to run (18 MB), not the symbol names, which crash reports need and which newer compilers make longer: the same program was 20.8 MB on Rust 1.85 and 22.5 MB on 1.98 while its code got smaller.
- A window is laid out again when fontconfig's answer or the computer's fonts come in, not only drawn again, so words first set in the stand-in font that now wrap or grow push what follows down instead of lying over it.
- A card is solid to the pointer: a press on a card floating over a button, such as a menu card, no longer reaches the button under it, with a finger too.
- `tools/release-build.sh` builds the toolkit's programs with no path from the build computer in them (`--remap-path-prefix` for the home, checkout, cargo and target folders); under `--cfg trindle_shipped`, `trindle-lang-check` no longer looks in the folder cargo fetched the toolkit to, the one path it held. RELEASING.md gives the same flags for programs built on the toolkit.
- A window the graphics chip draws tells the compositor it is opaque only when it is solid (`Window::set_solid`; windows start not solid). A shown window is solid unless it is see-through, fading or its page colour is see-through, so a see-through window such as a haze over the desktop no longer shows black.
- Windows are drawn by the graphics chip by default (`DrawnBy::Auto`) when every check passes, else by the processor. `DrawnBy::Processor` or `TRINDLE_DRAWN_BY_PROCESSOR=1` keeps a program on the processor.
- `Shown::run` gives back `Result<(), String>`: with no desktop or no window it returns the words saying so instead of ending the program, so the program's own clean-up runs. The examples and `trindle-new`'s template print the words and end with 1, as before.
- The `gpu` feature is on by default, so wayland-client goes through libwayland-client and `GpuSurface` has its raw pointers in every program. A program that wants a Wayland connection in pure Rust builds with `default-features = false, features = ["ui"]`.
- A window's page colour is `--trindle-<page>-page` for the last part of its app id, with no exception: the store's page is `--trindle-store-page`, and the sheet's `--trindle-apps-page` is renamed to it.
- Moving a file to the Trash on another drive makes the person's own folder in the drive's shared `.Trash` and checks it is a real folder that is theirs before using it; otherwise the drive's `.Trash-<uid>`, and if that is not theirs either, the unsafe error.
- `copy_named` refuses to copy a file onto itself, by the same name, a hard link or a symbolic link, with `EINVAL`, before anything is cut short.
- `xdg_open` passes no `--`, which xdg-open 1.2.1 refuses, and puts `./` before a relative name so it cannot be read as an option.
- The tests and `trindle-lang-check` no longer look for Trindle's old layout with the toolkit in its `crates` folder; the tests that read Trindle's source find it only where `TRINDLE_SOURCE_ROOT` says.
- The source is split into smaller files, none over 1,000 lines without comments, and long functions into named stages. Public items and behaviour are unchanged, checked against the documents' page and item lists. Four long functions that are mostly tables stay as they are.
- Under `testing`, `reach::ANSWER_WITHIN` is one second rather than 120 ms, so a busy computer is not taken for a silent share.
- `Bus::session` refuses, under `testing` or in tests, when the bus it would reach is the login's own or none is named, and says to use `dbus-run-session`: a test can no longer wake the person's keyring.
- `Node::on_drop_where(takes, f)`: as `on_drop`, where only some rows or choices take what is held; the others are not lit and letting go on them sends nothing.
- Text is shaped by HarfBuzz's own Rust port, `harfrust`, in place of `rustybuzz`, which is no longer looked after; `ttf-parser` goes with it. Bold in a variable font now kerns as HarfBuzz does, and a layout takes about a fifth of the time.
- `emoji::has_pictures` and `emoji::picture` take a `skrifa::FontRef`, and `picture` the place in the font's weights as well. Colour letters now draw sweeps the right way round, repeating and reflecting gradients, and nothing where the format says nothing is drawn.
- `tasks::Tasks::spawn_catching` and `spawn_sending_catching`: a panic in the work is delivered as a message made from its words, so a program waiting for the answer is not left waiting. `spawn` and `spawn_sending` are as they were.
- `ui::save_splits` and `load_splits` keep any key: `%`, `;`, spaces and control characters in a key are written as `%` and hex. A line saved before reads as it did unless a key held `%` and two hex digits.
- `table::Edited` has two new variants, `Copy(words)` and `Paste`, for Ctrl+C, Ctrl+X, Ctrl+V and Shift+Insert in a cell being typed in, and `Table::paste(words)` puts the clipboard's words in. A program that matches `Edited` without a catch-all arm needs two arms.
- `rich::parse` and `markup`: code can hold a backtick (`` \` ``) and a link address a `)` (`\)`); a backslash before a backslash is one in both. Code that held two backslashes in a row, written before this, reads as one.
- `spin::Spin` and `spin::number_text` show and read numbers the language's way (decimal mark, thousands mark, figures); "1.234,5" is 1234.5 in German and "1,234.5" in English; a thousands mark alone with other than three figures after it ("3,4" in English) is still read as the point.
- `common::ui::click_again` makes the first letter small only where the language file says so (`kit-click-again-small`, yes or no); German, Turkish and the languages without capitals say no. The sixteen translations were not reviewed by a speaker (marked in each file).
- `config::Config::read(key)` tells a key never set (`Ok(None)`) from one that cannot be read (an error); `get` is as it was.
- The find bar's count is as wide as its words need.
- The style sheet holds the sizes of the tab bar (`--trindle-tab-*`), the notice pill (`--trindle-pill-*`), a menu's group gap, edge and least row and a menu bar's title room (`--trindle-menu-*`) and the colour picker's swatch corner, with the values the code had. `pill::band()` gives the strip's height by the sheet; the constants `pill::HEIGHT`, `WIDEST`, `ROOM`, `FOOT` and `BAND` stay as the sheet's own start. `tabs::MORE` and `PAD` became functions (they were not public).
- `link::read` reads full-width letters in a host as the plain ones, as a browser does, so the label says where the link goes.
- `cellgrid::Screen::cell` and `cell_mut` panic on a column past the end, as they do on a row; before, such a column read as the start of the next row.
- Words dragged out of a text box and let go on it again are copied, not moved, when the person chose a copy while a move was on offer.
- Reading the style sheets again is done one at a time, so a burst of saves cannot leave the older one in use.

## 0.9.0-beta.1, 26 October 2026

The first public release. Nothing is promised stable; the toolkit changes as the programs built on it need.

Added for the first version:

- `trindle-draw`, the drawing service: one copy of the graphics driver drawing for every program of a user that asks with `TRINDLE_DRAWN_BY_SERVICE=1`. Off by default until it has been tried. See "The drawing service" in the README.
- `Node::context_menu`: any widget, and everything in it, gets the toolkit's own floating menu on a right click, a finger held still, the Menu key or Shift+F10 (first row lit), or a screen reader's show-menu action. The nearest one wins; text boxes, and drawings and lists with menus of their own, keep theirs; a greyed node offers nothing. Example `context_menu`.
- `dialog::confirm`, `choose` and `ask`: ready-made dialog pages for a yes or no, one of several answers, and an answer typed in; the keyboard starts on Cancel (in the box for `ask`), and Escape cancels. A dialog card in `gallery`.
- `range::Range` is at least 24 points high and tells a screen reader a square of at least 24 points for each handle; `bars::progress_bar` also tells its share in words ("50%"). A progress bar and range card, and a sideways strip card, in `gallery`.
- Menus: radio rows (`popup::Row::radio`, `MenuRow::radio`) drawn with a dot and told to a screen reader as menu radio items, and type-ahead (a letter or figure lights the next row that starts with it) in every menu, the menu bar's and the context menu's. Examples `menu_bar` and a context menu card in `gallery`.
- Quality checks now also draw every part in every state under high contrast and under large text, and check the keyboard's ring on every part it reaches; `cargo test -- --ignored shoot_the_states` writes the page of every part to look at. What they found: in high contrast the highlight was not raised for the keyboard ring and the edge under the pointer, so on a light page it stood only 3.1 to one; `Theme::lit_edge` is the highlight held to 4.5 to one there.
- Colour-blind safe palette: `--trindle-colour-blind-safe: 1` in a sheet, or `theme::colour_blind_safe(true)` in a program, draws the states and a pie's pieces in Okabe and Ito's colours (`--trindle-safe-*`), told apart in deuteranopia, protanopia and tritanopia (a test simulates each). Off, nothing changes.
- `text::built_in_fonts_only()`: every word in the built-in fonts alone, with the usual hinting and smoothing, whatever is installed and whatever fontconfig says, for a picture that is the same on every computer. `tests/golden/words.png` is drawn so and compared within the tolerance of the others.
- `bars::Tool::toggle(on)`: a toolbar tool that stays pressed, as Bold and Italic do; a soft square behind it while on, a tick on its More menu row, told to a screen reader as pressed. `Role::Toolbar` has a new `on` field.
- Week numbers in the date grid: `weeks` on `when::DateBox` and `RangeBox` puts each row's ISO 8601 week down the left, read as "Week 44"; `Date::week()`.
- `file_page::FilePage` and `font_page::FontPage`: a file chooser (open, save or a folder) and a font chooser, each a dialog page in the window, worked by keyboard, with red words under the list when an answer cannot be taken.
- Built-in fonts, so words look and measure the same on every computer: Noto Sans (regular, bold, italic, bold italic), Noto Sans Mono (regular, bold) and Noto Sans Arabic (regular, bold), 3.6 MB, under the SIL Open Font Licence 1.1 (`share/fonts/OFL.txt`). Text is Trindle Sans if installed, then the built-in Noto Sans; Arabic letters come from the built-in Noto Sans Arabic before any installed font; other scripts and missing letters from the computer's fonts. The fixed-width font is the person's own fontconfig monospace font if it is installed and fixed width, else the built-in Noto Sans Mono (`text::BUILT_IN_MONO`); the test harness always uses the built-in one so pictures match on every computer. DejaVu is no longer named anywhere.
- Named steps in the style sheet: a gap scale (`none` to `huge`, whole numbers of a 2 point unit) that the toolkit's fixed gaps and padding read, and size steps (`smaller` to `larger`) for the larger parts' sizes. Only Trindle's own sheet gives numbers for them; any other value is passed over, and `trindle-sheet-check` names it with its steps. `theme::gaps()` gives the scale. Radio buttons and tick boxes grow with large text.
- `docs/style-sheet-for-ai.md`: every key of the style sheet, what it takes and what it changes, with plain rules and a worked example for an AI writing a sheet.
- The `gpu` feature, off by default: `GpuSurface::display_ptr` and `surface_ptr` give the raw Wayland pointers EGL, Vulkan or wgpu need. It turns on wayland-client's `system` feature, so only a program that asks for it links libwayland-client; the default build stays pure Rust. Tested against the stand-in compositor, not yet with a GPU library on a real compositor.
- `Painter`: drawing recorded as a list and played onto a canvas byte for byte as drawing straight onto it. Every public paint function that takes a canvas (cell grid, terminal, board, tabs, table, icon grid, crop box, squiggle, pill and float shadow, `ui::fill`, labels, scene, rich words, the text editor, video frames) has an `_on` version that takes a `Painter`, and `tiles::Sketch` has `fit_on`, `strip_on` and `placed_on`; the canvas versions are unchanged for their callers and, but for `put_frame`, record through the new ones (docs/reference.md, "Drawing on a Painter").

- Drawing with the graphics chip: a window is drawn by an OpenGL ES 3.0 drawer into linux-dmabuf buffers when the machine allows it, and by the processor (tiny-skia) otherwise, with no setting needed. The libraries are opened at run time, so the default build still links nothing new. It falls back to the processor when the compositor takes no Argb8888 dmabufs, a library or render node is missing, the driver is software, a small test frame differs from the processor's by more than two levels, or a GL error or lost context happens later. `TRINDLE_DRAWN_BY_PROCESSOR=1` keeps every window on the processor. Only the spots that changed are drawn again and told to the compositor (damage, counting what each buffer missed), a fence tells the compositor when a frame is done, and the speed overlay (Ctrl+Shift+F11) says which drawer made the last frame. Figures and machine in MEASUREMENTS.md.

- Which drawer draws a window, said and chosen: every window prints `trindle: <program> window drawn by <gpu|processor> (<why>)` on standard error when it opens; `ui::drawn_by(DrawnBy::{Auto, Processor, Gpu})` lets a program choose (the environment variables still win; `Auto` is still the processor); `Drawer::Processor` now holds a `Why`. The graphics chip's atlases start at 512 x 512 and double to 2048 x 2048 before being emptied, and while every window is hidden or minimised its buffers, atlases and context are freed and made again on show, which lowers the program's memory.

- Example `bench`: a repeatable benchmark of the two drawers in a real window (a 10,000-row list scrolled at a set speed, a spinner, a progress bar, a clock and a changing text box), printing frame times, CPU time and memory, with `--both` to run each drawer and compare. It uses `Shown::drawer` (which drawer was chosen and, for the processor, why) and `Shown::on_painted` (each frame's start, time, size, scale and drawer).

- Quality checks: every part in every state is drawn in every look (dark, light, bold, the high-contrast look, and the other themes) at a wide and a narrow window with nothing cut off; three stored pictures in `tests/golden` compared pixel by pixel (`TRINDLE_GOLDEN=refresh` rewrites them); a fixed-seed fuzz test of random keys, composed words and pointer moves on a text area and a form; and a generous speed and memory budget for the gallery.
- Guides: one short page per widget group in `docs/guides` (their examples are doctests), a page on adding your own widget kind with `drawing`, and `docs/from-gtk-and-qt.md`, a table of GTK and Qt widget names against ours. `RELEASING.md` is the release checklist.
- `trindle-lang-check --missing [language]` lists each language's share of English done and every word it lacks.
- Japanese (`ja`) and Korean (`ko`) language files, machine drafted and marked in their first line for a native speaker to review; the toolkit now ships 17 languages (English and 16 others). Both start the week on Sunday and have only the `other` plural form.
- `rich(spans)`: bold, italic, code and links in one wrapped paragraph (`rich::Span`, `rich::parse` and `rich::markup` for a small markup); links in the highlight's colour, never underlined, followed by pointer, Tab and Enter, or a screen reader. The layout cache now tells italic and family apart.
- `editing::rich_box::RichBox`: a box of styled words, Ctrl+B, Ctrl+I and Ctrl+E, undo, saved as markup.
- Right-to-left writing: a text box's words stand at its right, the arrows move the caret the way they point in a text box, picked words and a text area, and a selection across both directions is washed band by band. `edit::right_to_left`, `edit::facing`.
- Figures by region, as the Unicode CLDR does: Eastern Arabic figures for ar and ar-SA, ar-EG, ar-IQ and the like, 0 to 9 for ar-MA, ar-DZ, ar-TN, ar-LY, ar-AE and ar-EH; Persian figures for fa and ps; Bengali, Devanagari (mr, ne), Burmese and Dzongkha their own; hi and ur 0 to 9 (ur-IN Persian figures). `lang::numerals::Numerals` (auto, latin, native) is the person's choice, answered from `Host::numerals()` and kept with `Numerals::save` and `read`; `settings::numerals_row` is the settings row. `lang::numerals::latin(|| ..)` and `Spin::latin()` write one field in 0 to 9; `Mask::Phone` and `Mask::Date` keep 0 to 9. Everything typed (numbers, number boxes, time, duration and date boxes, segments, phone, date) takes either set of figures or a mix, and chart words and spin boxes write the language's figures. Changed: ar-MA, ar-DZ, ar-TN, ar-LY, ar-AE and ar-EH now write 0 to 9 where they wrote Eastern Arabic; no shipped language changes otherwise.
- Figures per language: `kit-number-digits` (Arabic and Bengali their own), used by `lang::format::number`, `decimal` and sizes; `numbers::percent` (`kit-number-percent`) and `numbers::read`; the date grid starts the week on `kit-date-first-weekday` and writes the language's figures; `when::Date::written` and `when::Time::written`. Arabic's decimal and thousands marks are its own (٫ and ٬).
- `complete::Complete`: suggestions in a list under a text box, Up and Down while typing (`Node::on_up_down`), Enter, Escape, a combo box and a count for the screen reader.
- `code::view`: read-only code with line numbers, a small tokenizer for Rust and shell, and `--trindle-code-keyword`, `-type`, `-string`, `-number` and `-comment` in the sheet; `Theme::ink_on` holds any sheet colour readable on its ground.
- `help::markdown`, `help::html` and `help::view`: help pages of headings, paragraphs, lists, links, code and pictures' words.
- Actions (`action::Actions`): named actions with words, icon, shortcut and an enabled state, shared by menus, toolbars and the command palette (`palette::Palette`, a page).
- Checked boxes (`validate::CheckedBox`) for a phone number, a date and a hex colour, with red words in a line kept under the box.
- `dial::dial`, `dial::level_bar`, `dial::scale` (a slider with marks) and `dial::volume`, told to a screen reader as sliders with their numbers.
- `buttons::colour_button`, `font_button`, `file_button` and `lock_button`.
- `spin::spin`, `spin::time_spin` and `spin::month_year`; settings rows `rows::toggle_row`, `combo_row`, `entry_row` and `spin_row`.
- `dock::Dock` (panels at the sides, the foot or floating, moved by buttons or by dragging the handle on its head, with a line in the highlight at the edge it would land on; `Dock::update` takes a `dock::Change` and returns the `Moved` panel), `toolbox::toolbox`, `toolbox::tab_pages` (tabs inside tabs) and `info::info_panel` in place of banners and toasts.
- `model::ListModel` (sort, filter, group, change events) and `form::Form` (a form tied to a record).
- A test of Arabic's six plural forms.
- The desktop's light or dark setting is read from the desktop's own mode file where there is no settings portal, and followed live.
- `config::Config`: settings as a file per key in a versioned folder, as desktop settings systems keep them, with a watcher that names the keys changed.
- `Layer::Background` and `Layer::Bottom`, ready-made `Panel::bar`, `dock`, `display` and `wallpaper`, and `Panel::popup` to place an applet's popup beside its panel button.
- `notify::Heard::Pass`, the pass a notification gives to bring the window forward, and `Note::reply` with `Heard::Replied` for an answer typed in the notification.
- Tablet pens (`Input::Pen` and `PenOut`, with pointer events as a mouse makes) and `Window::hold_pointer` with `Input::PointerBy`.
- `global_keys::GlobalKeys` (the global shortcuts portal) and `badge::Badge` (a count, progress and urgency on the dock icon).
- `mpris::Media`: the player's controls offered to the desktop over MPRIS.
- `frame::framed` and `Window::needs_frame`, `begin_move` and `begin_resize`: a title bar with window buttons and resize edges where the compositor draws none, with `--trindle-title-bar-room` and `--trindle-frame-edge`.
- The self-drawn title bar is finished: a double press maximises or puts back, a right click asks for the compositor's window menu (`Framing::Menu`, `Window::show_window_menu`) or opens the toolkit's own, and a shadow falls round the window in `--trindle-frame-shadow` (`Window::set_shadow`, `Window::frame`, `Shown::frame_shadow`), kept out of its geometry and input region and dropped when maximised, full screen or tiled. `framed` now takes a `Frame`. The bar paints the title, so presses on the words move the window, and no gap is left between the edges and the bar. `Node::on_double_press` for drawings.
- `trindle-common`, a small crate in the same repository with no window, Wayland or bus code, for programs that do not want the toolkit: home and XDG folders (a relative XDG path is ignored, as the specification asks), a private file written whole, the settings file, the plugin notify line, a WAV header and the `build.rs` helper that lists the English word files.
- Every kind of button (icon, worded, quiet, word, segmented strip) is one height, `--trindle-button-height`; `--trindle-main-button-height` and `--trindle-word-button-height` are gone.
- `--trindle-ring-corner` (the one corner by default): a ring button (a tinted one) is a soft square with it, and Trindle draws the ring's app tiles with the same value.
- Nothing is a disc or pill but the exceptions: the eye button, the label on pointing, the badge, the person's picture and its edge (`edge_round`) are soft squares with the one corner. Still round: progress rings, spinner, the working line, the colour picker's marker, and the ends of thin slider and scroll lines (a thin line cannot have a corner bigger than half its width).
- One corner per kind of part in the sheet (`card`, `box`, `row`, `float`, `button`, `icon-button`, `ring`, `switch`, `knob`, `slider`, `tab`, `badge`, `chip`, `segmented`, `tooltip`, `scrollbar`, `picture`, each `--trindle-NAME-corner`), all starting as `--trindle-corner`; 0 is square and 999 fully round. No part reads a literal radius; `theme::Part`, `Sheet::corner_of` and `theme::corner_px` are the one helper. `Sheet::panel_radius`, `control_radius`, `row_radius`, `float_radius` and `ring_corner` are gone.
- `Part::Tick` (`--trindle-tick-corner`, 4, so a tick box stays a box) and `Part::Radio` (`--trindle-radio-corner`), plus a `tiles::tick_box(on)` widget; radio buttons now follow the sheet corner.
- `tick(on, name, change)`: a working tick box (press, Tab and Space, check box for the screen reader, hover edge, touch target), drawn with `--trindle-tick-corner`.
- `tree(rows, chosen, choose, toggle, open)`: a tree view built on the list (rows in sight, scrolling, hover and chosen look), with `tree_row(item, depth).branch(open)`. Each branch has an arrow to open or close it; Left and Right open, close and move between levels; Enter, Space and a double click open a branch or a leaf. The screen reader hears a tree with each row's level and whether it is open, and can open or close it. One new sheet value, `--trindle-tree-indent` (20, 8 to 40).
- Menus: a right-click menu row can show a tick (`MenuRow::checked`, `popup::Row::check`, a check item to the screen reader) and open a sub-menu beside it (`MenuRow::submenu`); the Menu key, Return and Space open a drawing's menu, and a left press a drawing sends nothing for opens it there (a toolbar's More).
- `menubar::MenuBar`: a menu bar across the window's top, its menus opening under their titles, with the arrows, Escape and F10, told as a menu bar.
- `spin::Spin`: a number box with typing, Up and Down, Page keys, the wheel, minus and plus buttons, limits, a step and a unit, told as a spin button; `Node::on_step` gives any text box the stepping keys.
- `dialog::dialog`: a dialog as a page in place of the window's page, with a title, lead sentence, content and answers, Cancel first, Escape cancelling, told as a dialog.
- `bars::progress_bar`: a bar in the soft highlight, with a known share or crossing while the end is unknown (still for less motion), told as a progress indicator.
- `range::Range`: a slider with two handles that never cross, told as two sliders.
- `bars::Toolbar`, `bars::status_bar` and `bars::path_bar`: tools that go into a More menu when they do not fit, the foot's status line, and a path whose long middle folds into a list.
- Gender in language files: a `[male]` / `[female]` / `*[other]` selector chosen by `gender = Gender::Female` (`lang::Gender`), alone or inside a plural choice; a translation may leave the choice out, or add it where English has none.
- A list with no name (`.called(..)`) is reported by `Ui::unnamed_lists` and warned of while a test build paints it; the gallery test checks every list has one.
- `segments::SegmentedTime`: a time in one box as two segments (hours and minutes) with keys, typing, a press and two spin buttons for the screen reader (`Role::Segments`).
- Menu bar: Alt with a letter opens the menu whose title has it (`MenuBar::alt`, `BarMsg::Alt`; a title may mark its letter with `&`, else the first letter is used); the letter shows in the highlight colour, never underlined.
- A toolbar names each tool when the pointer rests on it, under that tool (the same label on pointing as any widget); the screen reader can click a tool, and step (Increment, Decrement) or take either handle of a range slider.
- `split(key, first, second)` and `split_down`: two areas with a divider dragged between them, moved by the arrows, Home and End with the keyboard on it, put back by a double click, kept by key across views, each side at least `.least(...)` (the second squeezed first), started by `.start(Start::Part(f) or Start::Points(p))`, and told as a splitter. `Ui::split_places` and `set_split_places` (also on the window) and `save_splits` and `load_splits` keep the places in a settings file. Two sheet values, `--trindle-divider` (1, 1 to 4) and `--trindle-divider-lit` (4, 2 to 8), and a `RowResize` cursor.
- `scroll_x(key, inner)`, a sideways `scroll`: wheel with Shift, a sideways wheel or touchpad move, and a bar along the foot that looks and measures like the vertical one (same sheet values and scrollbar corner). It reaches the screen reader as a scroll view with left and right actions. Touch swipe and keys do not move it yet.
- Keyboard reveal and `Node::seek` work along a `scroll_x` (the part's start and length are then across, not down), so Tab in a wide row scrolls to the focused item; `Window::scrolled(id)` reads an area's offset in pixels, and `scroll_area(id, v)` sets it, down or along.
- `scroll_x` measures what it holds up to 8,000,000 points wide (was 100,000), so a very long row of fixed-width columns scrolls to its end.
- `nav`: a stack of pages with its switcher as a strip or a side bar (`stack`, `stack_beside`, `page_key`); a navigation view with history and a Back button (`History`, `nav_view`, `back_key`); a split view that shows one side at a time below `--trindle-split-narrow` (`split_view`, `is_narrow`); a drawer laid over a page from either edge and a bottom sheet (`drawer`, `bottom_sheet`).
- `flow(items)`, items at their own size wrapping like words; `aspect(ratio, inner)`, kept in shape in the middle of its room.
- `icongrid`: a grid of icons you paint with several chosen at once, keys, a rubber band and a screen reader table; `Selection`, the many-chosen model it uses.
- `pages`: a preferences window, a status page, a welcome page and an about page.
- `frames`: a named frame, a fixed layout, corner badges, a fading revealer, page dots and a pager of pages that slide over them (`frames::pager`).
- Kinetic scrolling: a swipe or touchpad flick coasts on and slows (`--trindle-fling-friction`); never with less movement asked for.
- `scroll_x(key, inner)`, a sideways `scroll`: wheel with Shift, a sideways wheel or touchpad move, and a bar along the foot that looks and measures like the vertical one (same sheet values and scrollbar corner). It reaches the screen reader as a scroll view with left and right actions.
- `scroll_x` by finger and keys: a swipe more across than down scrolls it, Left and Right a notch, Shift with the Page keys a page and with Home and End to either end, and Tab brings the widget it reaches into sight along it.
- `list(..).on_move_in(f)`: rows dragged from one list into another, with the line showing the gap and a message naming the list, row and gap.
- `Shortcuts::clashes()`, `Chord::kept()` and `Shortcuts::page(says)`: the chords the toolkit takes first are reported and shown on a page of every shortcut; named keys come from the language file.
- `history::History` and `Ui::on_undo(undo, redo)`: undo and redo across a window, after a text box's or text area's own steps run out.
- `pinch::Pinch`: two fingers as a pinch (zoom) and a twist (rotate) from the touch events.
- A touchpad's pinch and twist from the compositor's pointer gestures, as `Input::PadPinch` and `Input::PadPinchEnd`, which `pinch::Pinch` also takes.
- `landmark(kind)` for a screen reader's landmarks; a ring with `polite()` speaks its progress; a test that every focusable widget in the gallery has a name.
- `entry(..).purpose(p)` tells the input method what a box is for, for an on-screen keyboard; `emoji_picker::emoji_page`, a page of emoji found by name. The page has 176 emoji in eight groups; names come from the English language file, and another language falls back to them until it adds its own.
- Small widgets: arrow keys in `tiles::radio_group`, `combo::combo`, `chips::Chips` (with `Node::on_backspace_empty`), `rating::rating`, `duration::DurationBox`, `when::RangeBox`, `steps::wizard`, `find::Find`, `tiles::empty_picture` and `tiles::skeleton`, `badges::badge_count` and `badges::avatar_group`, `hint::help_over`, `picker::swatch_grid` and `picker::GradientBox`. In a radio group Tab stops only on the chosen row.
- `charts::Chart`: line, area, columns, sparkline, pie and ring charts from numbers and a name, drawn in the sheet's colours, with a screen reader summary of the figures and an optional hover value (`--trindle-chart-line` is the line's thickness).
- Picture viewer extras: `View::turn_right` and `turn_left` (quarter turns, shown whole), `Crop::turn_right` and `turn_left`, `picture::turned` and `picture::cropped`, and a fling that glides on (`Drag::pointer_at`, `glide`, `gliding`), off with reduced motion. `View::pan_by` returns how far it moved.
- `photos::gallery`: a large picture with arrows, a count and a strip of thumbnails, arrow keys, and "name, n of total" for the screen reader.
- The clipboard beyond text: `Window::copy_image` and `paste_image` (PNG), `copy_files` and `paste_files` (`text/uri-list`, with the file managers' copy note and plain paths), and `clipboard_has`.
- Tools for program authors (see "Tools for program authors" in the reference): the inspector (Ctrl+Shift+F12), the speed overlay (Ctrl+Shift+F11), problems in a saved own sheet written to the log, `trindle-sheet-check` (limits and WCAG 2.2 AA), the `testing::Harness` with its guide, `trindle-new` from a template the tests build, `showcase` and `examples/widgets.rs` with each widget's code beside it, plural lines a language never uses caught by the language check, an unsafe ledger, start and size budgets, a zero-config accessibility test and scale tests from 1.0 to 4.0.
- Drag and drop of data of any kind between programs: `Lifted::typed` offers `(mime, bytes)` pairs, and `Node::on_drop_of(kinds, would, take)` takes the first of its kinds that is offered as `Given::Typed(kind, bytes)`. `dnd::Kind::Typed` is new, so `Kind` is no longer `Copy`, and `Input::DragIn` carries the kinds offered. Widgets that take files or text are no longer lit by an offer of neither. The widget under something held is told to the screen reader as "Drop target".
- A band dragged across a list with `.many` picks every row it crosses. It begins in the room below the last row, or anywhere with Ctrl held (adding to what was picked), so it never takes a plain press meant for dragging a row; it is washed and edged in the highlight.
- One way to tell a screen reader what a drawing or group is: `Node::role(role::Role::..)` (button, check box, switch, slider, spin button or meter with its number, image, group, dialog, menu bar, toolbar, status line, path, two-handled slider, segments) and `Node::value(words)`; the toolkit's own dial, spin box, menu bar, toolbar, range slider and segmented time use it too; the reader steps a slider or spin button that takes keys by sending it Up and Down. The custom widgets guide shows a star rating told as a slider.
- The many-line text area checks spelling: `TextArea::spelling(Some(speller))` takes the same `spelling::Speller` as a text box, draws the same wavy line in `--trindle-spelling` (it was in the warning colour), and tells the screen reader of spelling mistakes in sight (`Node::misspelt`).
- `side_heading`, one sheet value (`--trindle-side-heading-size`, 11) for a side bar group's name, shared by Folders, Apps and Settings.
- Column view: `Table::set_tree`, `Branch`, `open_row` and `close_row` make a table whose rows open and close under a parent, set in by depth, with a 24 point control, arrow keys, Enter and double click, and a tree grid for the screen reader (level, open, place among siblings); a program builds a branch's rows only when it is opened. A column view card in `gallery`.
- Tables with several rows chosen, off by default: `Table::multiple`, `picked`, Ctrl and Shift clicks, Shift and the arrows, Ctrl+A (`Table::key`), and a band across the rows (`Gesture::band`, `paint_band`) that begins below the rows or with Ctrl. The screen reader is told which rows are chosen.
- A table you paint: headings that sort and resize, rows chosen by mouse or keys, only the rows in sight painted, and a table for the screen reader.
- Tables: cells typed in (double click, Enter keeps, Escape drops), headings dragged into a new order (`Column::id` keeps the data), and group headings between rows that never fold away.
- Tables by keys: `Table::step_column` puts the keys on a column, ringed with the keyboard's ring on the chosen row's cell (or its heading), `shift_column` moves it as a drag does, `edit_column` types in it when it can be, and a screen reader is on that cell as the table's active descendant (`SaidTable::column`). `kanban::Board::said` gives the chosen card's column the same way.
- Two more stored pictures in `tests/golden`: a table in the dark and light looks, with no words.
- A painted table holds a million rows in a thousand groups and asks only for the cells in sight (tested); `list` is measured against it for long data.
- A board of cards you paint (`kanban`): cards move by drag within and across columns, or by Ctrl and the arrows, and a screen reader hears a table of columns.
- `panel::side_panel`: a panel beside the content that slides in and out as the program says, gone from the window and the screen reader when out.
- `notices::Notices`: the bottom notice queued in order, shown one at a time for `--trindle-notice-stays`, over the foot of the window without moving the page.
- `outline::outline`, `Shown::go_to` and `Shown::reading`: a table of contents for a long page that goes to a heading and marks the one being read.
- `term::Term`: a terminal screen fed by a program, with ANSI colours from the sheet's `--trindle-term-*` held readable on every cell, cursor moves, erasing and a scroll-back.
- High contrast (`--trindle-high-contrast`) and large text (`--trindle-large-text`, 1 to 2) from the sheet, now growing the fixed sizes that hold or sit beside words too (listed in a test that fails when a new size is in neither the growing nor the staying list), tested for every theme and highlight; `motion::slide` and `motion::fade` timed by `--trindle-slide-time` and `--trindle-fade-time`, and `--trindle-reduce-motion` makes them instant.
- `settings::LookChoice` and `settings::look_page`: a program's own light or dark and highlight, kept in its settings file, with a ready settings page.
- A tab dragged off its bar is `Tearing` and `TornOff`; `Shown::open_beside` opens another window of the same program for it.
- `print::pdf` and `print::print_to`: widgets painted onto the pages of a PDF file, written without a new crate.
- A tab bar you paint: close boxes, middle click to close, drag to reorder, a button for the tabs that do not fit, and a tab list for the screen reader.
- A many-line text area ready to use as a node, with undo, wrapping, a scroll bar, the right-click menu and the clipboard.
- The mouse's selection: select to copy and middle click to paste, through the primary selection protocol.
- A screen of cells painted for a terminal: fills in runs, words in runs, wide characters on two cells, cursor shapes and a stream or block selection.
- A plain icon node, a round avatar and an image node.
- Settings rows and pages, a colour picker, and date and time entry.
- Fullscreen, maximise, minimise, size limits, child and modal windows and a window icon.
- Desktop notifications and a tray icon over D-Bus.
- The desktop's light or dark look, accent colour, contrast and reduced motion are followed through the settings portal.
- Keyboard stepping and a shortcut table for grids of tiles.
- Plumbing helpers: a private file write that is never half done, a settings file reader, a notification line, open a folder, and decoding in the background.
- `keep::Shape` keeps a window's size and maximised state per program, `Window::maximised()` reads it, and `keep::keep_open` and `opened_last` give back what was open on the next start.
- `Window::keep_awake` holds off the screen's sleep while a window shows (idle-inhibit-unstable-v1), and `locked::on_locked` hears the session lock and unlock from logind.
- `portal::screenshot`, `open_uri`, `open_file` and `camera` through the desktop portal; `chooser` now uses the same ask.
- `entry::Entry` writes a desktop entry (launcher, file kinds, address handlers) and an autostart entry; `make_default`, `default_for` and `openers` read and set what opens a kind.
- Tests of the gallery at scales 1.0, 1.25, 1.5, 1.75 and 2.0, and of a window moved between screens of different scales.
- A window kind for locking the session (ext-session-lock-v1), tested only against a stand-in compositor.
- A `debian/` source package, a guide, a gallery example and a note on how the toolkit changes.
- A `core` feature for a program with no window, which builds the plumbing and the windowless Wayland setup and none of the drawing, fonts or screen reader link.
- Words too long for their room are cut with an ellipsis and given whole on pointing, in every script. Fixed: a right-to-left line cut short (a label, button, choice, dropdown, list row, tab or table cell) was drawn far to the right of its room; tests with long words in seven scripts now hold this.
- Free shapes in a sketch: paths with curves, flat, linear and radial fills, dashed strokes, clipping and soft shadows (`tiles::Way`, `tiles::Fill`).
- `scene::Scene`, a canvas of items to choose, move by pointer or keys, zoom, pan and hit-test.
- `vector::vector`, an SVG drawn from its shapes at any size.
- `player::Player`, a video and sound player's controls and state; the program decodes through `player::Decoder`.
- `Window::gpu_surface`, a subsurface a program draws into with the GPU or a video decoder.
- `Fonts::chooser` and `Fonts::fallbacks`: families with faces, fixed width and colour, and which font draws each run of words.
- `.spelling(&speller)` on a text box: a wavy line in `--trindle-spelling` under unknown words, told to the screen reader.
- `tasks::Tasks`: threads and timers that hand their results back as messages.
- `watch::live` and `Watch::with_subfolders` for a live view of a folder.
- `open_with`: the apps that open a kind of file, the default first, setting the default, and the recent files list.
- `crash`: a panic written down as a report, and a page that shows it with a button to copy it.

Also new:

- Zoom and pan for a picture you paint, and a crop box with handles that can hold a ratio.
- A grid of tiles you paint, with click, double click, drag to reorder and a rubber band.
- Rows, choices and spinners tell a screen reader their place in the set, or that they are busy.
- The screen reader can be sent words after any message.
- The crate builds and tests on its own, outside Trindle's workspace, with `cargo +1.85 test --release --workspace`.
- The README now holds the quick start, and the full reference moved to `docs/reference.md`.
- Pages for security, contributing and the changelog, and pictures of the real toolkit.

- Text is placed on whole pixels along the line, as a browser sets it, instead of on quarter pixels, so stems no longer straddle two pixels and words read crisper on a 100% screen.
- One shape everywhere: every button, strip, tab, chip, switch, knob and label has a soft square corner (`--trindle-corner`, 8 points, 0 to 16). Icon buttons are plain icons with a soft square when pointed at or on, cards have no outline, a choice with arrows is a flat path with a small › between names, tabs are a soft box joined to the page, and `quiet_action` gives a quiet worded button with an icon. Only the ring stays round. Cards, boxes, rows and floating panels follow the same corner by default (`--trindle-card-corner`, `--trindle-box-corner` and `--trindle-row-corner` are `var(--trindle-corner)`), and a theme may still set each one.
- Plain text is 14 points (notes and column headings 13, headlines 15), down from 16, so words read sharp and light beside the worded buttons; each is still one value in the style sheet.

Fixed after a full code review:

- Pictures, masks, bitmaps and icon sets are size capped when read.
- A settings file can no longer inject extra lines.
- Bus connections and sends give up after a limit instead of waiting for ever, and a bus name is held firmly.
- The file chooser reports a failure and checks the addresses it is given.
- Notification numbers and text are escaped and kept right, and actions that arrive as signals are restored.
- Clipboard work no longer blocks, and the mount list is read as bytes.
- Plural forms, date ranges, dates, grapheme steps, right-to-left carets and CRLF line ends are handled correctly.
- Caret snapping, scroll bars, reading windows and popup rows were corrected.
- Many smaller faults in the editing pane, the grid, the cell screen, drag and drop, the watcher and the theme reader.
- Updated the xkbcommon, resvg and skrifa libraries. The accesskit and zbus libraries stay on versions that build on Rust 1.85, and the README warns against turning on zbus's `tokio` feature in the same build.
