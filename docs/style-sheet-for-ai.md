# The style sheet, for an AI writing one

This file is for an AI (or a person) asked to write a Trindle theme or a person's own style sheet. It lists every key the sheet has, what each one takes, Trindle's own value and what it changes, then gives the plain rules to follow and a full worked example. Follow it exactly: a value the sheet cannot take is passed over and Trindle's own is used in its place.

The keys here are read from `share/style.css`, Trindle's own sheet, which is built into the toolkit. A test holds this file to it, so every key the sheet has is listed below.

## Where a sheet goes

| Sheet | Where it lives | What it may set |
| --- | --- | --- |
| A person's own sheet | `~/.config/trindle/style.css` | Any key, `--trindle-highlight` too. It is laid over the theme and Trindle's own, key by key. |
| A theme | `~/.config/trindle/themes/<name>.css`, one file each, added in Settings, Appearance, Theme | Any key but `--trindle-highlight`, which stays the person's choice. It must also pass the theme check below. |

Either sheet holds only the keys it changes. Every key it leaves out is Trindle's own.

## How a sheet is written

```css
:root {
  --trindle-card-gap: 16;
}
@media (prefers-color-scheme: light) {
  :root {
    --trindle-card: #ffffff;
  }
}
```

- Every key goes in a `:root { ... }` block, one `--name: value;` a line.
- A value for the light look alone goes in a `:root` block inside `@media (prefers-color-scheme: light)`. Without one, the light look uses the same value as the dark.
- Comments are `/* ... */`. Nothing outside a `:root` block is read.

## What a value may be

Each key takes one kind of value, shown in the Takes column of the tables below.

| Takes | Write | Example |
| --- | --- | --- |
| a colour | `#rrggbb`, `#rrggbbaa`, `rgb()`, `rgba()`, `transparent`, another key's colour as `var(--trindle-text)`, or a mix: `shade(colour, by)` (lighter above 1, darker below), `alpha(colour, by)`, `mix(colour, colour, by)` | `#1c2833`, `shade(var(--trindle-card), 1.2)` |
| a number, A to B | A plain number in points, `px` allowed and ignored, or another key's number as `var(--trindle-gap)`. A number past A or B is held at A or B. | `16` |
| milliseconds, A to B | A time, as `250ms` or `3s`; held between A and B | `200ms` |
| megabytes, A to B | A plain number of megabytes; held between A and B | `1000` |
| a shadow | As CSS writes a box shadow, one to three layers parted by commas, or `none` | `0 6px 18px -6px rgba(0, 0, 0, 0.5)` |
| words in quotes | Plain words in double quotes | `"Harbour"` |
| dark, light or both | One of those three words | `dark` |
| a gap step | One gap step's name, from the gap scale below. Never a number. | `medium` |
| a size step | One size step's name, from the size steps below. Never a number. | `large` |

### The gap scale

The fixed room the toolkit's parts keep between and round their own pieces. In Trindle's own sheet each `--trindle-gap-<step>` key is a number of points; in any other sheet a key that takes a gap step names one of these steps, and gets Trindle's own number for it.

| Step | Points |
| --- | --- |
| `none` | 0 |
| `tiny` | 2 |
| `small` | 4 |
| `medium` | 8 |
| `roomy` | 12 |
| `large` | 16 |
| `huge` | 24 |

Every step is a whole number of the base unit, 2 points. To make every small gap in the toolkit a little roomier, write `--trindle-gap-small: medium;`.

### The size steps

A key that takes a size step is a size of one of the larger parts, such as a lane of a board. Its step makes Trindle's own size smaller or larger, rounded to a whole number of the base unit, 2 points.

| Step | Of Trindle's own size |
| --- | --- |
| `smaller` | three quarters |
| `small` | seven eighths |
| `normal` | the same |
| `large` | a quarter more |
| `larger` | half as much again |

So with Trindle's own lane of 260 points, `--trindle-board-lane: larger;` gives 390.

## Rules for an AI filling the sheet in

1. Use only the keys listed in this file, spelt exactly as they are. A key the sheet does not have is passed over and changes nothing.
2. Write only the keys you mean to change. Leave everything else out; it stays Trindle's own.
3. Give each key the kind of value its Takes column says, and nothing else.
4. A key that takes a gap step or a size step takes only a step's name, such as `small` or `larger`. Never write a number for it, such as `75px`: a number there is refused and Trindle's own value is used instead.
5. Keep every number inside the limits its Takes column gives. One outside them is held at the nearest limit, so it does not do what you meant.
6. Every word must stay readable. Words stand at least 4.5 to one from what they are drawn on, and icons, edges and lines at least 3 to one (WCAG 2.2 AA). When you change a pane's colour, check the words on it, in both looks if the sheet has both.
7. A theme never sets `--trindle-highlight`. Only a person's own sheet may.
8. A theme says what it is: `--trindle-theme-name`, `--trindle-theme-about`, `--trindle-theme-author`, `--trindle-theme-version` and `--trindle-theme-look`.
9. A theme is refused whole, and not shown, if it is larger than 64 KB, holds anything but words, brings in another file (`@import`, `url(`, `@font-face`, a web address or a backslash), has a value longer than 200 characters, gives a key a value of the wrong kind (a step key a number, for one), or has no key Trindle knows.
10. Use UK English and sentence case in the theme's own words, with no em dashes.
11. When you have written the sheet, check it with `trindle-sheet-check <file.css>`. It names each problem with its line, and says when there are none. Fix every problem it names.

## A full worked example

A made-up dark theme, Harbour: sea-grey panes, smaller corners, a little more room, wider lanes on a board and a bigger colour swatch. Every value in it is one this guide allows, and `trindle-sheet-check` finds no problems in it.

```css
/* Harbour: sea-grey panes with a little more room. */
:root {
  --trindle-theme-name: "Harbour";
  --trindle-theme-about: "Sea-grey panes, smaller corners and a little more room.";
  --trindle-theme-author: "John Smith";
  --trindle-theme-version: "1.0";
  --trindle-theme-look: dark;

  /* The panes, darkest first, and the words on them. */
  --trindle-page: #101820;
  --trindle-card: #1c2833;
  --trindle-box: #283746;
  --trindle-text: #eef3f7;
  --trindle-text-quiet: #b4c2cf;

  /* Numbers inside their limits. */
  --trindle-corner: 6;
  --trindle-card-gap: 16;
  --trindle-gap: 14;
  --trindle-slide-time: 200ms;

  /* Steps, never numbers. */
  --trindle-gap-small: medium;
  --trindle-gap-medium: roomy;
  --trindle-board-lane: larger;
  --trindle-colour-swatch: large;

  --trindle-card-shadow: 0 6px 18px -6px rgba(0, 0, 0, 0.5);
}
```

What it changes: the page, cards and boxes take the sea greys, and the words stay pale on them. Corners are 6 points, cards stand 16 apart and other things 14. Every gap the toolkit keeps as small becomes 8 points and every medium one 12. A board's lanes are 390 points wide, and the colour picker's swatch 46. A slide takes 200 milliseconds, and cards cast a softer, shorter shadow.

What would be refused: `--trindle-board-lane: 300;` (a number where a size step is asked), `--trindle-gap-small: 5px;` (a number where a gap step is asked), `--trindle-gap-small: wide;` (not one of the steps). Each is passed over, and the checker names it with its line, its steps and this guide.

## Every key

The keys in the order Trindle's own sheet gives them, with the words that sheet says of them. Trindle's own column is the value used when no sheet sets the key.

### Part 0: About this sheet

What a theme says of itself, shown on its tile in Settings: its name, one line about it, who made it, and its version. The look says which look the sheet is: dark, light, or both when it has a light block as this one does. A theme with one look is drawn in it whether Dark or Light is chosen.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-theme-name` | `"Trindle"` | words in quotes |
| `--trindle-theme-about` | `"Trindle's own look: a deep blue night, softly edged."` | words in quotes |
| `--trindle-theme-author` | `"Trindle"` | words in quotes |
| `--trindle-theme-version` | `"1.0"` | words in quotes |
| `--trindle-theme-look` | `both` | dark, light or both |

### Part 1: The theme

The panes, darkest to lightest: night behind, the window and the desktop behind cards; every card and tile; a box inside a card, the current item in a menu, and what floats.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-page` | `#0a111e` | a colour |
| `--trindle-card` | `#172640` | a colour |
| `--trindle-box` | `#233655` | a colour |

Words, and quieter words for hints.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-text` | `#ecf2fb` | a colour |
| `--trindle-text-quiet` | `#aebdd4` | a colour |

The highlight is chosen in Settings, Appearance (yellow by default); a sheet of your own may set it here instead and wins. Pointed at or pressed it is this much darker. Words on it are near black or white, whichever reads better, worked out by Trindle.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-highlight` | `#fbc711` | a colour |
| `--trindle-highlight-hover` | `shade(var(--trindle-highlight), 0.88)` | a colour |

Words on a dark pane that say how things went.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-shade-red` | `#ffa59c` | a colour |

A typing box is cut into its pane: darker than its card, with a clear cool rim.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-typing-box` | `#0c1524` | a colour |
| `--trindle-typing-box-edge` | `#7387a8` | a colour |

Corners. The base corner, and one corner for each kind of part, every one starting as the base. A theme sets any of them: 0 is square, a big number (999) is fully round (half the part's shorter side, a pill or a disc). No shape is set anywhere but here.

| Key | Trindle's own | Takes | What it changes |
| --- | --- | --- | --- |
| `--trindle-corner` | `8` | a number, 0 to 16 |  |
| `--trindle-card-corner` | `var(--trindle-corner)` | a number, 0 to 999 | cards, panels |
| `--trindle-box-corner` | `var(--trindle-corner)` | a number, 0 to 999 | typing boxes, dropdowns, lists, boxes in a card, side bars |
| `--trindle-float-corner` | `var(--trindle-corner)` | a number, 0 to 999 | menus and what floats |
| `--trindle-button-corner` | `var(--trindle-corner)` | a number, 0 to 999 | buttons with words, word buttons |
| `--trindle-icon-button-corner` | `var(--trindle-corner)` | a number, 0 to 999 | buttons with an icon, the eye |
| `--trindle-ring-corner` | `var(--trindle-corner)` | a number, 0 to 999 | buttons in the ring's middle |
| `--trindle-switch-corner` | `var(--trindle-corner)` | a number, 0 to 999 | a switch's track |
| `--trindle-knob-corner` | `var(--trindle-corner)` | a number, 0 to 999 | the knob of a switch or slider |
| `--trindle-slider-corner` | `var(--trindle-corner)` | a number, 0 to 999 | a slider's line |
| `--trindle-tab-corner` | `var(--trindle-corner)` | a number, 0 to 999 | a tab |
| `--trindle-badge-corner` | `var(--trindle-corner)` | a number, 0 to 999 | the badge behind a tile's icon |
| `--trindle-chip-corner` | `var(--trindle-corner)` | a number, 0 to 999 | small word tags |
| `--trindle-segmented-corner` | `var(--trindle-corner)` | a number, 0 to 999 | the strip of a segmented choice |
| `--trindle-tooltip-corner` | `var(--trindle-corner)` | a number, 0 to 999 | the label on pointing |
| `--trindle-scrollbar-corner` | `var(--trindle-corner)` | a number, 0 to 999 | a scroll bar's thumb and track |
| `--trindle-picture-corner` | `var(--trindle-corner)` | a number, 0 to 999 | a person's picture |
| `--trindle-radio-corner` | `var(--trindle-corner)` | a number, 0 to 999 | a radio button and its dot |
| `--trindle-tick-corner` | `4` | a number, 0 to 999 | a tick box: a quarter of its side, so it stays a box |
| `--trindle-row-corner` | `var(--trindle-corner)` | a number, 0 to 999 | a row, a chosen row, a menu line |

Room between cards; between any other two things; round the page; inside a card.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-card-gap` | `12` | a number, 4 to 32 |
| `--trindle-gap` | `12` | a number, 6 to 18 |
| `--trindle-page-padding` | `24` | a number, 8 to 48 |
| `--trindle-card-padding` | `18` | a number, 8 to 40 |

The gap scale: the fixed room the parts keep between and round their own pieces. Here each is a number of points; any other sheet names a step instead (none, tiny, small, medium, roomy, large, huge), which is the number given here. See docs/style-sheet-for-ai.md.

| Key | Trindle's own | Takes | What it changes |
| --- | --- | --- | --- |
| `--trindle-gap-tiny` | `2` | a gap step | a name over the line under it |
| `--trindle-gap-small` | `4` | a gap step | the pieces of one small part |
| `--trindle-gap-medium` | `8` | a gap step | the buttons in a bar, the rows of a short list |
| `--trindle-gap-roomy` | `12` | a gap step | the rows of a form, words beside their control |
| `--trindle-gap-large` | `16` | a gap step | the groups on a page, a page's edge |
| `--trindle-gap-huge` | `24` | a gap step | round a whole page of examples |

Buttons: every button (icon, worded, quiet, word and the segmented strip), dropdown and typing box is this tall, so they line up; an icon in one is this big.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-button-height` | `34` | a number, 28 to 52 |
| `--trindle-icon-size` | `16` | a number, 12 to 32 |

### Part 2: Colours

The palette: for pictures, charts and the highlight picker only, never for words or icons.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-red` | `#f34236` | a colour |
| `--trindle-pink` | `#e8207e` | a colour |
| `--trindle-purple` | `#9b26ae` | a colour |
| `--trindle-deep-purple` | `#6639ab` | a colour |
| `--trindle-indigo` | `#3d51b4` | a colour |
| `--trindle-blue` | `#2295f1` | a colour |
| `--trindle-light-blue` | `#03a8f3` | a colour |
| `--trindle-cyan` | `#00bcd3` | a colour |
| `--trindle-teal` | `#019587` | a colour |
| `--trindle-green` | `#4bae4f` | a colour |
| `--trindle-light-green` | `#8ac248` | a colour |
| `--trindle-lime` | `#cbdc38` | a colour |
| `--trindle-yellow` | `#ffea3a` | a colour |
| `--trindle-amber` | `#ffc006` | a colour |
| `--trindle-orange` | `#fe9700` | a colour |
| `--trindle-deep-orange` | `#ff5721` | a colour |
| `--trindle-brown` | `#795649` | a colour |
| `--trindle-grey` | `#9e9d9e` | a colour |
| `--trindle-blue-grey` | `#607c8b` | a colour |

How things are going, for dots, rings and bars, never words and never the highlight: calm, good (teal, and only as a dot or ring), busy, bad (also a failure's mark, and removing what cannot be got back), and quiet.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-calm` | `var(--trindle-blue)` | a colour |
| `--trindle-good` | `var(--trindle-teal)` | a colour |
| `--trindle-busy` | `var(--trindle-orange)` | a colour |
| `--trindle-bad` | `var(--trindle-red)` | a colour |
| `--trindle-quiet` | `var(--trindle-grey)` | a colour |

What the states and a pie's pieces are drawn in when `--trindle-colour-blind-safe` is 1: Okabe and Ito's palette (2008). The states take sky blue, blue, yellow and vermillion, which differ in lightness as well as hue; the pieces take blue, orange, sky blue, vermillion, bluish green and reddish purple, in that order. The light look has deeper values of its own, so each stands three to one from the page.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-safe-calm` | `#56b4e9` | a colour |
| `--trindle-safe-good` | `#0072b2` | a colour |
| `--trindle-safe-busy` | `#f0e442` | a colour |
| `--trindle-safe-bad` | `#d55e00` | a colour |
| `--trindle-safe-series-1` | `#0072b2` | a colour |
| `--trindle-safe-series-2` | `#e69f00` | a colour |
| `--trindle-safe-series-3` | `#56b4e9` | a colour |
| `--trindle-safe-series-4` | `#d55e00` | a colour |
| `--trindle-safe-series-5` | `#009e73` | a colour |
| `--trindle-safe-series-6` | `#cc79a7` | a colour |

A code view's words by kind (code.rs). Each is moved towards white or black just far enough to be read on what it is drawn on, so any colour here stays readable in the light and the dark look. Never red.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-code-keyword` | `var(--trindle-purple)` | a colour |
| `--trindle-code-type` | `var(--trindle-teal)` | a colour |
| `--trindle-code-string` | `var(--trindle-green)` | a colour |
| `--trindle-code-number` | `var(--trindle-orange)` | a colour |
| `--trindle-code-comment` | `var(--trindle-grey)` | a colour |

What is in use, one colour for each kind, written down once (the microphone is --trindle-busy, the camera --trindle-good and the screen being recorded --trindle-calm). These four are the rest: the mouse and keyboard taken by a program, files just copied, the home network reached and a USB device reached.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-use-input` | `#9b6fd0` | a colour |
| `--trindle-use-files` | `#26b5a8` | a colour |
| `--trindle-use-network` | `#d4537e` | a colour |
| `--trindle-use-usb` | `#e8c11a` | a colour |

A program's own page colour, `--trindle-<page>-page`, where it should differ from the window's. Apps (the store, app id `org.trindle.Store`) has it as `--trindle-store-page`, so a sheet or theme can give the Apps window a page of its own; it is held to the table of pairs like every other page. As it comes, it is the window's own page.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-store-page` | `var(--trindle-page)` | a colour |

What goes with part 1.

A box inside a box, one step on again.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-box-more` | `#2e4468` | a colour |

A control on the page itself is one step lighter than the page; on a card or a box Trindle draws it one step darker than what it sits on.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-control` | `#1b2c49` | a colour |

The ring: its disc, its pieces and its track are this one solid grey, never see-through (a transparency written here is ignored), and held like a card so the words and icons on it stay readable.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-ring-grey` | `#0a111e` | a colour |

A thin edge round the disc, its pieces and the window ring's track, so the ring stands out from any window behind it, light or dark. Kept opaque and, whatever is written here, moved until it stands three to one from both a white and a black window.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-ring-edge` | `#8da6cf` | a colour |

A row the pointer is on; every second row of a list, a band this much darker than what it sits on; a picked row in a list such as files, a quiet wash of the highlight with words in a pale shade of it.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-hover` | `#2b4063` | a colour |
| `--trindle-band` | `rgba(4, 9, 18, 0.12)` | a colour |
| `--trindle-picked` | `mix(var(--trindle-card), var(--trindle-highlight), 0.15)` | a colour |
| `--trindle-on-picked` | `mix(var(--trindle-highlight), #ffffff, 0.35)` | a colour |
| `--trindle-highlight-text` | `var(--trindle-highlight)` | a colour |

The highlight made softer, blended into the card: a switch that is on, a slider's fill, a ring or bar of work and a chart's bars. The full highlight is kept for the one main action and for what the pointer or the keyboard is on.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-highlight-soft` | `mix(var(--trindle-card), var(--trindle-highlight), 0.65)` | a colour |

A folder's icon, wherever a folder is shown as a thing: a pale, light tint of the highlight, mixed well towards white, so it follows whatever highlight is chosen. A folder given a colour of its own in Folders keeps that. Where this colour cannot be told from what it is on at three to one, Trindle draws the pale shade of a picked row's words there instead.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-folder-icon` | `mix(var(--trindle-highlight), #ffffff, 0.6)` | a colour |

Every other icon drawn in the words' colour (a file's, a toolbar's, a side bar's): a little softer than the words, a slightly off-white; in the light look, the words' own.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-icon` | `mix(var(--trindle-text), var(--trindle-card), 0.1)` | a colour |

The deepest dark, below the page, as behind the lock screen.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-well` | `#050a13` | a colour |

Edges: the fine light edge a card and a box each catch; controls have none. The line down a side bar's far edge is fainter still.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-edge` | `rgba(190, 215, 255, 0.08)` | a colour |
| `--trindle-box-edge` | `rgba(190, 215, 255, 0.11)` | a colour |
| `--trindle-control-edge` | `transparent` | a colour |
| `--trindle-side-line` | `rgba(190, 215, 255, 0.07)` | a colour |

The light line between two sections of a side bar, in Apps and in Settings: one rule for both.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-section-line` | `rgba(190, 215, 255, 0.16)` | a colour |
| `--trindle-section-line-width` | `1` | a number, 1 to 3 |

The two colours of pictures and charts: the panes everywhere, the highlight the only accent, and pictures use at most two other colours, the complement of the highlight first and a neighbour of it second, so they follow the highlight when it is changed in Appearance. Each line names the palette colour nearest the highlight, then the first and the second picture colour; pictures never name a colour of their own. Green and red are kept for a real status.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-pair-red-first` | `var(--trindle-cyan)` | a colour |
| `--trindle-pair-red-second` | `var(--trindle-teal)` | a colour |
| `--trindle-pair-pink-first` | `var(--trindle-teal)` | a colour |
| `--trindle-pair-pink-second` | `var(--trindle-cyan)` | a colour |
| `--trindle-pair-purple-first` | `var(--trindle-green)` | a colour |
| `--trindle-pair-purple-second` | `var(--trindle-teal)` | a colour |
| `--trindle-pair-deep-purple-first` | `var(--trindle-yellow)` | a colour |
| `--trindle-pair-deep-purple-second` | `var(--trindle-orange)` | a colour |
| `--trindle-pair-indigo-first` | `var(--trindle-yellow)` | a colour |
| `--trindle-pair-indigo-second` | `var(--trindle-orange)` | a colour |
| `--trindle-pair-blue-first` | `var(--trindle-orange)` | a colour |
| `--trindle-pair-blue-second` | `var(--trindle-yellow)` | a colour |
| `--trindle-pair-light-blue-first` | `var(--trindle-red)` | a colour |
| `--trindle-pair-light-blue-second` | `var(--trindle-pink)` | a colour |
| `--trindle-pair-cyan-first` | `var(--trindle-red)` | a colour |
| `--trindle-pair-cyan-second` | `var(--trindle-pink)` | a colour |
| `--trindle-pair-teal-first` | `var(--trindle-red)` | a colour |
| `--trindle-pair-teal-second` | `var(--trindle-pink)` | a colour |
| `--trindle-pair-green-first` | `var(--trindle-purple)` | a colour |
| `--trindle-pair-green-second` | `var(--trindle-deep-purple)` | a colour |
| `--trindle-pair-light-green-first` | `var(--trindle-purple)` | a colour |
| `--trindle-pair-light-green-second` | `var(--trindle-blue)` | a colour |
| `--trindle-pair-lime-first` | `var(--trindle-indigo)` | a colour |
| `--trindle-pair-lime-second` | `var(--trindle-blue)` | a colour |
| `--trindle-pair-yellow-first` | `var(--trindle-blue)` | a colour |
| `--trindle-pair-yellow-second` | `var(--trindle-cyan)` | a colour |
| `--trindle-pair-amber-first` | `var(--trindle-blue)` | a colour |
| `--trindle-pair-amber-second` | `var(--trindle-cyan)` | a colour |
| `--trindle-pair-orange-first` | `var(--trindle-blue)` | a colour |
| `--trindle-pair-orange-second` | `var(--trindle-cyan)` | a colour |
| `--trindle-pair-deep-orange-first` | `var(--trindle-cyan)` | a colour |
| `--trindle-pair-deep-orange-second` | `var(--trindle-teal)` | a colour |
| `--trindle-pair-brown-first` | `var(--trindle-blue)` | a colour |
| `--trindle-pair-brown-second` | `var(--trindle-orange)` | a colour |
| `--trindle-pair-grey-first` | `var(--trindle-blue)` | a colour |
| `--trindle-pair-grey-second` | `var(--trindle-orange)` | a colour |
| `--trindle-pair-blue-grey-first` | `var(--trindle-blue)` | a colour |
| `--trindle-pair-blue-grey-second` | `var(--trindle-orange)` | a colour |

Shadows, as CSS writes a box-shadow, up to three layers, each layer's colour giving only its strength, drawn in --trindle-shadow; down 0 to 32, blur 0 to 64, spread -32 to 16, or none. A card casts a gentle one; what floats a wide soft one over a short dark one; a typing box has a soft shade inside its top edge; a box has none.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-shadow` | `#02060e` | a colour |
| `--trindle-float-shadow` | `0 22px 54px 0 rgba(0, 0, 0, 0.55), 0 3px 10px 0 rgba(0, 0, 0, 0.40)` | a shadow |
| `--trindle-card-shadow` | `0 8px 22px -6px rgba(0, 0, 0, 0.42)` | a shadow |
| `--trindle-box-shadow` | `none` | a shadow |
| `--trindle-typing-box-shadow` | `inset 0 2px 5px 0 rgba(0, 0, 0, 0.45)` | a shadow |

### Part 3: Type

Trindle Sans: each kind's size in points and weight from 100 (thin) to 900 (black). Sentence case everywhere; a hint is never larger than ordinary words.

Two modes a person turns on in their own sheet. Large text draws every kind of words this many times larger, and what holds a line of them (buttons, typing boxes, rows) as much taller. High contrast, at 1, holds words and icons to seven to one on everything they lie on and every edge and line to four and a half. Motion: a slide and a fade take this long, and reduced motion, at 1, makes every one instant. Colour-blind safe, at 1, draws the states (calm, good, busy, bad) and a pie's pieces in Okabe and Ito's colour-universal palette, set in the `--trindle-safe-*` keys below, so they can be told apart in deuteranopia, protanopia and tritanopia; the look is not otherwise changed. A desktop that asks for more contrast or less motion through its settings gets the same, whatever its sheet says.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-large-text` | `1` | a number, 1 to 2 |
| `--trindle-high-contrast` | `0` | a number, 0 to 1 |
| `--trindle-slide-time` | `240ms` | milliseconds, 0 to 1000 (or seconds, as 3s) |
| `--trindle-fade-time` | `160ms` | milliseconds, 0 to 1000 (or seconds, as 3s) |
| `--trindle-reduce-motion` | `0` | a number, 0 to 1 |
| `--trindle-colour-blind-safe` | `0` | a number, 0 to 1 |

A page's own name, and a part of a page's; large names sit their letters this much closer, as a share of the size.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-page-title-size` | `36` | a number, 22 to 44 |
| `--trindle-page-title-weight` | `400` | a number, 100 to 900 |
| `--trindle-section-size` | `24` | a number, 18 to 36 |
| `--trindle-section-weight` | `400` | a number, 100 to 900 |
| `--trindle-title-tighten` | `0.02` | a number, 0 to 0.06 |

A larger sentence, as on the Welcome.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-lead-size` | `22` | a number, 16 to 30 |
| `--trindle-lead-weight` | `500` | a number, 100 to 900 |

A card's name.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-heading-size` | `16` | a number, 14 to 24 |
| `--trindle-heading-weight` | `600` | a number, 100 to 900 |

The one headline sentence under a page's title.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-headline-size` | `15` | a number, 14 to 24 |
| `--trindle-headline-weight` | `400` | a number, 100 to 900 |

A figure in a fact tile.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-figure-size` | `17` | a number, 14 to 48 |
| `--trindle-figure-weight` | `700` | a number, 100 to 900 |

A big figure in a stat tile.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-big-figure-size` | `40` | a number, 24 to 72 |
| `--trindle-big-figure-weight` | `600` | a number, 100 to 900 |

Ordinary words, a setting's name, and hints.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-text-size` | `14` | a number, 12 to 22 |
| `--trindle-text-weight` | `400` | a number, 100 to 900 |
| `--trindle-form-label-size` | `14` | a number, 12 to 20 |
| `--trindle-form-label-weight` | `600` | a number, 100 to 900 |
| `--trindle-note-size` | `13` | a number, 12 to 18 |
| `--trindle-note-weight` | `400` | a number, 100 to 900 |

A table's column headings.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-column-heading-size` | `13` | a number, 12 to 18 |
| `--trindle-column-heading-weight` | `600` | a number, 100 to 900 |

A group's name in a side bar (Places, Pinned; Apps and Settings sections), the same in every program.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-side-heading-size` | `11` | a number, 10 to 16 |
| `--trindle-side-heading-weight` | `400` | a number, 100 to 900 |

### Part 4: Parts

Round icon buttons, the main button, and the label on pointing.

Every button in the ring's middle, tray icons too, is one size (24 since 18 October 2026, a little smaller so more fit a row).

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-ring-button-size` | `24` | a number, 20 to 40 |

The gap between buttons in a band of the middle, the room round them inside the band's dark strip, and the gap between one band and the next, in every row alike.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-ring-button-gap` | `4` | a number, 2 to 12 |
| `--trindle-ring-band-room` | `4` | a number, 2 to 12 |
| `--trindle-ring-band-gap` | `4` | a number, 2 to 12 |

Each kind of job has its own colour for its buttons in the middle: the control colour with this share of the highlight blended in, so it is a very dark shade of the look's blue and follows whatever highlight is chosen. Settings, Apps, Activity and Drives; power, sound, the microphone, Bluetooth, Wi-Fi, the keyboard, brightness and updates; the tools (screenshot to keep awake); other programs' tray icons. Words and icons on each are kept to WCAG AA whatever the share.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-ring-job-programs` | `0.14` | a number, 0 to 0.3 |
| `--trindle-ring-job-computer` | `0` | a number, 0 to 0.3 |
| `--trindle-ring-job-tools` | `0.07` | a number, 0 to 0.3 |
| `--trindle-ring-job-tray` | `0.24` | a number, 0 to 0.3 |

The highlight's edge round an app's icon in the ring, under the pointer or as the best match: it follows the icon's own shape.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-ring-app-edge` | `2.5` | a number, 1 to 6 |

The ring of open windows, outside the favourites. Its width, and the least width worth drawing (below it the ring is left out), the gap between it and the favourites' disc, the room each side of a tab's icon, how much firmer a tab is than its part colour (per cent towards the ink), the gap between one window's part and the next (across, at the middle of the ring's width), the window's icon with its name beside it and the icon alone, the name's size, the dot on the window in front, and the live picture's width on pointing. Its colours are the look's own, so they follow light and dark: a part is `--trindle-control`, one under the pointer `--trindle-hover`, the dot `--trindle-text-quiet` and the names `--trindle-text`.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-ring-windows-width` | `88` | a number, 56 to 140 |
| `--trindle-ring-windows-width-least` | `56` | a number, 40 to 80 |
| `--trindle-ring-windows-gap` | `0` | a number, 0 to 20 |
| `--trindle-ring-windows-part-gap` | `4` | a number, 2 to 12 |
| `--trindle-ring-windows-icon-alone` | `36` | a number, 20 to 48 |
| `--trindle-ring-windows-track` | `0` | a number, 0 to 24 |
| `--trindle-ring-edge-width` | `1.5` | a number, 1 to 3 |

How much of the edge colour shows over the fill: a quiet line, not a bright one.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-ring-edge-strength` | `0.1` | a number, 0.1 to 1 |
| `--trindle-ring-windows-tab-firm` | `6` | a number, 0 to 20 |
| `--trindle-ring-windows-tab-front` | `16` | a number, 6 to 40 |
| `--trindle-ring-windows-tab-pad` | `6` | a number, 2 to 16 |
| `--trindle-ring-windows-name-width` | `64` | a number, 40 to 120 |
| `--trindle-ring-windows-title-size` | `14` | a number, 11 to 20 |
| `--trindle-ring-windows-title-gap` | `8` | a number, 4 to 16 |
| `--trindle-ring-windows-dot` | `6` | a number, 3 to 12 |
| `--trindle-ring-windows-picture` | `220` | a number, 120 to 320 |
| `--trindle-ring-windows-picture-margin` | `16` | a number, 6 to 40 |
| `--trindle-ring-windows-picture-corner` | `14` | a number, 0 to 40 |
| `--trindle-ring-windows-picture-icon` | `96` | a number, 48 to 160 |

The one main action on a screen: a pill in the highlight, its room at either end and between icon and words, and its words.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-action-padding` | `16` | a number, 8 to 32 |
| `--trindle-action-gap` | `8` | a number, 4 to 16 |
| `--trindle-action-size` | `14` | a number, 12 to 20 |
| `--trindle-action-weight` | `700` | a number, 100 to 900 |

A button that says its choice in a word, such as Dark and Light on a theme's tile: a pill at least this wide, with this much room at either end, its word this size. Its colours are the round buttons' own: the quiet wash of the highlight when it is in use.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-word-button-width` | `56` | a number, 44 to 72 |
| `--trindle-word-button-padding` | `10` | a number, 6 to 16 |
| `--trindle-word-button-size` | `13` | a number, 12 to 16 |

The label on pointing: a small floating pane of its own, a step lighter than a box, with light words, a fine light edge and a soft shadow.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-label-background` | `#2e4468` | a colour |
| `--trindle-label-ink` | `#f3f7fd` | a colour |
| `--trindle-label-edge` | `rgba(200, 222, 255, 0.20)` | a colour |
| `--trindle-label-outline` | `transparent` | a colour |
| `--trindle-label-shadow-y` | `6` | a number, 0 to 12 |
| `--trindle-label-shadow-blur` | `16` | a number, 0 to 24 |
| `--trindle-label-shadow-strength` | `0.5` | a number, 0 to 1 |
| `--trindle-label-size` | `12` | a number, 10 to 18 |
| `--trindle-label-weight` | `600` | a number, 100 to 900 |
| `--trindle-label-padding-x` | `12` | a number, 2 to 16 |
| `--trindle-label-padding-y` | `6` | a number, 1 to 10 |
| `--trindle-label-gap` | `6` | a number, 0 to 16 |

A label that shows the whole of something long (a thing copied, a name cut short) wraps inside this width, in points, and has no more than this many lines: the last then says how much more there is.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-label-width-most` | `420` | a number, 160 to 960 |
| `--trindle-label-lines-most` | `12` | a number, 3 to 40 |

It shows once the pointer has rested this long, and goes this long after, unless the person keeps labels while pointing. A label that tells the whole of something stays while the pointer rests on it.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-label-rest` | `400ms` | milliseconds, 0 to 2000 (or seconds, as 3s) |
| `--trindle-label-stays` | `1500ms` | milliseconds, 500 to 10000 (or seconds, as 3s) |

The notice at the foot of a window stays this long before the next waiting takes its place.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-notice-stays` | `5000ms` | milliseconds, 2000 to 20000 (or seconds, as 3s) |

A link's label shows the address it opens: the host in full, then as much of the path as fits this many characters, cut with dots.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-link-label-most` | `48` | a number, 24 to 80 |

The dots in a screen's top right corner that say the microphone, camera, screen or the mouse and keyboard is in use, and the label they show on pointing. The compositor draws them: across one dot, between two, from the screen's corner, the thin dark edge round each (its width and how dark), how far round them the pointer still counts as on them, and the room round the label's words.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-dots-size` | `28` | a number, 16 to 48 |
| `--trindle-dots-gap` | `8` | a number, 2 to 16 |
| `--trindle-dots-margin` | `0` | a number, 0 to 24 |
| `--trindle-dots-edge` | `2` | a number, 0 to 3 |
| `--trindle-dots-edge-strength` | `0.55` | a number, 0 to 1 |
| `--trindle-dots-reach` | `10` | a number, 0 to 20 |
| `--trindle-dots-label-padding` | `8` | a number, 4 to 16 |

The level shown low in the middle of the screen for a moment after a volume, microphone or brightness key, drawn by the compositor in the title bars' colours: the room before the sign and round the words, the sign, the room between its parts, the bar's length and thickness, the pill's height, how far it floats above the screen's bottom edge, how long it stays (milliseconds), and how much of the words' colour tints the bar's empty track.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-level-padding` | `16` | a number, 8 to 32 |
| `--trindle-level-icon` | `22` | a number, 16 to 32 |
| `--trindle-level-gap` | `12` | a number, 6 to 24 |
| `--trindle-level-bar-width` | `160` | a number, 80 to 320 |
| `--trindle-level-bar-height` | `6` | a number, 3 to 12 |
| `--trindle-level-height` | `48` | a number, 36 to 72 |
| `--trindle-level-bottom` | `64` | a number, 0 to 200 |
| `--trindle-level-stays` | `1500ms` | milliseconds, 500 to 5000 (or seconds, as 3s) |
| `--trindle-level-track` | `0.30` | a number, 0 to 0.5 |

The pill in the middle of the screen that says a part has stopped, drawn by the compositor in the title bars' colours: the room before its alert sign, the sign, the room round the words and the round button, the button, the pill's height (the button and the signs never outgrow it), the button's sign, and how much of the words' colour tints the button at rest and with the pointer on it.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-stopped-padding` | `16` | a number, 8 to 32 |
| `--trindle-stopped-icon` | `22` | a number, 16 to 32 |
| `--trindle-stopped-gap` | `12` | a number, 6 to 24 |
| `--trindle-stopped-button` | `34` | a number, 24 to 44 |
| `--trindle-stopped-height` | `52` | a number, 40 to 72 |
| `--trindle-stopped-again-icon` | `18` | a number, 12 to 28 |
| `--trindle-stopped-tint` | `0.14` | a number, 0 to 0.5 |
| `--trindle-stopped-tint-on` | `0.30` | a number, 0 to 0.6 |

Pointing at anything clickable edges it in the highlight, this thick; nothing moves or lifts.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-hover-edge` | `1` | a number, 1 to 3 |

The small round i beside a heading, that floats more words.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-i-size` | `24` | a number, 18 to 34 |

Remove asks twice: how long the first press keeps it armed.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-remove-armed` | `3s` | milliseconds, 2000 to 10000 (or seconds, as 3s) |

Work that ends says how it ended, in its row at the foot, this long, then the row goes; it is also the least time any work is shown.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-work-done` | `3s` | milliseconds, 1000 to 10000 (or seconds, as 3s) |

A copy or move this big (megabytes, 100 to 100000), or that has run this long, says in its row that other programs may feel slower.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-work-heavy-mb` | `1000` | megabytes, 100 to 100000 |
| `--trindle-work-heavy-after` | `10s` | milliseconds, 2000 to 120000 (or seconds, as 3s) |

Switches, sliders, typing boxes, strips and the keyboard's ring.

The keyboard's ring: this thick, this far outside the widget. On a table's cell or heading and an icon tile it lies just inside, so only the thickness is read.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-focus-width` | `2` | a number, 1 to 4 |
| `--trindle-focus-offset` | `3` | a number, 0 to 4 |

A switch, its knob with a little shade under it, and a slider's track.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-switch-width` | `44` | a number, 32 to 64 |
| `--trindle-switch-height` | `24` | a number, 16 to 36 |
| `--trindle-switch-knob` | `18` | a number, 10 to 30 |
| `--trindle-knob` | `#0a111e` | a colour |
| `--trindle-knob-shadow` | `rgba(0, 0, 0, 0.35)` | a colour |

A switch's knob while it is off: light on the dark track.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-knob-off` | `#c6d2e4` | a colour |
| `--trindle-slider-track` | `6` | a number, 2 to 12 |

A typing box's height, its words' room from its ends, the caret, and a selection as a share of the highlight.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-entry-height` | `var(--trindle-button-height)` | a number, 28 to 52 |
| `--trindle-entry-padding` | `12` | a number, 4 to 24 |

The eye at the right end of a password box, which shows what is typed while it is on: a round button this far across, kept inside the box's edge however tall the box is.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-entry-eye` | `26` | a number, 24 to 44 |
| `--trindle-caret` | `1.5` | a number, 1 to 4 |

The wavy line under a word a spell checker does not know: never red, which is kept for failures, and as thick as the caret.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-spelling` | `#6ab4f5` | a colour |
| `--trindle-selection` | `0.35` | a number, 0.1 to 0.8 |

Letters marked in a label or list row, such as what a search matched: a wash of the highlight behind them, this strong.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-mark` | `0.45` | a number, 0.1 to 0.8 |

A box that has just saved itself glows in the highlight: this long, this far out, this strong.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-glow-time` | `1600ms` | milliseconds, 300 to 5000 (or seconds, as 3s) |
| `--trindle-glow-width` | `5` | a number, 1 to 8 |
| `--trindle-glow-strength` | `0.45` | a number, 0.1 to 1 |

A program's bell flashes the window's edge in the highlight, this long, as wide as the glow.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-bell-time` | `250ms` | milliseconds, 100 to 1000 (or seconds, as 3s) |

The room inside the dark strip of a segmented choice, and round a small word tag.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-tabs-padding` | `3` | a number, 2 to 12 |
| `--trindle-chip-padding-x` | `12` | a number, 4 to 20 |
| `--trindle-chip-padding-y` | `3` | a number, 0 to 8 |

Lists, scroll bars, side bars and what shows work going on.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-row-height` | `40` | a number, 28 to 64 |

A rich row: a name in bold over a plain line, with its icon.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-rich-row-height` | `54` | a number, 40 to 80 |
| `--trindle-rich-row-gap` | `2` | a number, 0 to 8 |
| `--trindle-rich-row-icon` | `28` | a number, 16 to 48 |
| `--trindle-rich-row-weight` | `600` | a number, 100 to 900 |

How far each level of a tree view is set in from the one holding it.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-tree-indent` | `20` | a number, 8 to 40 |

A title bar a program draws itself, where the compositor draws none: the room round its buttons, the edges that resize the window, and the room round the window its float shadow falls in (none when maximised or tiled; 0 for no shadow).

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-title-bar-room` | `5` | a number, 2 to 12 |
| `--trindle-frame-edge` | `6` | a number, 2 to 12 |
| `--trindle-frame-shadow` | `24` | a number, 0 to 48 |

The line between a split's two sides, and how thick it grows under the pointer, while dragged and with the keyboard on it.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-divider` | `1` | a number, 1 to 4 |
| `--trindle-divider-lit` | `4` | a number, 2 to 8 |

Inside a snug card round a list, round what a side bar holds, and a flat line in a menu.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-snug-padding` | `6` | a number, 0 to 12 |
| `--trindle-sidebar-room` | `12` | a number, 4 to 20 |
| `--trindle-line-padding` | `6` | a number, 2 to 12 |

A scroll bar: thin, thicker under the pointer, in from the edge.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-scrollbar` | `4` | a number, 2 to 10 |
| `--trindle-scrollbar-hover` | `8` | a number, 4 to 16 |
| `--trindle-scrollbar-inset` | `2` | a number, 0 to 6 |

How strong its thumb is, in the quiet words' colour: at rest, a faint line that does not catch the eye at a window's edge, and under the pointer.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-scrollbar-rest` | `0.3` | a number, 0.2 to 0.8 |
| `--trindle-scrollbar-lit` | `0.8` | a number, 0.5 to 1 |

The ring that turns while something loads, and its line.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-spinner` | `36` | a number, 20 to 64 |
| `--trindle-spinner-line` | `4` | a number, 2 to 8 |

A panel beside a list showing what is chosen: its width until it is dragged, and the height of the picture at its top.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-preview-panel` | `300` | a number, 240 to 480 |
| `--trindle-preview-picture` | `220` | a number, 120 to 400 |

A card for one drive, at its narrowest, and the bar that shows how full something is.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-drive-card` | `300` | a number, 240 to 480 |
| `--trindle-fill-bar` | `10` | a number, 4 to 20 |

How thick the line of a line, area or spark chart is.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-chart-line` | `2.5` | a number, 1 to 6 |

A radio button's side and a tick box's side.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-radio-size` | `20` | a number, 16 to 28 |
| `--trindle-tick-size` | `18` | a number, 14 to 28 |

Sizes of the larger parts. Here each is a number of points; any other sheet names a step instead (smaller, small, normal, large, larger), which makes the number here smaller or larger.

| Key | Trindle's own | Takes | What it changes |
| --- | --- | --- | --- |
| `--trindle-colour-square-width` | `220` | a size step | the colour picker's square, across |
| `--trindle-colour-square-height` | `150` | a size step | the colour picker's square, down |
| `--trindle-colour-swatch` | `36` | a size step | the swatch of the colour chosen |
| `--trindle-colour-swatch-corner` | `6` | a number, 0 to 999 | the corners of that swatch |
| `--trindle-colour-code-width` | `120` | a size step | the box for a colour's code |
| `--trindle-gradient-bar` | `28` | a size step | a gradient's bar, down |
| `--trindle-volume-width` | `96` | a size step | a player's volume slider |
| `--trindle-photo-thumb` | `56` | a size step | a picture in a strip of photos |
| `--trindle-board-lane` | `260` | a size step | a lane of a board, across |
| `--trindle-board-card` | `56` | a size step | a card in a lane, down |
| `--trindle-board-head` | `40` | a size step | a lane's heading, down |
| `--trindle-table-row` | `36` | a size step | a row of a table, down |
| `--trindle-table-head` | `32` | a size step | the heading line of a table, down |
| `--trindle-info-icon` | `20` | a size step | the icon at the start of a line of information |

The small picture of amounts on a tile: how tall, each column at its widest and least, the room between and the corner.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-bars-height` | `40` | a number, 24 to 80 |
| `--trindle-bars-width` | `18` | a number, 6 to 32 |
| `--trindle-bars-least` | `6` | a number, 2 to 12 |
| `--trindle-bars-gap` | `6` | a number, 2 to 12 |
| `--trindle-bars-corner` | `4` | a number, 0 to 999 |

Grey lines where words will come: each line, the room between, and how much of the width the last takes.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-skeleton-line` | `12` | a number, 8 to 24 |
| `--trindle-skeleton-gap` | `10` | a number, 4 to 20 |
| `--trindle-skeleton-last` | `0.6` | a number, 0.3 to 1 |

How much of a drive hidden from Folders still shows in Drives: its name and picture are quieter, and still read easily.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-hidden-drive` | `0.7` | a number, 0.65 to 0.9 |

A card's lit edge, and the line of a job round its round button.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-lit-edge` | `3` | a number, 1 to 6 |
| `--trindle-working-line` | `3` | a number, 1 to 6 |

A state dot, as the update dot; the circle a card's icon sits in.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-state-dot` | `16` | a number, 10 to 24 |
| `--trindle-badge` | `var(--trindle-button-height)` | a number, 28 to 56 |
| `--trindle-badge-icon` | `var(--trindle-icon-size)` | a number, 12 to 32 |

Floating things: a small menu, as opens on a right click (a widget's `Node::context_menu` too), its width, one row, the room round its rows and its icons; and their corners.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-menu-width` | `272` | a number, 200 to 360 |
| `--trindle-menu-row` | `var(--trindle-button-height)` | a number, 24 to 44 |
| `--trindle-menu-padding` | `8` | a number, 2 to 12 |
| `--trindle-menu-icon` | `var(--trindle-icon-size)` | a number, 12 to 24 |

A sub-menu, which opens beside a row of a menu: how long the pointer rests on the row before it opens, and on another row before it shuts; how far it laps over its menu; and the arrow at the end of its row.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-menu-sub-pause` | `250ms` | milliseconds, 0 to 1000 (or seconds, as 3s) |
| `--trindle-menu-sub-overlap` | `4` | a number, 0 to 12 |
| `--trindle-menu-sub-arrow` | `12` | a number, 8 to 24 |

The room each side of a title in a menu bar; the room between one group of a menu's rows and the next; the room left between a menu and the window's edge; and the least a row is squeezed to in a short window.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-menu-title-pad` | `16` | a number, 0 to 32 |
| `--trindle-menu-group-gap` | `8` | a number, 0 to 24 |
| `--trindle-menu-edge` | `4` | a number, 0 to 16 |
| `--trindle-menu-row-least` | `24` | a number, 16 to 44 |

A bar of tabs: its height, the narrowest and widest a tab goes, its close box, the button for the tabs that did not fit, the room each side of a tab's words, how far a press moves before it is a drag, how far above or below the bar a tab is torn off it, and the room for a count's figures.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-tab-height` | `40` | a number, 28 to 72 |
| `--trindle-tab-least` | `112` | a number, 60 to 240 |
| `--trindle-tab-most` | `240` | a number, 120 to 480 |
| `--trindle-tab-close` | `24` | a number, 24 to 48 |
| `--trindle-tab-more` | `36` | a number, 24 to 64 |
| `--trindle-tab-pad` | `10` | a number, 0 to 24 |
| `--trindle-tab-slop` | `6` | a number, 0 to 16 |
| `--trindle-tab-tear` | `48` | a number, 16 to 160 |
| `--trindle-tab-count` | `28` | a number, 16 to 64 |

The notice's pill: its height, the room each side of its words, the widest it grows, the clear space round it for its shadow, how far its foot is from the foot of the screen, and the size of its words.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-pill-height` | `34` | a number, 24 to 64 |
| `--trindle-pill-side` | `16` | a number, 4 to 40 |
| `--trindle-pill-widest` | `640` | a number, 240 to 1200 |
| `--trindle-pill-room` | `40` | a number, 8 to 120 |
| `--trindle-pill-foot` | `48` | a number, 8 to 160 |
| `--trindle-pill-words` | `14` | a number, 10 to 24 |

A dropdown: its closed box's height, one row of its open list, how many rows show before the list scrolls and gains its search box, and how wide the open list may be.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-dropdown-height` | `var(--trindle-button-height)` | a number, 28 to 52 |
| `--trindle-dropdown-row` | `32` | a number, 24 to 44 |
| `--trindle-dropdown-rows` | `8` | a number, 3 to 16 |
| `--trindle-dropdown-width` | `460` | a number, 200 to 720 |

Pages and panels: a side bar of pages beside the page it shows; the window width under which a split view shows one side at a time; a drawer laid over the page, and the room of the page always left beside it to press.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-sidebar-width` | `240` | a number, 180 to 360 |
| `--trindle-split-narrow` | `600` | a number, 360 to 1200 |
| `--trindle-drawer-width` | `320` | a number, 240 to 480 |
| `--trindle-drawer-room` | `56` | a number, 44 to 120 |

A tile of a grid of icons, square, and the icon in it.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-icon-tile` | `112` | a number, 88 to 160 |
| `--trindle-icon-tile-icon` | `48` | a number, 32 to 96 |

The large icon at the top of a status, welcome or about page.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-status-icon` | `96` | a number, 48 to 160 |

A pager's dot, and the target round it a press finds.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-dot` | `8` | a number, 6 to 16 |
| `--trindle-dot-target` | `24` | a number, 24 to 44 |

How fast a scrolling area flicked by a finger or a touchpad slows, a share of its speed each second: more stops it sooner.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-fling-friction` | `4` | a number, 1 to 12 |

Puzzle grids of cards: each tile a whole number of twelfths of the width. A card of settings, a tile of one figure, the page's picture: the fewest and the most. The narrowest a card of words, or of a figure, may be, in points.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-bento-card-least` | `4` | a number, 3 to 6 |
| `--trindle-bento-card-most` | `6` | a number, 4 to 8 |
| `--trindle-bento-figure-least` | `3` | a number, 2 to 4 |
| `--trindle-bento-figure-most` | `4` | a number, 2 to 6 |
| `--trindle-bento-picture-least` | `6` | a number, 4 to 8 |
| `--trindle-bento-picture-most` | `8` | a number, 5 to 9 |
| `--trindle-bento-words` | `300px` | a number, 200 to 420 |
| `--trindle-bento-figure-words` | `180px` | a number, 100 to 260 |

How tall the picture of what a page is about is at the least.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-picture-height` | `240` | a number, 160 to 360 |

### Part 5: Places

The Welcome, the tour at the first login, in points: the window when there is room, the smallest it is laid out for, the room kept at a page's top for its name and two lines of words, an app tile's height at most and at least, and one run of a moving picture in seconds.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-welcome-width` | `1040px` | a number, 640 to 1800 |
| `--trindle-welcome-height` | `680px` | a number, 440 to 1200 |
| `--trindle-welcome-least-width` | `860px` | a number, 640 to 1200 |
| `--trindle-welcome-least-height` | `540px` | a number, 440 to 900 |
| `--trindle-welcome-header` | `108px` | a number, 80 to 160 |
| `--trindle-welcome-tile` | `64px` | a number, 40 to 72 |
| `--trindle-welcome-tile-least` | `40px` | a number, 36 to 52 |
| `--trindle-welcome-run` | `7` | a number, 4 to 14 |

The page Apps shows for one app, in points. It is laid out from these and the window's width alone, so every app's page is the same: the width under which it takes its narrow arrangement; the app's icon; the room kept for the one main action; the picture's frame, this share of the page's width tall and held between a least and a most; the rating's ring and the room for its figure; and a bar of the spread of stars with the room for the star's number and for its count.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-app-page-narrow` | `640px` | a number, 480 to 1000 |
| `--trindle-app-icon` | `48px` | a number, 32 to 96 |
| `--trindle-app-action-width` | `132px` | a number, 96 to 240 |
| `--trindle-app-picture-shape` | `0.5` | a number, 0.3 to 0.8 |
| `--trindle-app-picture-least` | `200px` | a number, 120 to 360 |
| `--trindle-app-picture-most` | `560px` | a number, 240 to 720 |
| `--trindle-app-rating-ring` | `72px` | a number, 48 to 120 |
| `--trindle-app-rating-figure` | `56px` | a number, 40 to 96 |
| `--trindle-app-rating-bar` | `8px` | a number, 4 to 16 |
| `--trindle-app-rating-bar-name` | `36px` | a number, 24 to 64 |
| `--trindle-app-rating-bar-count` | `48px` | a number, 32 to 96 |

Apps' lists, in points. Every list of apps is one design, a picture, a name, one line and what can be done at the far end: as a tile in a grid of equal columns (Discover, a kind, results) or as a row of a list (Installed, Updates, Plugins and the rest). The least a tile is wide, which sets how many columns fit; the picture in a tile and in a row; the least a plugin's card is wide, which has more to say; and how much of the picture of a thing set aside still shows, such as an update held back, out of one.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-apps-tile-least` | `280px` | a number, 220 to 420 |
| `--trindle-apps-tile-icon` | `48px` | a number, 32 to 72 |
| `--trindle-apps-row-icon` | `36px` | a number, 24 to 56 |
| `--trindle-apps-card-least` | `360px` | a number, 280 to 520 |
| `--trindle-apps-held` | `0.4` | a number, 0.2 to 0.8 |

-------------------------------------------------------------------------- Folders: how close its rows sit (11 October 2026). Folders' own settings offer three levels, Compact, Normal and Roomy, and each level has its own set of sizes here, in points: a row of the List view with one line of words, and with two (a name over what it is); a line of the Compact view; a row of the sidebar; the room between a pane's parts (its tabs, its path bar, the headings, the list and the line at the foot); and how much wider and taller than its icon a cell of the Icons view is. Roomy is Folders as it was before the levels; Normal is what everyone starts on. No row may be under 24 points, so it can always be pressed, and a row of two lines keeps room for both. --------------------------------------------------------------------------

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-folders-compact-row` | `26` | a number, 24 to 64 |
| `--trindle-folders-compact-rich-row` | `42` | a number, 40 to 80 |
| `--trindle-folders-compact-line` | `24` | a number, 24 to 48 |
| `--trindle-folders-compact-side-row` | `26` | a number, 24 to 48 |
| `--trindle-folders-compact-gap` | `4` | a number, 2 to 16 |
| `--trindle-folders-compact-icon-across` | `44` | a number, 40 to 80 |
| `--trindle-folders-compact-icon-down` | `44` | a number, 44 to 72 |
| `--trindle-folders-normal-row` | `34` | a number, 24 to 64 |
| `--trindle-folders-normal-rich-row` | `46` | a number, 40 to 80 |
| `--trindle-folders-normal-line` | `26` | a number, 24 to 48 |
| `--trindle-folders-normal-side-row` | `30` | a number, 24 to 48 |
| `--trindle-folders-normal-gap` | `6` | a number, 2 to 16 |
| `--trindle-folders-normal-icon-across` | `52` | a number, 40 to 80 |
| `--trindle-folders-normal-icon-down` | `48` | a number, 44 to 72 |
| `--trindle-folders-roomy-row` | `var(--trindle-rich-row-height)` | a number, 24 to 64 |
| `--trindle-folders-roomy-rich-row` | `var(--trindle-rich-row-height)` | a number, 40 to 80 |
| `--trindle-folders-roomy-line` | `28` | a number, 24 to 48 |
| `--trindle-folders-roomy-side-row` | `34` | a number, 24 to 48 |
| `--trindle-folders-roomy-gap` | `8` | a number, 2 to 16 |
| `--trindle-folders-roomy-icon-across` | `60` | a number, 40 to 80 |
| `--trindle-folders-roomy-icon-down` | `52` | a number, 44 to 72 |

-------------------------------------------------------------------------- The areas of a window (11 October 2026): the bar across its top, the bar down its side, and the line of words at the foot of its work. A sheet gives each a surface of its own, and the step of grey between one area and the next is the boundary: in this look (chosen 20 October 2026, "I don't really want the line there at all") the top bar is a step darker than the page, the side bar a step lighter, and the foot is the page's own shade inside its card, with no line anywhere. A sheet that wants lines (a high-contrast one) sets --trindle-area-line, this thick, which runs under the top bar, over the foot and down the side bar's far edge. Folders is the first program to draw its areas by these names. The light look has its own top bar below. --------------------------------------------------------------------------

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-top-bar` | `shade(var(--trindle-page), 0.55)` | a colour |
| `--trindle-side-bar` | `mix(var(--trindle-page), var(--trindle-card), 0.65)` | a colour |
| `--trindle-foot` | `var(--trindle-page)` | a colour |
| `--trindle-area-line` | `transparent` | a colour |
| `--trindle-area-line-width` | `1` | a number, 1 to 3 |

-------------------------------------------------------------------------- Folders' two panes (11 October 2026). In dual view each pane is a card of its own, and between the two is a narrow strip in the side bar's shade (--trindle-side-bar, the page where that is see-through) that holds the buttons acting on both panes; the strip is this wide, in points, and never narrower than a round button with a little room. The card of the pane in use has an edge of its own, this colour and this thick, inside the card's ordinary edge: a step lighter than the card in a dark look and a step darker in a light one, so it follows the words' colour, and thicker, so it is found by more than its shade. It is never the highlight, which is kept for what the pointer or the keyboard is on. Any card marked as the one in use of several alike is drawn with this edge. --------------------------------------------------------------------------

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-folders-strip` | `42` | a number, 36 to 72 |
| `--trindle-current-card-edge` | `alpha(var(--trindle-text), 0.38)` | a colour |
| `--trindle-current-card-edge-width` | `2` | a number, 1 to 3 |

The foot a change on trial is asked in (11 October 2026): a box along the bottom of Settings' page, as wide as the page, there while a theme or a display's arrangement waits to be kept and gone the moment it is answered or its time runs out. How tall it is and the room inside it, in points; its surface; and the line along its top. While a theme is on trial it is drawn from Trindle's own values, whatever the theme says, so it can always be found and read.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-trial-foot-height` | `72` | a number, 56 to 96 |
| `--trindle-trial-foot-padding` | `12` | a number, 4 to 12 |
| `--trindle-trial-foot` | `var(--trindle-box)` | a colour |
| `--trindle-trial-foot-line` | `var(--trindle-typing-box-edge)` | a colour |

A terminal's sixteen colours, as programs in it name them: the eight, then the same brighter. They come from the palette, and each is moved just far enough to be read on the cell it is drawn on (4.5 to one), so no theme can make a terminal unreadable.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-term-black` | `#000000` | a colour |
| `--trindle-term-red` | `var(--trindle-red)` | a colour |
| `--trindle-term-green` | `var(--trindle-green)` | a colour |
| `--trindle-term-yellow` | `var(--trindle-amber)` | a colour |
| `--trindle-term-blue` | `var(--trindle-blue)` | a colour |
| `--trindle-term-magenta` | `var(--trindle-purple)` | a colour |
| `--trindle-term-cyan` | `var(--trindle-cyan)` | a colour |
| `--trindle-term-white` | `var(--trindle-grey)` | a colour |
| `--trindle-term-bright-black` | `shade(var(--trindle-grey), 0.6)` | a colour |
| `--trindle-term-bright-red` | `shade(var(--trindle-red), 1.25)` | a colour |
| `--trindle-term-bright-green` | `var(--trindle-light-green)` | a colour |
| `--trindle-term-bright-yellow` | `var(--trindle-yellow)` | a colour |
| `--trindle-term-bright-blue` | `var(--trindle-light-blue)` | a colour |
| `--trindle-term-bright-magenta` | `var(--trindle-pink)` | a colour |
| `--trindle-term-bright-cyan` | `shade(var(--trindle-cyan), 1.25)` | a colour |
| `--trindle-term-bright-white` | `#ffffff` | a colour |

A notification: how heavy its title is, and how long it takes to slide out and back, in thousandths of a second.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-note-title-weight` | `700` | a number, 100 to 900 |
| `--trindle-note-slide` | `0` | a number, 0 to 600 |

A notification as a circle in a corner of the screen: the circle across at most (and at least, at rest), the daylight between two of them, the room from the corner, and the share of the ring centre's button size its buttons are (two thirds), and the most a circle grows to on pointing, to hold the whole message, as a share of the screen's height. In points. Its surface and edge are a card's; the label beside it is the label on pointing's.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-circle-size` | `160` | a number, 144 to 176 |

At rest a circle is as small as its icon, title and message allow, no smaller than this and no bigger than the size above.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-circle-rest-least` | `64` | a number, 48 to 144 |
| `--trindle-circle-gap` | `12` | a number, 4 to 24 |
| `--trindle-circle-margin` | `24` | a number, 16 to 48 |
| `--trindle-circle-button-share` | `0.67` | a number, 0.5 to 0.9 |
| `--trindle-circle-most` | `0.45` | a number, 0.25 to 0.6 |
| `--trindle-circle-pointed-most` | `0.9` | a number, 0.45 to 0.95 |

A pinned circle is moved by holding the left button on it this long (thousandths of a second); a key press with Alt moves it this many points.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-circle-hold` | `2000` | a number, 1000 to 4000 |
| `--trindle-circle-step` | `24` | a number, 8 to 96 |

The ring: how far the desktop behind it is dimmed while it is open.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-ring-dim` | `0.5` | a number, 0 to 0.8 |

How far every row of a page in the ring's middle (notifications, the clipboard, sound and the rest) keeps from the edge of its disc.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-ring-page-margin` | `10` | a number, 4 to 24 |

Words the ring shows outside its disc (nothing found, what was heard, what went wrong) are in the label on pointing's own pill, under the disc: this is the most its words are wide before they are cut short.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-ring-words-width` | `480` | a number, 240 to 800 |

Title bars, drawn by the compositor round every window that has one: the window in use, a shade lighter than the page, and the others, the page itself. They take plain colours or var(). The thin see-through line round a window that draws its own title bar keeps it apart from what is behind.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-title-background` | `#0e1728` | a colour |
| `--trindle-title-ink` | `var(--trindle-text)` | a colour |
| `--trindle-title-border` | `rgba(190, 215, 255, 0.09)` | a colour |
| `--trindle-title-away-background` | `var(--trindle-page)` | a colour |
| `--trindle-title-away-ink` | `var(--trindle-text-quiet)` | a colour |
| `--trindle-title-away-border` | `rgba(190, 215, 255, 0.06)` | a colour |
| `--trindle-window-edge` | `rgba(190, 215, 255, 0.10)` | a colour |

The light look, for the Light choice in Settings, Appearance: flat, in pale greys, as it was before the dark look turned blue. It names every colour and shadow the dark look sets, so nothing of the night shows through: no edges and no shadow on a card or in a typing box, only under what floats. Corners, room and sizes are the same in both looks; so are the label on pointing and a switch's knob while it is on, which are dark in either.

### The light look

These keys have a second value for the light look, written in a `:root` block inside `@media (prefers-color-scheme: light)`. Each takes the same kind of value as above.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-page` | `#ededed` | a colour |
| `--trindle-card` | `#ffffff` | a colour |
| `--trindle-box` | `#f2f2f2` | a colour |
| `--trindle-box-more` | `mix(var(--trindle-box), var(--trindle-text), 0.06)` | a colour |
| `--trindle-text` | `#111111` | a colour |
| `--trindle-text-quiet` | `#4f4f4f` | a colour |
| `--trindle-typing-box` | `#ffffff` | a colour |
| `--trindle-typing-box-edge` | `#7a7a7a` | a colour |
| `--trindle-typing-box-shadow` | `none` | a shadow |

Red words, deep enough to read on every pale ground: 4.9 to one even on a row pointed at on the page, where #c62828 stood 4.25.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-shade-red` | `#b71c1c` | a colour |
| `--trindle-control` | `var(--trindle-card)` | a colour |
| `--trindle-ring-grey` | `#d6d8dd` | a colour |
| `--trindle-ring-edge` | `#5f636c` | a colour |
| `--trindle-hover` | `var(--trindle-box)` | a colour |
| `--trindle-band` | `rgba(0, 0, 0, 0.04)` | a colour |
| `--trindle-edge` | `transparent` | a colour |
| `--trindle-box-edge` | `transparent` | a colour |
| `--trindle-side-line` | `#d6d6d6` | a colour |
| `--trindle-top-bar` | `mix(var(--trindle-page), var(--trindle-text), 0.07)` | a colour |
| `--trindle-section-line` | `#dedede` | a colour |
| `--trindle-card-shadow` | `none` | a shadow |
| `--trindle-well` | `#e0e0e0` | a colour |
| `--trindle-highlight-text` | `shade(var(--trindle-highlight), 0.5)` | a colour |
| `--trindle-highlight-soft` | `mix(var(--trindle-card), var(--trindle-highlight-text), 0.72)` | a colour |
| `--trindle-folder-icon` | `mix(var(--trindle-card), var(--trindle-highlight-text), 0.72)` | a colour |
| `--trindle-icon` | `var(--trindle-text)` | a colour |
| `--trindle-picked` | `mix(var(--trindle-highlight), #ffffff, 0.88)` | a colour |
| `--trindle-on-picked` | `mix(var(--trindle-highlight), #000000, 0.62)` | a colour |
| `--trindle-shadow` | `#10121e` | a colour |
| `--trindle-float-shadow` | `0 10px 28px 0 rgba(0, 0, 0, 0.18)` | a shadow |
| `--trindle-title-background` | `var(--trindle-card)` | a colour |
| `--trindle-title-border` | `var(--trindle-side-line)` | a colour |
| `--trindle-title-away-background` | `var(--trindle-page)` | a colour |
| `--trindle-title-away-ink` | `#52525b` | a colour |
| `--trindle-title-away-border` | `var(--trindle-side-line)` | a colour |

A switch's knob while it is off: dark on the pale track.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-knob-off` | `#6b7480` | a colour |
| `--trindle-spelling` | `#1565c0` | a colour |
| `--trindle-window-edge` | `rgba(0, 0, 0, 0.10)` | a colour |

The pane in use: the dark look's quieter edge would fall short on white.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-current-card-edge` | `alpha(var(--trindle-text), 0.5)` | a colour |
| `--trindle-ring-dim` | `0.22` | a number, 0 to 0.8 |

On the light control colour a share shows less, so each is larger.

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-ring-job-programs` | `0.16` | a number, 0 to 0.3 |
| `--trindle-ring-job-computer` | `0` | a number, 0 to 0.3 |
| `--trindle-ring-job-tools` | `0.08` | a number, 0 to 0.3 |
| `--trindle-ring-job-tray` | `0.26` | a number, 0 to 0.3 |

The blue, teal and orange of the small state dots, deep enough to stand three to one from the page, a card and a box in the light look (the dark look's stood about 2.7, 2.4 and 1.9).

| Key | Trindle's own | Takes |
| --- | --- | --- |
| `--trindle-calm` | `#1976d2` | a colour |
| `--trindle-good` | `#00796b` | a colour |
| `--trindle-busy` | `#b86200` | a colour |
| `--trindle-use-input` | `#7a4cb8` | a colour |
| `--trindle-use-files` | `#5b7f00` | a colour |
| `--trindle-use-network` | `#c2185b` | a colour |
| `--trindle-use-usb` | `#8a6d00` | a colour |
