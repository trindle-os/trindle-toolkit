# machine translated (German), not yet reviewed by a native speaker.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = Kopieren

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = Alles auswählen

# Right-click menu on a typing box that has words in it.
kit-cut = Ausschneiden

# Right-click menu on a typing box.
kit-paste = Einfügen

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
kit-show-password = Passwort anzeigen

# Name of the eye beside a PIN box while the PIN is hidden.
kit-show-pin = PIN anzeigen

# Name of the eye beside a key box while the key is hidden.
kit-show-key = Schlüssel anzeigen

# Name of the eye beside a password box while the password shows.
kit-hide-password = Passwort verbergen

# Name of the eye beside a PIN box while the PIN shows.
kit-hide-pin = PIN verbergen

# Name of the eye beside a key box while the key shows.
kit-hide-key = Schlüssel verbergen

# Said by the screen reader after the eye beside a password box is pressed to show it.
kit-password-shown = Passwort angezeigt

# Said by the screen reader after the eye beside a password box is pressed to hide it.
kit-password-hidden = Passwort verborgen

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
kit-pin-shown = PIN angezeigt

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
kit-pin-hidden = PIN verborgen

# Said by the screen reader after the eye beside a key box is pressed to show it.
kit-key-shown = Schlüssel angezeigt

# Said by the screen reader after the eye beside a key box is pressed to hide it.
kit-key-hidden = Schlüssel verborgen

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = Eines wählen

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = Suchen

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = Nichts passt

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, { $n } von { $total }

# Read out by the screen reader for a spinner that is turning.
kit-loading = Wird geladen

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
# SAFETY: needs a human review before it ships
kit-click-again = Noch einmal klicken: { $action }

# The same, when the button has no name to repeat.
# SAFETY: needs a human review before it ships
kit-click-again-bare = Noch einmal klicken

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
# SAFETY: needs a human review before it ships
kit-click-again-small = no

kit-job-wait = Warte, bis die Aufgabe fertig ist.

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + weitere

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
        [one] +{ $n } Zeile
       *[other] +{ $n } Zeilen
    }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = Mit den Elementen, die du abgelegt hast, konnte Trindle nichts anfangen.

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = Was du abgelegt hast, ist zu groß, um es hineinzuziehen.

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = Was du abgelegt hast, kam zu langsam an, deshalb hat Trindle nicht länger gewartet.

# Said when too many things were dropped at once.
kit-dropped-too-many = Du hast zu viele Elemente auf einmal abgelegt. Lege weniger auf einmal ab.

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = Die Elemente, die du abgelegt hast, sind keine Dateien auf diesem Computer.

# Label of a link button whose address is not a web address.
kit-link-not-web = Keine Webadresse

# Label of a link button whose address opens a mail program.
kit-link-mailto = E-Mail-Adresse

# Label of a link button whose address opens a file.
kit-link-file = Datei auf diesem Computer

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = Link zu { $label }

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = Das Auswahlfenster hat nicht reagiert. Versuche es noch einmal.

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = Zeile { $line }: { $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = Diese Designdatei ist zu groß. Die maximale Größe beträgt 64 KB.

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = Diese Datei ist kein Stylesheet. Sie enthält mehr als reinen Text.

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = ein Design kann keine andere Datei und keine Webadresse importieren.

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = Der Wert von --{ $name } ist zu lang.

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = Trindle kann --{ $name } nicht als Farbe lesen.

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = Trindle kann --{ $name } nicht als Zahl lesen.

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = Trindle kann --{ $name } nicht als Schatten lesen.

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = Trindle erkennt in dieser Datei keine Einstellungen, deshalb würde sich nichts ändern.

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = Fortschritt

# The name a screen reader gives a menu that opens over a window.
kit-menu = Menü


kit-search-of = { $n } von { $total }
kit-divider = Trennlinie
kit-row-sub =
    { $n ->
        [one] { $words }, Untermenü, 1 Zeile
       *[other] { $words }, Untermenü, { $n } Zeilen
    }
kit-row-keys = { $words }, { $keys }
kit-drop-target = Ablegebereich
kit-theme-not-step = --{ $name } darf nur einer dieser Werte sein: { $steps }. Trindle ignoriert jeden anderen Wert. Siehe { $guide }.
kit-sheet-not-watched = Starte neu, um Änderungen am Stylesheet in { $folder } zu sehen. { $why }
kit-menu-bar = Menüleiste
kit-cancel = Abbrechen
kit-toolbar = Werkzeuge
kit-more-tools = Weitere Werkzeuge
kit-status = Status
kit-path = Pfad
kit-path-more = Weitere Orte
kit-range-low = unteres Ende
kit-range-high = oberes Ende
kit-spin-less = { $what } verringern
kit-spin-more = { $what } erhöhen
kit-spin-not-number = Gib eine Zahl von { $min } bis { $max } ein
kit-tab-close = Schließen
kit-tab-more = Weitere Tabs
kit-area-cut = Ausschneiden
kit-area-copy = Kopieren
kit-area-paste = Einfügen
kit-area-select-all = Alles auswählen
kit-colour-square = Farbton, Sättigung und Helligkeit
kit-colour-swatch = Gewählte Farbe { $colour }
kit-colour-hue = Farbton
kit-colour-hex = Hex-Code
kit-look-title = Aussehen
kit-look-lead = Lege fest, wie dieses Programm aussieht, unabhängig vom Rest der Arbeitsoberfläche.
kit-look-group = Aussehen
kit-look-shade = Hell oder dunkel
kit-look-shade-help = Der Arbeitsoberfläche folgen oder immer hell oder dunkel verwenden.
kit-look-desktop = Wie die Arbeitsoberfläche
kit-look-dark = Dunkel
kit-look-light = Hell
kit-look-highlight = Hervorhebungsfarbe
kit-look-highlight-help = Die Farbe von Hauptschaltflächen und ausgewählten Elementen.
kit-look-highlight-desktop = Wie die Arbeitsoberfläche
kit-date-previous = Zurück
kit-date-next = Weiter
kit-date-month-year = { $month } { $year }
kit-date-full = { $weekday } { $day }. { $month } { $year }
kit-time-hours = Stunden
kit-time-minutes = Minuten
kit-month-1 = Januar
kit-month-2 = Februar
kit-month-3 = März
kit-month-4 = April
kit-month-5 = Mai
kit-month-6 = Juni
kit-month-7 = Juli
kit-month-8 = August
kit-month-9 = September
kit-month-10 = Oktober
kit-month-11 = November
kit-month-12 = Dezember
# A month as it reads inside a whole date, such as "12 November 2026". Some languages change the ending there, so it has its own key.
kit-month-in-date-1 = Januar
kit-month-in-date-2 = Februar
kit-month-in-date-3 = März
kit-month-in-date-4 = April
kit-month-in-date-5 = Mai
kit-month-in-date-6 = Juni
kit-month-in-date-7 = Juli
kit-month-in-date-8 = August
kit-month-in-date-9 = September
kit-month-in-date-10 = Oktober
kit-month-in-date-11 = November
kit-month-in-date-12 = Dezember
kit-weekday-1 = Mo
kit-weekday-2 = Di
kit-weekday-3 = Mi
kit-weekday-4 = Do
kit-weekday-5 = Fr
kit-weekday-6 = Sa
kit-weekday-7 = So
kit-weekday-full-1 = Montag
kit-weekday-full-2 = Dienstag
kit-weekday-full-3 = Mittwoch
kit-weekday-full-4 = Donnerstag
kit-weekday-full-5 = Freitag
kit-weekday-full-6 = Samstag
kit-weekday-full-7 = Sonntag
kit-key-ctrl = Strg
kit-key-shift = Umschalt
kit-key-alt = Alt
kit-key-logo = Super
kit-key-left = Links
kit-key-right = Rechts
kit-key-up = Nach oben
kit-key-down = Nach unten
kit-key-page-up = Bild hoch
kit-key-page-down = Bild runter
kit-key-home = Pos1
kit-key-end = Ende
kit-key-enter = Eingabe
kit-key-escape = Esc
kit-key-tab = Tab
kit-key-space = Leertaste
kit-key-backspace = Rücktaste
kit-key-delete = Entf
kit-key-insert = Einfg
kit-shortcuts = Tastenkürzel
kit-shortcut-kept-window = Diese Taste ist zum Wechseln zwischen Bedienelementen und zum Schließen des Fensters reserviert.
kit-shortcut-kept-typing = Beim Tippen bekommt zuerst das Textfeld diese Taste.
kit-emoji = Emoji
kit-emoji-search = Emoji nach Namen suchen
kit-emoji-none = Kein Emoji hat diesen Namen.
kit-emoji-faces = Gesichter
kit-emoji-hands = Hände
kit-emoji-animals = Tiere
kit-emoji-nature = Natur
kit-emoji-food = Essen und Trinken
kit-emoji-travel = Reisen und Orte
kit-emoji-objects = Gegenstände
kit-emoji-symbols = Symbole
kit-emoji-grin = Grinsen
kit-emoji-tears-of-joy = Freudentränen
kit-emoji-smile = Lächeln
kit-emoji-wink = Zwinkern
kit-emoji-heart-eyes = Herzaugen
kit-emoji-thinking = Nachdenken
kit-emoji-crying = Weinen
kit-emoji-surprised = Überrascht
kit-emoji-blush = Erröten
kit-emoji-halo = Heiligenschein
kit-emoji-cool = Cool
kit-emoji-sleeping = Schlafend
kit-emoji-sobbing = Schluchzen
kit-emoji-angry = Wütend
kit-emoji-scream = Schreien
kit-emoji-eye-roll = Augenrollen
kit-emoji-relieved = Erleichtert
kit-emoji-rolling-laugh = Lachend auf dem Boden
kit-emoji-blowing-a-kiss = Kusshand
kit-emoji-straight-face = Ausdrucksloses Gesicht
kit-emoji-flushed = Verlegen
kit-emoji-hug = Umarmung
kit-emoji-unwell = Unwohl
kit-emoji-celebrating = Feiern
kit-emoji-thumbs-up = Daumen hoch
kit-emoji-thumbs-down = Daumen runter
kit-emoji-clapping = Klatschen
kit-emoji-thank-you = Danke
kit-emoji-waving = Winken
kit-emoji-victory = Victory-Zeichen
kit-emoji-ok = OK
kit-emoji-crossed-fingers = Gedrückte Daumen
kit-emoji-strong = Stark
kit-emoji-pointing-up = Zeigt nach oben
kit-emoji-pointing-down = Zeigt nach unten
kit-emoji-pointing-left = Zeigt nach links
kit-emoji-pointing-right = Zeigt nach rechts
kit-emoji-raised-hand = Erhobene Hand
kit-emoji-handshake = Händedruck
kit-emoji-fist = Faust
kit-emoji-raised-hands = Erhobene Hände
kit-emoji-call-me = Ruf mich an
kit-emoji-writing = Schreiben
kit-emoji-shrug = Schulterzucken
kit-emoji-dog = Hund
kit-emoji-cat = Katze
kit-emoji-mouse = Maus
kit-emoji-rabbit = Hase
kit-emoji-fox = Fuchs
kit-emoji-bear = Bär
kit-emoji-panda = Panda
kit-emoji-tiger = Tiger
kit-emoji-lion = Löwe
kit-emoji-cow = Kuh
kit-emoji-pig = Schwein
kit-emoji-frog = Frosch
kit-emoji-monkey = Affe
kit-emoji-chicken = Huhn
kit-emoji-penguin = Pinguin
kit-emoji-bird = Vogel
kit-emoji-duck = Ente
kit-emoji-owl = Eule
kit-emoji-horse = Pferd
kit-emoji-bee = Biene
kit-emoji-butterfly = Schmetterling
kit-emoji-turtle = Schildkröte
kit-emoji-fish = Fisch
kit-emoji-dolphin = Delfin
kit-emoji-blossom = Blüte
kit-emoji-tree = Baum
kit-emoji-sun = Sonne
kit-emoji-rain = Regen
kit-emoji-rose = Rose
kit-emoji-sunflower = Sonnenblume
kit-emoji-tulip = Tulpe
kit-emoji-evergreen = Nadelbaum
kit-emoji-clover = Kleeblatt
kit-emoji-maple-leaf = Ahornblatt
kit-emoji-cactus = Kaktus
kit-emoji-moon = Mond
kit-emoji-rainbow = Regenbogen
kit-emoji-cloud = Wolke
kit-emoji-snowflake = Schneeflocke
kit-emoji-lightning = Blitz
kit-emoji-wave = Welle
kit-emoji-earth = Erde
kit-emoji-mountain = Berg
kit-emoji-palm-tree = Palme
kit-emoji-apple = Apfel
kit-emoji-bread = Brot
kit-emoji-cake = Kuchen
kit-emoji-tea = Tee
kit-emoji-banana = Banane
kit-emoji-orange = Orange
kit-emoji-strawberry = Erdbeere
kit-emoji-grapes = Trauben
kit-emoji-watermelon = Wassermelone
kit-emoji-lemon = Zitrone
kit-emoji-carrot = Karotte
kit-emoji-corn = Mais
kit-emoji-pizza = Pizza
kit-emoji-burger = Burger
kit-emoji-chips = Pommes
kit-emoji-pasta = Nudeln
kit-emoji-fried-egg = Spiegelei
kit-emoji-cheese = Käse
kit-emoji-biscuit = Keks
kit-emoji-chocolate = Schokolade
kit-emoji-ice-cream = Eis
kit-emoji-beer = Bier
kit-emoji-wine = Wein
kit-emoji-milk = Milch
kit-emoji-car = Auto
kit-emoji-bus = Bus
kit-emoji-bicycle = Fahrrad
kit-emoji-train = Zug
kit-emoji-aeroplane = Flugzeug
kit-emoji-rocket = Rakete
kit-emoji-ship = Schiff
kit-emoji-sailing-boat = Segelboot
kit-emoji-helicopter = Hubschrauber
kit-emoji-house = Haus
kit-emoji-office = Bürogebäude
kit-emoji-hospital = Krankenhaus
kit-emoji-school = Schule
kit-emoji-church = Kirche
kit-emoji-beach = Strand
kit-emoji-tower = Turm
kit-emoji-bridge = Brücke
kit-emoji-petrol-pump = Zapfsäule
kit-emoji-traffic-light = Ampel
kit-emoji-map = Karte
kit-emoji-phone = Telefon
kit-emoji-laptop = Laptop
kit-emoji-keyboard = Tastatur
kit-emoji-camera = Kamera
kit-emoji-light-bulb = Glühbirne
kit-emoji-key = Schlüssel
kit-emoji-lock = Schloss
kit-emoji-books = Bücher
kit-emoji-memo = Notizzettel
kit-emoji-pencil = Bleistift
kit-emoji-paperclip = Büroklammer
kit-emoji-calendar = Kalender
kit-emoji-email = E-Mail
kit-emoji-gift = Geschenk
kit-emoji-music-note = Musiknote
kit-emoji-headphones = Kopfhörer
kit-emoji-alarm-clock = Wecker
kit-emoji-bell = Glocke
kit-emoji-spanner = Schraubenschlüssel
kit-emoji-money-bag = Geldsack
kit-emoji-shopping-trolley = Einkaufswagen
kit-emoji-football = Fußball
kit-emoji-heart = Herz
kit-emoji-star = Stern
kit-emoji-done = Erledigt
kit-emoji-question = Frage
kit-emoji-party = Party
kit-emoji-fire = Feuer
kit-emoji-cross = Kreuz
kit-emoji-exclamation = Ausrufezeichen
kit-emoji-warning = Warnung
kit-emoji-hundred = Hundert
kit-emoji-broken-heart = Gebrochenes Herz
kit-emoji-blue-heart = Blaues Herz
kit-emoji-green-heart = Grünes Herz
kit-emoji-sparkles = Funkeln
kit-emoji-plus = Plus
kit-emoji-minus = Minus
kit-emoji-right-arrow = Pfeil nach rechts
kit-emoji-left-arrow = Pfeil nach links
kit-emoji-refresh = Aktualisieren
kit-emoji-forbidden = Verboten
kit-emoji-recycling = Recycling
kit-emoji-sleep = Schlaf
kit-only-your-own = Nur du kannst diese Schaltfläche drücken.
kit-no-layout = Trindle konnte die Tastaturbelegung nicht lesen, deshalb kannst du nichts eingeben.
kit-scene =
    { $count ->
        [one] Eine Szene mit 1 Element
       *[other] Eine Szene mit { $count } Elementen
    }
kit-scene-chosen =
    { $count ->
        [one] Eine Szene mit 1 Element, { $name } gewählt
       *[other] Eine Szene mit { $count } Elementen, { $name } gewählt
    }
kit-player-play = Wiedergeben
kit-player-pause = Pause
kit-player-back = 5 Sekunden zurück
kit-player-forward = 5 Sekunden vor
kit-player-seek = Position
kit-player-mute = Stummschalten
kit-player-unmute = Ton einschalten
kit-player-volume = Lautstärke
kit-player-time = { $at } von { $length }
kit-crash-title = Etwas ist schiefgelaufen
kit-crash-lead = { $program } wurde beendet, weil etwas schiefgelaufen ist. Kopiere den Bericht unten und sende ihn mit deiner Fehlermeldung, damit das Problem behoben werden kann.
kit-crash-copy = Bericht kopieren
kit-help-code = Code
kit-complete-count =
    { $n ->
        [one] 1 Vorschlag
       *[other] { $n } Vorschläge
    }
kit-frame-minimise = Minimieren
kit-frame-maximise = Maximieren
kit-frame-restore = Wiederherstellen
kit-frame-close = Schließen
kit-frame-move = Verschieben
kit-frame-resize = Größe ändern
kit-numerals-title = Zahlendarstellung
kit-numerals-help = Wie Zahlen, Daten und Uhrzeiten geschrieben werden.
kit-numerals-auto = Üblich für deine Region
kit-numerals-latin = Immer 0 bis 9
kit-numerals-native = Die eigenen Ziffern der Sprache
kit-date-week-short = Wo
kit-date-week = Woche { $week }
