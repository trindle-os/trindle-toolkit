
# machine translated (Indonesian), not yet reviewed by a person.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = Salin

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = Pilih semua

# Right-click menu on a typing box that has words in it.
kit-cut = Potong

# Right-click menu on a typing box.
kit-paste = Tempel

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
# SAFETY: needs a human review before it ships
kit-show-password = Tampilkan kata sandi

# Name of the eye beside a PIN box while the PIN is hidden.
# SAFETY: needs a human review before it ships
kit-show-pin = Tampilkan PIN

# Name of the eye beside a key box while the key is hidden.
# SAFETY: needs a human review before it ships
kit-show-key = Tampilkan kunci

# Name of the eye beside a password box while the password shows.
kit-hide-password = Sembunyikan kata sandi

# Name of the eye beside a PIN box while the PIN shows.
kit-hide-pin = Sembunyikan PIN

# Name of the eye beside a key box while the key shows.
kit-hide-key = Sembunyikan kunci

# Said by the screen reader after the eye beside a password box is pressed to show it.
kit-password-shown = Kata sandi ditampilkan

# Said by the screen reader after the eye beside a password box is pressed to hide it.
kit-password-hidden = Kata sandi disembunyikan

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
kit-pin-shown = PIN ditampilkan

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
kit-pin-hidden = PIN disembunyikan

# Said by the screen reader after the eye beside a key box is pressed to show it.
kit-key-shown = Kunci ditampilkan

# Said by the screen reader after the eye beside a key box is pressed to hide it.
kit-key-hidden = Kunci disembunyikan

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = Pilih salah satu

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = Cari

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = Tidak ada yang cocok

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, { $n } dari { $total }

# Read out by the screen reader for a spinner that is turning.
kit-loading = Memuat

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
# SAFETY: needs a human review before it ships
kit-click-again = Klik lagi untuk { $action }

# The same, when the button has no name to repeat.
# SAFETY: needs a human review before it ships
kit-click-again-bare = Klik lagi

kit-job-wait = Tunggu sampai pekerjaan selesai.

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + lainnya

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
       *[other] +{ $n } baris
    }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = Tidak ada hasil dari yang dijatuhkan.

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = Yang dijatuhkan terlalu besar untuk diambil dengan cara ini.

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = Yang dijatuhkan terlalu lama datang, jadi dilewati.

# Said when too many things were dropped at once.
kit-dropped-too-many = Terlalu banyak yang dijatuhkan sekaligus. Jatuhkan lebih sedikit sekali waktu.

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = Yang dijatuhkan bukan daftar file di komputer ini.

# Label of a link button whose address is not a web address.
kit-link-not-web = Bukan alamat web

# Label of a link button whose address opens a mail program.
kit-link-mailto = Membuka alamat mailto

# Label of a link button whose address opens a file.
kit-link-file = Membuka file

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = Tautan ke { $label }

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = Jendela pemilihan tidak menjawab. Coba lagi.

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = Baris { $line }: { $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = File terlalu besar untuk menjadi tema. Tema paling besar 64 KB.

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = File ini bukan lembar gaya: isinya lebih dari sekadar kata.

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = tema tidak bisa memuat file lain atau alamat web.

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = --{ $name } terlalu panjang untuk menjadi nilai.

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = --{ $name } bukan warna yang bisa dibaca Trindle.

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = --{ $name } bukan angka yang bisa dibaca Trindle.

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = --{ $name } bukan bayangan yang bisa dibaca Trindle.

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = Tidak ada isi file yang berupa nilai yang dikenal Trindle, jadi tidak ada yang berubah.

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = Kemajuan

# The name a screen reader gives a menu that opens over a window.
kit-menu = Menu

# Shown at the end of a dropdown's search box: how many rows match out of how many there are.
kit-search-of = { $n } dari { $total }

# A menu row that opens a sub-menu. $words is the row's own words, $n how many lines the sub-menu holds.
kit-row-sub =
    { $n ->
       *[other] { $words }, submenu, { $n } baris
    }

# A menu row with a shortcut. $words is the row's own words, $keys the shortcut.
kit-row-keys = { $words }, { $keys }

# Said by a screen reader for the close box in a tab.
kit-tab-close = Tutup

# Said by a screen reader for the button that lists the tabs that did not fit.
kit-tab-more = Tab lainnya

# Right-click menu of a text area, one row each.
kit-area-cut = Potong

kit-area-copy = Salin

kit-area-paste = Tempel

kit-area-select-all = Pilih semua

# The name a screen reader gives the painted square of a colour picker.
kit-colour-square = Warna, kejenuhan, dan kecerahan

# The name a screen reader gives the swatch showing the colour chosen. $colour is its hex code.
kit-colour-swatch = Warna terpilih { $colour }

kit-colour-hue = Rona

kit-colour-hex = Kode heksadesimal

# Buttons either side of the month in a date picker.
kit-date-previous = Sebelumnya

kit-date-next = Berikutnya

# The month shown in a date picker. $month is the month's name, $year the year.
kit-date-month-year = { $month } { $year }

# What a screen reader says for a day in a date picker.
kit-date-full = { $weekday } { $day } { $month } { $year }

# The names of the two boxes in a time entry.
kit-time-hours = Jam

kit-time-minutes = Menit

# Months and weekdays; weekdays are short in the grid and full when read out.
kit-month-1 = Januari

kit-month-2 = Februari

kit-month-3 = Maret

kit-month-4 = April

kit-month-5 = Mei

kit-month-6 = Juni

kit-month-7 = Juli

kit-month-8 = Agustus

kit-month-9 = September

kit-month-10 = Oktober

kit-month-11 = November

kit-month-12 = Desember

kit-weekday-1 = Sen

kit-weekday-2 = Sel

kit-weekday-3 = Rab

kit-weekday-4 = Kam

kit-weekday-5 = Jum

kit-weekday-6 = Sab

kit-weekday-7 = Min

kit-weekday-full-1 = Senin

kit-weekday-full-2 = Selasa

kit-weekday-full-3 = Rabu

kit-weekday-full-4 = Kamis

kit-weekday-full-5 = Jumat

kit-weekday-full-6 = Sabtu

kit-weekday-full-7 = Minggu

# The names of the modifier keys in a shortcut such as Ctrl+Shift+C.
kit-key-ctrl = Ctrl

kit-key-shift = Shift

kit-key-alt = Alt

kit-key-logo = Logo

# Said in a window of Trindle's own when something tried to press a button that only the person may press.
kit-only-your-own = Di sini hanya tekanan Anda sendiri yang berfungsi.

# Said in a window when keys are pressed but the keyboard layout could not be read.
kit-no-layout = Tata letak papan ketik tidak dapat dibaca, jadi mengetik tidak berpengaruh.



# Read out by the screen reader for the line between two areas of a window that can be dragged to make one bigger.
kit-divider = Pembatas

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
kit-click-again-small = ya

# Read out by the screen reader for the control under something dragged, when letting go there would drop it in.
kit-drop-target = Target jatuhan

# Fault in a theme or style sheet: a setting that takes only named steps was given something else. $steps is the list of steps, $guide the file that explains them.
kit-theme-not-step = --{ $name } hanya menerima satu dari langkah ini: { $steps }. Yang lain dilewati. Lihat { $guide }.

# Standard error, when the folder of the person's style sheet cannot be watched. $folder is the folder, $why the system's reason.
kit-sheet-not-watched = Perubahan pada lembar gaya di { $folder } baru tampil setelah dimulai ulang: { $why }

# What a screen reader calls the row of menu titles across the top of a window.
kit-menu-bar = Bilah menu

# The answer at the foot of every dialog page that leaves it with nothing done.
kit-cancel = Batal

# What a screen reader calls a row of tool buttons.
kit-toolbar = Alat

# The button at the end of a toolbar that opens a menu of the tools that did not fit.
kit-more-tools = Alat lainnya

# What a screen reader calls the line at a window's foot that says how things stand.
kit-status = Status

# What a screen reader calls a place's path, such as a folder's.
kit-path = Jalur

# The list in the middle of a long path, holding the places not shown.
kit-path-more = Tempat lainnya

# Said after a range slider's name for its low handle.
kit-range-low = ujung bawah

# Said after a range slider's name for its high handle.
kit-range-high = ujung atas

# The button that takes a step off a number box's number. $what is the box's name.
kit-spin-less = Kurangi { $what }

# The button that adds a step to a number box's number. $what is the box's name.
kit-spin-more = Tambah { $what }

# Red words under a number box whose typing is not a number.
kit-spin-not-number = Ketik angka dari { $min } sampai { $max }

# Said by a screen reader for a tab that carries a count, such as unread messages: its title, then the count.
kit-tab-count = { $title }, { $count }

# The ready settings page for a program's own look (settings::look_page).
kit-look-title = Tampilan

kit-look-lead = Bagaimana program ini tampil, apa pun yang dilakukan bagian desktop lainnya.

kit-look-group = Tampilan

kit-look-shade = Terang atau gelap

kit-look-shade-help = Ikuti desktop, atau selalu satu tampilan.

kit-look-desktop = Seperti desktop

kit-look-dark = Gelap

kit-look-light = Terang

kit-look-highlight = Warna sorotan

kit-look-highlight-help = Warna tindakan utama dan apa yang dipilih.

# A button that gives the highlight back to the desktop's choice.
kit-look-highlight-desktop = Pakai milik desktop

# Over the column of week numbers in a date picker; keep it as short as the short weekday names.
kit-date-week-short = Mg

# What a screen reader says for a week number in a date picker. $week is the number.
kit-date-week = Minggu { $week }

# The names of keys that are words, in a shortcut such as Ctrl+Page down.
kit-key-left = Kiri

kit-key-right = Kanan

kit-key-up = Atas

kit-key-down = Bawah

kit-key-page-up = Halaman atas

kit-key-page-down = Halaman bawah

kit-key-home = Home

kit-key-end = End

kit-key-enter = Enter

kit-key-escape = Escape

kit-key-tab = Tab

kit-key-space = Spasi

kit-key-backspace = Backspace

kit-key-delete = Delete

kit-key-insert = Insert

# The heading of the page that lists a program's keyboard shortcuts.
kit-shortcuts = Pintasan papan ketik

# Under a shortcut on that page whose keys the window always takes first.
kit-shortcut-kept-window = Jendela menyimpan tombol ini untuk berpindah antarbagian dan menutup.

# Under a shortcut on that page whose keys a text box takes first while you type.
kit-shortcut-kept-typing = Kotak teks mengambil tombol ini lebih dulu saat Anda mengetik.

# The heading of the page of emoji to pick from.
kit-emoji = Emoji

# The name of the box on that page where an emoji's name is typed to find it.
kit-emoji-search = Cari emoji berdasarkan nama

# Shown on that page when no emoji's name has what was typed.
kit-emoji-none = Tidak ada emoji dengan nama itu.

# The groups of emoji on that page, in order. Another language may leave out any
# name below; the page then shows the English one, so a name is never missing.
kit-emoji-faces = Wajah

kit-emoji-hands = Tangan

kit-emoji-animals = Hewan

kit-emoji-nature = Alam

kit-emoji-food = Makanan dan minuman

kit-emoji-travel = Perjalanan dan tempat

kit-emoji-objects = Benda

kit-emoji-symbols = Simbol

# Each emoji's name, spoken by a screen reader and found by typing.
kit-emoji-grin = Nyengir

kit-emoji-tears-of-joy = Air mata bahagia

kit-emoji-smile = Senyum

kit-emoji-wink = Kedip

kit-emoji-heart-eyes = Mata hati

kit-emoji-thinking = Berpikir

kit-emoji-crying = Menangis

kit-emoji-surprised = Terkejut

kit-emoji-blush = Tersipu

kit-emoji-halo = Lingkaran suci

kit-emoji-cool = Keren

kit-emoji-sleeping = Tidur

kit-emoji-sobbing = Terisak

kit-emoji-angry = Marah

kit-emoji-scream = Menjerit

kit-emoji-eye-roll = Memutar mata

kit-emoji-relieved = Lega

kit-emoji-rolling-laugh = Tertawa terguling

kit-emoji-blowing-a-kiss = Mengirim ciuman

kit-emoji-straight-face = Wajah datar

kit-emoji-flushed = Wajah merah

kit-emoji-hug = Pelukan

kit-emoji-unwell = Tidak enak badan

kit-emoji-celebrating = Merayakan

kit-emoji-thumbs-up = Jempol ke atas

kit-emoji-thumbs-down = Jempol ke bawah

kit-emoji-clapping = Bertepuk tangan

kit-emoji-thank-you = Terima kasih

kit-emoji-waving = Melambai

kit-emoji-victory = Kemenangan

kit-emoji-ok = OK

kit-emoji-crossed-fingers = Jari bersilang

kit-emoji-strong = Kuat

kit-emoji-pointing-up = Menunjuk ke atas

kit-emoji-pointing-down = Menunjuk ke bawah

kit-emoji-pointing-left = Menunjuk ke kiri

kit-emoji-pointing-right = Menunjuk ke kanan

kit-emoji-raised-hand = Tangan terangkat

kit-emoji-handshake = Jabat tangan

kit-emoji-fist = Kepalan tangan

kit-emoji-raised-hands = Tangan-tangan terangkat

kit-emoji-call-me = Telepon saya

kit-emoji-writing = Menulis

kit-emoji-shrug = Angkat bahu

kit-emoji-dog = Anjing

kit-emoji-cat = Kucing

kit-emoji-mouse = Tikus

kit-emoji-rabbit = Kelinci

kit-emoji-fox = Rubah

kit-emoji-bear = Beruang

kit-emoji-panda = Panda

kit-emoji-tiger = Harimau

kit-emoji-lion = Singa

kit-emoji-cow = Sapi

kit-emoji-pig = Babi

kit-emoji-frog = Katak

kit-emoji-monkey = Monyet

kit-emoji-chicken = Ayam

kit-emoji-penguin = Pinguin

kit-emoji-bird = Burung

kit-emoji-duck = Bebek

kit-emoji-owl = Burung hantu

kit-emoji-horse = Kuda

kit-emoji-bee = Lebah

kit-emoji-butterfly = Kupu-kupu

kit-emoji-turtle = Kura-kura

kit-emoji-fish = Ikan

kit-emoji-dolphin = Lumba-lumba

kit-emoji-blossom = Bunga mekar

kit-emoji-tree = Pohon

kit-emoji-sun = Matahari

kit-emoji-rain = Hujan

kit-emoji-rose = Mawar

kit-emoji-sunflower = Bunga matahari

kit-emoji-tulip = Tulip

kit-emoji-evergreen = Pohon cemara

kit-emoji-clover = Semanggi

kit-emoji-maple-leaf = Daun maple

kit-emoji-cactus = Kaktus

kit-emoji-moon = Bulan

kit-emoji-rainbow = Pelangi

kit-emoji-cloud = Awan

kit-emoji-snowflake = Kepingan salju

kit-emoji-lightning = Kilat

kit-emoji-wave = Ombak

kit-emoji-earth = Bumi

kit-emoji-mountain = Gunung

kit-emoji-palm-tree = Pohon palem

kit-emoji-apple = Apel

kit-emoji-bread = Roti

kit-emoji-cake = Kue

kit-emoji-tea = Teh

kit-emoji-banana = Pisang

kit-emoji-orange = Jeruk

kit-emoji-strawberry = Stroberi

kit-emoji-grapes = Anggur

kit-emoji-watermelon = Semangka

kit-emoji-lemon = Lemon

kit-emoji-carrot = Wortel

kit-emoji-corn = Jagung

kit-emoji-pizza = Pizza

kit-emoji-burger = Burger

kit-emoji-chips = Kentang goreng

kit-emoji-pasta = Pasta

kit-emoji-fried-egg = Telur goreng

kit-emoji-cheese = Keju

kit-emoji-biscuit = Biskuit

kit-emoji-chocolate = Cokelat

kit-emoji-ice-cream = Es krim

kit-emoji-beer = Bir

kit-emoji-wine = Anggur minuman

kit-emoji-milk = Susu

kit-emoji-car = Mobil

kit-emoji-bus = Bus

kit-emoji-bicycle = Sepeda

kit-emoji-train = Kereta

kit-emoji-aeroplane = Pesawat

kit-emoji-rocket = Roket

kit-emoji-ship = Kapal

kit-emoji-sailing-boat = Perahu layar

kit-emoji-helicopter = Helikopter

kit-emoji-house = Rumah

kit-emoji-office = Kantor

kit-emoji-hospital = Rumah sakit

kit-emoji-school = Sekolah

kit-emoji-church = Gereja

kit-emoji-beach = Pantai

kit-emoji-tower = Menara

kit-emoji-bridge = Jembatan

kit-emoji-petrol-pump = Pompa bensin

kit-emoji-traffic-light = Lampu lalu lintas

kit-emoji-map = Peta

kit-emoji-phone = Telepon

kit-emoji-laptop = Laptop

kit-emoji-keyboard = Papan ketik

kit-emoji-camera = Kamera

kit-emoji-light-bulb = Bola lampu

kit-emoji-key = Kunci

kit-emoji-lock = Gembok

kit-emoji-books = Buku-buku

kit-emoji-memo = Memo

kit-emoji-pencil = Pensil

kit-emoji-paperclip = Penjepit kertas

kit-emoji-calendar = Kalender

kit-emoji-email = Email

kit-emoji-gift = Hadiah

kit-emoji-music-note = Not musik

kit-emoji-headphones = Headphone

kit-emoji-alarm-clock = Jam weker

kit-emoji-bell = Lonceng

kit-emoji-spanner = Kunci inggris

kit-emoji-money-bag = Kantong uang

kit-emoji-shopping-trolley = Troli belanja

kit-emoji-football = Sepak bola

kit-emoji-heart = Hati

kit-emoji-star = Bintang

kit-emoji-done = Selesai

kit-emoji-question = Pertanyaan

kit-emoji-party = Pesta

kit-emoji-fire = Api

kit-emoji-cross = Silang

kit-emoji-exclamation = Seru

kit-emoji-warning = Peringatan

kit-emoji-hundred = Seratus

kit-emoji-broken-heart = Hati patah

kit-emoji-blue-heart = Hati biru

kit-emoji-green-heart = Hati hijau

kit-emoji-sparkles = Kilauan

kit-emoji-plus = Plus

kit-emoji-minus = Minus

kit-emoji-right-arrow = Panah kanan

kit-emoji-left-arrow = Panah kiri

kit-emoji-refresh = Segarkan

kit-emoji-forbidden = Dilarang

kit-emoji-recycling = Daur ulang

kit-emoji-sleep = Tidur lelap

# What a screen reader is told of a scene of items: how many, and the one chosen.
kit-scene =
    { $count ->
       *[other] Adegan berisi { $count } item
    }

kit-scene-chosen =
    { $count ->
       *[other] Adegan berisi { $count } item, { $name } dipilih
    }

## A video or sound player's controls
kit-player-play = Putar

kit-player-pause = Jeda

kit-player-back = Mundur 5 detik

kit-player-forward = Maju 5 detik

kit-player-seek = Posisi

kit-player-mute = Bisukan

kit-player-unmute = Suara hidup

kit-player-volume = Volume

# How far in, out of how long, as clock times.
kit-player-time = { $at } dari { $length }

## The page shown after a program crashed or met a fault
kit-crash-title = Terjadi kesalahan

kit-crash-lead = { $program } berhenti karena sebuah gangguan. Salin laporan di bawah dan kirim bersama laporan bug Anda, agar dapat diperbaiki.

kit-crash-copy = Salin laporan

# What a screen reader calls a block of code on a help page.
kit-help-code = Kode

# What a screen reader calls the list of suggestions under a text box; $name is what the box is called.
kit-complete-list = Saran untuk { $name }

# Under a text box with suggestions: how many there are, said by a screen reader as they change.
kit-complete-count =
    { $n ->
       *[other] { $n } saran
    }

## A title bar the program draws, where the compositor draws none
kit-frame-minimise = Minimalkan

kit-frame-maximise = Maksimalkan

kit-frame-restore = Pulihkan

kit-frame-close = Tutup

kit-frame-move = Pindahkan

kit-frame-resize = Ubah ukuran

# Settings: which figures numbers are written in.
kit-numerals-title = Angka

kit-numerals-help = Cara angka, tanggal, dan waktu ditulis.

kit-numerals-auto = Seperti yang biasa ditulis di wilayah itu

kit-numerals-latin = Selalu 0 sampai 9

kit-numerals-native = Angka milik bahasa itu sendiri

