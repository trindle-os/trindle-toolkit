# machine translated (Vietnamese), not yet reviewed by a person.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = Sao chép

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = Chọn tất cả

# Right-click menu on a typing box that has words in it.
kit-cut = Cắt

# Right-click menu on a typing box.
kit-paste = Dán

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
# SAFETY: needs a human review before it ships
kit-show-password = Hiện mật khẩu

# Name of the eye beside a PIN box while the PIN is hidden.
kit-show-pin = Hiện mã PIN

# Name of the eye beside a key box while the key is hidden.
kit-show-key = Hiện khóa

# Name of the eye beside a password box while the password shows.
# SAFETY: needs a human review before it ships
kit-hide-password = Ẩn mật khẩu

# Name of the eye beside a PIN box while the PIN shows.
kit-hide-pin = Ẩn mã PIN

# Name of the eye beside a key box while the key shows.
kit-hide-key = Ẩn khóa

# Said by the screen reader after the eye beside a password box is pressed to show it.
kit-password-shown = Đã hiện mật khẩu

# Said by the screen reader after the eye beside a password box is pressed to hide it.
kit-password-hidden = Đã ẩn mật khẩu

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
kit-pin-shown = Đã hiện mã PIN

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
kit-pin-hidden = Đã ẩn mã PIN

# Said by the screen reader after the eye beside a key box is pressed to show it.
kit-key-shown = Đã hiện khóa

# Said by the screen reader after the eye beside a key box is pressed to hide it.
kit-key-hidden = Đã ẩn khóa

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = Chọn một mục

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = Tìm kiếm

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = Không có gì khớp

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, { $n }/{ $total }

# Read out by the screen reader for a spinner that is turning.
kit-loading = Đang tải

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
# SAFETY: needs a human review before it ships
kit-click-again = Nhấn lại để { $action }

# The same, when the button has no name to repeat.
# SAFETY: needs a human review before it ships
kit-click-again-bare = Nhấn lại

kit-job-wait = Hãy đợi công việc hoàn tất.

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + còn nữa

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
       *[other] +{ $n } dòng
    }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = Thứ vừa thả vào không có gì để nhận.

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = Thứ vừa thả vào quá lớn để nhận bằng cách này.

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = Thứ vừa thả vào đến quá chậm nên đã bị bỏ.

# Said when too many things were dropped at once.
kit-dropped-too-many = Đã thả quá nhiều thứ cùng lúc. Mỗi lần hãy thả ít hơn.

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = Thứ vừa thả vào không phải danh sách tệp trên máy này.

# Label of a link button whose address is not a web address.
kit-link-not-web = Không phải địa chỉ web

# Label of a link button whose address opens a mail program.
kit-link-mailto = Mở một địa chỉ mailto

# Label of a link button whose address opens a file.
kit-link-file = Mở một tệp

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = Liên kết tới { $label }

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = Cửa sổ chọn không phản hồi. Hãy thử lại.

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = Dòng { $line }: { $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = Tệp quá lớn để làm giao diện. Một giao diện tối đa 64 KB.

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = Tệp này không phải bảng kiểu dáng: nó chứa nhiều hơn chữ.

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = giao diện không thể nạp thêm tệp khác hay địa chỉ web.

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = --{ $name } quá dài để làm một giá trị.

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = --{ $name } không phải màu mà Trindle đọc được.

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = --{ $name } không phải số mà Trindle đọc được.

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = --{ $name } không phải bóng đổ mà Trindle đọc được.

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = Không có gì trong tệp là giá trị mà Trindle biết, nên nó sẽ không thay đổi gì.

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = Tiến độ

# The name a screen reader gives a menu that opens over a window.
kit-menu = Trình đơn

# Shown at the end of a dropdown's search box: how many rows match out of how many there are.
kit-search-of = { $n } trên { $total }

# A menu row that opens a sub-menu. $words is the row's own words, $n how many lines the sub-menu holds.
kit-row-sub =
    { $n ->
       *[other] { $words }, menu con, { $n } dòng
    }

# A menu row with a shortcut. $words is the row's own words, $keys the shortcut.
kit-row-keys = { $words }, { $keys }

# Said by a screen reader for the close box in a tab.
kit-tab-close = Đóng

# Said by a screen reader for the button that lists the tabs that did not fit.
kit-tab-more = Thêm thẻ

# Right-click menu of a text area, one row each.
kit-area-cut = Cắt

kit-area-copy = Sao chép

kit-area-paste = Dán

kit-area-select-all = Chọn tất cả

# The name a screen reader gives the painted square of a colour picker.
kit-colour-square = Màu, độ bão hòa và độ sáng

# The name a screen reader gives the swatch showing the colour chosen. $colour is its hex code.
kit-colour-swatch = Màu đã chọn { $colour }

kit-colour-hue = Sắc độ

kit-colour-hex = Mã thập lục phân

# Buttons either side of the month in a date picker.
kit-date-previous = Trước

kit-date-next = Sau

# The month shown in a date picker. $month is the month's name, $year the year.
kit-date-month-year = { $month } { $year }

# What a screen reader says for a day in a date picker.
kit-date-full = { $weekday } { $day } { $month } { $year }

# The names of the two boxes in a time entry.
kit-time-hours = Giờ

kit-time-minutes = Phút

# Months and weekdays; weekdays are short in the grid and full when read out.
kit-month-1 = Tháng 1

kit-month-2 = Tháng 2

kit-month-3 = Tháng 3

kit-month-4 = Tháng 4

kit-month-5 = Tháng 5

kit-month-6 = Tháng 6

kit-month-7 = Tháng 7

kit-month-8 = Tháng 8

kit-month-9 = Tháng 9

kit-month-10 = Tháng 10

kit-month-11 = Tháng 11

kit-month-12 = Tháng 12

kit-weekday-1 = T2

kit-weekday-2 = T3

kit-weekday-3 = T4

kit-weekday-4 = T5

kit-weekday-5 = T6

kit-weekday-6 = T7

kit-weekday-7 = CN

kit-weekday-full-1 = Thứ Hai

kit-weekday-full-2 = Thứ Ba

kit-weekday-full-3 = Thứ Tư

kit-weekday-full-4 = Thứ Năm

kit-weekday-full-5 = Thứ Sáu

kit-weekday-full-6 = Thứ Bảy

kit-weekday-full-7 = Chủ Nhật

# The names of the modifier keys in a shortcut such as Ctrl+Shift+C.
kit-key-ctrl = Ctrl

kit-key-shift = Shift

kit-key-alt = Alt

kit-key-logo = Logo

# Said in a window of Trindle's own when something tried to press a button that only the person may press.
kit-only-your-own = Ở đây chỉ thao tác nhấn của chính bạn mới có tác dụng.

# Said in a window when keys are pressed but the keyboard layout could not be read.
kit-no-layout = Không đọc được bố cục bàn phím nên gõ phím sẽ không có tác dụng.



# Read out by the screen reader for the line between two areas of a window that can be dragged to make one bigger.
kit-divider = Đường phân cách

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
kit-click-again-small = có

# Read out by the screen reader for the control under something dragged, when letting go there would drop it in.
kit-drop-target = Nơi thả

# Fault in a theme or style sheet: a setting that takes only named steps was given something else. $steps is the list of steps, $guide the file that explains them.
kit-theme-not-step = --{ $name } chỉ nhận một trong các bước này: { $steps }. Mọi giá trị khác bị bỏ qua. Xem { $guide }.

# Standard error, when the folder of the person's style sheet cannot be watched. $folder is the folder, $why the system's reason.
kit-sheet-not-watched = Các thay đổi của bảng định kiểu trong { $folder } chỉ hiện sau khi khởi động lại: { $why }

# What a screen reader calls the row of menu titles across the top of a window.
kit-menu-bar = Thanh menu

# The answer at the foot of every dialog page that leaves it with nothing done.
kit-cancel = Huỷ

# What a screen reader calls a row of tool buttons.
kit-toolbar = Công cụ

# The button at the end of a toolbar that opens a menu of the tools that did not fit.
kit-more-tools = Thêm công cụ

# What a screen reader calls the line at a window's foot that says how things stand.
kit-status = Trạng thái

# What a screen reader calls a place's path, such as a folder's.
kit-path = Đường dẫn

# The list in the middle of a long path, holding the places not shown.
kit-path-more = Thêm vị trí

# Said after a range slider's name for its low handle.
kit-range-low = đầu thấp

# Said after a range slider's name for its high handle.
kit-range-high = đầu cao

# The button that takes a step off a number box's number. $what is the box's name.
kit-spin-less = Giảm { $what }

# The button that adds a step to a number box's number. $what is the box's name.
kit-spin-more = Tăng { $what }

# Red words under a number box whose typing is not a number.
kit-spin-not-number = Hãy gõ một số từ { $min } đến { $max }

# Said by a screen reader for a tab that carries a count, such as unread messages: its title, then the count.
kit-tab-count = { $title }, { $count }

# The ready settings page for a program's own look (settings::look_page).
kit-look-title = Giao diện

kit-look-lead = Chương trình này trông thế nào, bất kể phần còn lại của màn hình nền làm gì.

kit-look-group = Giao diện

kit-look-shade = Sáng hoặc tối

kit-look-shade-help = Theo màn hình nền, hoặc luôn một giao diện.

kit-look-desktop = Theo màn hình nền

kit-look-dark = Tối

kit-look-light = Sáng

kit-look-highlight = Màu nhấn

kit-look-highlight-help = Màu của hành động chính và của mục đang được chọn.

# A button that gives the highlight back to the desktop's choice.
kit-look-highlight-desktop = Dùng màu của màn hình nền

# Over the column of week numbers in a date picker; keep it as short as the short weekday names.
kit-date-week-short = Tuần

# What a screen reader says for a week number in a date picker. $week is the number.
kit-date-week = Tuần { $week }

# The names of keys that are words, in a shortcut such as Ctrl+Page down.
kit-key-left = Trái

kit-key-right = Phải

kit-key-up = Lên

kit-key-down = Xuống

kit-key-page-up = Trang lên

kit-key-page-down = Trang xuống

kit-key-home = Home

kit-key-end = End

kit-key-enter = Enter

kit-key-escape = Escape

kit-key-tab = Tab

kit-key-space = Dấu cách

kit-key-backspace = Backspace

kit-key-delete = Delete

kit-key-insert = Insert

# The heading of the page that lists a program's keyboard shortcuts.
kit-shortcuts = Phím tắt

# Under a shortcut on that page whose keys the window always takes first.
kit-shortcut-kept-window = Cửa sổ giữ phím này để di chuyển giữa các phần và để đóng.

# Under a shortcut on that page whose keys a text box takes first while you type.
kit-shortcut-kept-typing = Ô nhập văn bản nhận phím này trước khi bạn gõ.

# The heading of the page of emoji to pick from.
kit-emoji = Emoji

# The name of the box on that page where an emoji's name is typed to find it.
kit-emoji-search = Tìm emoji theo tên

# Shown on that page when no emoji's name has what was typed.
kit-emoji-none = Không có emoji nào mang tên đó.

# The groups of emoji on that page, in order. Another language may leave out any
# name below; the page then shows the English one, so a name is never missing.
kit-emoji-faces = Khuôn mặt

kit-emoji-hands = Bàn tay

kit-emoji-animals = Động vật

kit-emoji-nature = Thiên nhiên

kit-emoji-food = Đồ ăn thức uống

kit-emoji-travel = Du lịch và địa điểm

kit-emoji-objects = Đồ vật

kit-emoji-symbols = Biểu tượng

# Each emoji's name, spoken by a screen reader and found by typing.
kit-emoji-grin = Cười toe

kit-emoji-tears-of-joy = Khóc vì cười

kit-emoji-smile = Mỉm cười

kit-emoji-wink = Nháy mắt

kit-emoji-heart-eyes = Mắt trái tim

kit-emoji-thinking = Suy nghĩ

kit-emoji-crying = Khóc

kit-emoji-surprised = Ngạc nhiên

kit-emoji-blush = Đỏ mặt

kit-emoji-halo = Vầng hào quang

kit-emoji-cool = Ngầu

kit-emoji-sleeping = Ngủ

kit-emoji-sobbing = Khóc nức nở

kit-emoji-angry = Giận dữ

kit-emoji-scream = Hét lên

kit-emoji-eye-roll = Đảo mắt

kit-emoji-relieved = Nhẹ nhõm

kit-emoji-rolling-laugh = Cười lăn lộn

kit-emoji-blowing-a-kiss = Gửi nụ hôn

kit-emoji-straight-face = Mặt lạnh tanh

kit-emoji-flushed = Mặt ửng đỏ

kit-emoji-hug = Ôm

kit-emoji-unwell = Không khoẻ

kit-emoji-celebrating = Ăn mừng

kit-emoji-thumbs-up = Giơ ngón cái lên

kit-emoji-thumbs-down = Giơ ngón cái xuống

kit-emoji-clapping = Vỗ tay

kit-emoji-thank-you = Cảm ơn

kit-emoji-waving = Vẫy tay

kit-emoji-victory = Chiến thắng

kit-emoji-ok = OK

kit-emoji-crossed-fingers = Bắt chéo ngón tay

kit-emoji-strong = Mạnh mẽ

kit-emoji-pointing-up = Chỉ lên

kit-emoji-pointing-down = Chỉ xuống

kit-emoji-pointing-left = Chỉ sang trái

kit-emoji-pointing-right = Chỉ sang phải

kit-emoji-raised-hand = Giơ tay

kit-emoji-handshake = Bắt tay

kit-emoji-fist = Nắm đấm

kit-emoji-raised-hands = Giơ hai tay

kit-emoji-call-me = Gọi cho tôi

kit-emoji-writing = Viết

kit-emoji-shrug = Nhún vai

kit-emoji-dog = Chó

kit-emoji-cat = Mèo

kit-emoji-mouse = Chuột

kit-emoji-rabbit = Thỏ

kit-emoji-fox = Cáo

kit-emoji-bear = Gấu

kit-emoji-panda = Gấu trúc

kit-emoji-tiger = Hổ

kit-emoji-lion = Sư tử

kit-emoji-cow = Bò

kit-emoji-pig = Lợn

kit-emoji-frog = Ếch

kit-emoji-monkey = Khỉ

kit-emoji-chicken = Gà

kit-emoji-penguin = Chim cánh cụt

kit-emoji-bird = Chim

kit-emoji-duck = Vịt

kit-emoji-owl = Cú

kit-emoji-horse = Ngựa

kit-emoji-bee = Ong

kit-emoji-butterfly = Bướm

kit-emoji-turtle = Rùa

kit-emoji-fish = Cá

kit-emoji-dolphin = Cá heo

kit-emoji-blossom = Hoa nở

kit-emoji-tree = Cây

kit-emoji-sun = Mặt trời

kit-emoji-rain = Mưa

kit-emoji-rose = Hoa hồng

kit-emoji-sunflower = Hoa hướng dương

kit-emoji-tulip = Hoa tulip

kit-emoji-evergreen = Cây thường xanh

kit-emoji-clover = Cỏ ba lá

kit-emoji-maple-leaf = Lá phong

kit-emoji-cactus = Xương rồng

kit-emoji-moon = Mặt trăng

kit-emoji-rainbow = Cầu vồng

kit-emoji-cloud = Mây

kit-emoji-snowflake = Bông tuyết

kit-emoji-lightning = Sét

kit-emoji-wave = Sóng

kit-emoji-earth = Trái đất

kit-emoji-mountain = Núi

kit-emoji-palm-tree = Cây cọ

kit-emoji-apple = Táo

kit-emoji-bread = Bánh mì

kit-emoji-cake = Bánh ngọt

kit-emoji-tea = Trà

kit-emoji-banana = Chuối

kit-emoji-orange = Cam

kit-emoji-strawberry = Dâu tây

kit-emoji-grapes = Nho

kit-emoji-watermelon = Dưa hấu

kit-emoji-lemon = Chanh

kit-emoji-carrot = Cà rốt

kit-emoji-corn = Ngô

kit-emoji-pizza = Pizza

kit-emoji-burger = Bánh mì kẹp thịt

kit-emoji-chips = Khoai tây chiên

kit-emoji-pasta = Mì Ý

kit-emoji-fried-egg = Trứng ốp la

kit-emoji-cheese = Phô mai

kit-emoji-biscuit = Bánh quy

kit-emoji-chocolate = Sô cô la

kit-emoji-ice-cream = Kem

kit-emoji-beer = Bia

kit-emoji-wine = Rượu vang

kit-emoji-milk = Sữa

kit-emoji-car = Ô tô

kit-emoji-bus = Xe buýt

kit-emoji-bicycle = Xe đạp

kit-emoji-train = Tàu hoả

kit-emoji-aeroplane = Máy bay

kit-emoji-rocket = Tên lửa

kit-emoji-ship = Tàu thuỷ

kit-emoji-sailing-boat = Thuyền buồm

kit-emoji-helicopter = Trực thăng

kit-emoji-house = Nhà

kit-emoji-office = Văn phòng

kit-emoji-hospital = Bệnh viện

kit-emoji-school = Trường học

kit-emoji-church = Nhà thờ

kit-emoji-beach = Bãi biển

kit-emoji-tower = Tháp

kit-emoji-bridge = Cầu

kit-emoji-petrol-pump = Cây xăng

kit-emoji-traffic-light = Đèn giao thông

kit-emoji-map = Bản đồ

kit-emoji-phone = Điện thoại

kit-emoji-laptop = Laptop

kit-emoji-keyboard = Bàn phím

kit-emoji-camera = Máy ảnh

kit-emoji-light-bulb = Bóng đèn

kit-emoji-key = Chìa khoá

kit-emoji-lock = Ổ khoá

kit-emoji-books = Sách

kit-emoji-memo = Ghi nhớ

kit-emoji-pencil = Bút chì

kit-emoji-paperclip = Kẹp giấy

kit-emoji-calendar = Lịch

kit-emoji-email = Thư điện tử

kit-emoji-gift = Quà tặng

kit-emoji-music-note = Nốt nhạc

kit-emoji-headphones = Tai nghe

kit-emoji-alarm-clock = Đồng hồ báo thức

kit-emoji-bell = Chuông

kit-emoji-spanner = Cờ lê

kit-emoji-money-bag = Túi tiền

kit-emoji-shopping-trolley = Xe đẩy mua sắm

kit-emoji-football = Bóng đá

kit-emoji-heart = Trái tim

kit-emoji-star = Ngôi sao

kit-emoji-done = Xong

kit-emoji-question = Dấu hỏi

kit-emoji-party = Tiệc

kit-emoji-fire = Lửa

kit-emoji-cross = Dấu nhân

kit-emoji-exclamation = Dấu chấm than

kit-emoji-warning = Cảnh báo

kit-emoji-hundred = Một trăm

kit-emoji-broken-heart = Trái tim tan vỡ

kit-emoji-blue-heart = Trái tim xanh dương

kit-emoji-green-heart = Trái tim xanh lá

kit-emoji-sparkles = Lấp lánh

kit-emoji-plus = Dấu cộng

kit-emoji-minus = Dấu trừ

kit-emoji-right-arrow = Mũi tên phải

kit-emoji-left-arrow = Mũi tên trái

kit-emoji-refresh = Làm mới

kit-emoji-forbidden = Cấm

kit-emoji-recycling = Tái chế

kit-emoji-sleep = Ngủ

# What a screen reader is told of a scene of items: how many, and the one chosen.
kit-scene =
    { $count ->
       *[other] Một cảnh gồm { $count } mục
    }

kit-scene-chosen =
    { $count ->
       *[other] Một cảnh gồm { $count } mục, đã chọn { $name }
    }

## A video or sound player's controls
kit-player-play = Phát

kit-player-pause = Tạm dừng

kit-player-back = Lùi 5 giây

kit-player-forward = Tới 5 giây

kit-player-seek = Vị trí

kit-player-mute = Tắt tiếng

kit-player-unmute = Bật tiếng

kit-player-volume = Âm lượng

# How far in, out of how long, as clock times.
kit-player-time = { $at } trên { $length }

## The page shown after a program crashed or met a fault
kit-crash-title = Đã xảy ra sự cố

kit-crash-lead = { $program } đã dừng vì một lỗi. Hãy sao chép báo cáo bên dưới và gửi cùng báo cáo lỗi của bạn, để lỗi được sửa.

kit-crash-copy = Sao chép báo cáo

# What a screen reader calls a block of code on a help page.
kit-help-code = Mã

# What a screen reader calls the list of suggestions under a text box; $name is what the box is called.
kit-complete-list = Gợi ý cho { $name }

# Under a text box with suggestions: how many there are, said by a screen reader as they change.
kit-complete-count =
    { $n ->
       *[other] { $n } gợi ý
    }

## A title bar the program draws, where the compositor draws none
kit-frame-minimise = Thu nhỏ

kit-frame-maximise = Phóng to

kit-frame-restore = Khôi phục

kit-frame-close = Đóng

kit-frame-move = Di chuyển

kit-frame-resize = Đổi cỡ

# Settings: which figures numbers are written in.
kit-numerals-title = Chữ số

kit-numerals-help = Cách viết số, ngày và giờ.

kit-numerals-auto = Như khu vực thường viết

kit-numerals-latin = Luôn từ 0 đến 9

kit-numerals-native = Chữ số riêng của ngôn ngữ

