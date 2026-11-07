# machine translated (Korean), not yet reviewed by a native speaker.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = 복사

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = 모두 선택

# Right-click menu on a typing box that has words in it.
kit-cut = 잘라내기

# Right-click menu on a typing box.
kit-paste = 붙여넣기

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
kit-show-password = 비밀번호 표시

# Name of the eye beside a PIN box while the PIN is hidden.
kit-show-pin = PIN 표시

# Name of the eye beside a key box while the key is hidden.
kit-show-key = 키 표시

# Name of the eye beside a password box while the password shows.
kit-hide-password = 비밀번호 숨기기

# Name of the eye beside a PIN box while the PIN shows.
kit-hide-pin = PIN 숨기기

# Name of the eye beside a key box while the key shows.
kit-hide-key = 키 숨기기

# Said by the screen reader after the eye beside a password box is pressed to show it.
kit-password-shown = 비밀번호를 표시했습니다

# Said by the screen reader after the eye beside a password box is pressed to hide it.
kit-password-hidden = 비밀번호를 숨겼습니다

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
kit-pin-shown = PIN을 표시했습니다

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
kit-pin-hidden = PIN을 숨겼습니다

# Said by the screen reader after the eye beside a key box is pressed to show it.
kit-key-shown = 키를 표시했습니다

# Said by the screen reader after the eye beside a key box is pressed to hide it.
kit-key-hidden = 키를 숨겼습니다

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = 하나를 선택하세요

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = 검색

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = 일치하는 항목이 없습니다

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, 전체 { $total }개 중 { $n }개

# Shown at the end of a dropdown's search box: how many rows match out of how many there are.
kit-search-of = 전체 { $total }개 중 { $n }개

# Read out by the screen reader for a spinner that is turning.
kit-loading = 불러오는 중

# Read out by the screen reader for the line between two areas of a window that can be dragged to make one bigger.
kit-divider = 구분선

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
# SAFETY: needs a human review before it ships
kit-click-again = 다시 클릭: { $action }

# The same, when the button has no name to repeat.
kit-click-again-bare = 다시 클릭하세요

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
# SAFETY: needs a human review before it ships
kit-click-again-small = no

kit-job-wait = 작업이 끝날 때까지 기다려 주세요.

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + 더 보기

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
       *[other] +{ $n }줄 더
    }

# A menu row that opens a sub-menu. $words is the row's own words, $n how many lines the sub-menu holds.
kit-row-sub =
    { $n ->
       *[other] { $words }, 하위 메뉴, { $n }개 항목
    }

# A menu row with a shortcut. $words is the row's own words, $keys the shortcut.
kit-row-keys = { $words }, { $keys }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = 놓은 항목에서 아무것도 가져오지 못했습니다.

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = 놓은 항목이 너무 커서 이 방법으로는 받을 수 없습니다.

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = 놓은 항목이 너무 늦게 도착해서 받지 않았습니다.

# Said when too many things were dropped at once.
kit-dropped-too-many = 한꺼번에 너무 많은 항목을 놓았습니다. 한 번에 적게 놓아 주세요.

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = 놓은 항목은 이 컴퓨터에 있는 파일 목록이 아닙니다.

# Read out by the screen reader for the control under something dragged, when letting go there would drop it in.
kit-drop-target = 놓을 위치

# Label of a link button whose address is not a web address.
kit-link-not-web = 웹 주소가 아닙니다

# Label of a link button whose address opens a mail program.
kit-link-mailto = 메일 주소를 엽니다

# Label of a link button whose address opens a file.
kit-link-file = 파일을 엽니다

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = { $label }(으)로 연결되는 링크

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = 선택 창이 응답하지 않았습니다. 다시 시도해 주세요.

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = { $line }번째 줄: { $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = 파일이 너무 커서 테마로 쓸 수 없습니다. 테마는 최대 64 KB입니다.

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = 이 파일은 스타일 시트가 아닙니다. 글자 외의 내용이 들어 있습니다.

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = 테마에서 다른 파일이나 웹 주소를 불러올 수 없습니다.

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = --{ $name }이(가) 너무 길어서 값으로 쓸 수 없습니다.

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = --{ $name }은(는) Trindle이 읽을 수 있는 색이 아닙니다.

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = --{ $name }은(는) Trindle이 읽을 수 있는 숫자가 아닙니다.

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = --{ $name }은(는) Trindle이 읽을 수 있는 그림자가 아닙니다.

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = 파일에 Trindle이 아는 값이 없어서 아무것도 바뀌지 않습니다.

# Standard error, when the folder of the person's style sheet cannot be watched. $folder is the folder, $why the system's reason.
kit-sheet-not-watched = { $folder }에 있는 스타일 시트의 변경 사항은 다시 시작한 뒤에 적용됩니다: { $why }

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = 진행률

# The name a screen reader gives a menu that opens over a window.
kit-menu = 메뉴

# What a screen reader calls the row of menu titles across the top of a window.
kit-menu-bar = 메뉴 막대

# The answer at the foot of every dialog page that leaves it with nothing done.
kit-cancel = 취소

# What a screen reader calls a row of tool buttons.
kit-toolbar = 도구

# The button at the end of a toolbar that opens a menu of the tools that did not fit.
kit-more-tools = 도구 더 보기

# What a screen reader calls the line at a window's foot that says how things stand.
kit-status = 상태

# What a screen reader calls a place's path, such as a folder's.
kit-path = 경로

# The list in the middle of a long path, holding the places not shown.
kit-path-more = 다른 위치

# Said after a range slider's name for its low handle.
kit-range-low = 최솟값

# Said after a range slider's name for its high handle.
kit-range-high = 최댓값

# The button that takes a step off a number box's number. $what is the box's name.
kit-spin-less = { $what } 줄이기

# The button that adds a step to a number box's number. $what is the box's name.
kit-spin-more = { $what } 늘리기

# Red words under a number box whose typing is not a number.
kit-spin-not-number = { $min }부터 { $max }까지의 숫자를 입력하세요

# Said by a screen reader for the close box in a tab.
kit-tab-close = 닫기

# Said by a screen reader for the button that lists the tabs that did not fit.
kit-tab-more = 탭 더 보기

# Right-click menu of a text area, one row each.
kit-area-cut = 잘라내기

kit-area-copy = 복사

kit-area-paste = 붙여넣기

kit-area-select-all = 모두 선택

# The name a screen reader gives the painted square of a colour picker.
kit-colour-square = 색상, 채도, 밝기

# The name a screen reader gives the swatch showing the colour chosen. $colour is its hex code.
kit-colour-swatch = 선택한 색 { $colour }

kit-colour-hue = 색조

kit-colour-hex = 16진수 코드

# The ready settings page for a program's own look (settings::look_page).
kit-look-title = 모양

kit-look-lead = 데스크톱 설정과 관계없이 이 프로그램의 모양을 정합니다.

kit-look-group = 모양

kit-look-shade = 밝게 또는 어둡게

kit-look-shade-help = 데스크톱에 맞추거나, 항상 한쪽으로 고정합니다.

kit-look-desktop = 데스크톱에 맞춤

kit-look-dark = 어둡게

kit-look-light = 밝게

kit-look-highlight = 강조 색

kit-look-highlight-help = 주요 동작과 선택한 항목의 색입니다.

# A button that gives the highlight back to the desktop's choice.
kit-look-highlight-desktop = 데스크톱 색 사용

# Buttons either side of the month in a date picker.
kit-date-previous = 이전

kit-date-next = 다음

# The month shown in a date picker. $month is the month's name, $year the year.
kit-date-month-year = { $year }년 { $month }

# What a screen reader says for a day in a date picker.
kit-date-full = { $year }년 { $month } { $day }일 { $weekday }

# The names of the two boxes in a time entry.
kit-time-hours = 시

kit-time-minutes = 분

# Months and weekdays; weekdays are short in the grid and full when read out.
kit-month-1 = 1월

kit-month-2 = 2월

kit-month-3 = 3월

kit-month-4 = 4월

kit-month-5 = 5월

kit-month-6 = 6월

kit-month-7 = 7월

kit-month-8 = 8월

kit-month-9 = 9월

kit-month-10 = 10월

kit-month-11 = 11월

kit-month-12 = 12월

kit-weekday-1 = 월

kit-weekday-2 = 화

kit-weekday-3 = 수

kit-weekday-4 = 목

kit-weekday-5 = 금

kit-weekday-6 = 토

kit-weekday-7 = 일

kit-weekday-full-1 = 월요일

kit-weekday-full-2 = 화요일

kit-weekday-full-3 = 수요일

kit-weekday-full-4 = 목요일

kit-weekday-full-5 = 금요일

kit-weekday-full-6 = 토요일

kit-weekday-full-7 = 일요일

# The names of the modifier keys in a shortcut such as Ctrl+Shift+C.
kit-key-ctrl = Ctrl

kit-key-shift = Shift

kit-key-alt = Alt

kit-key-logo = 로고

# The names of keys that are words, in a shortcut such as Ctrl+Page down.
kit-key-left = 왼쪽

kit-key-right = 오른쪽

kit-key-up = 위쪽

kit-key-down = 아래쪽

kit-key-page-up = 페이지 위로

kit-key-page-down = 페이지 아래로

kit-key-home = Home

kit-key-end = End

kit-key-enter = Enter

kit-key-escape = Esc

kit-key-tab = Tab

kit-key-space = 스페이스

kit-key-backspace = Backspace

kit-key-delete = Delete

kit-key-insert = Insert

# The heading of the page that lists a program's keyboard shortcuts.
kit-shortcuts = 키보드 단축키

# Under a shortcut on that page whose keys the window always takes first.
kit-shortcut-kept-window = 이 키는 부분 간 이동과 창 닫기를 위해 창이 먼저 사용합니다.

# Under a shortcut on that page whose keys a text box takes first while you type.
kit-shortcut-kept-typing = 입력하는 동안에는 이 키를 텍스트 상자가 먼저 사용합니다.

# The heading of the page of emoji to pick from.
kit-emoji = 이모지

# The name of the box on that page where an emoji's name is typed to find it.
kit-emoji-search = 이름으로 이모지 찾기

# Shown on that page when no emoji's name has what was typed.
kit-emoji-none = 해당 이름의 이모지가 없습니다.

# The groups of emoji on that page, in order. Another language may leave out any
# name below; the page then shows the English one, so a name is never missing.
kit-emoji-faces = 얼굴

kit-emoji-hands = 손

kit-emoji-animals = 동물

kit-emoji-nature = 자연

kit-emoji-food = 음식과 음료

kit-emoji-travel = 여행과 장소

kit-emoji-objects = 사물

kit-emoji-symbols = 기호

# Each emoji's name, spoken by a screen reader and found by typing.
kit-emoji-grin = 활짝 웃음

kit-emoji-tears-of-joy = 기쁨의 눈물

kit-emoji-smile = 미소

kit-emoji-wink = 윙크

kit-emoji-heart-eyes = 하트 눈

kit-emoji-thinking = 생각 중

kit-emoji-crying = 우는 얼굴

kit-emoji-surprised = 놀람

kit-emoji-blush = 부끄러움

kit-emoji-halo = 천사

kit-emoji-cool = 멋짐

kit-emoji-sleeping = 자는 얼굴

kit-emoji-sobbing = 펑펑 움

kit-emoji-angry = 화남

kit-emoji-scream = 비명

kit-emoji-eye-roll = 눈 굴리기

kit-emoji-relieved = 안도

kit-emoji-rolling-laugh = 데굴데굴 웃음

kit-emoji-blowing-a-kiss = 날리는 키스

kit-emoji-straight-face = 무표정

kit-emoji-flushed = 얼굴 붉힘

kit-emoji-hug = 포옹

kit-emoji-unwell = 몸이 안 좋음

kit-emoji-celebrating = 축하

kit-emoji-thumbs-up = 좋아요

kit-emoji-thumbs-down = 싫어요

kit-emoji-clapping = 박수

kit-emoji-thank-you = 감사

kit-emoji-waving = 손 흔들기

kit-emoji-victory = 승리

kit-emoji-ok = OK

kit-emoji-crossed-fingers = 행운을 비는 손가락

kit-emoji-strong = 근육

kit-emoji-pointing-up = 위를 가리킴

kit-emoji-pointing-down = 아래를 가리킴

kit-emoji-pointing-left = 왼쪽을 가리킴

kit-emoji-pointing-right = 오른쪽을 가리킴

kit-emoji-raised-hand = 든 손

kit-emoji-handshake = 악수

kit-emoji-fist = 주먹

kit-emoji-raised-hands = 만세

kit-emoji-call-me = 전화해 줘

kit-emoji-writing = 글쓰기

kit-emoji-shrug = 어깨 으쓱

kit-emoji-dog = 개

kit-emoji-cat = 고양이

kit-emoji-mouse = 쥐

kit-emoji-rabbit = 토끼

kit-emoji-fox = 여우

kit-emoji-bear = 곰

kit-emoji-panda = 판다

kit-emoji-tiger = 호랑이

kit-emoji-lion = 사자

kit-emoji-cow = 소

kit-emoji-pig = 돼지

kit-emoji-frog = 개구리

kit-emoji-monkey = 원숭이

kit-emoji-chicken = 닭

kit-emoji-penguin = 펭귄

kit-emoji-bird = 새

kit-emoji-duck = 오리

kit-emoji-owl = 부엉이

kit-emoji-horse = 말

kit-emoji-bee = 벌

kit-emoji-butterfly = 나비

kit-emoji-turtle = 거북이

kit-emoji-fish = 물고기

kit-emoji-dolphin = 돌고래

kit-emoji-blossom = 꽃

kit-emoji-tree = 나무

kit-emoji-sun = 해

kit-emoji-rain = 비

kit-emoji-rose = 장미

kit-emoji-sunflower = 해바라기

kit-emoji-tulip = 튤립

kit-emoji-evergreen = 상록수

kit-emoji-clover = 클로버

kit-emoji-maple-leaf = 단풍잎

kit-emoji-cactus = 선인장

kit-emoji-moon = 달

kit-emoji-rainbow = 무지개

kit-emoji-cloud = 구름

kit-emoji-snowflake = 눈송이

kit-emoji-lightning = 번개

kit-emoji-wave = 파도

kit-emoji-earth = 지구

kit-emoji-mountain = 산

kit-emoji-palm-tree = 야자수

kit-emoji-apple = 사과

kit-emoji-bread = 빵

kit-emoji-cake = 케이크

kit-emoji-tea = 차

kit-emoji-banana = 바나나

kit-emoji-orange = 오렌지

kit-emoji-strawberry = 딸기

kit-emoji-grapes = 포도

kit-emoji-watermelon = 수박

kit-emoji-lemon = 레몬

kit-emoji-carrot = 당근

kit-emoji-corn = 옥수수

kit-emoji-pizza = 피자

kit-emoji-burger = 햄버거

kit-emoji-chips = 감자튀김

kit-emoji-pasta = 파스타

kit-emoji-fried-egg = 계란프라이

kit-emoji-cheese = 치즈

kit-emoji-biscuit = 비스킷

kit-emoji-chocolate = 초콜릿

kit-emoji-ice-cream = 아이스크림

kit-emoji-beer = 맥주

kit-emoji-wine = 와인

kit-emoji-milk = 우유

kit-emoji-car = 자동차

kit-emoji-bus = 버스

kit-emoji-bicycle = 자전거

kit-emoji-train = 기차

kit-emoji-aeroplane = 비행기

kit-emoji-rocket = 로켓

kit-emoji-ship = 배

kit-emoji-sailing-boat = 요트

kit-emoji-helicopter = 헬리콥터

kit-emoji-house = 집

kit-emoji-office = 사무실

kit-emoji-hospital = 병원

kit-emoji-school = 학교

kit-emoji-church = 교회

kit-emoji-beach = 해변

kit-emoji-tower = 타워

kit-emoji-bridge = 다리

kit-emoji-petrol-pump = 주유기

kit-emoji-traffic-light = 신호등

kit-emoji-map = 지도

kit-emoji-phone = 전화

kit-emoji-laptop = 노트북

kit-emoji-keyboard = 키보드

kit-emoji-camera = 카메라

kit-emoji-light-bulb = 전구

kit-emoji-key = 열쇠

kit-emoji-lock = 자물쇠

kit-emoji-books = 책

kit-emoji-memo = 메모

kit-emoji-pencil = 연필

kit-emoji-paperclip = 클립

kit-emoji-calendar = 달력

kit-emoji-email = 이메일

kit-emoji-gift = 선물

kit-emoji-music-note = 음표

kit-emoji-headphones = 헤드폰

kit-emoji-alarm-clock = 알람 시계

kit-emoji-bell = 종

kit-emoji-spanner = 스패너

kit-emoji-money-bag = 돈주머니

kit-emoji-shopping-trolley = 쇼핑 카트

kit-emoji-football = 축구공

kit-emoji-heart = 하트

kit-emoji-star = 별

kit-emoji-done = 완료

kit-emoji-question = 물음표

kit-emoji-party = 파티

kit-emoji-fire = 불

kit-emoji-cross = 엑스

kit-emoji-exclamation = 느낌표

kit-emoji-warning = 경고

kit-emoji-hundred = 백 점

kit-emoji-broken-heart = 깨진 하트

kit-emoji-blue-heart = 파란 하트

kit-emoji-green-heart = 초록 하트

kit-emoji-sparkles = 반짝임

kit-emoji-plus = 더하기

kit-emoji-minus = 빼기

kit-emoji-right-arrow = 오른쪽 화살표

kit-emoji-left-arrow = 왼쪽 화살표

kit-emoji-refresh = 새로 고침

kit-emoji-forbidden = 금지

kit-emoji-recycling = 재활용

kit-emoji-sleep = 수면

# Said in a window of Trindle's own when something tried to press a button that only the person may press.
kit-only-your-own = 여기서는 직접 누른 것만 작동합니다.

# Said in a window when keys are pressed but the keyboard layout could not be read.
kit-no-layout = 키보드 배열을 읽지 못해서 입력할 수 없습니다.

# What a screen reader is told of a scene of items: how many, and the one chosen.
kit-scene =
    { $count ->
       *[other] { $count }개 항목이 있는 장면
    }

kit-scene-chosen =
    { $count ->
       *[other] { $count }개 항목이 있는 장면, { $name } 선택됨
    }

kit-player-play = 재생

kit-player-pause = 일시 정지

kit-player-back = 5초 뒤로

kit-player-forward = 5초 앞으로

kit-player-seek = 재생 위치

kit-player-mute = 음소거

kit-player-unmute = 소리 켜기

kit-player-volume = 음량

# How far in, out of how long, as clock times.
kit-player-time = { $length } 중 { $at }

kit-crash-title = 문제가 발생했습니다

kit-crash-lead = { $program }이(가) 문제로 인해 멈췄습니다. 아래 보고서를 복사해서 버그 신고와 함께 보내 주세요. 문제를 고치는 데 도움이 됩니다.

kit-crash-copy = 보고서 복사

# What a screen reader calls a block of code on a help page.
kit-help-code = 코드

# Under a text box with suggestions: how many there are, said by a screen reader as they change.
kit-complete-count =
    { $n ->
       *[other] 추천 { $n }개
    }

kit-frame-minimise = 최소화

kit-frame-maximise = 최대화

kit-frame-restore = 원래 크기로

kit-frame-close = 닫기

kit-frame-move = 이동

kit-frame-resize = 크기 조절

kit-theme-not-step = --{ $name }은(는) 다음 단계 중 하나만 받습니다: { $steps }. 그 밖의 값은 건너뜁니다. { $guide }을(를) 참고하세요.

kit-numerals-title = 숫자

kit-numerals-help = 숫자, 날짜, 시간을 쓰는 방식입니다.

kit-numerals-auto = 지역에서 보통 쓰는 방식

kit-numerals-latin = 항상 0~9

kit-numerals-native = 언어 고유의 숫자
kit-date-week-short = 주
kit-date-week = { $week }주차
