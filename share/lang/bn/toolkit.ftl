# machine translated (Bengali), not yet reviewed by a person.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = কপি

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = সব নির্বাচন করুন

# Right-click menu on a typing box that has words in it.
kit-cut = কাটুন

# Right-click menu on a typing box.
kit-paste = পেস্ট

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
# SAFETY: needs a human review before it ships
kit-show-password = পাসওয়ার্ড দেখান

# Name of the eye beside a PIN box while the PIN is hidden.
# SAFETY: needs a human review before it ships
kit-show-pin = PIN দেখান

# Name of the eye beside a key box while the key is hidden.
# SAFETY: needs a human review before it ships
kit-show-key = কী দেখান

# Name of the eye beside a password box while the password shows.
# SAFETY: needs a human review before it ships
kit-hide-password = পাসওয়ার্ড লুকান

# Name of the eye beside a PIN box while the PIN shows.
# SAFETY: needs a human review before it ships
kit-hide-pin = PIN লুকান

# Name of the eye beside a key box while the key shows.
# SAFETY: needs a human review before it ships
kit-hide-key = কী লুকান

# Said by the screen reader after the eye beside a password box is pressed to show it.
# SAFETY: needs a human review before it ships
kit-password-shown = পাসওয়ার্ড দেখানো হচ্ছে

# Said by the screen reader after the eye beside a password box is pressed to hide it.
# SAFETY: needs a human review before it ships
kit-password-hidden = পাসওয়ার্ড লুকানো হয়েছে

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
# SAFETY: needs a human review before it ships
kit-pin-shown = PIN দেখানো হচ্ছে

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
# SAFETY: needs a human review before it ships
kit-pin-hidden = PIN লুকানো হয়েছে

# Said by the screen reader after the eye beside a key box is pressed to show it.
# SAFETY: needs a human review before it ships
kit-key-shown = কী দেখানো হচ্ছে

# Said by the screen reader after the eye beside a key box is pressed to hide it.
# SAFETY: needs a human review before it ships
kit-key-hidden = কী লুকানো হয়েছে

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = একটি বেছে নিন

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = খুঁজুন

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = কিছু মেলেনি

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, { $total }টির মধ্যে { $n }

# Read out by the screen reader for a spinner that is turning.
kit-loading = লোড হচ্ছে

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
# SAFETY: needs a human review before it ships
kit-click-again = { $action } করতে আবার ক্লিক করুন

# The same, when the button has no name to repeat.
# SAFETY: needs a human review before it ships
kit-click-again-bare = আবার ক্লিক করুন

kit-job-wait = কাজটি শেষ হওয়া পর্যন্ত অপেক্ষা করুন।

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + আরও

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
        [one] +{ $n }টি লাইন
       *[other] +{ $n }টি লাইন
    }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = যা ফেলা হয়েছিল তা থেকে কিছু পাওয়া যায়নি।

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = যা ফেলা হয়েছে তা এভাবে নেওয়ার জন্য অনেক বড়।

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = যা ফেলা হয়েছিল তা আসতে অনেক সময় নিয়েছে, তাই বাদ দেওয়া হয়েছে।

# Said when too many things were dropped at once.
kit-dropped-too-many = একসঙ্গে অনেক কিছু ফেলা হয়েছে। একবারে কম করে ফেলুন।

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = যা ফেলা হয়েছে তা এই কম্পিউটারের ফাইলের তালিকা নয়।

# Label of a link button whose address is not a web address.
kit-link-not-web = ওয়েব ঠিকানা নয়

# Label of a link button whose address opens a mail program.
kit-link-mailto = একটি mailto ঠিকানা খোলে

# Label of a link button whose address opens a file.
kit-link-file = একটি ফাইল খোলে

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = { $label }-এর লিংক

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = বেছে নেওয়ার উইন্ডো সাড়া দেয়নি। আবার চেষ্টা করুন।

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = লাইন { $line }: { $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = ফাইলটি থিম হওয়ার জন্য অনেক বড়। একটি থিম সর্বোচ্চ 64 KB হতে পারে।

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = ফাইলটি স্টাইল শিট নয়: এতে লেখার চেয়ে বেশি কিছু আছে।

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = থিম অন্য কোনো ফাইল বা ওয়েব ঠিকানা আনতে পারে না।

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = --{ $name } মান হওয়ার জন্য অনেক লম্বা।

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = --{ $name } এমন রং নয় যা Trindle পড়তে পারে।

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = --{ $name } এমন সংখ্যা নয় যা Trindle পড়তে পারে।

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = --{ $name } এমন ছায়া নয় যা Trindle পড়তে পারে।

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = ফাইলের কোনো কিছুই Trindle-এর জানা মান নয়, তাই এটি কিছুই বদলাবে না।

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = অগ্রগতি

# The name a screen reader gives a menu that opens over a window.
kit-menu = মেনু

# Shown at the end of a dropdown's search box: how many rows match out of how many there are.
kit-search-of = { $total }-এর মধ্যে { $n }

# A menu row that opens a sub-menu. $words is the row's own words, $n how many lines the sub-menu holds.
kit-row-sub =
    { $n ->
        [one] { $words }, উপ-মেনু, { $n }টি লাইন
       *[other] { $words }, উপ-মেনু, { $n }টি লাইন
    }

# A menu row with a shortcut. $words is the row's own words, $keys the shortcut.
kit-row-keys = { $words }, { $keys }

# Said by a screen reader for the close box in a tab.
kit-tab-close = বন্ধ করুন

# Said by a screen reader for the button that lists the tabs that did not fit.
kit-tab-more = আরও ট্যাব

# Right-click menu of a text area, one row each.
kit-area-cut = কাটুন

kit-area-copy = কপি করুন

kit-area-paste = পেস্ট করুন

kit-area-select-all = সব বেছে নিন

# The name a screen reader gives the painted square of a colour picker.
kit-colour-square = রং, সম্পৃক্ততা ও উজ্জ্বলতা

# The name a screen reader gives the swatch showing the colour chosen. $colour is its hex code.
kit-colour-swatch = বেছে নেওয়া রং { $colour }

kit-colour-hue = রঙের আভা

kit-colour-hex = হেক্স কোড

# Buttons either side of the month in a date picker.
kit-date-previous = আগেরটি

kit-date-next = পরেরটি

# The month shown in a date picker. $month is the month's name, $year the year.
kit-date-month-year = { $month } { $year }

# What a screen reader says for a day in a date picker.
kit-date-full = { $weekday } { $day } { $month } { $year }

# The names of the two boxes in a time entry.
kit-time-hours = ঘণ্টা

kit-time-minutes = মিনিট

# Months and weekdays; weekdays are short in the grid and full when read out.
kit-month-1 = জানুয়ারি

kit-month-2 = ফেব্রুয়ারি

kit-month-3 = মার্চ

kit-month-4 = এপ্রিল

kit-month-5 = মে

kit-month-6 = জুন

kit-month-7 = জুলাই

kit-month-8 = আগস্ট

kit-month-9 = সেপ্টেম্বর

kit-month-10 = অক্টোবর

kit-month-11 = নভেম্বর

kit-month-12 = ডিসেম্বর

kit-weekday-1 = সোম

kit-weekday-2 = মঙ্গল

kit-weekday-3 = বুধ

kit-weekday-4 = বৃহ

kit-weekday-5 = শুক্র

kit-weekday-6 = শনি

kit-weekday-7 = রবি

kit-weekday-full-1 = সোমবার

kit-weekday-full-2 = মঙ্গলবার

kit-weekday-full-3 = বুধবার

kit-weekday-full-4 = বৃহস্পতিবার

kit-weekday-full-5 = শুক্রবার

kit-weekday-full-6 = শনিবার

kit-weekday-full-7 = রবিবার

# The names of the modifier keys in a shortcut such as Ctrl+Shift+C.
kit-key-ctrl = Ctrl

kit-key-shift = Shift

kit-key-alt = Alt

kit-key-logo = Logo

# Said in a window of Trindle's own when something tried to press a button that only the person may press.
kit-only-your-own = এখানে শুধু আপনার নিজের চাপই কাজ করে।

# Said in a window when keys are pressed but the keyboard layout could not be read.
kit-no-layout = কীবোর্ড লেআউট পড়া যায়নি, তাই টাইপ করলে কিছু হয় না।



# Read out by the screen reader for the line between two areas of a window that can be dragged to make one bigger.
kit-divider = বিভাজক

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
kit-click-again-small = হ্যাঁ

# Read out by the screen reader for the control under something dragged, when letting go there would drop it in.
kit-drop-target = ফেলার জায়গা

# Fault in a theme or style sheet: a setting that takes only named steps was given something else. $steps is the list of steps, $guide the file that explains them.
kit-theme-not-step = --{ $name } এই ধাপগুলোর মধ্যে শুধু একটি নেয়: { $steps }। বাকি সব বাদ যায়। দেখুন { $guide }।

# Standard error, when the folder of the person's style sheet cannot be watched. $folder is the folder, $why the system's reason.
kit-sheet-not-watched = { $folder }-এর স্টাইল শিটের বদল শুধু আবার চালুর পরে দেখা যায়: { $why }

# What a screen reader calls the row of menu titles across the top of a window.
kit-menu-bar = মেনু বার

# The answer at the foot of every dialog page that leaves it with nothing done.
kit-cancel = বাতিল করুন

# What a screen reader calls a row of tool buttons.
kit-toolbar = সরঞ্জাম

# The button at the end of a toolbar that opens a menu of the tools that did not fit.
kit-more-tools = আরও সরঞ্জাম

# What a screen reader calls the line at a window's foot that says how things stand.
kit-status = অবস্থা

# What a screen reader calls a place's path, such as a folder's.
kit-path = পথ

# The list in the middle of a long path, holding the places not shown.
kit-path-more = আরও জায়গা

# Said after a range slider's name for its low handle.
kit-range-low = নিচের প্রান্ত

# Said after a range slider's name for its high handle.
kit-range-high = উপরের প্রান্ত

# The button that takes a step off a number box's number. $what is the box's name.
kit-spin-less = { $what } কমান

# The button that adds a step to a number box's number. $what is the box's name.
kit-spin-more = { $what } বাড়ান

# Red words under a number box whose typing is not a number.
kit-spin-not-number = { $min } থেকে { $max } পর্যন্ত একটি সংখ্যা লিখুন

# Said by a screen reader for a tab that carries a count, such as unread messages: its title, then the count.
kit-tab-count = { $title }, { $count }

# The ready settings page for a program's own look (settings::look_page).
kit-look-title = চেহারা

kit-look-lead = ডেস্কটপের বাকি অংশ যা-ই করুক, এই প্রোগ্রাম কেমন দেখাবে।

kit-look-group = চেহারা

kit-look-shade = হালকা বা গাঢ়

kit-look-shade-help = ডেস্কটপ অনুসরণ করুন, অথবা সবসময় একই চেহারা।

kit-look-desktop = ডেস্কটপের মতো

kit-look-dark = গাঢ়

kit-look-light = হালকা

kit-look-highlight = হাইলাইট রং

kit-look-highlight-help = প্রধান কাজের এবং বেছে নেওয়া জিনিসের রং।

# A button that gives the highlight back to the desktop's choice.
kit-look-highlight-desktop = ডেস্কটপেরটি ব্যবহার করুন

# Over the column of week numbers in a date picker; keep it as short as the short weekday names.
kit-date-week-short = সপ্তাহ

# What a screen reader says for a week number in a date picker. $week is the number.
kit-date-week = সপ্তাহ { $week }

# The names of keys that are words, in a shortcut such as Ctrl+Page down.
kit-key-left = বাঁ

kit-key-right = ডান

kit-key-up = উপর

kit-key-down = নিচ

kit-key-page-up = পেজ আপ

kit-key-page-down = পেজ ডাউন

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
kit-shortcuts = কিবোর্ড শর্টকাট

# Under a shortcut on that page whose keys the window always takes first.
kit-shortcut-kept-window = উইন্ডো এই কী-টি অংশগুলোর মধ্যে যাওয়া ও বন্ধ করার জন্য রাখে।

# Under a shortcut on that page whose keys a text box takes first while you type.
kit-shortcut-kept-typing = আপনি টাইপ করার সময় লেখার বাক্স আগে এই কী-টি নেয়।

# The heading of the page of emoji to pick from.
kit-emoji = ইমোজি

# The name of the box on that page where an emoji's name is typed to find it.
kit-emoji-search = নাম দিয়ে ইমোজি খুঁজুন

# Shown on that page when no emoji's name has what was typed.
kit-emoji-none = এই নামে কোনো ইমোজি নেই।

# The groups of emoji on that page, in order. Another language may leave out any
# name below; the page then shows the English one, so a name is never missing.
kit-emoji-faces = মুখ

kit-emoji-hands = হাত

kit-emoji-animals = প্রাণী

kit-emoji-nature = প্রকৃতি

kit-emoji-food = খাবার ও পানীয়

kit-emoji-travel = ভ্রমণ ও জায়গা

kit-emoji-objects = জিনিসপত্র

kit-emoji-symbols = চিহ্ন

# Each emoji's name, spoken by a screen reader and found by typing.
kit-emoji-grin = দাঁত বের করা হাসি

kit-emoji-tears-of-joy = আনন্দের কান্না

kit-emoji-smile = হাসি

kit-emoji-wink = চোখ টিপ

kit-emoji-heart-eyes = হৃদয় চোখ

kit-emoji-thinking = ভাবনা

kit-emoji-crying = কান্না

kit-emoji-surprised = অবাক

kit-emoji-blush = লাজুক

kit-emoji-halo = জ্যোতিবলয়

kit-emoji-cool = দারুণ

kit-emoji-sleeping = ঘুমন্ত

kit-emoji-sobbing = ফুঁপিয়ে কান্না

kit-emoji-angry = রাগান্বিত

kit-emoji-scream = চিৎকার

kit-emoji-eye-roll = চোখ ঘোরানো

kit-emoji-relieved = স্বস্তি

kit-emoji-rolling-laugh = হেসে গড়াগড়ি

kit-emoji-blowing-a-kiss = চুমু ছোড়া

kit-emoji-straight-face = নির্বিকার মুখ

kit-emoji-flushed = লজ্জায় লাল

kit-emoji-hug = আলিঙ্গন

kit-emoji-unwell = অসুস্থ

kit-emoji-celebrating = উদ্যাপন

kit-emoji-thumbs-up = বুড়ো আঙুল উপরে

kit-emoji-thumbs-down = বুড়ো আঙুল নিচে

kit-emoji-clapping = হাততালি

kit-emoji-thank-you = ধন্যবাদ

kit-emoji-waving = হাত নাড়া

kit-emoji-victory = বিজয়

kit-emoji-ok = ঠিক আছে

kit-emoji-crossed-fingers = আঙুল আড়াআড়ি

kit-emoji-strong = শক্তিশালী

kit-emoji-pointing-up = উপরে দেখানো

kit-emoji-pointing-down = নিচে দেখানো

kit-emoji-pointing-left = বাঁয়ে দেখানো

kit-emoji-pointing-right = ডানে দেখানো

kit-emoji-raised-hand = তোলা হাত

kit-emoji-handshake = করমর্দন

kit-emoji-fist = মুঠো

kit-emoji-raised-hands = দুই হাত তোলা

kit-emoji-call-me = ফোন করুন

kit-emoji-writing = লেখা

kit-emoji-shrug = কাঁধ ঝাঁকানো

kit-emoji-dog = কুকুর

kit-emoji-cat = বিড়াল

kit-emoji-mouse = ইঁদুর

kit-emoji-rabbit = খরগোশ

kit-emoji-fox = শিয়াল

kit-emoji-bear = ভালুক

kit-emoji-panda = পান্ডা

kit-emoji-tiger = বাঘ

kit-emoji-lion = সিংহ

kit-emoji-cow = গরু

kit-emoji-pig = শূকর

kit-emoji-frog = ব্যাঙ

kit-emoji-monkey = বানর

kit-emoji-chicken = মুরগি

kit-emoji-penguin = পেঙ্গুইন

kit-emoji-bird = পাখি

kit-emoji-duck = হাঁস

kit-emoji-owl = পেঁচা

kit-emoji-horse = ঘোড়া

kit-emoji-bee = মৌমাছি

kit-emoji-butterfly = প্রজাপতি

kit-emoji-turtle = কচ্ছপ

kit-emoji-fish = মাছ

kit-emoji-dolphin = শুশুক

kit-emoji-blossom = ফুলের কুঁড়ি

kit-emoji-tree = গাছ

kit-emoji-sun = সূর্য

kit-emoji-rain = বৃষ্টি

kit-emoji-rose = গোলাপ

kit-emoji-sunflower = সূর্যমুখী

kit-emoji-tulip = টিউলিপ

kit-emoji-evergreen = চিরসবুজ

kit-emoji-clover = ক্লোভার

kit-emoji-maple-leaf = ম্যাপল পাতা

kit-emoji-cactus = ক্যাকটাস

kit-emoji-moon = চাঁদ

kit-emoji-rainbow = রামধনু

kit-emoji-cloud = মেঘ

kit-emoji-snowflake = তুষারকণা

kit-emoji-lightning = বিদ্যুৎ চমক

kit-emoji-wave = ঢেউ

kit-emoji-earth = পৃথিবী

kit-emoji-mountain = পাহাড়

kit-emoji-palm-tree = তালগাছ

kit-emoji-apple = আপেল

kit-emoji-bread = রুটি

kit-emoji-cake = কেক

kit-emoji-tea = চা

kit-emoji-banana = কলা

kit-emoji-orange = কমলা

kit-emoji-strawberry = স্ট্রবেরি

kit-emoji-grapes = আঙুর

kit-emoji-watermelon = তরমুজ

kit-emoji-lemon = লেবু

kit-emoji-carrot = গাজর

kit-emoji-corn = ভুট্টা

kit-emoji-pizza = পিৎজা

kit-emoji-burger = বার্গার

kit-emoji-chips = চিপস

kit-emoji-pasta = পাস্তা

kit-emoji-fried-egg = ডিম ভাজা

kit-emoji-cheese = পনির

kit-emoji-biscuit = বিস্কুট

kit-emoji-chocolate = চকলেট

kit-emoji-ice-cream = আইসক্রিম

kit-emoji-beer = বিয়ার

kit-emoji-wine = ওয়াইন

kit-emoji-milk = দুধ

kit-emoji-car = গাড়ি

kit-emoji-bus = বাস

kit-emoji-bicycle = সাইকেল

kit-emoji-train = ট্রেন

kit-emoji-aeroplane = উড়োজাহাজ

kit-emoji-rocket = রকেট

kit-emoji-ship = জাহাজ

kit-emoji-sailing-boat = পালতোলা নৌকা

kit-emoji-helicopter = হেলিকপ্টার

kit-emoji-house = বাড়ি

kit-emoji-office = অফিস

kit-emoji-hospital = হাসপাতাল

kit-emoji-school = স্কুল

kit-emoji-church = গির্জা

kit-emoji-beach = সৈকত

kit-emoji-tower = মিনার

kit-emoji-bridge = সেতু

kit-emoji-petrol-pump = পেট্রল পাম্প

kit-emoji-traffic-light = ট্রাফিক বাতি

kit-emoji-map = মানচিত্র

kit-emoji-phone = ফোন

kit-emoji-laptop = ল্যাপটপ

kit-emoji-keyboard = কিবোর্ড

kit-emoji-camera = ক্যামেরা

kit-emoji-light-bulb = বাল্ব

kit-emoji-key = চাবি

kit-emoji-lock = তালা

kit-emoji-books = বই

kit-emoji-memo = স্মারকলিপি

kit-emoji-pencil = পেনসিল

kit-emoji-paperclip = পেপার ক্লিপ

kit-emoji-calendar = ক্যালেন্ডার

kit-emoji-email = ইমেল

kit-emoji-gift = উপহার

kit-emoji-music-note = সুরের চিহ্ন

kit-emoji-headphones = হেডফোন

kit-emoji-alarm-clock = অ্যালার্ম ঘড়ি

kit-emoji-bell = ঘণ্টা

kit-emoji-spanner = স্প্যানার

kit-emoji-money-bag = টাকার থলি

kit-emoji-shopping-trolley = কেনাকাটার ট্রলি

kit-emoji-football = ফুটবল

kit-emoji-heart = হৃদয়

kit-emoji-star = তারা

kit-emoji-done = শেষ

kit-emoji-question = প্রশ্ন

kit-emoji-party = পার্টি

kit-emoji-fire = আগুন

kit-emoji-cross = ক্রস

kit-emoji-exclamation = বিস্ময়

kit-emoji-warning = সতর্কতা

kit-emoji-hundred = একশো

kit-emoji-broken-heart = ভাঙা হৃদয়

kit-emoji-blue-heart = নীল হৃদয়

kit-emoji-green-heart = সবুজ হৃদয়

kit-emoji-sparkles = ঝিলিক

kit-emoji-plus = যোগ

kit-emoji-minus = বিয়োগ

kit-emoji-right-arrow = ডান তীর

kit-emoji-left-arrow = বাঁ তীর

kit-emoji-refresh = রিফ্রেশ

kit-emoji-forbidden = নিষিদ্ধ

kit-emoji-recycling = পুনর্ব্যবহার

kit-emoji-sleep = ঘুম

# What a screen reader is told of a scene of items: how many, and the one chosen.
kit-scene =
    { $count ->
        [one] 1টি আইটেমের একটি দৃশ্য
       *[other] { $count }টি আইটেমের একটি দৃশ্য
    }

kit-scene-chosen =
    { $count ->
        [one] 1টি আইটেমের একটি দৃশ্য, { $name } বাছা
       *[other] { $count }টি আইটেমের একটি দৃশ্য, { $name } বাছা
    }

## A video or sound player's controls
kit-player-play = চালান

kit-player-pause = বিরতি

kit-player-back = 5 সেকেন্ড পিছনে

kit-player-forward = 5 সেকেন্ড সামনে

kit-player-seek = অবস্থান

kit-player-mute = শব্দ বন্ধ করুন

kit-player-unmute = শব্দ চালু

kit-player-volume = শব্দের মাত্রা

# How far in, out of how long, as clock times.
kit-player-time = { $length }-এর মধ্যে { $at }

## The page shown after a program crashed or met a fault
kit-crash-title = কিছু একটা ভুল হয়েছে

kit-crash-lead = একটি ত্রুটির কারণে { $program } থেমে গেছে। নিচের প্রতিবেদনটি কপি করে আপনার বাগ রিপোর্টের সঙ্গে পাঠান, যাতে এটি ঠিক করা যায়।

kit-crash-copy = প্রতিবেদন কপি করুন

# What a screen reader calls a block of code on a help page.
kit-help-code = কোড

# What a screen reader calls the list of suggestions under a text box; $name is what the box is called.
kit-complete-list = { $name }-এর পরামর্শ

# Under a text box with suggestions: how many there are, said by a screen reader as they change.
kit-complete-count =
    { $n ->
        [one] 1টি পরামর্শ
       *[other] { $n }টি পরামর্শ
    }

## A title bar the program draws, where the compositor draws none
kit-frame-minimise = ছোট করুন

kit-frame-maximise = বড় করুন

kit-frame-restore = আগের মাপে ফেরান

kit-frame-close = বন্ধ করুন

kit-frame-move = সরান

kit-frame-resize = মাপ বদলান

# Settings: which figures numbers are written in.
kit-numerals-title = সংখ্যা

kit-numerals-help = সংখ্যা, তারিখ ও সময় কীভাবে লেখা হবে।

kit-numerals-auto = অঞ্চলে সাধারণত যেভাবে লেখা হয়

kit-numerals-latin = সবসময় 0 থেকে 9

kit-numerals-native = ভাষার নিজের সংখ্যা

