## The tools for people who write programs: the sheet checker, the inspector and the speed overlay

# The style sheet checker: a number written past the limits Trindle's sheet gives it. $line is the line in the file, $name the setting.
kit-sheet-out-of-range = Line { $line }: --{ $name } is { $value }, outside its limits of { $min } to { $max }.
# The style sheet checker: a colour the sheet sets cannot be told from what it is drawn on. $look is dark or light, $ink the setting, $on what it is drawn on, $stands and $floor contrast ratios.
kit-sheet-short = In the { $look } look, --{ $ink } on { $on } stands { $stands } to one, below the { $floor } to one it needs.
# The two looks, as named in the line above.
kit-sheet-dark = dark
kit-sheet-light = light
# The style sheet checker: a file with nothing wrong. $file is its name.
kit-sheet-fine = { $file }: no problems found.
# The style sheet checker: a file it could not read. $file is its name.
kit-sheet-unread = { $file } could not be read.
# The style sheet checker's help, shown by trindle-sheet-check --help.
kit-sheet-usage =
    Usage: trindle-sheet-check [file.css ...]
    Checks each sheet (by default your own sheet and themes) against Trindle's limits and WCAG 2.2 AA.

# The inspector, Ctrl+Shift+F12, for a program's author: the widget under the pointer's size and place in pixels.
kit-inspect-size = { $w } by { $h } pixels at { $x }, { $y }
# The inspector: the widget's state, $state being the words below joined by commas.
kit-inspect-state = State: { $state }
kit-inspect-keyboard = keyboard on it
kit-inspect-pointer = pointer on it
kit-inspect-pressed = pressed
kit-inspect-takes-keys = takes the keyboard
kit-inspect-off = cannot be used
kit-inspect-rest = at rest
# The inspector: what a screen reader is told. $role is its kind, such as Button; $name its name.
kit-inspect-reader = Screen reader: { $role } "{ $name }"
kit-inspect-unread = Screen reader: told nothing of it

# The speed overlay, Ctrl+Shift+F11: how long the last frame took to paint.
kit-speed-paint = Paint { $ms } ms
# The speed overlay: memory the program holds, in megabytes.
kit-speed-memory = Memory { $mb } MB
# The speed overlay: how much of the window the last frame painted again.
kit-speed-damage = Painted again: { $share } percent of the window
# The speed overlay: what drew the last frame.
kit-speed-gpu = Drawn by the graphics card
kit-speed-processor = Drawn by the processor

# trindle-new, which makes a new program from the template: its help.
kit-new-usage =
    Usage: trindle-new <name> [--toolkit <path>]
    Makes a folder <name> holding a program with one window, a style sheet, a language file and a test. With --toolkit, it uses the toolkit at that path.
# trindle-new: the name given cannot be a program's name. $name is what was given.
kit-new-bad-name = { $name } cannot be a program's name: use small letters, numbers and hyphens, starting with a letter.
# trindle-new: a file or folder of that name is already there.
kit-new-exists = { $name } is already there, so nothing was written.
# trindle-new: the toolkit folder given cannot be found.
kit-new-no-toolkit = No toolkit was found at { $path }.
# trindle-new: a file could not be written.
kit-new-unwritten = { $path } could not be written.
# trindle-new: the program was made. $name is its folder.
kit-new-made = Made { $name }. Run it with cargo run, and test it with cargo test.

# trindle-lang-check, which lists the words a language still lacks: its help.
kit-lang-check-usage =
    Usage: trindle-lang-check [--missing [language]] [folder]
    Checks the language files under the folder (default: this one).
    --missing lists every word each language (or the one named) lacks.
# trindle-lang-check: there is no English folder to check against. $folder is where it looked.
kit-lang-check-no-english = No share/lang/en folder was found in { $folder }.

## The widget gallery that is also the docs (src/showcase.rs, examples/widgets.rs): sample words on each widget.

kit-showcase-title = Widgets and their code
kit-showcase-words = Plain words
kit-showcase-heading = A heading
kit-showcase-note = A quieter line under something
kit-showcase-add = Add
kit-showcase-save = Save
kit-showcase-wifi = Wi-Fi
kit-showcase-agree = I agree
kit-showcase-volume = Volume
kit-showcase-name = Your name
kit-showcase-small = Small
kit-showcase-large = Large
kit-showcase-size = Size
kit-showcase-town = Town
kit-showcase-town-first = Northfield
kit-showcase-town-second = Eastwick
kit-showcase-in-card = Words in a card
