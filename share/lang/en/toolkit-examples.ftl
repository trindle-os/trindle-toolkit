## Said by Shown::run when a program of one window cannot start

# Standard error, when no Wayland desktop answers.
kit-run-no-desktop = No Wayland desktop answered, so the window can't open
# Standard error, when the desktop would not open the window.
kit-run-no-window = The desktop couldn't open a window.

## The example programs (examples/)

# The hello example: the window's title, its button and what it says about the presses.
kit-example-hello-title = Hello
kit-example-hello-button = Press me
kit-example-hello-none = Not pressed yet
kit-example-hello-pressed =
    { $n ->
        [one] Pressed once
       *[other] Pressed { $n } times
    }

# The names example: the window's title and heading.
kit-example-names-title = Names

# The notes example: the window's title, the box's label and hint, and what it says of the saved note.
kit-example-notes-title = Notes
kit-example-notes-label = Your note
kit-example-notes-hint = Type a note
kit-example-notes-none = Nothing saved yet
kit-example-notes-saved = Saved: { $words }

# The cells example: the window's title when the first line is empty.
kit-example-cells-title = Cells

# The gallery example: one window with the controls you can use, each doing something you can see.
kit-example-gallery-title = Widget gallery
kit-example-gallery-heading = Controls
kit-example-gallery-buttons = Buttons
kit-example-gallery-add = Add
kit-example-gallery-main = Save
kit-example-gallery-remove = Remove
kit-example-gallery-count = Pressed { $n } times
kit-example-gallery-choices = Choices
kit-example-gallery-switch-name = Notifications
kit-example-gallery-size-name = Size
kit-example-gallery-size-small = Small
kit-example-gallery-size-medium = Medium
kit-example-gallery-size-large = Large
kit-example-gallery-town-name = Town
kit-example-gallery-volume-name = Volume
kit-example-gallery-volume = Volume { $n } of 100
kit-example-gallery-typing = Typing
kit-example-gallery-entry-hint = Type your name
kit-example-gallery-hello = Hello, { $name }
kit-example-gallery-progress = Progress
# The gallery's number box: the card's heading, and the name of the box.
kit-example-gallery-copies = Number field
kit-example-gallery-copies-name = Copies
# The word after the number box's number.
kit-example-gallery-copies-unit = copies
# The gallery's progress bar and range slider card: the heading, the bar's name (it follows the volume slider), the two-handled slider's name, and the span it shows.
kit-example-gallery-span = Progress bar and range
kit-example-gallery-span-progress = Volume
kit-example-gallery-span-name = Price
kit-example-gallery-span-now = From { $low } to { $high }
# The gallery's dialog card: its heading, how it behaves, the button that opens the page, and the page's title, message and main answer.
kit-example-gallery-dialog = Dialogue page
kit-example-gallery-dialog-how = Questions fill the whole window instead of popping up. Cancel is selected first, and Escape cancels.
kit-example-gallery-dialog-open = Reset the count
kit-example-gallery-ask-title = Reset the count?
kit-example-gallery-ask-message = This will reset the press count to zero.
kit-example-gallery-ask-yes = Reset
# The gallery's sideways card: the heading, how to move it, and the words on each of twelve buttons in the strip.
kit-example-gallery-along = Horizontal strip
kit-example-gallery-along-how = Swipe it, hold Shift and turn the mouse wheel, or use the Left and Right arrow keys. Press Tab to reach a button out of sight.
kit-example-gallery-along-item = Item { $n }
kit-example-gallery-split = Split
kit-example-gallery-split-first = Drag the divider, or select it and use the arrow keys.
kit-example-gallery-split-second = Press Home or End to move the divider to either end.
kit-example-gallery-menu = Context menu
kit-example-gallery-menu-how = Right-click here, press and hold, or press the Menu key. Type a letter to highlight a row.

# The context menu example: the window's title, what it says to do, the card's words, its menu's rows, and what shows once it is removed.
kit-example-context-title = Context menu
kit-example-context-how = Right-click the card, press and hold it, or press the Menu key
kit-example-context-note = Shopping list
kit-example-context-pinned = Pinned
kit-example-context-unpinned = Not pinned
kit-example-context-pin = Pin
kit-example-context-remove = Remove
kit-example-context-removed = The card was removed
kit-example-context-back = Put it back

# The bench example: the window's title, the clock, the progress bar's name, the text box's hint and the list's name.
kit-example-bench-title = Drawing benchmark
kit-example-bench-clock = Time: { $secs } s
kit-example-bench-progress = Progress
kit-example-bench-hint = Text that changes
kit-example-bench-rows = Ten thousand rows

# The menu bar example: the window's title, what it says to do, the menus' titles and rows, and what shows once a row is chosen.
kit-example-bar-title = Menu bar
kit-example-bar-how = Click a menu name, or press Tab to reach the menu bar and use the arrow keys, Enter and Escape. Alt with a menu's marked letter opens it. Type a letter to highlight the next row that starts with it.
kit-example-bar-file = &File
kit-example-bar-view = &View
kit-example-bar-new = New
kit-example-bar-open = Open
kit-example-bar-quit = Quit
kit-example-bar-wrap = Wrap lines
kit-example-bar-size = Text size
kit-example-bar-small = Small
kit-example-bar-medium = Medium
kit-example-bar-large = Large
kit-example-bar-chose = You chose { $what }
# The gallery's column view card: its heading, how it behaves, its name for a screen reader, the two column headings, and the names of the folders and files in it.
kit-example-gallery-columns = Column view
kit-example-gallery-columns-how = Click the arrow or double-click a folder to open it, or use the Right, Left and Enter keys. Hold Ctrl or Shift to select several rows, or drag across rows to select them.
kit-example-gallery-columns-name = Files
kit-example-gallery-columns-head-name = Name
kit-example-gallery-columns-head-kind = Kind
kit-example-gallery-columns-folder = Folder
kit-example-gallery-columns-file = File
kit-example-gallery-columns-documents = Documents
kit-example-gallery-columns-pictures = Pictures
kit-example-gallery-columns-letters = Letters
kit-example-gallery-columns-reports = Reports
kit-example-gallery-columns-holiday = Holiday
kit-example-gallery-columns-notes = Notes
