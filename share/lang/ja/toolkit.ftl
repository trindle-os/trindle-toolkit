# machine translated (Japanese), not yet reviewed by a native speaker.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = コピー

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = すべて選択

# Right-click menu on a typing box that has words in it.
kit-cut = 切り取り

# Right-click menu on a typing box.
kit-paste = 貼り付け

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
kit-show-password = パスワードを表示

# Name of the eye beside a PIN box while the PIN is hidden.
kit-show-pin = PIN を表示

# Name of the eye beside a key box while the key is hidden.
kit-show-key = キーを表示

# Name of the eye beside a password box while the password shows.
kit-hide-password = パスワードを隠す

# Name of the eye beside a PIN box while the PIN shows.
kit-hide-pin = PIN を隠す

# Name of the eye beside a key box while the key shows.
kit-hide-key = キーを隠す

# Said by the screen reader after the eye beside a password box is pressed to show it.
kit-password-shown = パスワードを表示しました

# Said by the screen reader after the eye beside a password box is pressed to hide it.
kit-password-hidden = パスワードを隠しました

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
kit-pin-shown = PIN を表示しました

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
kit-pin-hidden = PIN を隠しました

# Said by the screen reader after the eye beside a key box is pressed to show it.
kit-key-shown = キーを表示しました

# Said by the screen reader after the eye beside a key box is pressed to hide it.
kit-key-hidden = キーを隠しました

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = 選択してください

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = 検索

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = 一致するものがありません

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }、{ $total } 件中 { $n } 件

# Shown at the end of a dropdown's search box: how many rows match out of how many there are.
kit-search-of = { $total } 件中 { $n } 件

# Read out by the screen reader for a spinner that is turning.
kit-loading = 読み込み中

# Read out by the screen reader for the line between two areas of a window that can be dragged to make one bigger.
kit-divider = 区切り線

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
# SAFETY: needs a human review before it ships
kit-click-again = もう一度クリック：{ $action }

# The same, when the button has no name to repeat.
kit-click-again-bare = もう一度クリックしてください

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
# SAFETY: needs a human review before it ships
kit-click-again-small = no

kit-job-wait = 処理が終わるまでお待ちください。

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + ほか

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
       *[other] +{ $n } 行
    }

# A menu row that opens a sub-menu. $words is the row's own words, $n how many lines the sub-menu holds.
kit-row-sub =
    { $n ->
       *[other] { $words }、サブメニュー、{ $n } 項目
    }

# A menu row with a shortcut. $words is the row's own words, $keys the shortcut.
kit-row-keys = { $words }、{ $keys }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = ドロップしたものからは何も得られませんでした。

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = ドロップしたものが大きすぎるため、この方法では受け取れません。

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = ドロップしたものの到着に時間がかかりすぎたため、破棄しました。

# Said when too many things were dropped at once.
kit-dropped-too-many = 一度にドロップしたものが多すぎます。数を減らしてもう一度お試しください。

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = ドロップしたものは、このコンピューター上のファイルの一覧ではありません。

# Read out by the screen reader for the control under something dragged, when letting go there would drop it in.
kit-drop-target = ドロップ先

# Label of a link button whose address is not a web address.
kit-link-not-web = ウェブアドレスではありません

# Label of a link button whose address opens a mail program.
kit-link-mailto = メールアドレスを開きます

# Label of a link button whose address opens a file.
kit-link-file = ファイルを開きます

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = { $label } へのリンク

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = 選択ウィンドウから応答がありませんでした。もう一度お試しください。

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = { $line } 行目：{ $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = ファイルが大きすぎてテーマにできません。テーマは最大 64 KB です。

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = このファイルはスタイルシートではありません。文字以外のものが含まれています。

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = テーマから別のファイルやウェブアドレスを読み込むことはできません。

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = --{ $name } が長すぎて値にできません。

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = --{ $name } は Trindle が読み取れる色ではありません。

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = --{ $name } は Trindle が読み取れる数ではありません。

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = --{ $name } は Trindle が読み取れる影ではありません。

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = ファイルに Trindle が認識できる値がないため、何も変わりません。

# Standard error, when the folder of the person's style sheet cannot be watched. $folder is the folder, $why the system's reason.
kit-sheet-not-watched = { $folder } にあるスタイルシートの変更は、再起動後に反映されます：{ $why }

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = 進行状況

# The name a screen reader gives a menu that opens over a window.
kit-menu = メニュー

# What a screen reader calls the row of menu titles across the top of a window.
kit-menu-bar = メニューバー

# The answer at the foot of every dialog page that leaves it with nothing done.
kit-cancel = キャンセル

# What a screen reader calls a row of tool buttons.
kit-toolbar = ツール

# The button at the end of a toolbar that opens a menu of the tools that did not fit.
kit-more-tools = その他のツール

# What a screen reader calls the line at a window's foot that says how things stand.
kit-status = 状態

# What a screen reader calls a place's path, such as a folder's.
kit-path = パス

# The list in the middle of a long path, holding the places not shown.
kit-path-more = その他の場所

# Said after a range slider's name for its low handle.
kit-range-low = 下限

# Said after a range slider's name for its high handle.
kit-range-high = 上限

# The button that takes a step off a number box's number. $what is the box's name.
kit-spin-less = { $what }を減らす

# The button that adds a step to a number box's number. $what is the box's name.
kit-spin-more = { $what }を増やす

# Red words under a number box whose typing is not a number.
kit-spin-not-number = { $min } から { $max } までの数を入力してください

# Said by a screen reader for the close box in a tab.
kit-tab-close = 閉じる

# Said by a screen reader for the button that lists the tabs that did not fit.
kit-tab-more = その他のタブ

# Right-click menu of a text area, one row each.
kit-area-cut = 切り取り

kit-area-copy = コピー

kit-area-paste = 貼り付け

kit-area-select-all = すべて選択

# The name a screen reader gives the painted square of a colour picker.
kit-colour-square = 色、彩度、明るさ

# The name a screen reader gives the swatch showing the colour chosen. $colour is its hex code.
kit-colour-swatch = 選択中の色 { $colour }

kit-colour-hue = 色相

kit-colour-hex = 16 進コード

# The ready settings page for a program's own look (settings::look_page).
kit-look-title = 外観

kit-look-lead = デスクトップの設定に関係なく、このプログラムの外観を決めます。

kit-look-group = 外観

kit-look-shade = ライトまたはダーク

kit-look-shade-help = デスクトップに合わせるか、常にどちらかに固定します。

kit-look-desktop = デスクトップに合わせる

kit-look-dark = ダーク

kit-look-light = ライト

kit-look-highlight = 強調色

kit-look-highlight-help = 主な操作と選択中の項目の色です。

# A button that gives the highlight back to the desktop's choice.
kit-look-highlight-desktop = デスクトップの色を使う

# Buttons either side of the month in a date picker.
kit-date-previous = 前へ

kit-date-next = 次へ

# The month shown in a date picker. $month is the month's name, $year the year.
kit-date-month-year = { $year }年{ $month }

# What a screen reader says for a day in a date picker.
kit-date-full = { $year }年{ $month }{ $day }日 { $weekday }

# The names of the two boxes in a time entry.
kit-time-hours = 時

kit-time-minutes = 分

# Months and weekdays; weekdays are short in the grid and full when read out.
kit-month-1 = 1月

kit-month-2 = 2月

kit-month-3 = 3月

kit-month-4 = 4月

kit-month-5 = 5月

kit-month-6 = 6月

kit-month-7 = 7月

kit-month-8 = 8月

kit-month-9 = 9月

kit-month-10 = 10月

kit-month-11 = 11月

kit-month-12 = 12月

kit-weekday-1 = 月

kit-weekday-2 = 火

kit-weekday-3 = 水

kit-weekday-4 = 木

kit-weekday-5 = 金

kit-weekday-6 = 土

kit-weekday-7 = 日

kit-weekday-full-1 = 月曜日

kit-weekday-full-2 = 火曜日

kit-weekday-full-3 = 水曜日

kit-weekday-full-4 = 木曜日

kit-weekday-full-5 = 金曜日

kit-weekday-full-6 = 土曜日

kit-weekday-full-7 = 日曜日

# The names of the modifier keys in a shortcut such as Ctrl+Shift+C.
kit-key-ctrl = Ctrl

kit-key-shift = Shift

kit-key-alt = Alt

kit-key-logo = ロゴ

# The names of keys that are words, in a shortcut such as Ctrl+Page down.
kit-key-left = 左

kit-key-right = 右

kit-key-up = 上

kit-key-down = 下

kit-key-page-up = ページアップ

kit-key-page-down = ページダウン

kit-key-home = Home

kit-key-end = End

kit-key-enter = Enter

kit-key-escape = Esc

kit-key-tab = Tab

kit-key-space = スペース

kit-key-backspace = Backspace

kit-key-delete = Delete

kit-key-insert = Insert

# The heading of the page that lists a program's keyboard shortcuts.
kit-shortcuts = キーボードショートカット

# Under a shortcut on that page whose keys the window always takes first.
kit-shortcut-kept-window = このキーは、パーツ間の移動とウィンドウを閉じる操作のためにウィンドウが先に使います。

# Under a shortcut on that page whose keys a text box takes first while you type.
kit-shortcut-kept-typing = 入力中は、このキーをテキストボックスが先に使います。

# The heading of the page of emoji to pick from.
kit-emoji = 絵文字

# The name of the box on that page where an emoji's name is typed to find it.
kit-emoji-search = 名前で絵文字を探す

# Shown on that page when no emoji's name has what was typed.
kit-emoji-none = その名前の絵文字はありません。

# The groups of emoji on that page, in order. Another language may leave out any
# name below; the page then shows the English one, so a name is never missing.
kit-emoji-faces = 顔

kit-emoji-hands = 手

kit-emoji-animals = 動物

kit-emoji-nature = 自然

kit-emoji-food = 食べ物と飲み物

kit-emoji-travel = 乗り物と場所

kit-emoji-objects = 物

kit-emoji-symbols = 記号

# Each emoji's name, spoken by a screen reader and found by typing.
kit-emoji-grin = にっこり

kit-emoji-tears-of-joy = うれし泣き

kit-emoji-smile = 笑顔

kit-emoji-wink = ウインク

kit-emoji-heart-eyes = 目がハート

kit-emoji-thinking = 考え中

kit-emoji-crying = 泣き顔

kit-emoji-surprised = 驚き

kit-emoji-blush = 照れ笑い

kit-emoji-halo = 天使

kit-emoji-cool = サングラス

kit-emoji-sleeping = 寝顔

kit-emoji-sobbing = 号泣

kit-emoji-angry = 怒り

kit-emoji-scream = 絶叫

kit-emoji-eye-roll = 白目

kit-emoji-relieved = ほっとした顔

kit-emoji-rolling-laugh = 転げ回って笑う

kit-emoji-blowing-a-kiss = 投げキッス

kit-emoji-straight-face = 真顔

kit-emoji-flushed = 赤面

kit-emoji-hug = ハグ

kit-emoji-unwell = 具合が悪い

kit-emoji-celebrating = お祝い

kit-emoji-thumbs-up = いいね

kit-emoji-thumbs-down = よくないね

kit-emoji-clapping = 拍手

kit-emoji-thank-you = ありがとう

kit-emoji-waving = 手を振る

kit-emoji-victory = ピース

kit-emoji-ok = OK

kit-emoji-crossed-fingers = 指をクロス

kit-emoji-strong = 力こぶ

kit-emoji-pointing-up = 上を指す

kit-emoji-pointing-down = 下を指す

kit-emoji-pointing-left = 左を指す

kit-emoji-pointing-right = 右を指す

kit-emoji-raised-hand = 挙手

kit-emoji-handshake = 握手

kit-emoji-fist = こぶし

kit-emoji-raised-hands = 万歳

kit-emoji-call-me = 電話して

kit-emoji-writing = 書く手

kit-emoji-shrug = 肩をすくめる

kit-emoji-dog = 犬

kit-emoji-cat = 猫

kit-emoji-mouse = ネズミ

kit-emoji-rabbit = ウサギ

kit-emoji-fox = キツネ

kit-emoji-bear = クマ

kit-emoji-panda = パンダ

kit-emoji-tiger = トラ

kit-emoji-lion = ライオン

kit-emoji-cow = ウシ

kit-emoji-pig = ブタ

kit-emoji-frog = カエル

kit-emoji-monkey = サル

kit-emoji-chicken = ニワトリ

kit-emoji-penguin = ペンギン

kit-emoji-bird = 鳥

kit-emoji-duck = アヒル

kit-emoji-owl = フクロウ

kit-emoji-horse = ウマ

kit-emoji-bee = ミツバチ

kit-emoji-butterfly = チョウ

kit-emoji-turtle = カメ

kit-emoji-fish = 魚

kit-emoji-dolphin = イルカ

kit-emoji-blossom = 花

kit-emoji-tree = 木

kit-emoji-sun = 太陽

kit-emoji-rain = 雨

kit-emoji-rose = バラ

kit-emoji-sunflower = ヒマワリ

kit-emoji-tulip = チューリップ

kit-emoji-evergreen = 常緑樹

kit-emoji-clover = クローバー

kit-emoji-maple-leaf = もみじ

kit-emoji-cactus = サボテン

kit-emoji-moon = 月

kit-emoji-rainbow = 虹

kit-emoji-cloud = 雲

kit-emoji-snowflake = 雪の結晶

kit-emoji-lightning = 稲妻

kit-emoji-wave = 波

kit-emoji-earth = 地球

kit-emoji-mountain = 山

kit-emoji-palm-tree = ヤシの木

kit-emoji-apple = リンゴ

kit-emoji-bread = パン

kit-emoji-cake = ケーキ

kit-emoji-tea = お茶

kit-emoji-banana = バナナ

kit-emoji-orange = オレンジ

kit-emoji-strawberry = イチゴ

kit-emoji-grapes = ブドウ

kit-emoji-watermelon = スイカ

kit-emoji-lemon = レモン

kit-emoji-carrot = ニンジン

kit-emoji-corn = トウモロコシ

kit-emoji-pizza = ピザ

kit-emoji-burger = ハンバーガー

kit-emoji-chips = フライドポテト

kit-emoji-pasta = パスタ

kit-emoji-fried-egg = 目玉焼き

kit-emoji-cheese = チーズ

kit-emoji-biscuit = ビスケット

kit-emoji-chocolate = チョコレート

kit-emoji-ice-cream = アイスクリーム

kit-emoji-beer = ビール

kit-emoji-wine = ワイン

kit-emoji-milk = 牛乳

kit-emoji-car = 車

kit-emoji-bus = バス

kit-emoji-bicycle = 自転車

kit-emoji-train = 電車

kit-emoji-aeroplane = 飛行機

kit-emoji-rocket = ロケット

kit-emoji-ship = 船

kit-emoji-sailing-boat = ヨット

kit-emoji-helicopter = ヘリコプター

kit-emoji-house = 家

kit-emoji-office = オフィス

kit-emoji-hospital = 病院

kit-emoji-school = 学校

kit-emoji-church = 教会

kit-emoji-beach = ビーチ

kit-emoji-tower = タワー

kit-emoji-bridge = 橋

kit-emoji-petrol-pump = ガソリンスタンド

kit-emoji-traffic-light = 信号機

kit-emoji-map = 地図

kit-emoji-phone = 電話

kit-emoji-laptop = ノートパソコン

kit-emoji-keyboard = キーボード

kit-emoji-camera = カメラ

kit-emoji-light-bulb = 電球

kit-emoji-key = 鍵

kit-emoji-lock = 錠

kit-emoji-books = 本

kit-emoji-memo = メモ

kit-emoji-pencil = 鉛筆

kit-emoji-paperclip = クリップ

kit-emoji-calendar = カレンダー

kit-emoji-email = メール

kit-emoji-gift = プレゼント

kit-emoji-music-note = 音符

kit-emoji-headphones = ヘッドホン

kit-emoji-alarm-clock = 目覚まし時計

kit-emoji-bell = ベル

kit-emoji-spanner = スパナ

kit-emoji-money-bag = お金の袋

kit-emoji-shopping-trolley = ショッピングカート

kit-emoji-football = サッカーボール

kit-emoji-heart = ハート

kit-emoji-star = 星

kit-emoji-done = 完了

kit-emoji-question = 疑問符

kit-emoji-party = パーティー

kit-emoji-fire = 炎

kit-emoji-cross = バツ

kit-emoji-exclamation = 感嘆符

kit-emoji-warning = 警告

kit-emoji-hundred = 100点

kit-emoji-broken-heart = ハートブレイク

kit-emoji-blue-heart = 青いハート

kit-emoji-green-heart = 緑のハート

kit-emoji-sparkles = きらきら

kit-emoji-plus = プラス

kit-emoji-minus = マイナス

kit-emoji-right-arrow = 右矢印

kit-emoji-left-arrow = 左矢印

kit-emoji-refresh = 更新

kit-emoji-forbidden = 禁止

kit-emoji-recycling = リサイクル

kit-emoji-sleep = 睡眠

# Said in a window of Trindle's own when something tried to press a button that only the person may press.
kit-only-your-own = ここでは、あなた自身の操作だけが有効です。

# Said in a window when keys are pressed but the keyboard layout could not be read.
kit-no-layout = キーボード配列を読み取れなかったため、入力できません。

# What a screen reader is told of a scene of items: how many, and the one chosen.
kit-scene =
    { $count ->
       *[other] { $count } 項目のシーン
    }

kit-scene-chosen =
    { $count ->
       *[other] { $count } 項目のシーン、{ $name } を選択中
    }

kit-player-play = 再生

kit-player-pause = 一時停止

kit-player-back = 5 秒戻る

kit-player-forward = 5 秒進む

kit-player-seek = 再生位置

kit-player-mute = ミュート

kit-player-unmute = 音を出す

kit-player-volume = 音量

# How far in, out of how long, as clock times.
kit-player-time = { $length } 中 { $at }

kit-crash-title = 問題が発生しました

kit-crash-lead = { $program } は問題のため停止しました。下のレポートをコピーして、不具合の報告と一緒に送ってください。修正に役立ちます。

kit-crash-copy = レポートをコピー

# What a screen reader calls a block of code on a help page.
kit-help-code = コード

# Under a text box with suggestions: how many there are, said by a screen reader as they change.
kit-complete-count =
    { $n ->
       *[other] 候補 { $n } 件
    }

kit-frame-minimise = 最小化

kit-frame-maximise = 最大化

kit-frame-restore = 元に戻す

kit-frame-close = 閉じる

kit-frame-move = 移動

kit-frame-resize = サイズ変更

kit-theme-not-step = --{ $name } に使えるステップは次のいずれかだけです: { $steps }。それ以外は無視されます。{ $guide } を参照してください。

kit-numerals-title = 数字

kit-numerals-help = 数字、日付、時刻の書き方です。

kit-numerals-auto = 地域の通常の書き方に従う

kit-numerals-latin = 常に 0 から 9

kit-numerals-native = その言語固有の数字
kit-date-week-short = 週
kit-date-week = 第{ $week }週
