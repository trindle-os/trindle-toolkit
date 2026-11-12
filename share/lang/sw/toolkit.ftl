# machine translated (Swahili), not yet reviewed by a person.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = Nakili

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = Chagua vyote

# Right-click menu on a typing box that has words in it.
kit-cut = Kata

# Right-click menu on a typing box.
kit-paste = Bandika

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
# SAFETY: needs a human review before it ships
kit-show-password = Onyesha nenosiri

# Name of the eye beside a PIN box while the PIN is hidden.
# SAFETY: needs a human review before it ships
kit-show-pin = Onyesha PIN

# Name of the eye beside a key box while the key is hidden.
# SAFETY: needs a human review before it ships
kit-show-key = Onyesha ufunguo

# Name of the eye beside a password box while the password shows.
# SAFETY: needs a human review before it ships
kit-hide-password = Ficha nenosiri

# Name of the eye beside a PIN box while the PIN shows.
# SAFETY: needs a human review before it ships
kit-hide-pin = Ficha PIN

# Name of the eye beside a key box while the key shows.
# SAFETY: needs a human review before it ships
kit-hide-key = Ficha ufunguo

# Said by the screen reader after the eye beside a password box is pressed to show it.
# SAFETY: needs a human review before it ships
kit-password-shown = Nenosiri linaonyeshwa

# Said by the screen reader after the eye beside a password box is pressed to hide it.
# SAFETY: needs a human review before it ships
kit-password-hidden = Nenosiri limefichwa

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
# SAFETY: needs a human review before it ships
kit-pin-shown = PIN inaonyeshwa

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
# SAFETY: needs a human review before it ships
kit-pin-hidden = PIN imefichwa

# Said by the screen reader after the eye beside a key box is pressed to show it.
# SAFETY: needs a human review before it ships
kit-key-shown = Ufunguo unaonyeshwa

# Said by the screen reader after the eye beside a key box is pressed to hide it.
# SAFETY: needs a human review before it ships
kit-key-hidden = Ufunguo umefichwa

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = Chagua moja

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = Tafuta

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = Hakuna kinacholingana

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, { $n } kati ya { $total }

# Read out by the screen reader for a spinner that is turning.
kit-loading = Inapakia

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
# SAFETY: needs a human review before it ships
kit-click-again = Bofya tena ili { $action }

# The same, when the button has no name to repeat.
# SAFETY: needs a human review before it ships
kit-click-again-bare = Bofya tena

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
# SAFETY: needs a human review before it ships
kit-click-again-small = yes

kit-job-wait = Subiri kazi ikamilike.

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + zaidi

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
        [one] +mstari { $n }
       *[other] +mistari { $n }
    }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = Trindle haikuweza kufanya lolote na vitu ulivyodondosha.

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = Ulichodondosha ni kikubwa mno kuburutwa hapa.

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = Ulichodondosha kilichukua muda mrefu mno kufika, kwa hiyo Trindle iliacha kusubiri.

# Said when too many things were dropped at once.
kit-dropped-too-many = Umedondosha vitu vingi mno kwa wakati mmoja. Dondosha vichache kwa wakati.

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = Vitu ulivyodondosha si faili zilizo kwenye kompyuta hii.

# Label of a link button whose address is not a web address.
kit-link-not-web = Si anwani ya wavuti

# Label of a link button whose address opens a mail program.
kit-link-mailto = Anwani ya barua pepe

# Label of a link button whose address opens a file.
kit-link-file = Faili kwenye kompyuta hii

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = Kiungo cha { $label }

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = Dirisha la kuchagua halikujibu. Jaribu tena.

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = Mstari wa { $line }: { $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = Faili hii ya mandhari ni kubwa mno. Ukubwa wa juu ni 64 KB.

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = Faili hii si laha ya mitindo. Ina zaidi ya maandishi matupu.

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = mandhari haiwezi kuingiza faili nyingine wala anwani ya wavuti.

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = Thamani ya --{ $name } ni ndefu mno.

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = Trindle haiwezi kusoma --{ $name } kama rangi.

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = Trindle haiwezi kusoma --{ $name } kama namba.

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = Trindle haiwezi kusoma --{ $name } kama kivuli.

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = Trindle haitambui mipangilio yoyote kwenye faili hii, kwa hiyo hakuna kitakachobadilika.

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = Maendeleo

# The name a screen reader gives a menu that opens over a window.
kit-menu = Menyu

kit-search-of = { $n } kati ya { $total }
kit-divider = Kigawanyo
kit-row-sub =
    { $n ->
        [one] { $words }, menyu ndogo, mstari 1
       *[other] { $words }, menyu ndogo, mistari { $n }
    }
kit-row-keys = { $words }, { $keys }
kit-drop-target = Eneo la kudondosha
kit-theme-not-step = --{ $name } inaweza kuwa moja tu kati ya hizi: { $steps }. Trindle hupuuza thamani nyingine yoyote. Tazama { $guide }.
kit-sheet-not-watched = Anzisha upya ili uone mabadiliko ya laha ya mitindo katika { $folder }. { $why }
kit-menu-bar = Upau wa menyu
kit-cancel = Ghairi
kit-toolbar = Zana
kit-more-tools = Zana zaidi
kit-status = Hali
kit-path = Njia
kit-path-more = Mahali zaidi
kit-range-low = ncha ya chini
kit-range-high = ncha ya juu
kit-spin-less = Punguza { $what }
kit-spin-more = Ongeza { $what }
kit-spin-not-number = Andika nambari kutoka { $min } hadi { $max }
kit-tab-close = Funga
kit-tab-more = Vichupo zaidi
kit-area-cut = Kata
kit-area-copy = Nakili
kit-area-paste = Bandika
kit-area-select-all = Chagua vyote
kit-colour-square = Rangi, kueneza na mwangaza
kit-colour-swatch = Rangi iliyochaguliwa { $colour }
kit-colour-hue = Kivuli cha rangi
kit-colour-hex = Msimbo wa hex
kit-look-title = Mwonekano
kit-look-lead = Chagua jinsi programu hii inavyoonekana, tofauti na kwingineko kwenye eneo-kazi.
kit-look-group = Mwonekano
kit-look-shade = Mwanga au giza
kit-look-shade-help = Fuata eneo-kazi, au tumia mwanga au giza kila mara.
kit-look-desktop = Sawa na eneo-kazi
kit-look-dark = Giza
kit-look-light = Mwanga
kit-look-highlight = Rangi ya kuangazia
kit-look-highlight-help = Rangi ya vitufe vikuu na vitu vilivyochaguliwa.
kit-look-highlight-desktop = Sawa na eneo-kazi
kit-date-previous = Iliyotangulia
kit-date-next = Inayofuata
kit-date-month-year = { $month } { $year }
kit-date-full = { $weekday } { $day } { $month } { $year }
kit-time-hours = Saa
kit-time-minutes = Dakika
kit-month-1 = Januari
kit-month-2 = Februari
kit-month-3 = Machi
kit-month-4 = Aprili
kit-month-5 = Mei
kit-month-6 = Juni
kit-month-7 = Julai
kit-month-8 = Agosti
kit-month-9 = Septemba
kit-month-10 = Oktoba
kit-month-11 = Novemba
kit-month-12 = Desemba
# A month as it reads inside a whole date, such as "12 November 2026". Some languages change the ending there, so it has its own key.
kit-month-in-date-1 = Januari
kit-month-in-date-2 = Februari
kit-month-in-date-3 = Machi
kit-month-in-date-4 = Aprili
kit-month-in-date-5 = Mei
kit-month-in-date-6 = Juni
kit-month-in-date-7 = Julai
kit-month-in-date-8 = Agosti
kit-month-in-date-9 = Septemba
kit-month-in-date-10 = Oktoba
kit-month-in-date-11 = Novemba
kit-month-in-date-12 = Desemba
kit-weekday-1 = Jtatu
kit-weekday-2 = Jnne
kit-weekday-3 = Jtano
kit-weekday-4 = Alh
kit-weekday-5 = Ijm
kit-weekday-6 = Jmosi
kit-weekday-7 = Jpili
kit-weekday-full-1 = Jumatatu
kit-weekday-full-2 = Jumanne
kit-weekday-full-3 = Jumatano
kit-weekday-full-4 = Alhamisi
kit-weekday-full-5 = Ijumaa
kit-weekday-full-6 = Jumamosi
kit-weekday-full-7 = Jumapili
kit-key-ctrl = Ctrl
kit-key-shift = Shift
kit-key-alt = Alt
kit-key-logo = Super
kit-key-left = Kushoto
kit-key-right = Kulia
kit-key-up = Juu
kit-key-down = Chini
kit-key-page-up = Ukurasa juu
kit-key-page-down = Ukurasa chini
kit-key-home = Mwanzo
kit-key-end = Mwisho
kit-key-enter = Enter
kit-key-escape = Escape
kit-key-tab = Tab
kit-key-space = Nafasi
kit-key-backspace = Backspace
kit-key-delete = Futa
kit-key-insert = Insert
kit-shortcuts = Njia za mkato za kibodi
kit-shortcut-kept-window = Kitufe hiki kimetengwa kwa kuhama kati ya vidhibiti na kufunga dirisha.
kit-shortcut-kept-typing = Unapoandika, kitufe hiki huenda kwanza kwenye kisanduku cha maandishi.
kit-emoji = Emoji
kit-emoji-search = Tafuta emoji kwa jina
kit-emoji-none = Hakuna emoji yenye jina hilo.
kit-emoji-faces = Nyuso
kit-emoji-hands = Mikono
kit-emoji-animals = Wanyama
kit-emoji-nature = Asili
kit-emoji-food = Chakula na kinywaji
kit-emoji-travel = Safari na mahali
kit-emoji-objects = Vitu
kit-emoji-symbols = Alama
kit-emoji-grin = Kuchekelea
kit-emoji-tears-of-joy = Machozi ya furaha
kit-emoji-smile = Tabasamu
kit-emoji-wink = Kukonyeza
kit-emoji-heart-eyes = Macho ya moyo
kit-emoji-thinking = Kufikiri
kit-emoji-crying = Kulia
kit-emoji-surprised = Mshangao
kit-emoji-blush = Kuona aibu
kit-emoji-halo = Kitanga
kit-emoji-cool = Poa
kit-emoji-sleeping = Kulala
kit-emoji-sobbing = Kulia kwa kwikwi
kit-emoji-angry = Hasira
kit-emoji-scream = Kupiga yowe
kit-emoji-eye-roll = Kuzungusha macho
kit-emoji-relieved = Afueni
kit-emoji-rolling-laugh = Kucheka hadi kubiringika
kit-emoji-blowing-a-kiss = Kutuma busu
kit-emoji-straight-face = Uso bila hisia
kit-emoji-flushed = Uso uliobadilika rangi
kit-emoji-hug = Kumbatio
kit-emoji-unwell = Mgonjwa
kit-emoji-celebrating = Kusherehekea
kit-emoji-thumbs-up = Gumba juu
kit-emoji-thumbs-down = Gumba chini
kit-emoji-clapping = Makofi
kit-emoji-thank-you = Asante
kit-emoji-waving = Kupunga mkono
kit-emoji-victory = Ushindi
kit-emoji-ok = Sawa
kit-emoji-crossed-fingers = Vidole vilivyovukwa
kit-emoji-strong = Nguvu
kit-emoji-pointing-up = Kuelekeza juu
kit-emoji-pointing-down = Kuelekeza chini
kit-emoji-pointing-left = Kuelekeza kushoto
kit-emoji-pointing-right = Kuelekeza kulia
kit-emoji-raised-hand = Mkono ulioinuliwa
kit-emoji-handshake = Kupeana mikono
kit-emoji-fist = Ngumi
kit-emoji-raised-hands = Mikono iliyoinuliwa
kit-emoji-call-me = Nipigie simu
kit-emoji-writing = Kuandika
kit-emoji-shrug = Kuinua mabega
kit-emoji-dog = Mbwa
kit-emoji-cat = Paka
kit-emoji-mouse = Panya
kit-emoji-rabbit = Sungura
kit-emoji-fox = Mbweha
kit-emoji-bear = Dubu
kit-emoji-panda = Panda
kit-emoji-tiger = Chui madoa
kit-emoji-lion = Simba
kit-emoji-cow = Ng'ombe
kit-emoji-pig = Nguruwe
kit-emoji-frog = Chura
kit-emoji-monkey = Tumbili
kit-emoji-chicken = Kuku
kit-emoji-penguin = Pengwini
kit-emoji-bird = Ndege
kit-emoji-duck = Bata
kit-emoji-owl = Bundi
kit-emoji-horse = Farasi
kit-emoji-bee = Nyuki
kit-emoji-butterfly = Kipepeo
kit-emoji-turtle = Kobe
kit-emoji-fish = Samaki
kit-emoji-dolphin = Pomboo
kit-emoji-blossom = Ua
kit-emoji-tree = Mti
kit-emoji-sun = Jua
kit-emoji-rain = Mvua
kit-emoji-rose = Waridi
kit-emoji-sunflower = Alizeti
kit-emoji-tulip = Tulipu
kit-emoji-evergreen = Mti wa kijani kibichi
kit-emoji-clover = Shamrock
kit-emoji-maple-leaf = Jani la mpera
kit-emoji-cactus = Kaktasi
kit-emoji-moon = Mwezi
kit-emoji-rainbow = Upinde wa mvua
kit-emoji-cloud = Wingu
kit-emoji-snowflake = Chembe ya theluji
kit-emoji-lightning = Radi
kit-emoji-wave = Wimbi
kit-emoji-earth = Dunia
kit-emoji-mountain = Mlima
kit-emoji-palm-tree = Mnazi
kit-emoji-apple = Tufaha
kit-emoji-bread = Mkate
kit-emoji-cake = Keki
kit-emoji-tea = Chai
kit-emoji-banana = Ndizi
kit-emoji-orange = Chungwa
kit-emoji-strawberry = Stroberi
kit-emoji-grapes = Zabibu
kit-emoji-watermelon = Tikiti maji
kit-emoji-lemon = Limau
kit-emoji-carrot = Karoti
kit-emoji-corn = Mahindi
kit-emoji-pizza = Piza
kit-emoji-burger = Baga
kit-emoji-chips = Chipsi
kit-emoji-pasta = Tambi
kit-emoji-fried-egg = Yai la kukaanga
kit-emoji-cheese = Jibini
kit-emoji-biscuit = Biskuti
kit-emoji-chocolate = Chokoleti
kit-emoji-ice-cream = Aiskrimu
kit-emoji-beer = Bia
kit-emoji-wine = Divai
kit-emoji-milk = Maziwa
kit-emoji-car = Gari
kit-emoji-bus = Basi
kit-emoji-bicycle = Baiskeli
kit-emoji-train = Treni
kit-emoji-aeroplane = Ndege
kit-emoji-rocket = Roketi
kit-emoji-ship = Meli
kit-emoji-sailing-boat = Mashua
kit-emoji-helicopter = Helikopta
kit-emoji-house = Nyumba
kit-emoji-office = Ofisi
kit-emoji-hospital = Hospitali
kit-emoji-school = Shule
kit-emoji-church = Kanisa
kit-emoji-beach = Ufuo
kit-emoji-tower = Mnara
kit-emoji-bridge = Daraja
kit-emoji-petrol-pump = Pampu ya petroli
kit-emoji-traffic-light = Taa za barabarani
kit-emoji-map = Ramani
kit-emoji-phone = Simu
kit-emoji-laptop = Kompyuta mpakato
kit-emoji-keyboard = Kibodi
kit-emoji-camera = Kamera
kit-emoji-light-bulb = Balbu
kit-emoji-key = Ufunguo
kit-emoji-lock = Kufuli
kit-emoji-books = Vitabu
kit-emoji-memo = Kumbukumbu
kit-emoji-pencil = Penseli
kit-emoji-paperclip = Kipande cha karatasi
kit-emoji-calendar = Kalenda
kit-emoji-email = Barua pepe
kit-emoji-gift = Zawadi
kit-emoji-music-note = Noti ya muziki
kit-emoji-headphones = Vipokea sauti
kit-emoji-alarm-clock = Saa ya kengele
kit-emoji-bell = Kengele
kit-emoji-spanner = Spana
kit-emoji-money-bag = Mfuko wa pesa
kit-emoji-shopping-trolley = Kigari cha manunuzi
kit-emoji-football = Mpira wa miguu
kit-emoji-heart = Moyo
kit-emoji-star = Nyota
kit-emoji-done = Imekamilika
kit-emoji-question = Swali
kit-emoji-party = Sherehe
kit-emoji-fire = Moto
kit-emoji-cross = Msalaba
kit-emoji-exclamation = Mshangao
kit-emoji-warning = Onyo
kit-emoji-hundred = Mia moja
kit-emoji-broken-heart = Moyo uliovunjika
kit-emoji-blue-heart = Moyo wa buluu
kit-emoji-green-heart = Moyo wa kijani
kit-emoji-sparkles = Mng'ao
kit-emoji-plus = Kujumlisha
kit-emoji-minus = Kutoa
kit-emoji-right-arrow = Mshale wa kulia
kit-emoji-left-arrow = Mshale wa kushoto
kit-emoji-refresh = Onyesha upya
kit-emoji-forbidden = Imekatazwa
kit-emoji-recycling = Kuchakata upya
kit-emoji-sleep = Usingizi
kit-only-your-own = Ni wewe tu unayeweza kubonyeza kitufe hiki.
kit-no-layout = Trindle haikuweza kusoma mpangilio wa kibodi, kwa hiyo huwezi kuandika.
kit-scene =
    { $count ->
        [one] Onyesho la kipengee 1
       *[other] Onyesho la vipengee { $count }
    }
kit-scene-chosen =
    { $count ->
        [one] Onyesho la kipengee 1, { $name } kimechaguliwa
       *[other] Onyesho la vipengee { $count }, { $name } kimechaguliwa
    }
kit-player-play = Cheza
kit-player-pause = Simamisha
kit-player-back = Rudi sekunde 5
kit-player-forward = Songa sekunde 5
kit-player-seek = Nafasi
kit-player-mute = Nyamazisha
kit-player-unmute = Washa sauti
kit-player-volume = Sauti
kit-player-time = { $at } kati ya { $length }
kit-crash-title = Kuna hitilafu
kit-crash-lead = { $program } imesimama kwa sababu kitu kimekwenda vibaya. Nakili ripoti iliyo hapa chini na uitume pamoja na ripoti yako ya hitilafu ili tatizo lirekebishwe.
kit-crash-copy = Nakili ripoti
kit-help-code = Msimbo
kit-complete-count =
    { $n ->
        [one] Pendekezo 1
       *[other] Mapendekezo { $n }
    }
kit-frame-minimise = Punguza
kit-frame-maximise = Kuza
kit-frame-restore = Rejesha
kit-frame-close = Funga
kit-frame-move = Hamisha
kit-frame-resize = Badilisha ukubwa
kit-numerals-title = Mtindo wa namba
kit-numerals-help = Jinsi nambari, tarehe na saa zinavyoandikwa.
kit-numerals-auto = Kawaida kwa eneo lako
kit-numerals-latin = Daima 0 hadi 9
kit-numerals-native = Tarakimu za lugha yenyewe
kit-date-week-short = Wk
kit-date-week = Wiki { $week }
