# Coming from GTK or Qt

The toolkit is not object based. A view is a function from your state to a tree of `Node<Msg>`; a press or an edit sends a `Msg` to your `update`, and the view is built again. There are no widget handles to keep, no signals to connect and no layout managers to add children to. Looks come from the style sheet, not from code.

| GTK | Qt | Trindle | Notes |
|---|---|---|---|
| `GtkWindow` | `QMainWindow` | `window::Kind::Window`, `ui::Shown::run` | One `App` per window |
| `GtkBox` (vertical) | `QVBoxLayout` | `column(items)` | `.gap()`, `.padding()` |
| `GtkBox` (horizontal) | `QHBoxLayout` | `row(items)` | |
| `GtkGrid`, `GtkFlowBox` | `QGridLayout`, `QFlowLayout` | `grid`, `even_grid`, `bento` | Tiles fill bands of the width |
| `GtkOverlay` | stacked layouts | `layers(items)`, `placed` | |
| `GtkFrame`, `GtkCard` | `QGroupBox`, `QFrame` | `card(inner)` | Heading with `heading(text)` |
| `GtkLabel` | `QLabel` | `label`, `note`, `title`, `heading`, `form_label` | One function per role |
| `GtkButton` | `QPushButton`, `QToolButton` | `button`, `main_word`, `main_action`, `quiet_action`, `word_button` | `.on_press(msg)` |
| `GtkLinkButton` | `QLabel` with a link | `link`, `link_named` | No underline |
| `GtkToggleButton` | checkable `QPushButton` | `button(..).on(true)` | |
| linked `GtkToggleButton`s | non-exclusive `QButtonGroup` | `bars::Toolbar` of `tool(..).toggle(on)` | One Tab stop, the arrows move along |
| `GtkSwitch` | none | `switch(on, tip, change)` | |
| `GtkCheckButton` | `QCheckBox` | `tick(on, tip, change)` | |
| `GtkRadioButton` | `QRadioButton` | `choice(options, chosen, tip, choose)` | One strip, no groups to join |
| `GtkDropDown`, `GtkComboBox` | `QComboBox` | `dropdown(..)`, `one_of(..)` | Its list floats; no pop-up window |
| `GtkScale` | `QSlider` | `slider(value, min, max, step, tip, change)` | |
| `GtkSpinButton` | `QSpinBox` | `spin::spin`, `spin::Spin` | Less and More buttons, arrows, Page keys and wheel |
| `GtkEntry` | `QLineEdit` | `entry(text, tip, change)` | `.secret()` for passwords |
| `GtkPasswordEntry` | `QLineEdit` echo mode | `entry(..).secret()` | |
| `GtkTextView` | `QPlainTextEdit` | `editing::area::TextArea` | Undo, wrapping, clipboard |
| `GtkSearchEntry` | `QLineEdit` with a clear button | `entry`, `find::Find` | |
| `GtkListBox`, `GtkListView` | `QListWidget`, `QListView` | `list(rows, chosen, choose)` with `item(text)` | |
| `GtkColumnView`, `GtkTreeView` (flat) | `QTableView` | `table::Table` | Sorts, resizes, paints only visible rows |
| `GtkTreeView` (tree) | `QTreeView` | `tree(rows, chosen, choose, toggle, open)` | Rows built with `tree_row(item, depth)` |
| `GtkNotebook`, `GtkStackSwitcher`, `GtkStack`, `GtkStackSidebar` | `QTabWidget`, `QTabBar`, `QStackedWidget` | `tabs::Tabs` (a painted tab bar), `toolbox::tab_pages`, `nav::stack`, `nav::stack_beside`, `choice` | |
| `GtkScrolledWindow` | `QScrollArea` | `scroll(key, inner)`, `scroll_x(key, inner)` | `key` names the area |
| `GtkProgressBar`, `GtkLevelBar` | `QProgressBar` | `bars::progress_bar`, `dial::level_bar`, `ring(share, size)` | Bar for progress or fullness; ring when the end can be guessed |
| `GtkSpinner` | `QProgressBar` busy | `spinner()` | |
| `GtkImage`, `GtkPicture` | `QLabel` with a pixmap | `image_view`, `icon_image`, `avatar` | |
| `GtkDrawingArea` | `QWidget::paintEvent` | `drawing(size, tip, paint)` | See [custom widgets](guides/custom-widgets.md) |
| `GtkColorButton` | `QColorDialog` | `picker::ColourBox` | |
| `GtkCalendar` | `QCalendarWidget` | `when::DateBox`, `when::TimeBox` | |
| `GtkSeparator` | `QFrame::HLine` | `section_rule()` | |
| `GtkPaned` | `QSplitter` | `split`, `split_down` | Drag the divider or use the arrow keys |
| `GtkFixed` | `QWidget` with `move` | `frames::fixed`, `placed` | |
| `GtkRevealer` | `QPropertyAnimation` on height | `frames::revealer` | Moved with `motion::tween` |
| `GtkCarousel`, `AdwCarousel` | none | `frames::pager`, `frames::dots` (a picture viewer with thumbnails: `photos::gallery`) | |
| `AdwNavigationView` | `QStackedWidget` with a back button | `nav::nav_view` | Escape goes back |
| `AdwNavigationSplitView`, `AdwLeaflet` | `QSplitter` | `nav::split_view` | Shows one side when narrow |
| `AdwPreferencesWindow`, `AdwPreferencesPage` | `QDialog` of pages | `pages::preferences`, `settings::setting_page` | |
| `AdwActionRow`, `AdwSwitchRow`, `AdwComboRow`, `AdwEntryRow`, `AdwSpinRow` | `QFormLayout` rows | `settings::setting`, `rows::toggle_row`, `combo_row`, `entry_row`, `spin_row` | |
| `AdwStatusPage` | none | `pages::status_page`, `tiles::empty_state` | |
| `AdwAboutWindow`, `GtkAboutDialog` | `QMessageBox::about` | `pages::about_page` | A page, not a dialog |
| `AdwWelcome` | none | `pages::welcome_page` | |
| `AdwBottomSheet` | none | `nav::bottom_sheet` | |
| `AdwOverlaySplitView` | `QDockWidget`, `QDrawer` | `nav::drawer`, `side_panel`, `dock::Dock` | |
| `AdwAvatar` | none | `avatar`, `badges::avatar_group` | |
| `GtkAssistant` | `QWizard` | `steps::wizard` | One page at a time |
| `GtkFontButton`, `GtkColorDialogButton`, `GtkFileChooserButton`, `GtkLockButton` | `QFontDialog`, `QColorDialog`, `QFileDialog` | `buttons::font_button`, `colour_button`, `file_button`, `lock_button` | The press is the program's |
| `GtkScaleButton`, `GtkVolumeButton` | none | `dial::volume`, `dial::dial` | |
| `GtkLevelBar` | `QProgressBar` | `dial::level_bar`, `tiles::fill_bar` | |
| `GtkEmojiChooser` | none | `emoji_picker::emoji_page` | A page, not a pop-up |
| `GtkMenuBar`, `GtkPopoverMenuBar` | `QMenuBar` | `menubar::MenuBar` | |
| `GtkShortcutsWindow` | none | `shortcuts::Shortcuts::page` | |
| `GtkStatusbar` | `QStatusBar` | `bars::status_bar` | |
| `GtkPathBar` | none | `bars::path_bar` | |
| `GtkToolbar`, `GtkToolPalette` | `QToolBar`, `QToolBox` | `bars::Toolbar`, `toolbox::toolbox` | |
| `GtkVideo`, `GtkMediaControls` | `QMediaPlayer`, `QVideoWidget` | `player::view`, `moving` | |
| none | none | `rating::rating` | Stars you press |
| `GtkCalendar` range | `QCalendarWidget` | `when::RangeBox` | |
| `GtkCalendar` week numbers | `QCalendarWidget::setVerticalHeaderFormat` | `weeks: true` on `when::DateBox` or `RangeBox` | ISO 8601 weeks |
| `GtkFileChooserDialog`, `GtkFileDialog` | `QFileDialog` | `file_page::FilePage`, or the portal with `chooser` | A page, not a pop-up |
| `GtkFontChooserDialog`, `GtkFontDialog` | `QFontDialog` | `font_page::FontPage` | A page, not a pop-up |
| `GtkGLArea` | `QOpenGLWidget` | `Window::gpu_surface` | |
| `GtkTextView` with tags | `QTextEdit` | `rich`, `editing::rich_box::RichBox` | |
| `AdwSkeleton` (loading placeholder) | none | `tiles::skeleton`, `tiles::loading_state` | |
| `GtkListStore`, `GListModel` | `QAbstractItemModel` | `model::ListModel` | |
| `GtkComboBox` with entry | `QComboBox` editable | `combo::combo` | |
| `GtkPicture` vector | `QSvgWidget` | `vector::vector` | |
| `GtkPrintOperation` | `QPrinter` | `print` | |
| `GtkPopover`, `GtkMenu` | `QMenu` | `popup`, `Node::menu` | Floats over the page |
| `GtkDialog`, `GtkMessageDialog` | `QDialog`, `QMessageBox` | a page or a card in the window | No pop-ups, no dialogs; the sign-in is a page |
| `GtkInfoBar`, toast | `QStatusBar` message | `note(..).tone(Tone::Problem)` under the box, `info::info_panel`, `notices::Notices` | Problems show beside what failed; news sits in the page |
| `GtkTooltip` | `QToolTip` | the `tip` argument | Shows on pointing; a screen reader says it |
| `GtkExpander` | `QToolBox` | none, by design | No folding sections |
| `GtkHeaderBar`, `GtkActionBar` | `QToolBar` | `row` of buttons | |
| `GtkShortcutController` | `QShortcut` | `shortcuts` | |
| `GtkClipboard` | `QClipboard` | `host` clipboard calls | |
| `GtkDragSource`, `GtkDropTarget` | `QDrag`, `QDropEvent` | `dnd` | |
| CSS themes | style sheets, `QPalette` | `share/style.css` and themes | Each value has a range comment |
| `g_signal_connect` | `connect(..)` | a `Msg` and `update` | |
| `GtkBuilder`, `.ui`, QML | `.ui`, QML | none | The view is Rust |
| `GSettings`, `QSettings` | `QSettings` | `settings` | One file per program |
| `gettext`, `tr()` | `tr()` | `t!("key")` and `share/lang` | Every word is in a language file |
| `AT-SPI` through GTK | `QAccessible` | automatic, AccessKit | Name every picture with `.called(..)` |

Things to unlearn:

- Do not keep a handle to a widget. Change your state and return a new view.
- Do not set colours, fonts or corners in code. Change the style sheet; a program that must choose a look sets a theme.
- Do not open a second window for a question. Show a page or card in the same window.
- Do not fold content behind a heading. Show it, or move it to its own page.
