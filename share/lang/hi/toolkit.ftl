
# machine translated (Hindi), not yet reviewed by a person.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = कॉपी करें

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = सभी चुनें

# Right-click menu on a typing box that has words in it.
kit-cut = काटें

# Right-click menu on a typing box.
kit-paste = चिपकाएँ

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
kit-show-password = पासवर्ड दिखाएँ

# Name of the eye beside a PIN box while the PIN is hidden.
kit-show-pin = PIN दिखाएँ

# Name of the eye beside a key box while the key is hidden.
kit-show-key = कुंजी दिखाएँ

# Name of the eye beside a password box while the password shows.
kit-hide-password = पासवर्ड छिपाएँ

# Name of the eye beside a PIN box while the PIN shows.
kit-hide-pin = PIN छिपाएँ

# Name of the eye beside a key box while the key shows.
kit-hide-key = कुंजी छिपाएँ

# Said by the screen reader after the eye beside a password box is pressed to show it.
kit-password-shown = पासवर्ड दिख रहा है

# Said by the screen reader after the eye beside a password box is pressed to hide it.
kit-password-hidden = पासवर्ड छिपा है

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
kit-pin-shown = PIN दिख रहा है

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
kit-pin-hidden = PIN छिपा है

# Said by the screen reader after the eye beside a key box is pressed to show it.
kit-key-shown = कुंजी दिख रही है

# Said by the screen reader after the eye beside a key box is pressed to hide it.
kit-key-hidden = कुंजी छिपी है

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = एक चुनें

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = खोजें

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = कुछ मेल नहीं खाता

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, { $total } में से { $n }

# Read out by the screen reader for a spinner that is turning.
kit-loading = लोड हो रहा है

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
# SAFETY: needs a human review before it ships
kit-click-again = फिर क्लिक करें: { $action }

# The same, when the button has no name to repeat.
# SAFETY: needs a human review before it ships
kit-click-again-bare = फिर क्लिक करें

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + और

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
        [one] +{ $n } पंक्ति
       *[other] +{ $n } पंक्तियाँ
    }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = Trindle आपके छोड़े गए आइटम के साथ कुछ नहीं कर सका।

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = आपने जो छोड़ा वह खींचकर लाने के लिए बहुत बड़ा है।

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = आपने जो छोड़ा उसे आने में बहुत देर लगी, इसलिए Trindle ने इंतज़ार बंद कर दिया।

# Said when too many things were dropped at once.
kit-dropped-too-many = आपने एक साथ बहुत सारे आइटम छोड़ दिए। एक बार में कम आइटम छोड़ें।

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = आपके छोड़े गए आइटम इस कंप्यूटर की फ़ाइलें नहीं हैं।

# Label of a link button whose address is not a web address.
kit-link-not-web = वेब पता नहीं है

# Label of a link button whose address opens a mail program.
kit-link-mailto = ईमेल पता

# Label of a link button whose address opens a file.
kit-link-file = इस कंप्यूटर की फ़ाइल

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = { $label } का लिंक

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = चुनने वाली विंडो ने जवाब नहीं दिया। फिर कोशिश करें।

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = पंक्ति { $line }: { $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = यह थीम फ़ाइल बहुत बड़ी है। अधिकतम आकार 64 KB है।

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = यह फ़ाइल स्टाइल शीट नहीं है। इसमें सादे टेक्स्ट से ज़्यादा कुछ है।

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = थीम किसी दूसरी फ़ाइल या वेब पते को इम्पोर्ट नहीं कर सकती।

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = --{ $name } का मान बहुत लंबा है।

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = Trindle --{ $name } को रंग के रूप में नहीं पढ़ सकता।

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = Trindle --{ $name } को संख्या के रूप में नहीं पढ़ सकता।

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = Trindle --{ $name } को छाया के रूप में नहीं पढ़ सकता।

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = Trindle इस फ़ाइल की कोई सेटिंग नहीं पहचानता, इसलिए कुछ नहीं बदलेगा।

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = प्रगति

# The name a screen reader gives a menu that opens over a window.
kit-menu = मेनू

kit-job-wait = कार्य पूरा होने तक प्रतीक्षा करें।

# Shown at the end of a dropdown's search box: how many rows match out of how many there are.
kit-search-of = { $total } में से { $n }

# A menu row that opens a sub-menu. $words is the row's own words, $n how many lines the sub-menu holds.
kit-row-sub =
    { $n ->
        [one] { $words }, उप-मेनू, { $n } पंक्ति
       *[other] { $words }, उप-मेनू, { $n } पंक्तियाँ
    }

# A menu row with a shortcut. $words is the row's own words, $keys the shortcut.
kit-row-keys = { $words }, { $keys }

# Said by a screen reader for the close box in a tab.
kit-tab-close = बंद करें

# Said by a screen reader for the button that lists the tabs that did not fit.
kit-tab-more = और टैब

# Right-click menu of a text area, one row each.
kit-area-cut = काटें

kit-area-copy = कॉपी करें

kit-area-paste = चिपकाएँ

kit-area-select-all = सभी चुनें

# The name a screen reader gives the painted square of a colour picker.
kit-colour-square = रंग, संतृप्ति और चमक

# The name a screen reader gives the swatch showing the colour chosen. $colour is its hex code.
kit-colour-swatch = चुना गया रंग { $colour }

kit-colour-hue = रंगत

kit-colour-hex = हेक्स कोड

# Buttons either side of the month in a date picker.
kit-date-previous = पिछला

kit-date-next = अगला

# The month shown in a date picker. $month is the month's name, $year the year.
kit-date-month-year = { $month } { $year }

# What a screen reader says for a day in a date picker.
kit-date-full = { $weekday } { $day } { $month } { $year }

# The names of the two boxes in a time entry.
kit-time-hours = घंटे

kit-time-minutes = मिनट

# Months and weekdays; weekdays are short in the grid and full when read out.
kit-month-1 = जनवरी

kit-month-2 = फ़रवरी

kit-month-3 = मार्च

kit-month-4 = अप्रैल

kit-month-5 = मई

kit-month-6 = जून

kit-month-7 = जुलाई

kit-month-8 = अगस्त

kit-month-9 = सितंबर

kit-month-10 = अक्टूबर

kit-month-11 = नवंबर

kit-month-12 = दिसंबर
# A month as it reads inside a whole date, such as "12 November 2026". Some languages change the ending there, so it has its own key.
kit-month-in-date-1 = जनवरी
kit-month-in-date-2 = फ़रवरी
kit-month-in-date-3 = मार्च
kit-month-in-date-4 = अप्रैल
kit-month-in-date-5 = मई
kit-month-in-date-6 = जून
kit-month-in-date-7 = जुलाई
kit-month-in-date-8 = अगस्त
kit-month-in-date-9 = सितंबर
kit-month-in-date-10 = अक्टूबर
kit-month-in-date-11 = नवंबर
kit-month-in-date-12 = दिसंबर

kit-weekday-1 = सोम

kit-weekday-2 = मंगल

kit-weekday-3 = बुध

kit-weekday-4 = गुरु

kit-weekday-5 = शुक्र

kit-weekday-6 = शनि

kit-weekday-7 = रवि

kit-weekday-full-1 = सोमवार

kit-weekday-full-2 = मंगलवार

kit-weekday-full-3 = बुधवार

kit-weekday-full-4 = गुरुवार

kit-weekday-full-5 = शुक्रवार

kit-weekday-full-6 = शनिवार

kit-weekday-full-7 = रविवार

# The names of the modifier keys in a shortcut such as Ctrl+Shift+C.
kit-key-ctrl = Ctrl

kit-key-shift = Shift

kit-key-alt = Alt

kit-key-logo = Super

# Said in a window of Trindle's own when something tried to press a button that only the person may press.
kit-only-your-own = इस बटन को केवल आप दबा सकते हैं।

# Said in a window when keys are pressed but the keyboard layout could not be read.
kit-no-layout = Trindle कीबोर्ड लेआउट नहीं पढ़ सका, इसलिए आप टाइप नहीं कर सकते।



# Read out by the screen reader for the line between two areas of a window that can be dragged to make one bigger.
kit-divider = विभाजक

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
kit-click-again-small = yes

# Read out by the screen reader for the control under something dragged, when letting go there would drop it in.
kit-drop-target = छोड़ने का क्षेत्र

# Fault in a theme or style sheet: a setting that takes only named steps was given something else. $steps is the list of steps, $guide the file that explains them.
kit-theme-not-step = --{ $name } केवल इनमें से एक हो सकता है: { $steps }। Trindle बाकी सभी मान अनदेखे कर देता है। देखें { $guide }।

# Standard error, when the folder of the person's style sheet cannot be watched. $folder is the folder, $why the system's reason.
kit-sheet-not-watched = { $folder } की स्टाइल शीट में बदलाव देखने के लिए रीस्टार्ट करें। { $why }

# What a screen reader calls the row of menu titles across the top of a window.
kit-menu-bar = मेनू पट्टी

# The answer at the foot of every dialog page that leaves it with nothing done.
kit-cancel = रद्द करें

# What a screen reader calls a row of tool buttons.
kit-toolbar = औज़ार

# The button at the end of a toolbar that opens a menu of the tools that did not fit.
kit-more-tools = और औज़ार

# What a screen reader calls the line at a window's foot that says how things stand.
kit-status = स्थिति

# What a screen reader calls a place's path, such as a folder's.
kit-path = पथ

# The list in the middle of a long path, holding the places not shown.
kit-path-more = और जगहें

# Said after a range slider's name for its low handle.
kit-range-low = निचला सिरा

# Said after a range slider's name for its high handle.
kit-range-high = ऊपरी सिरा

# The button that takes a step off a number box's number. $what is the box's name.
kit-spin-less = { $what } कम करें

# The button that adds a step to a number box's number. $what is the box's name.
kit-spin-more = { $what } बढ़ाएँ

# Red words under a number box whose typing is not a number.
kit-spin-not-number = { $min } से { $max } तक की संख्या लिखें

# Said by a screen reader for a tab that carries a count, such as unread messages: its title, then the count.
kit-tab-count = { $title }, { $count }

# The ready settings page for a program's own look (settings::look_page).
kit-look-title = दिखावट

kit-look-lead = चुनें कि यह प्रोग्राम कैसा दिखे, बाकी डेस्कटॉप से अलग।

kit-look-group = दिखावट

kit-look-shade = हल्का या गहरा

kit-look-shade-help = डेस्कटॉप के अनुसार रखें, या हमेशा हल्का या गहरा रखें।

kit-look-desktop = डेस्कटॉप जैसा

kit-look-dark = गहरा

kit-look-light = हल्का

kit-look-highlight = हाइलाइट का रंग

kit-look-highlight-help = मुख्य बटनों और चुनी हुई चीज़ों का रंग।

# A button that gives the highlight back to the desktop's choice.
kit-look-highlight-desktop = डेस्कटॉप जैसा

# Over the column of week numbers in a date picker; keep it as short as the short weekday names.
kit-date-week-short = सप्ताह

# What a screen reader says for a week number in a date picker. $week is the number.
kit-date-week = सप्ताह { $week }

# The names of keys that are words, in a shortcut such as Ctrl+Page down.
kit-key-left = Left

kit-key-right = Right

kit-key-up = Up

kit-key-down = Down

kit-key-page-up = पेज ऊपर

kit-key-page-down = पेज नीचे

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
kit-shortcuts = कीबोर्ड शॉर्टकट

# Under a shortcut on that page whose keys the window always takes first.
kit-shortcut-kept-window = यह कुंजी नियंत्रणों के बीच जाने और विंडो बंद करने के लिए आरक्षित है।

# Under a shortcut on that page whose keys a text box takes first while you type.
kit-shortcut-kept-typing = जब आप टाइप कर रहे हों, तो यह कुंजी पहले टेक्स्ट बॉक्स को जाती है।

# The heading of the page of emoji to pick from.
kit-emoji = इमोजी

# The name of the box on that page where an emoji's name is typed to find it.
kit-emoji-search = नाम से इमोजी खोजें

# Shown on that page when no emoji's name has what was typed.
kit-emoji-none = इस नाम का कोई इमोजी नहीं है।

# The groups of emoji on that page, in order. Another language may leave out any
# name below; the page then shows the English one, so a name is never missing.
kit-emoji-faces = चेहरे

kit-emoji-hands = हाथ

kit-emoji-animals = जानवर

kit-emoji-nature = प्रकृति

kit-emoji-food = खाना-पीना

kit-emoji-travel = यात्रा और जगहें

kit-emoji-objects = वस्तुएँ

kit-emoji-symbols = चिह्न

# Each emoji's name, spoken by a screen reader and found by typing.
kit-emoji-grin = दाँत दिखाकर हँसी

kit-emoji-tears-of-joy = खुशी के आँसू

kit-emoji-smile = मुस्कान

kit-emoji-wink = आँख मारना

kit-emoji-heart-eyes = दिल वाली आँखें

kit-emoji-thinking = सोचना

kit-emoji-crying = रोना

kit-emoji-surprised = हैरान

kit-emoji-blush = शर्माना

kit-emoji-halo = प्रभामंडल

kit-emoji-cool = मस्त

kit-emoji-sleeping = सोया हुआ

kit-emoji-sobbing = फूट-फूटकर रोना

kit-emoji-angry = गुस्सा

kit-emoji-scream = चीख

kit-emoji-eye-roll = आँखें घुमाना

kit-emoji-relieved = राहत

kit-emoji-rolling-laugh = हँसी से लोटपोट

kit-emoji-blowing-a-kiss = फ्लाइंग किस

kit-emoji-straight-face = सपाट चेहरा

kit-emoji-flushed = लाल चेहरा

kit-emoji-hug = गले लगाना

kit-emoji-unwell = तबीयत खराब

kit-emoji-celebrating = जश्न

kit-emoji-thumbs-up = अंगूठा ऊपर

kit-emoji-thumbs-down = अंगूठा नीचे

kit-emoji-clapping = तालियाँ

kit-emoji-thank-you = धन्यवाद

kit-emoji-waving = हाथ हिलाना

kit-emoji-victory = जीत

kit-emoji-ok = ठीक है

kit-emoji-crossed-fingers = उँगलियाँ क्रॉस

kit-emoji-strong = ताकतवर

kit-emoji-pointing-up = ऊपर इशारा

kit-emoji-pointing-down = नीचे इशारा

kit-emoji-pointing-left = बाएँ इशारा

kit-emoji-pointing-right = दाएँ इशारा

kit-emoji-raised-hand = उठा हुआ हाथ

kit-emoji-handshake = हाथ मिलाना

kit-emoji-fist = मुट्ठी

kit-emoji-raised-hands = उठे हुए हाथ

kit-emoji-call-me = मुझे फ़ोन करो

kit-emoji-writing = लिखना

kit-emoji-shrug = कंधे उचकाना

kit-emoji-dog = कुत्ता

kit-emoji-cat = बिल्ली

kit-emoji-mouse = चूहा

kit-emoji-rabbit = खरगोश

kit-emoji-fox = लोमड़ी

kit-emoji-bear = भालू

kit-emoji-panda = पांडा

kit-emoji-tiger = बाघ

kit-emoji-lion = शेर

kit-emoji-cow = गाय

kit-emoji-pig = सूअर

kit-emoji-frog = मेंढक

kit-emoji-monkey = बंदर

kit-emoji-chicken = मुर्गी

kit-emoji-penguin = पेंगुइन

kit-emoji-bird = चिड़िया

kit-emoji-duck = बत्तख

kit-emoji-owl = उल्लू

kit-emoji-horse = घोड़ा

kit-emoji-bee = मधुमक्खी

kit-emoji-butterfly = तितली

kit-emoji-turtle = कछुआ

kit-emoji-fish = मछली

kit-emoji-dolphin = डॉल्फ़िन

kit-emoji-blossom = फूल

kit-emoji-tree = पेड़

kit-emoji-sun = सूरज

kit-emoji-rain = बारिश

kit-emoji-rose = गुलाब

kit-emoji-sunflower = सूरजमुखी

kit-emoji-tulip = ट्यूलिप

kit-emoji-evergreen = सदाबहार पेड़

kit-emoji-clover = तिपतिया

kit-emoji-maple-leaf = मेपल की पत्ती

kit-emoji-cactus = कैक्टस

kit-emoji-moon = चाँद

kit-emoji-rainbow = इंद्रधनुष

kit-emoji-cloud = बादल

kit-emoji-snowflake = बर्फ़ का फाहा

kit-emoji-lightning = बिजली

kit-emoji-wave = लहर

kit-emoji-earth = पृथ्वी

kit-emoji-mountain = पहाड़

kit-emoji-palm-tree = ताड़ का पेड़

kit-emoji-apple = सेब

kit-emoji-bread = ब्रेड

kit-emoji-cake = केक

kit-emoji-tea = चाय

kit-emoji-banana = केला

kit-emoji-orange = संतरा

kit-emoji-strawberry = स्ट्रॉबेरी

kit-emoji-grapes = अंगूर

kit-emoji-watermelon = तरबूज़

kit-emoji-lemon = नींबू

kit-emoji-carrot = गाजर

kit-emoji-corn = मक्का

kit-emoji-pizza = पिज़्ज़ा

kit-emoji-burger = बर्गर

kit-emoji-chips = चिप्स

kit-emoji-pasta = पास्ता

kit-emoji-fried-egg = तला अंडा

kit-emoji-cheese = पनीर

kit-emoji-biscuit = बिस्कुट

kit-emoji-chocolate = चॉकलेट

kit-emoji-ice-cream = आइसक्रीम

kit-emoji-beer = बियर

kit-emoji-wine = वाइन

kit-emoji-milk = दूध

kit-emoji-car = कार

kit-emoji-bus = बस

kit-emoji-bicycle = साइकिल

kit-emoji-train = रेलगाड़ी

kit-emoji-aeroplane = हवाई जहाज़

kit-emoji-rocket = रॉकेट

kit-emoji-ship = जहाज़

kit-emoji-sailing-boat = पाल वाली नाव

kit-emoji-helicopter = हेलीकॉप्टर

kit-emoji-house = घर

kit-emoji-office = दफ़्तर

kit-emoji-hospital = अस्पताल

kit-emoji-school = स्कूल

kit-emoji-church = चर्च

kit-emoji-beach = समुद्र तट

kit-emoji-tower = मीनार

kit-emoji-bridge = पुल

kit-emoji-petrol-pump = पेट्रोल पंप

kit-emoji-traffic-light = ट्रैफ़िक लाइट

kit-emoji-map = नक्शा

kit-emoji-phone = फ़ोन

kit-emoji-laptop = लैपटॉप

kit-emoji-keyboard = कीबोर्ड

kit-emoji-camera = कैमरा

kit-emoji-light-bulb = बल्ब

kit-emoji-key = चाबी

kit-emoji-lock = ताला

kit-emoji-books = किताबें

kit-emoji-memo = ज्ञापन

kit-emoji-pencil = पेंसिल

kit-emoji-paperclip = पेपर क्लिप

kit-emoji-calendar = कैलेंडर

kit-emoji-email = ईमेल

kit-emoji-gift = उपहार

kit-emoji-music-note = संगीत का स्वर

kit-emoji-headphones = हेडफ़ोन

kit-emoji-alarm-clock = अलार्म घड़ी

kit-emoji-bell = घंटी

kit-emoji-spanner = रिंच

kit-emoji-money-bag = पैसों की थैली

kit-emoji-shopping-trolley = खरीदारी की ट्रॉली

kit-emoji-football = फ़ुटबॉल

kit-emoji-heart = दिल

kit-emoji-star = तारा

kit-emoji-done = पूरा हुआ

kit-emoji-question = सवाल

kit-emoji-party = पार्टी

kit-emoji-fire = आग

kit-emoji-cross = क्रॉस

kit-emoji-exclamation = विस्मयादिबोधक

kit-emoji-warning = चेतावनी

kit-emoji-hundred = सौ

kit-emoji-broken-heart = टूटा दिल

kit-emoji-blue-heart = नीला दिल

kit-emoji-green-heart = हरा दिल

kit-emoji-sparkles = चमक

kit-emoji-plus = जमा

kit-emoji-minus = घटा

kit-emoji-right-arrow = दायाँ तीर

kit-emoji-left-arrow = बायाँ तीर

kit-emoji-refresh = ताज़ा करें

kit-emoji-forbidden = मनाही

kit-emoji-recycling = रीसाइक्लिंग

kit-emoji-sleep = नींद

# What a screen reader is told of a scene of items: how many, and the one chosen.
kit-scene =
    { $count ->
        [one] 1 चीज़ का दृश्य
       *[other] { $count } चीज़ों का दृश्य
    }

kit-scene-chosen =
    { $count ->
        [one] 1 चीज़ का दृश्य, { $name } चुनी गई
       *[other] { $count } चीज़ों का दृश्य, { $name } चुनी गई
    }

## A video or sound player's controls
kit-player-play = चलाएँ

kit-player-pause = रोकें

kit-player-back = 5 सेकंड पीछे

kit-player-forward = 5 सेकंड आगे

kit-player-seek = स्थिति

kit-player-mute = म्यूट करें

kit-player-unmute = म्यूट हटाएँ

kit-player-volume = आवाज़

# How far in, out of how long, as clock times.
kit-player-time = { $length } में से { $at }

## The page shown after a program crashed or met a fault
kit-crash-title = कुछ गड़बड़ हो गई

kit-crash-lead = कुछ गड़बड़ होने के कारण { $program } रुक गया। नीचे की रिपोर्ट कॉपी करें और अपनी बग रिपोर्ट के साथ भेजें, ताकि समस्या ठीक की जा सके।

kit-crash-copy = रिपोर्ट कॉपी करें

# What a screen reader calls a block of code on a help page.
kit-help-code = कोड

# What a screen reader calls the list of suggestions under a text box; $name is what the box is called.
kit-complete-list = { $name } के लिए सुझाव

# Under a text box with suggestions: how many there are, said by a screen reader as they change.
kit-complete-count =
    { $n ->
        [one] 1 सुझाव
       *[other] { $n } सुझाव
    }

## A title bar the program draws, where the compositor draws none
kit-frame-minimise = छोटा करें

kit-frame-maximise = बड़ा करें

kit-frame-restore = पहले जैसा करें

kit-frame-close = बंद करें

kit-frame-move = खिसकाएँ

kit-frame-resize = आकार बदलें

# Settings: which figures numbers are written in.
kit-numerals-title = अंक शैली

kit-numerals-help = संख्याएँ, तारीखें और समय कैसे लिखे जाते हैं।

kit-numerals-auto = आपके क्षेत्र में प्रचलित

kit-numerals-latin = हमेशा 0 से 9

kit-numerals-native = भाषा के अपने अंक

