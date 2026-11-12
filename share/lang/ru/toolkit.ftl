# machine translated (Russian), not yet reviewed by a native speaker.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = Копировать

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = Выделить всё

# Right-click menu on a typing box that has words in it.
kit-cut = Вырезать

# Right-click menu on a typing box.
kit-paste = Вставить

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
# SAFETY: needs a human review before it ships
kit-show-password = Показать пароль

# Name of the eye beside a PIN box while the PIN is hidden.
# SAFETY: needs a human review before it ships
kit-show-pin = Показать PIN-код

# Name of the eye beside a key box while the key is hidden.
# SAFETY: needs a human review before it ships
kit-show-key = Показать ключ

# Name of the eye beside a password box while the password shows.
kit-hide-password = Скрыть пароль

# Name of the eye beside a PIN box while the PIN shows.
kit-hide-pin = Скрыть PIN-код

# Name of the eye beside a key box while the key shows.
kit-hide-key = Скрыть ключ

# Said by the screen reader after the eye beside a password box is pressed to show it.
kit-password-shown = Пароль показан

# Said by the screen reader after the eye beside a password box is pressed to hide it.
kit-password-hidden = Пароль скрыт

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
kit-pin-shown = PIN-код показан

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
kit-pin-hidden = PIN-код скрыт

# Said by the screen reader after the eye beside a key box is pressed to show it.
kit-key-shown = Ключ показан

# Said by the screen reader after the eye beside a key box is pressed to hide it.
kit-key-hidden = Ключ скрыт

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = Выберите один

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = Поиск

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = Ничего не найдено

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, { $n } из { $total }

# Read out by the screen reader for a spinner that is turning.
kit-loading = Загрузка

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
# SAFETY: needs a human review before it ships
kit-click-again = Нажмите ещё раз, чтобы { $action }

# The same, when the button has no name to repeat.
# SAFETY: needs a human review before it ships
kit-click-again-bare = Нажмите ещё раз

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
# SAFETY: needs a human review before it ships
kit-click-again-small = yes

kit-job-wait = Дождитесь окончания задачи.

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + ещё

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
        [one] +{ $n } строка
        [few] +{ $n } строки
        [many] +{ $n } строк
       *[other] +{ $n } строки
    }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = С перетащенными элементами ничего не удалось сделать.

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = Перетащенные элементы слишком велики, чтобы их принять.

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = Перетащенные элементы поступали слишком долго, поэтому ожидание прекращено.

# Said when too many things were dropped at once.
kit-dropped-too-many = Вы перетащили слишком много элементов сразу. Перетаскивайте меньше за раз.

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = Перетащенные элементы не являются файлами на этом компьютере.

# Label of a link button whose address is not a web address.
kit-link-not-web = Не веб-адрес

# Label of a link button whose address opens a mail program.
kit-link-mailto = Адрес электронной почты

# Label of a link button whose address opens a file.
kit-link-file = Файл на этом компьютере

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = Ссылка на { $label }

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = Окно выбора не ответило. Попробуйте ещё раз.

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = Строка { $line }: { $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = Файл темы слишком велик. Максимальный размер: 64 КБ.

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = Этот файл не является таблицей стилей. В нём не только простой текст.

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = тема не может импортировать другой файл или веб-адрес.

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = Значение --{ $name } слишком длинное.

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = Trindle не может прочитать --{ $name } как цвет.

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = Trindle не может прочитать --{ $name } как число.

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = Trindle не может прочитать --{ $name } как тень.

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = Trindle не распознаёт в этом файле ни одной настройки, поэтому ничего не изменится.

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = Ход выполнения

# The name a screen reader gives a menu that opens over a window.
kit-menu = Меню


kit-search-of = { $n } из { $total }
kit-divider = Разделитель
kit-row-sub =
    { $n ->
        [one] { $words }, подменю, { $n } строка
        [few] { $words }, подменю, { $n } строки
        [many] { $words }, подменю, { $n } строк
       *[other] { $words }, подменю, { $n } строки
    }
kit-row-keys = { $words }, { $keys }
kit-drop-target = Область для перетаскивания
kit-theme-not-step = --{ $name } может принимать только одно из этих значений: { $steps }. Остальные значения Trindle игнорирует. См. { $guide }.
kit-sheet-not-watched = Перезапустите программу, чтобы увидеть изменения таблицы стилей в { $folder }. { $why }
kit-menu-bar = Строка меню
kit-cancel = Отмена
kit-toolbar = Инструменты
kit-more-tools = Другие инструменты
kit-status = Состояние
kit-path = Путь
kit-path-more = Другие места
kit-range-low = нижний предел
kit-range-high = верхний предел
kit-spin-less = Уменьшить: { $what }
kit-spin-more = Увеличить: { $what }
kit-spin-not-number = Введите число от { $min } до { $max }
kit-tab-close = Закрыть
kit-tab-more = Другие вкладки
kit-area-cut = Вырезать
kit-area-copy = Копировать
kit-area-paste = Вставить
kit-area-select-all = Выделить всё
kit-colour-square = Цвет, насыщенность и яркость
kit-colour-swatch = Выбранный цвет: { $colour }
kit-colour-hue = Оттенок
kit-colour-hex = Шестнадцатеричный код
kit-look-title = Внешний вид
kit-look-lead = Выберите, как выглядит эта программа, независимо от остального рабочего стола.
kit-look-group = Внешний вид
kit-look-shade = Светлое или тёмное
kit-look-shade-help = Как на рабочем столе, либо всегда светлая, либо всегда тёмная.
kit-look-desktop = Как на рабочем столе
kit-look-dark = Тёмное
kit-look-light = Светлое
kit-look-highlight = Цвет выделения
kit-look-highlight-help = Цвет главных кнопок и выбранных элементов.
kit-look-highlight-desktop = Как на рабочем столе
kit-date-previous = Назад
kit-date-next = Вперёд
kit-date-month-year = { $month } { $year }
kit-date-full = { $weekday } { $day } { $month } { $year }
kit-time-hours = Часы
kit-time-minutes = Минуты
kit-month-1 = январь
kit-month-2 = февраль
kit-month-3 = март
kit-month-4 = апрель
kit-month-5 = май
kit-month-6 = июнь
kit-month-7 = июль
kit-month-8 = август
kit-month-9 = сентябрь
kit-month-10 = октябрь
kit-month-11 = ноябрь
kit-month-12 = декабрь
# A month as it reads inside a whole date, such as "12 November 2026". Some languages change the ending there, so it has its own key.
kit-month-in-date-1 = января
kit-month-in-date-2 = февраля
kit-month-in-date-3 = марта
kit-month-in-date-4 = апреля
kit-month-in-date-5 = мая
kit-month-in-date-6 = июня
kit-month-in-date-7 = июля
kit-month-in-date-8 = августа
kit-month-in-date-9 = сентября
kit-month-in-date-10 = октября
kit-month-in-date-11 = ноября
kit-month-in-date-12 = декабря
kit-weekday-1 = Пн
kit-weekday-2 = Вт
kit-weekday-3 = Ср
kit-weekday-4 = Чт
kit-weekday-5 = Пт
kit-weekday-6 = Сб
kit-weekday-7 = Вс
kit-weekday-full-1 = Понедельник
kit-weekday-full-2 = Вторник
kit-weekday-full-3 = Среда
kit-weekday-full-4 = Четверг
kit-weekday-full-5 = Пятница
kit-weekday-full-6 = Суббота
kit-weekday-full-7 = Воскресенье
kit-key-ctrl = Ctrl
kit-key-shift = Shift
kit-key-alt = Alt
kit-key-logo = Super
kit-key-left = Влево
kit-key-right = Вправо
kit-key-up = Вверх
kit-key-down = Вниз
kit-key-page-up = Page Up
kit-key-page-down = Page Down
kit-key-home = Home
kit-key-end = End
kit-key-enter = Enter
kit-key-escape = Esc
kit-key-tab = Tab
kit-key-space = Пробел
kit-key-backspace = Backspace
kit-key-delete = Delete
kit-key-insert = Insert
kit-shortcuts = Сочетания клавиш
kit-shortcut-kept-window = Эта клавиша зарезервирована для перехода между элементами и закрытия окна.
kit-shortcut-kept-typing = Пока вы печатаете, эта клавиша сначала достаётся текстовому полю.
kit-emoji = Эмодзи
kit-emoji-search = Найти эмодзи по названию
kit-emoji-none = Эмодзи с таким названием нет.
kit-emoji-faces = Лица
kit-emoji-hands = Руки
kit-emoji-animals = Животные
kit-emoji-nature = Природа
kit-emoji-food = Еда и напитки
kit-emoji-travel = Путешествия и места
kit-emoji-objects = Предметы
kit-emoji-symbols = Символы
kit-emoji-grin = Ухмылка
kit-emoji-tears-of-joy = Слёзы радости
kit-emoji-smile = Улыбка
kit-emoji-wink = Подмигивание
kit-emoji-heart-eyes = Глаза-сердечки
kit-emoji-thinking = Размышление
kit-emoji-crying = Плач
kit-emoji-surprised = Удивление
kit-emoji-blush = Румянец
kit-emoji-halo = Нимб
kit-emoji-cool = Крутой
kit-emoji-sleeping = Сон
kit-emoji-sobbing = Рыдание
kit-emoji-angry = Злость
kit-emoji-scream = Крик
kit-emoji-eye-roll = Закатывание глаз
kit-emoji-relieved = Облегчение
kit-emoji-rolling-laugh = Катаюсь от смеха
kit-emoji-blowing-a-kiss = Воздушный поцелуй
kit-emoji-straight-face = Невозмутимое лицо
kit-emoji-flushed = Смущение
kit-emoji-hug = Объятия
kit-emoji-unwell = Нездоровится
kit-emoji-celebrating = Праздник
kit-emoji-thumbs-up = Большой палец вверх
kit-emoji-thumbs-down = Большой палец вниз
kit-emoji-clapping = Аплодисменты
kit-emoji-thank-you = Спасибо
kit-emoji-waving = Машу рукой
kit-emoji-victory = Победа
kit-emoji-ok = Хорошо
kit-emoji-crossed-fingers = Скрещённые пальцы
kit-emoji-strong = Сильный
kit-emoji-pointing-up = Указывает вверх
kit-emoji-pointing-down = Указывает вниз
kit-emoji-pointing-left = Указывает влево
kit-emoji-pointing-right = Указывает вправо
kit-emoji-raised-hand = Поднятая рука
kit-emoji-handshake = Рукопожатие
kit-emoji-fist = Кулак
kit-emoji-raised-hands = Поднятые руки
kit-emoji-call-me = Позвони мне
kit-emoji-writing = Пишу
kit-emoji-shrug = Пожимание плечами
kit-emoji-dog = Собака
kit-emoji-cat = Кошка
kit-emoji-mouse = Мышь
kit-emoji-rabbit = Кролик
kit-emoji-fox = Лиса
kit-emoji-bear = Медведь
kit-emoji-panda = Панда
kit-emoji-tiger = Тигр
kit-emoji-lion = Лев
kit-emoji-cow = Корова
kit-emoji-pig = Свинья
kit-emoji-frog = Лягушка
kit-emoji-monkey = Обезьяна
kit-emoji-chicken = Курица
kit-emoji-penguin = Пингвин
kit-emoji-bird = Птица
kit-emoji-duck = Утка
kit-emoji-owl = Сова
kit-emoji-horse = Лошадь
kit-emoji-bee = Пчела
kit-emoji-butterfly = Бабочка
kit-emoji-turtle = Черепаха
kit-emoji-fish = Рыба
kit-emoji-dolphin = Дельфин
kit-emoji-blossom = Цветок
kit-emoji-tree = Дерево
kit-emoji-sun = Солнце
kit-emoji-rain = Дождь
kit-emoji-rose = Роза
kit-emoji-sunflower = Подсолнух
kit-emoji-tulip = Тюльпан
kit-emoji-evergreen = Вечнозелёное дерево
kit-emoji-clover = Клевер
kit-emoji-maple-leaf = Кленовый лист
kit-emoji-cactus = Кактус
kit-emoji-moon = Луна
kit-emoji-rainbow = Радуга
kit-emoji-cloud = Облако
kit-emoji-snowflake = Снежинка
kit-emoji-lightning = Молния
kit-emoji-wave = Волна
kit-emoji-earth = Земля
kit-emoji-mountain = Гора
kit-emoji-palm-tree = Пальма
kit-emoji-apple = Яблоко
kit-emoji-bread = Хлеб
kit-emoji-cake = Торт
kit-emoji-tea = Чай
kit-emoji-banana = Банан
kit-emoji-orange = Апельсин
kit-emoji-strawberry = Клубника
kit-emoji-grapes = Виноград
kit-emoji-watermelon = Арбуз
kit-emoji-lemon = Лимон
kit-emoji-carrot = Морковь
kit-emoji-corn = Кукуруза
kit-emoji-pizza = Пицца
kit-emoji-burger = Бургер
kit-emoji-chips = Картофель фри
kit-emoji-pasta = Паста
kit-emoji-fried-egg = Яичница
kit-emoji-cheese = Сыр
kit-emoji-biscuit = Печенье
kit-emoji-chocolate = Шоколад
kit-emoji-ice-cream = Мороженое
kit-emoji-beer = Пиво
kit-emoji-wine = Вино
kit-emoji-milk = Молоко
kit-emoji-car = Автомобиль
kit-emoji-bus = Автобус
kit-emoji-bicycle = Велосипед
kit-emoji-train = Поезд
kit-emoji-aeroplane = Самолёт
kit-emoji-rocket = Ракета
kit-emoji-ship = Корабль
kit-emoji-sailing-boat = Парусник
kit-emoji-helicopter = Вертолёт
kit-emoji-house = Дом
kit-emoji-office = Офис
kit-emoji-hospital = Больница
kit-emoji-school = Школа
kit-emoji-church = Церковь
kit-emoji-beach = Пляж
kit-emoji-tower = Башня
kit-emoji-bridge = Мост
kit-emoji-petrol-pump = Заправка
kit-emoji-traffic-light = Светофор
kit-emoji-map = Карта
kit-emoji-phone = Телефон
kit-emoji-laptop = Ноутбук
kit-emoji-keyboard = Клавиатура
kit-emoji-camera = Фотоаппарат
kit-emoji-light-bulb = Лампочка
kit-emoji-key = Ключ
kit-emoji-lock = Замок
kit-emoji-books = Книги
kit-emoji-memo = Записка
kit-emoji-pencil = Карандаш
kit-emoji-paperclip = Скрепка
kit-emoji-calendar = Календарь
kit-emoji-email = Электронная почта
kit-emoji-gift = Подарок
kit-emoji-music-note = Нота
kit-emoji-headphones = Наушники
kit-emoji-alarm-clock = Будильник
kit-emoji-bell = Колокольчик
kit-emoji-spanner = Гаечный ключ
kit-emoji-money-bag = Мешок денег
kit-emoji-shopping-trolley = Тележка для покупок
kit-emoji-football = Футбол
kit-emoji-heart = Сердце
kit-emoji-star = Звезда
kit-emoji-done = Готово
kit-emoji-question = Вопрос
kit-emoji-party = Вечеринка
kit-emoji-fire = Огонь
kit-emoji-cross = Крест
kit-emoji-exclamation = Восклицание
kit-emoji-warning = Предупреждение
kit-emoji-hundred = Сто
kit-emoji-broken-heart = Разбитое сердце
kit-emoji-blue-heart = Синее сердце
kit-emoji-green-heart = Зелёное сердце
kit-emoji-sparkles = Искры
kit-emoji-plus = Плюс
kit-emoji-minus = Минус
kit-emoji-right-arrow = Стрелка вправо
kit-emoji-left-arrow = Стрелка влево
kit-emoji-refresh = Обновить
kit-emoji-forbidden = Запрещено
kit-emoji-recycling = Переработка
kit-emoji-sleep = Сон
kit-only-your-own = Эту кнопку можете нажать только вы.
kit-no-layout = Trindle не удалось прочитать раскладку клавиатуры, поэтому ввод невозможен.
kit-scene =
    { $count ->
        [one] Сцена из { $count } элемента
        [few] Сцена из { $count } элементов
        [many] Сцена из { $count } элементов
       *[other] Сцена из { $count } элемента
    }
kit-scene-chosen =
    { $count ->
        [one] Сцена из { $count } элемента, выбран «{ $name }»
        [few] Сцена из { $count } элементов, выбран «{ $name }»
        [many] Сцена из { $count } элементов, выбран «{ $name }»
       *[other] Сцена из { $count } элемента, выбран «{ $name }»
    }
kit-player-play = Воспроизвести
kit-player-pause = Пауза
kit-player-back = Назад на 5 секунд
kit-player-forward = Вперёд на 5 секунд
kit-player-seek = Положение
kit-player-mute = Выключить звук
kit-player-unmute = Включить звук
kit-player-volume = Громкость
kit-player-time = { $at } из { $length }
kit-crash-title = Что-то пошло не так
kit-crash-lead = Программа { $program } остановилась, потому что что-то пошло не так. Скопируйте отчёт ниже и отправьте его вместе с сообщением об ошибке, чтобы проблему можно было исправить.
kit-crash-copy = Копировать отчёт
kit-help-code = Код
kit-complete-count =
    { $n ->
        [one] { $n } подсказка
        [few] { $n } подсказки
        [many] { $n } подсказок
       *[other] { $n } подсказки
    }
kit-frame-minimise = Свернуть
kit-frame-maximise = Развернуть
kit-frame-restore = Восстановить
kit-frame-close = Закрыть
kit-frame-move = Переместить
kit-frame-resize = Изменить размер
kit-numerals-title = Вид цифр
kit-numerals-help = Как записываются числа, даты и время.
kit-numerals-auto = Обычный для вашего региона
kit-numerals-latin = Всегда от 0 до 9
kit-numerals-native = Собственные цифры языка
kit-date-week-short = Нед.
kit-date-week = Неделя { $week }
