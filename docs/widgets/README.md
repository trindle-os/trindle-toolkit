# Widget guides

One short guide for each widget in the README's table: what it is for, a picture, the keys it answers to, what a screen reader is told and an example that is compiled with the tests. The pictures are painted from each guide's own code with the built-in fonts; `tools/widget-pictures.sh` paints them again.

For a whole kind of job rather than one widget, see the [guides](../guide.md) on buttons, choices, typing, lists, layout, progress, testing and your own widgets.

## Layout

- [`column`, `row`](row-and-column.md)
- [`grid`, `even_grid`, `bento`](grids.md)
- [`flow`](flow.md)
- [`card`, `frames::frame`](card-and-frame.md)
- [`layers`, `placed`, `frames::fixed`](layers.md)
- [`aspect`](aspect.md)
- [`space`, `section_rule`](space-and-rule.md)
- [`scroll`, `scroll_x`, `slide`](scrolling.md)
- [`split`, `split_down`](split.md)
- [`frames::badged`, `frames::revealer`](badged-and-revealer.md)
- [`side_panel`, `dock::Dock`](side-panel-and-dock.md)

## Text

- [`label`, `note`, `figure`](label-note-figure.md)
- [`title`, `page_title`, `lead`, `heading`, `headline`, `section`, `side_heading`, `form_label`, `column_heading`](text-roles.md)
- [`rich`, `editing::rich_box::RichBox`](rich-text.md)
- [`code::view`, `help::view`](code-and-help-views.md)
- [`link`, `link_named`](link.md)

## Buttons and choices

- [`button`, `main_word`, `main_action`, `quiet_action`, `word_button`](buttons.md)
- [`remove`, `remove_by_hand`](remove-buttons.md)
- [`switch`, `tick`, `tiles::tick_box`](switch-and-tick.md)
- [`choice`, `dropdown`, `one_of`, `tiles::radio_group`](choices-and-dropdowns.md)
- [`buttons::colour_button`, `font_button`, `file_button`, `lock_button`](device-buttons.md)
- [`rating::rating`](rating.md)
- [`icon_circle`, `icon_image`](icons.md)

## Entries

- [`entry`](entry.md)
- [`slider`, `dial::scale`, `dial::dial`, `range::Range`](sliders.md)
- [`dial::volume`](volume.md)
- [`spin::spin`, `spin_with`, `spin::Spin`, `spin::time_spin`, `spin::month_year`](spin-boxes.md)
- [`combo::combo`, `complete::Complete`, `chips::Chips`](combo-boxes.md)
- [`validate::CheckedBox`, `form::Form`](checked-box-and-form.md)
- [`picker::ColourBox`, `GradientBox`, `swatch_grid`](colour-pickers.md)
- [`when::DateBox`, `TimeBox`, `RangeBox`, `duration::DurationBox`](date-and-time-boxes.md)
- [`editing::area::TextArea`, `editing::editor::Pane`](text-area.md)
- [`find::Find`](find-bar.md)
- [`emoji_picker::emoji_page`](emoji-picker.md)
- [`rows::toggle_row`, `combo_row`, `entry_row`, `spin_row`](settings-rows.md)

## Lists and trees

- [`list` with `item`](list.md)
- [`tree` with `tree_row`](tree.md)
- [`table::Table`](table.md)
- [`kanban::Board`](kanban.md)
- [`model::ListModel`](list-model.md)
- [`outline::outline`](outline.md)
- [`open_with::chooser`](open-with.md)
- [`cellgrid::Screen`, `term::Term`](cell-grid-and-terminal.md)

## Bars and progress

- [`ring`, `tiles::standard_ring`, `standard_ring_with`](ring.md)
- [`spinner`](spinner.md)
- [`bars::progress_bar`, `dial::level_bar`, `tiles::fill_bar`](progress-bars.md)
- [`tiles::time_pie`, `tiles::bars`](time-pie-and-bars.md)
- [`bars::Toolbar`, `action::Actions::toolbar`](toolbar.md)
- [`bars::status_bar`, `bars::path_bar`](status-and-path-bars.md)
- [`tiles::skeleton`, `tiles::loading_state`](skeleton-and-loading.md)

## Pages and navigation

- [`nav::stack`, `stack_beside`](stack.md)
- [`nav::nav_view`, `nav::split_view`](nav-view-and-split-view.md)
- [`history::History`](history.md)
- [`nav::drawer`, `nav::bottom_sheet`](drawer-and-bottom-sheet.md)
- [`tabs::Tabs`, `toolbox::tab_pages`, `toolbox::toolbox`](tabs.md)
- [`pages::preferences`, `settings::setting_page`, `settings::setting`](preferences-and-settings.md)
- [`pages::welcome_page`, `status_page`, `about_page`](welcome-status-about.md)
- [`steps::wizard`](wizard.md)
- [`crash::page`, `settings::look_page`, `shortcuts::Shortcuts::page`](crash-look-shortcuts.md)
- [`file_page::FilePage`, `font_page::FontPage`](file-and-font-pages.md)

## Menus and dialogs

- [`menubar::MenuBar`](menu-bar.md)
- [`popup::Popup`, `Node::menu`](popup-menu.md)
- [`Node::context_menu`](context-menu.md)
- [`popup::dismissing`, `popup::under_the_i`, `hint::help_over`](i-panel.md)
- [`dialog::dialog`, `confirm`, `choose`, `ask`](dialog.md)
- [`palette::Palette`](palette.md)
- [`notices::Notices::over`](notices.md)
- [`info::info_panel`](info-panel.md)
- [`tiles::empty_state`, `empty_picture`, `error_state`](empty-and-error-states.md)
- [`typing_over`](typing-over.md)

## Charts and pictures

- [`charts::Chart`](charts.md)
- [`image_view`, `avatar`](image-and-avatar.md)
- [`badges::badge_count`, `badges::avatar_group`](badges.md)
- [`photos::gallery`, `frames::pager`, `frames::dots`](photo-gallery-and-pager.md)
- [`vector::vector`](vector.md)
- [`moving`, `player::view`](video-and-player.md)
- [`tiles::sketch`, `tiles::picture`, `tiles::tile_head`](tile-pictures.md)
- [`scene::Scene`](scene.md)
- [`drawing`](drawing.md)
- [`Painter`](painter.md)

## Desktop and system

- [`framed`](framed.md)
