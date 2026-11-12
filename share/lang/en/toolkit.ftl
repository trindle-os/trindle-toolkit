# The Trindle Toolkit's own words, in English. This is the source file: every
# other language is a copy of it with the right-hand sides translated. See
# README.md in this folder.
#
# Keys start with kit-. Write words in sentence case, UK English, no em dashes.

## Menus on text

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = Copy
# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = Select all
# Right-click menu on a typing box that has words in it.
kit-cut = Cut
# Right-click menu on a typing box.
kit-paste = Paste

## The eye beside a secret box

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
kit-show-password = Show the password
# Name of the eye beside a PIN box while the PIN is hidden.
kit-show-pin = Show the PIN
# Name of the eye beside a key box while the key is hidden.
kit-show-key = Show the key
# Name of the eye beside a password box while the password shows.
kit-hide-password = Hide the password
# Name of the eye beside a PIN box while the PIN shows.
kit-hide-pin = Hide the PIN
# Name of the eye beside a key box while the key shows.
kit-hide-key = Hide the key
# Said by the screen reader after the eye beside a password box is pressed to show it.
kit-password-shown = Password shown
# Said by the screen reader after the eye beside a password box is pressed to hide it.
kit-password-hidden = Password hidden
# Said by the screen reader after the eye beside a PIN box is pressed to show it.
kit-pin-shown = PIN shown
# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
kit-pin-hidden = PIN hidden
# Said by the screen reader after the eye beside a key box is pressed to show it.
kit-key-shown = Key shown
# Said by the screen reader after the eye beside a key box is pressed to hide it.
kit-key-hidden = Key hidden

## Lists and dropdowns

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = Choose one
# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = Search
# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = Nothing matches
# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, { $n } of { $total }
# Shown at the end of a dropdown's search box: how many rows match out of how many there are.
kit-search-of = { $n } of { $total }
# Read out by the screen reader for a spinner that is turning.
kit-loading = Loading
# Read out by the screen reader for the line between two areas of a window that can be dragged to make one bigger.
kit-divider = Divider

## Buttons that need a second press

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small where the language does that (see kit-click-again-small).
kit-click-again = Click again to { $action }
# The same, when the button has no name to repeat.
kit-click-again-bare = Click again

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
kit-click-again-small = yes

kit-job-wait = Wait for the task to finish.

## Long label on pointing

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + more
# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
        [one] +{ $n } line
       *[other] +{ $n } lines
    }

## What a screen reader says of a menu row

# A menu row that opens a sub-menu. $words is the row's own words, $n how many lines the sub-menu holds.
kit-row-sub =
    { $n ->
        [one] { $words }, sub-menu, 1 line
       *[other] { $words }, sub-menu, { $n } lines
    }
# A menu row with a shortcut. $words is the row's own words, $keys the shortcut.
kit-row-keys = { $words }, { $keys }

## Things dropped on a window

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = Trindle couldn't do anything with the items you dropped.
# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = What you dropped is too big to drag in.
# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = What you dropped took too long to arrive, so Trindle stopped waiting.
# Said when too many things were dropped at once.
kit-dropped-too-many = You dropped too many items at once. Drop fewer at a time.
# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = The items you dropped aren't files on this computer.
# Read out by the screen reader for the control under something dragged, when letting go there would drop it in.
kit-drop-target = Drop area

## Links

# Label of a link button whose address is not a web address.
kit-link-not-web = Not a web address
# Label of a link button whose address opens a mail program.
kit-link-mailto = Email address
# Label of a link button whose address opens a file.
kit-link-file = File on this computer
# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = Link to { $label }

## The window for choosing a file

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = The chooser window didn't respond. Try again.

## Checking a theme file

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = Line { $line }: { $why }
# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = This theme file is too big. The maximum size is 64 KB.
# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = This file isn't a style sheet. It contains more than plain text.
# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = a theme can't import another file or a web address.
# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = The value of --{ $name } is too long.
# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = Trindle can't read --{ $name } as a colour.
# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = Trindle can't read --{ $name } as a number.
# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = Trindle can't read --{ $name } as a shadow.
# Fault in a theme or style sheet: a setting that takes only named steps was given something else. $steps is the list of steps, $guide the file that explains them.
kit-theme-not-step = --{ $name } can only be one of these: { $steps }. Trindle ignores any other value. See { $guide }.
# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = Trindle doesn't recognise any settings in this file, so nothing would change.
# Standard error, when the folder of the person's style sheet cannot be watched. $folder is the folder, $why the system's reason.
kit-sheet-not-watched = Restart to see changes to the style sheet in { $folder }. { $why }

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = Progress

# The name a screen reader gives a menu that opens over a window.
kit-menu = Menu
# What a screen reader calls the row of menu titles across the top of a window.
kit-menu-bar = Menu bar

## A dialog page

# The answer at the foot of every dialog page that leaves it with nothing done.
kit-cancel = Cancel

## Bars

# What a screen reader calls a row of tool buttons.
kit-toolbar = Tools
# The button at the end of a toolbar that opens a menu of the tools that did not fit.
kit-more-tools = More tools
# What a screen reader calls the line at a window's foot that says how things stand.
kit-status = Status
# What a screen reader calls a place's path, such as a folder's.
kit-path = Path
# The list in the middle of a long path, holding the places not shown.
kit-path-more = More places

## A slider with two handles

# Said after a range slider's name for its low handle.
kit-range-low = lower end
# Said after a range slider's name for its high handle.
kit-range-high = upper end

## A number box

# The button that takes a step off a number box's number. $what is the box's name.
kit-spin-less = Decrease { $what }
# The button that adds a step to a number box's number. $what is the box's name.
kit-spin-more = Increase { $what }
# Red words under a number box whose typing is not a number.
kit-spin-not-number = Type a number from { $min } to { $max }

## Tabs a program paints

# Said by a screen reader for a tab that carries a count, such as unread messages: its title, then the count.
kit-tab-count = { $title }, { $count }
# Said by a screen reader for the close box in a tab.
kit-tab-close = Close
# Said by a screen reader for the button that lists the tabs that did not fit.
kit-tab-more = More tabs

## The text area

# Right-click menu of a text area, one row each.
kit-area-cut = Cut
kit-area-copy = Copy
kit-area-paste = Paste
kit-area-select-all = Select all

## Settings pages, colour and date

# The name a screen reader gives the painted square of a colour picker.
kit-colour-square = Colour, saturation and brightness
# The name a screen reader gives the swatch showing the colour chosen. $colour is its hex code.
kit-colour-swatch = Chosen colour { $colour }
kit-colour-hue = Hue
kit-colour-hex = Hex code

# The ready settings page for a program's own look (settings::look_page).
kit-look-title = Appearance
kit-look-lead = Choose how this program looks, separately from the rest of the desktop.
kit-look-group = Appearance
kit-look-shade = Light or dark
kit-look-shade-help = Match the desktop, or always use light or dark.
kit-look-desktop = Same as the desktop
kit-look-dark = Dark
kit-look-light = Light
kit-look-highlight = Highlight colour
kit-look-highlight-help = The colour of main buttons and selected items.
# A button that gives the highlight back to the desktop's choice.
kit-look-highlight-desktop = Same as the desktop

# Buttons either side of the month in a date picker.
kit-date-previous = Previous
kit-date-next = Next
# The month shown in a date picker. $month is the month's name, $year the year.
kit-date-month-year = { $month } { $year }
# What a screen reader says for a day in a date picker.
kit-date-full = { $weekday } { $day } { $month } { $year }
# Over the column of week numbers in a date picker; keep it as short as the short weekday names.
kit-date-week-short = Wk
# What a screen reader says for a week number in a date picker. $week is the number.
kit-date-week = Week { $week }

# The names of the two boxes in a time entry.
kit-time-hours = Hours
kit-time-minutes = Minutes

# Months and weekdays; weekdays are short in the grid and full when read out.
kit-month-1 = January
kit-month-2 = February
kit-month-3 = March
kit-month-4 = April
kit-month-5 = May
kit-month-6 = June
kit-month-7 = July
kit-month-8 = August
kit-month-9 = September
kit-month-10 = October
kit-month-11 = November
kit-month-12 = December
# A month as it reads inside a whole date, such as "12 November 2026". Some languages change the ending there, so it has its own key.
kit-month-in-date-1 = January
kit-month-in-date-2 = February
kit-month-in-date-3 = March
kit-month-in-date-4 = April
kit-month-in-date-5 = May
kit-month-in-date-6 = June
kit-month-in-date-7 = July
kit-month-in-date-8 = August
kit-month-in-date-9 = September
kit-month-in-date-10 = October
kit-month-in-date-11 = November
kit-month-in-date-12 = December
kit-weekday-1 = Mon
kit-weekday-2 = Tue
kit-weekday-3 = Wed
kit-weekday-4 = Thu
kit-weekday-5 = Fri
kit-weekday-6 = Sat
kit-weekday-7 = Sun
kit-weekday-full-1 = Monday
kit-weekday-full-2 = Tuesday
kit-weekday-full-3 = Wednesday
kit-weekday-full-4 = Thursday
kit-weekday-full-5 = Friday
kit-weekday-full-6 = Saturday
kit-weekday-full-7 = Sunday

## Shortcut keys

# The names of the modifier keys in a shortcut such as Ctrl+Shift+C.
kit-key-ctrl = Ctrl
kit-key-shift = Shift
kit-key-alt = Alt
kit-key-logo = Super
# The names of keys that are words, in a shortcut such as Ctrl+Page down.
kit-key-left = Left
kit-key-right = Right
kit-key-up = Up
kit-key-down = Down
kit-key-page-up = Page up
kit-key-page-down = Page down
kit-key-home = Home
kit-key-end = End
kit-key-enter = Enter
kit-key-escape = Escape
kit-key-tab = Tab
kit-key-space = Space
kit-key-backspace = Backspace
kit-key-delete = Delete
kit-key-insert = Insert
# The heading of the page that lists a program's keyboard shortcuts.
kit-shortcuts = Keyboard shortcuts
# Under a shortcut on that page whose keys the window always takes first.
kit-shortcut-kept-window = This key is reserved for moving between controls and closing the window.
# Under a shortcut on that page whose keys a text box takes first while you type.
kit-shortcut-kept-typing = While you're typing, this key goes to the text box first.

## Emoji page

# The heading of the page of emoji to pick from.
kit-emoji = Emoji
# The name of the box on that page where an emoji's name is typed to find it.
kit-emoji-search = Find an emoji by name
# Shown on that page when no emoji's name has what was typed.
kit-emoji-none = No emoji has that name.
# The groups of emoji on that page, in order. Another language may leave out any
# name below; the page then shows the English one, so a name is never missing.
kit-emoji-faces = Faces
kit-emoji-hands = Hands
kit-emoji-animals = Animals
kit-emoji-nature = Nature
kit-emoji-food = Food and drink
kit-emoji-travel = Travel and places
kit-emoji-objects = Objects
kit-emoji-symbols = Symbols
# Each emoji's name, spoken by a screen reader and found by typing.
kit-emoji-grin = Grin
kit-emoji-tears-of-joy = Tears of joy
kit-emoji-smile = Smile
kit-emoji-wink = Wink
kit-emoji-heart-eyes = Heart eyes
kit-emoji-thinking = Thinking
kit-emoji-crying = Crying
kit-emoji-surprised = Surprised
kit-emoji-blush = Blush
kit-emoji-halo = Halo
kit-emoji-cool = Cool
kit-emoji-sleeping = Sleeping
kit-emoji-sobbing = Sobbing
kit-emoji-angry = Angry
kit-emoji-scream = Scream
kit-emoji-eye-roll = Eye roll
kit-emoji-relieved = Relieved
kit-emoji-rolling-laugh = Rolling with laughter
kit-emoji-blowing-a-kiss = Blowing a kiss
kit-emoji-straight-face = Straight face
kit-emoji-flushed = Flushed
kit-emoji-hug = Hug
kit-emoji-unwell = Unwell
kit-emoji-celebrating = Celebrating
kit-emoji-thumbs-up = Thumbs up
kit-emoji-thumbs-down = Thumbs down
kit-emoji-clapping = Clapping
kit-emoji-thank-you = Thank you
kit-emoji-waving = Waving
kit-emoji-victory = Victory
kit-emoji-ok = OK
kit-emoji-crossed-fingers = Crossed fingers
kit-emoji-strong = Strong
kit-emoji-pointing-up = Pointing up
kit-emoji-pointing-down = Pointing down
kit-emoji-pointing-left = Pointing left
kit-emoji-pointing-right = Pointing right
kit-emoji-raised-hand = Raised hand
kit-emoji-handshake = Handshake
kit-emoji-fist = Fist
kit-emoji-raised-hands = Raised hands
kit-emoji-call-me = Call me
kit-emoji-writing = Writing
kit-emoji-shrug = Shrug
kit-emoji-dog = Dog
kit-emoji-cat = Cat
kit-emoji-mouse = Mouse
kit-emoji-rabbit = Rabbit
kit-emoji-fox = Fox
kit-emoji-bear = Bear
kit-emoji-panda = Panda
kit-emoji-tiger = Tiger
kit-emoji-lion = Lion
kit-emoji-cow = Cow
kit-emoji-pig = Pig
kit-emoji-frog = Frog
kit-emoji-monkey = Monkey
kit-emoji-chicken = Chicken
kit-emoji-penguin = Penguin
kit-emoji-bird = Bird
kit-emoji-duck = Duck
kit-emoji-owl = Owl
kit-emoji-horse = Horse
kit-emoji-bee = Bee
kit-emoji-butterfly = Butterfly
kit-emoji-turtle = Turtle
kit-emoji-fish = Fish
kit-emoji-dolphin = Dolphin
kit-emoji-blossom = Blossom
kit-emoji-tree = Tree
kit-emoji-sun = Sun
kit-emoji-rain = Rain
kit-emoji-rose = Rose
kit-emoji-sunflower = Sunflower
kit-emoji-tulip = Tulip
kit-emoji-evergreen = Evergreen
kit-emoji-clover = Clover
kit-emoji-maple-leaf = Maple leaf
kit-emoji-cactus = Cactus
kit-emoji-moon = Moon
kit-emoji-rainbow = Rainbow
kit-emoji-cloud = Cloud
kit-emoji-snowflake = Snowflake
kit-emoji-lightning = Lightning
kit-emoji-wave = Wave
kit-emoji-earth = Earth
kit-emoji-mountain = Mountain
kit-emoji-palm-tree = Palm tree
kit-emoji-apple = Apple
kit-emoji-bread = Bread
kit-emoji-cake = Cake
kit-emoji-tea = Tea
kit-emoji-banana = Banana
kit-emoji-orange = Orange
kit-emoji-strawberry = Strawberry
kit-emoji-grapes = Grapes
kit-emoji-watermelon = Watermelon
kit-emoji-lemon = Lemon
kit-emoji-carrot = Carrot
kit-emoji-corn = Corn
kit-emoji-pizza = Pizza
kit-emoji-burger = Burger
kit-emoji-chips = Chips
kit-emoji-pasta = Pasta
kit-emoji-fried-egg = Fried egg
kit-emoji-cheese = Cheese
kit-emoji-biscuit = Biscuit
kit-emoji-chocolate = Chocolate
kit-emoji-ice-cream = Ice cream
kit-emoji-beer = Beer
kit-emoji-wine = Wine
kit-emoji-milk = Milk
kit-emoji-car = Car
kit-emoji-bus = Bus
kit-emoji-bicycle = Bicycle
kit-emoji-train = Train
kit-emoji-aeroplane = Aeroplane
kit-emoji-rocket = Rocket
kit-emoji-ship = Ship
kit-emoji-sailing-boat = Sailing boat
kit-emoji-helicopter = Helicopter
kit-emoji-house = House
kit-emoji-office = Office
kit-emoji-hospital = Hospital
kit-emoji-school = School
kit-emoji-church = Church
kit-emoji-beach = Beach
kit-emoji-tower = Tower
kit-emoji-bridge = Bridge
kit-emoji-petrol-pump = Petrol pump
kit-emoji-traffic-light = Traffic light
kit-emoji-map = Map
kit-emoji-phone = Phone
kit-emoji-laptop = Laptop
kit-emoji-keyboard = Keyboard
kit-emoji-camera = Camera
kit-emoji-light-bulb = Light bulb
kit-emoji-key = Key
kit-emoji-lock = Lock
kit-emoji-books = Books
kit-emoji-memo = Memo
kit-emoji-pencil = Pencil
kit-emoji-paperclip = Paperclip
kit-emoji-calendar = Calendar
kit-emoji-email = Email
kit-emoji-gift = Gift
kit-emoji-music-note = Music note
kit-emoji-headphones = Headphones
kit-emoji-alarm-clock = Alarm clock
kit-emoji-bell = Bell
kit-emoji-spanner = Spanner
kit-emoji-money-bag = Money bag
kit-emoji-shopping-trolley = Shopping trolley
kit-emoji-football = Football
kit-emoji-heart = Heart
kit-emoji-star = Star
kit-emoji-done = Done
kit-emoji-question = Question
kit-emoji-party = Party
kit-emoji-fire = Fire
kit-emoji-cross = Cross
kit-emoji-exclamation = Exclamation
kit-emoji-warning = Warning
kit-emoji-hundred = Hundred
kit-emoji-broken-heart = Broken heart
kit-emoji-blue-heart = Blue heart
kit-emoji-green-heart = Green heart
kit-emoji-sparkles = Sparkles
kit-emoji-plus = Plus
kit-emoji-minus = Minus
kit-emoji-right-arrow = Right arrow
kit-emoji-left-arrow = Left arrow
kit-emoji-refresh = Refresh
kit-emoji-forbidden = Forbidden
kit-emoji-recycling = Recycling
kit-emoji-sleep = Sleep

## Refused presses

# Said in a window of Trindle's own when something tried to press a button that only the person may press.
kit-only-your-own = Only you can press this button.

# Said in a window when keys are pressed but the keyboard layout could not be read.
kit-no-layout = Trindle couldn't read the keyboard layout, so you can't type.

# What a screen reader is told of a scene of items: how many, and the one chosen.
kit-scene =
    { $count ->
        [one] A scene of 1 item
       *[other] A scene of { $count } items
    }
kit-scene-chosen =
    { $count ->
        [one] A scene of 1 item, { $name } chosen
       *[other] A scene of { $count } items, { $name } chosen
    }

## A video or sound player's controls
kit-player-play = Play
kit-player-pause = Pause
kit-player-back = Back 5 seconds
kit-player-forward = Forward 5 seconds
kit-player-seek = Position
kit-player-mute = Mute
kit-player-unmute = Unmute
kit-player-volume = Volume
# How far in, out of how long, as clock times.
kit-player-time = { $at } of { $length }

## The page shown after a program crashed or met a fault
kit-crash-title = Something went wrong
kit-crash-lead = { $program } stopped because something went wrong. Copy the report below and send it with your bug report so the problem can be fixed.
kit-crash-copy = Copy report

# What a screen reader calls a block of code on a help page.
kit-help-code = Code

# What a screen reader calls the list of suggestions under a text box; $name is what the box is called.
kit-complete-list = Suggestions for { $name }

# Under a text box with suggestions: how many there are, said by a screen reader as they change.
kit-complete-count =
    { $n ->
        [one] 1 suggestion
       *[other] { $n } suggestions
    }

## A title bar the program draws, where the compositor draws none
kit-frame-minimise = Minimise
kit-frame-maximise = Maximise
kit-frame-restore = Restore
kit-frame-close = Close
kit-frame-move = Move
kit-frame-resize = Resize

# Settings: which figures numbers are written in.
kit-numerals-title = Number style
kit-numerals-help = How numbers, dates and times are written.
kit-numerals-auto = Usual for your region
kit-numerals-latin = Always 0 to 9
kit-numerals-native = The language's own digits
