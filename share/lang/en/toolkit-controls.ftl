# Controls, actions and data models: the command palette, checked boxes,
# dials, rows of a settings page, docked panels and the like.

## Command palette

# What the box at the top of the command palette is called; the screen reader says it and it shows while the box is empty.
kit-palette-search = Search actions

# What the palette's list of matching actions is called; the screen reader says it.
kit-palette-list = Matching actions

# Shown under the palette's box when no action matches what was typed.
kit-palette-none = No matching actions.

## Checked boxes

# Red words under a phone number box when what was typed cannot be a phone number.
kit-check-phone = Enter a phone number with 7 to 15 digits.

# Red words under a date box when what was typed is not a real date.
kit-check-date = Enter a date as year-month-day, such as 2026-10-30.

# Red words under a colour box when what was typed is not a colour code.
kit-check-hex = Enter a colour code, such as #fbc711.

## Volume

# Tip of the button beside a volume slider that silences the sound.
kit-volume-mute = Mute

# Tip of the same button while the sound is silenced.
kit-volume-unmute = Unmute

# What the screen reader calls the volume slider.
kit-volume = Volume

## Buttons that show what they hold

# What the screen reader hears for a button filled with a colour: the button's name, then the colour's code such as #fbc711.
kit-colour-button = { $name }, { $code }

# The words on a font button: the font's name and its size in points, such as Trindle Sans 11.
kit-font-button = { $family } { $size }

# The words on a file button while no file has been chosen.
kit-file-none = Choose a file

# Tip of the lock over settings while they are locked.
kit-lock-unlock = Unlock to make changes

# Tip of the lock over settings while they are unlocked.
kit-lock-lock = Lock to stop changes

## Spin boxes

# What the screen reader calls the year in a month and year picker.
kit-year = Year

## Docked panels

# Tip of the button on a panel's head that moves it to the left of the window.
kit-dock-left = Dock on the left

# Tip of the button on a panel's head that moves it to the right of the window.
kit-dock-right = Dock on the right

# Tip of the button on a panel's head that moves it to the foot of the window.
kit-dock-bottom = Dock at the bottom

# Tip of the button on a panel's head that lets it float over the main area.
kit-dock-float = Float above the page

# Name of the handle on a panel's head that is dragged to move the panel to another edge.
kit-dock-grip = Drag to another edge

## Forms

# Red words under a form's box for a number when what was typed is not a number.
kit-form-number = Enter a number, such as 12 or 3.5.

## File chooser page

# The sentence under a file chooser page's title, by what it is for.
kit-files-open-lead = Choose a file to open.
kit-files-save-lead = Choose a folder, then type a name for the file.
kit-files-folder-lead = Choose a folder, or stay in the one you're in.
# The main answer at the foot of a file chooser page.
kit-files-open = Open
kit-files-save = Save
kit-files-folder = Choose this folder
# The button that goes to the folder holding the one shown.
kit-files-up = Up
# The first part of the path, for the top of the computer's folders.
kit-files-computer = Computer
# The switch that shows files whose names start with a dot.
kit-files-hidden = Show hidden files
# What a screen reader calls the list of folders and files.
kit-files-list = Folders and files
# The box for the name of the file to save.
kit-files-name = File name
# Shown in place of the list when the folder holds nothing.
kit-files-empty = This folder is empty.
# Red words when the folder cannot be read.
kit-filepage-unreadable = Trindle couldn't open this folder. Choose a different one.
# Red words after Open with no file chosen.
kit-files-choose-first = Choose a file first.
# Red words after Save with no name, or a name that cannot be a file's.
kit-files-name-needed = Type a name for the file. It can't contain a slash.

## Font chooser page

kit-fonts-lead = Choose a font and a size.
# The box that finds fonts by name.
kit-fonts-search = Find a font by name
# What a screen reader calls the list of fonts.
kit-fonts-list = Fonts
# Beside a font whose letters are all as wide as each other.
kit-fonts-monospace = Monospace
# Shown in place of the list when no font's name matches.
kit-fonts-none = No font has that name.
kit-fonts-size = Size
# The words drawn in the chosen font.
kit-fonts-sample = The quick brown fox jumps over the lazy dog
# What a screen reader says for the sample. $family is the font, $words the sample.
kit-fonts-sample-tip = { $words }, in { $family }
# The main answer at the foot of the font chooser page.
kit-fonts-use = Use this font
# Red words after the answer with no font chosen.
kit-fonts-choose-first = Choose a font first.
