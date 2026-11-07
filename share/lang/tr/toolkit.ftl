# machine translated (Turkish), not yet reviewed by a native speaker.
# machine translated (Turkish), not yet reviewed by a person.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = Kopyala

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = Tümünü seç

# Right-click menu on a typing box that has words in it.
kit-cut = Kes

# Right-click menu on a typing box.
kit-paste = Yapıştır

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
# SAFETY: needs a human review before it ships
kit-show-password = Parolayı göster

# Name of the eye beside a PIN box while the PIN is hidden.
# SAFETY: needs a human review before it ships
kit-show-pin = PIN'i göster

# Name of the eye beside a key box while the key is hidden.
# SAFETY: needs a human review before it ships
kit-show-key = Anahtarı göster

# Name of the eye beside a password box while the password shows.
# SAFETY: needs a human review before it ships
kit-hide-password = Parolayı gizle

# Name of the eye beside a PIN box while the PIN shows.
# SAFETY: needs a human review before it ships
kit-hide-pin = PIN'i gizle

# Name of the eye beside a key box while the key shows.
# SAFETY: needs a human review before it ships
kit-hide-key = Anahtarı gizle

# Said by the screen reader after the eye beside a password box is pressed to show it.
# SAFETY: needs a human review before it ships
kit-password-shown = Parola gösteriliyor

# Said by the screen reader after the eye beside a password box is pressed to hide it.
# SAFETY: needs a human review before it ships
kit-password-hidden = Parola gizlendi

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
# SAFETY: needs a human review before it ships
kit-pin-shown = PIN gösteriliyor

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
# SAFETY: needs a human review before it ships
kit-pin-hidden = PIN gizlendi

# Said by the screen reader after the eye beside a key box is pressed to show it.
# SAFETY: needs a human review before it ships
kit-key-shown = Anahtar gösteriliyor

# Said by the screen reader after the eye beside a key box is pressed to hide it.
# SAFETY: needs a human review before it ships
kit-key-hidden = Anahtar gizlendi

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = Birini seçin

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = Ara

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = Eşleşen yok

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, { $total } içinden { $n }

# Read out by the screen reader for a spinner that is turning.
kit-loading = Yükleniyor

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
# SAFETY: needs a human review before it ships
kit-click-again = Tekrar tıklayın: { $action }

# The same, when the button has no name to repeat.
# SAFETY: needs a human review before it ships
kit-click-again-bare = Tekrar tıklayın

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
# SAFETY: needs a human review before it ships
kit-click-again-small = no

kit-job-wait = İşin bitmesini bekleyin.

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + daha fazla

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
        [one] +{ $n } satır
       *[other] +{ $n } satır
    }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = Bırakılandan bir sonuç çıkmadı.

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = Bırakılan öğe bu yolla alınamayacak kadar büyük.

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = Bırakılan öğenin gelmesi çok uzun sürdü, bu yüzden bırakıldı.

# Said when too many things were dropped at once.
kit-dropped-too-many = Bir seferde çok fazla öğe bırakıldı. Her seferinde daha azını bırakın.

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = Bırakılan, bu bilgisayardaki dosyaların bir listesi değil.

# Label of a link button whose address is not a web address.
kit-link-not-web = Web adresi değil

# Label of a link button whose address opens a mail program.
kit-link-mailto = Bir mailto adresini açar

# Label of a link button whose address opens a file.
kit-link-file = Bir dosyayı açar

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = { $label } bağlantısı

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = Seçme penceresi yanıt vermedi. Yeniden deneyin.

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = { $line }. satır: { $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = Dosya tema olamayacak kadar büyük. Bir tema en fazla 64 KB olabilir.

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = Dosya bir stil sayfası değil: sözcüklerden fazlasını içeriyor.

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = bir tema başka bir dosyayı ya da web adresini içe alamaz.

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = --{ $name } değer olamayacak kadar uzun.

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = --{ $name } Trindle'ın okuyabileceği bir renk değil.

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = --{ $name } Trindle'ın okuyabileceği bir sayı değil.

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = --{ $name } Trindle'ın okuyabileceği bir gölge değil.

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = Dosyadaki hiçbir şey Trindle'ın bildiği bir değer değil, bu yüzden hiçbir şeyi değiştirmez.

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = İlerleme

# The name a screen reader gives a menu that opens over a window.
kit-menu = Menü


kit-search-of = { $n } / { $total }
kit-divider = Ayırıcı
kit-row-sub =
    { $n ->
        [one] { $words }, alt menü, 1 satır
       *[other] { $words }, alt menü, { $n } satır
    }
kit-row-keys = { $words }, { $keys }
kit-drop-target = Bırakma hedefi
kit-theme-not-step = --{ $name } yalnızca şu adımlardan birini alır: { $steps }. Başka her şey atlanır. Bkz. { $guide }.
kit-sheet-not-watched = { $folder } içindeki stil sayfasındaki değişiklikler yalnızca yeniden başlatmadan sonra görünür: { $why }
kit-menu-bar = Menü çubuğu
kit-cancel = İptal
kit-toolbar = Araçlar
kit-more-tools = Daha fazla araç
kit-status = Durum
kit-path = Yol
kit-path-more = Daha fazla yer
kit-range-low = alt uç
kit-range-high = üst uç
kit-spin-less = { $what } azalt
kit-spin-more = { $what } artır
kit-spin-not-number = { $min } ile { $max } arasında bir sayı yazın
kit-tab-close = Kapat
kit-tab-more = Daha fazla sekme
kit-area-cut = Kes
kit-area-copy = Kopyala
kit-area-paste = Yapıştır
kit-area-select-all = Tümünü seç
kit-colour-square = Renk, doygunluk ve parlaklık
kit-colour-swatch = Seçilen renk { $colour }
kit-colour-hue = Ton
kit-colour-hex = Onaltılı kod
kit-look-title = Görünüm
kit-look-lead = Masaüstünün geri kalanı ne yaparsa yapsın, bu programın nasıl göründüğü.
kit-look-group = Görünüm
kit-look-shade = Açık ya da koyu
kit-look-shade-help = Masaüstünü izleyin ya da hep tek bir görünüm kullanın.
kit-look-desktop = Masaüstü gibi
kit-look-dark = Koyu
kit-look-light = Açık
kit-look-highlight = Vurgu rengi
kit-look-highlight-help = Ana eylemin ve seçilen şeyin rengi.
kit-look-highlight-desktop = Masaüstününkini kullan
kit-date-previous = Önceki
kit-date-next = Sonraki
kit-date-month-year = { $month } { $year }
kit-date-full = { $day } { $month } { $year } { $weekday }
kit-time-hours = Saat
kit-time-minutes = Dakika
kit-month-1 = Ocak
kit-month-2 = Şubat
kit-month-3 = Mart
kit-month-4 = Nisan
kit-month-5 = Mayıs
kit-month-6 = Haziran
kit-month-7 = Temmuz
kit-month-8 = Ağustos
kit-month-9 = Eylül
kit-month-10 = Ekim
kit-month-11 = Kasım
kit-month-12 = Aralık
kit-weekday-1 = Pzt
kit-weekday-2 = Sal
kit-weekday-3 = Çar
kit-weekday-4 = Per
kit-weekday-5 = Cum
kit-weekday-6 = Cmt
kit-weekday-7 = Paz
kit-weekday-full-1 = Pazartesi
kit-weekday-full-2 = Salı
kit-weekday-full-3 = Çarşamba
kit-weekday-full-4 = Perşembe
kit-weekday-full-5 = Cuma
kit-weekday-full-6 = Cumartesi
kit-weekday-full-7 = Pazar
kit-key-ctrl = Ctrl
kit-key-shift = Shift
kit-key-alt = Alt
kit-key-logo = Logo
kit-key-left = Sol
kit-key-right = Sağ
kit-key-up = Yukarı
kit-key-down = Aşağı
kit-key-page-up = Sayfa yukarı
kit-key-page-down = Sayfa aşağı
kit-key-home = Home
kit-key-end = End
kit-key-enter = Enter
kit-key-escape = Esc
kit-key-tab = Tab
kit-key-space = Boşluk
kit-key-backspace = Geri sil
kit-key-delete = Delete
kit-key-insert = Insert
kit-shortcuts = Klavye kısayolları
kit-shortcut-kept-window = Pencere bu tuşu bölümler arasında gezinmek ve kapatmak için ayırır.
kit-shortcut-kept-typing = Siz yazarken bu tuşu önce metin kutusu alır.
kit-emoji = Emoji
kit-emoji-search = Adıyla bir emoji bulun
kit-emoji-none = Bu adda emoji yok.
kit-emoji-faces = Yüzler
kit-emoji-hands = Eller
kit-emoji-animals = Hayvanlar
kit-emoji-nature = Doğa
kit-emoji-food = Yiyecek ve içecek
kit-emoji-travel = Seyahat ve yerler
kit-emoji-objects = Nesneler
kit-emoji-symbols = Semboller
kit-emoji-grin = Sırıtma
kit-emoji-tears-of-joy = Sevinç gözyaşları
kit-emoji-smile = Gülümseme
kit-emoji-wink = Göz kırpma
kit-emoji-heart-eyes = Kalp gözler
kit-emoji-thinking = Düşünme
kit-emoji-crying = Ağlama
kit-emoji-surprised = Şaşırmış
kit-emoji-blush = Kızarma
kit-emoji-halo = Hale
kit-emoji-cool = Havalı
kit-emoji-sleeping = Uyuyan
kit-emoji-sobbing = Hıçkırarak ağlama
kit-emoji-angry = Kızgın
kit-emoji-scream = Çığlık
kit-emoji-eye-roll = Göz devirme
kit-emoji-relieved = Rahatlamış
kit-emoji-rolling-laugh = Yerlerde gülme
kit-emoji-blowing-a-kiss = Öpücük gönderme
kit-emoji-straight-face = Ifadesiz yüz
kit-emoji-flushed = Utanmış
kit-emoji-hug = Sarılma
kit-emoji-unwell = Hasta
kit-emoji-celebrating = Kutlama
kit-emoji-thumbs-up = Başparmak yukarı
kit-emoji-thumbs-down = Başparmak aşağı
kit-emoji-clapping = Alkış
kit-emoji-thank-you = Teşekkürler
kit-emoji-waving = El sallama
kit-emoji-victory = Zafer
kit-emoji-ok = Tamam
kit-emoji-crossed-fingers = Çapraz parmaklar
kit-emoji-strong = Güçlü
kit-emoji-pointing-up = Yukarı işaret
kit-emoji-pointing-down = Aşağı işaret
kit-emoji-pointing-left = Sola işaret
kit-emoji-pointing-right = Sağa işaret
kit-emoji-raised-hand = Kalkık el
kit-emoji-handshake = El sıkışma
kit-emoji-fist = Yumruk
kit-emoji-raised-hands = Kalkık eller
kit-emoji-call-me = Beni ara
kit-emoji-writing = Yazma
kit-emoji-shrug = Omuz silkme
kit-emoji-dog = Köpek
kit-emoji-cat = Kedi
kit-emoji-mouse = Fare
kit-emoji-rabbit = Tavşan
kit-emoji-fox = Tilki
kit-emoji-bear = Ayı
kit-emoji-panda = Panda
kit-emoji-tiger = Kaplan
kit-emoji-lion = Aslan
kit-emoji-cow = İnek
kit-emoji-pig = Domuz
kit-emoji-frog = Kurbağa
kit-emoji-monkey = Maymun
kit-emoji-chicken = Tavuk
kit-emoji-penguin = Penguen
kit-emoji-bird = Kuş
kit-emoji-duck = Ördek
kit-emoji-owl = Baykuş
kit-emoji-horse = At
kit-emoji-bee = Arı
kit-emoji-butterfly = Kelebek
kit-emoji-turtle = Kaplumbağa
kit-emoji-fish = Balık
kit-emoji-dolphin = Yunus
kit-emoji-blossom = Çiçek
kit-emoji-tree = Ağaç
kit-emoji-sun = Güneş
kit-emoji-rain = Yağmur
kit-emoji-rose = Gül
kit-emoji-sunflower = Ayçiçeği
kit-emoji-tulip = Lale
kit-emoji-evergreen = Herdem yeşil
kit-emoji-clover = Yonca
kit-emoji-maple-leaf = Akçaağaç yaprağı
kit-emoji-cactus = Kaktüs
kit-emoji-moon = Ay
kit-emoji-rainbow = Gökkuşağı
kit-emoji-cloud = Bulut
kit-emoji-snowflake = Kar tanesi
kit-emoji-lightning = Yıldırım
kit-emoji-wave = Dalga
kit-emoji-earth = Dünya
kit-emoji-mountain = Dağ
kit-emoji-palm-tree = Palmiye
kit-emoji-apple = Elma
kit-emoji-bread = Ekmek
kit-emoji-cake = Pasta
kit-emoji-tea = Çay
kit-emoji-banana = Muz
kit-emoji-orange = Portakal
kit-emoji-strawberry = Çilek
kit-emoji-grapes = Üzüm
kit-emoji-watermelon = Karpuz
kit-emoji-lemon = Limon
kit-emoji-carrot = Havuç
kit-emoji-corn = Mısır
kit-emoji-pizza = Pizza
kit-emoji-burger = Hamburger
kit-emoji-chips = Patates kızartması
kit-emoji-pasta = Makarna
kit-emoji-fried-egg = Sahanda yumurta
kit-emoji-cheese = Peynir
kit-emoji-biscuit = Bisküvi
kit-emoji-chocolate = Çikolata
kit-emoji-ice-cream = Dondurma
kit-emoji-beer = Bira
kit-emoji-wine = Şarap
kit-emoji-milk = Süt
kit-emoji-car = Araba
kit-emoji-bus = Otobüs
kit-emoji-bicycle = Bisiklet
kit-emoji-train = Tren
kit-emoji-aeroplane = Uçak
kit-emoji-rocket = Roket
kit-emoji-ship = Gemi
kit-emoji-sailing-boat = Yelkenli
kit-emoji-helicopter = Helikopter
kit-emoji-house = Ev
kit-emoji-office = Ofis
kit-emoji-hospital = Hastane
kit-emoji-school = Okul
kit-emoji-church = Kilise
kit-emoji-beach = Plaj
kit-emoji-tower = Kule
kit-emoji-bridge = Köprü
kit-emoji-petrol-pump = Benzin pompası
kit-emoji-traffic-light = Trafik lambası
kit-emoji-map = Harita
kit-emoji-phone = Telefon
kit-emoji-laptop = Dizüstü bilgisayar
kit-emoji-keyboard = Klavye
kit-emoji-camera = Kamera
kit-emoji-light-bulb = Ampul
kit-emoji-key = Anahtar
kit-emoji-lock = Kilit
kit-emoji-books = Kitaplar
kit-emoji-memo = Not
kit-emoji-pencil = Kalem
kit-emoji-paperclip = Ataş
kit-emoji-calendar = Takvim
kit-emoji-email = E-posta
kit-emoji-gift = Hediye
kit-emoji-music-note = Müzik notası
kit-emoji-headphones = Kulaklık
kit-emoji-alarm-clock = Çalar saat
kit-emoji-bell = Zil
kit-emoji-spanner = İngiliz anahtarı
kit-emoji-money-bag = Para çantası
kit-emoji-shopping-trolley = Alışveriş arabası
kit-emoji-football = Futbol
kit-emoji-heart = Kalp
kit-emoji-star = Yıldız
kit-emoji-done = Bitti
kit-emoji-question = Soru
kit-emoji-party = Parti
kit-emoji-fire = Ateş
kit-emoji-cross = Çarpı
kit-emoji-exclamation = Ünlem
kit-emoji-warning = Uyarı
kit-emoji-hundred = Yüz
kit-emoji-broken-heart = Kırık kalp
kit-emoji-blue-heart = Mavi kalp
kit-emoji-green-heart = Yeşil kalp
kit-emoji-sparkles = Parıltılar
kit-emoji-plus = Artı
kit-emoji-minus = Eksi
kit-emoji-right-arrow = Sağ ok
kit-emoji-left-arrow = Sol ok
kit-emoji-refresh = Yenile
kit-emoji-forbidden = Yasak
kit-emoji-recycling = Geri dönüşüm
kit-emoji-sleep = Uyku
kit-only-your-own = Burada yalnızca kendi basışınız işe yarar.
kit-no-layout = Klavye düzeni okunamadı, bu yüzden yazmak bir şey yapmaz.
kit-scene =
    { $count ->
        [one] 1 öğelik bir sahne
       *[other] { $count } öğelik bir sahne
    }
kit-scene-chosen =
    { $count ->
        [one] 1 öğelik bir sahne, { $name } seçildi
       *[other] { $count } öğelik bir sahne, { $name } seçildi
    }
kit-player-play = Oynat
kit-player-pause = Duraklat
kit-player-back = 5 saniye geri
kit-player-forward = 5 saniye ileri
kit-player-seek = Konum
kit-player-mute = Sesi kapat
kit-player-unmute = Sesi aç
kit-player-volume = Ses
kit-player-time = { $at } / { $length }
kit-crash-title = Bir şeyler ters gitti
kit-crash-lead = { $program } bir hata yüzünden durdu. Aşağıdaki raporu kopyalayın ve hata bildiriminizle birlikte gönderin, böylece düzeltilebilir.
kit-crash-copy = Raporu kopyala
kit-help-code = Kod
kit-complete-count =
    { $n ->
        [one] 1 öneri
       *[other] { $n } öneri
    }
kit-frame-minimise = Küçült
kit-frame-maximise = Büyüt
kit-frame-restore = Geri yükle
kit-frame-close = Kapat
kit-frame-move = Taşı
kit-frame-resize = Yeniden boyutlandır
kit-numerals-title = Rakamlar
kit-numerals-help = Sayıların, tarihlerin ve saatlerin nasıl yazıldığı.
kit-numerals-auto = Bölgenin genelde yazdığı gibi
kit-numerals-latin = Her zaman 0 ile 9
kit-numerals-native = Dilin kendi rakamları
kit-date-week-short = Hf.
kit-date-week = { $week }. hafta
