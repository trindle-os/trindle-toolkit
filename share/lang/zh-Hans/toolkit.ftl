# machine translated (Simplified Chinese), not yet reviewed by a native speaker.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = 复制

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = 全选

# Right-click menu on a typing box that has words in it.
kit-cut = 剪切

# Right-click menu on a typing box.
kit-paste = 粘贴

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
kit-show-password = 显示密码

# Name of the eye beside a PIN box while the PIN is hidden.
kit-show-pin = 显示 PIN 码

# Name of the eye beside a key box while the key is hidden.
kit-show-key = 显示密钥

# Name of the eye beside a password box while the password shows.
kit-hide-password = 隐藏密码

# Name of the eye beside a PIN box while the PIN shows.
kit-hide-pin = 隐藏 PIN 码

# Name of the eye beside a key box while the key shows.
kit-hide-key = 隐藏密钥

# Said by the screen reader after the eye beside a password box is pressed to show it.
kit-password-shown = 密码已显示

# Said by the screen reader after the eye beside a password box is pressed to hide it.
kit-password-hidden = 密码已隐藏

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
kit-pin-shown = PIN 码已显示

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
kit-pin-hidden = PIN 码已隐藏

# Said by the screen reader after the eye beside a key box is pressed to show it.
kit-key-shown = 密钥已显示

# Said by the screen reader after the eye beside a key box is pressed to hide it.
kit-key-hidden = 密钥已隐藏

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = 请选择一项

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = 搜索

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = 没有匹配项

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }，第 { $n } 项，共 { $total } 项

# Read out by the screen reader for a spinner that is turning.
kit-loading = 正在加载

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
# SAFETY: needs a human review before it ships
kit-click-again = 再点一次即可{ $action }

# The same, when the button has no name to repeat.
kit-click-again-bare = 再点一次

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
# SAFETY: needs a human review before it ships
kit-click-again-small = no

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + 更多

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
       *[other] +{ $n } 行
    }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = Trindle 无法处理拖入的项目。

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = 拖入的内容太大，无法接收。

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = 拖入的内容送达太慢，所以 Trindle 不再等待。

# Said when too many things were dropped at once.
kit-dropped-too-many = 一次拖入的项目太多。请每次少拖一些。

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = 拖入的项目不是这台电脑上的文件。

# Label of a link button whose address is not a web address.
kit-link-not-web = 不是网址

# Label of a link button whose address opens a mail program.
kit-link-mailto = 邮件地址

# Label of a link button whose address opens a file.
kit-link-file = 这台电脑上的文件

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = 指向 { $label } 的链接

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = 选择器窗口没有响应。请再试一次。

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = 第 { $line } 行：{ $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = 这个主题文件太大。最大为 64 KB。

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = 这个文件不是样式表。它包含的不只是纯文本。

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = 主题不能导入其他文件或网址。

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = --{ $name } 的值太长。

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = Trindle 无法将 --{ $name } 识别为颜色。

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = Trindle 无法将 --{ $name } 识别为数字。

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = Trindle 无法将 --{ $name } 识别为阴影。

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
# SAFETY: needs a human review before it ships
kit-theme-nothing-known = Trindle 不认识这个文件中的任何设置，所以不会改变任何内容。

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = 进度

# The name a screen reader gives a menu that opens over a window.
kit-menu = 菜单


kit-job-wait = 请等待任务完成。


kit-search-of = 第 { $n } 项，共 { $total } 项

kit-divider = 分隔线

kit-row-sub =
    { $n ->
       *[other] { $words }，子菜单，{ $n } 行
    }

kit-row-keys = { $words }，{ $keys }

kit-drop-target = 放置区域

kit-theme-not-step = --{ $name } 只能是以下之一：{ $steps }。Trindle 会忽略其他任何值。请参阅 { $guide }。

kit-sheet-not-watched = 请重启以查看对 { $folder } 中样式表的更改。{ $why }

kit-menu-bar = 菜单栏

kit-cancel = 取消

kit-toolbar = 工具

kit-more-tools = 更多工具

kit-status = 状态

kit-path = 路径

kit-path-more = 更多位置

kit-range-low = 下限

kit-range-high = 上限

kit-spin-less = 减小{ $what }

kit-spin-more = 增大{ $what }

kit-spin-not-number = 请输入 { $min } 到 { $max } 之间的数字

kit-tab-close = 关闭

kit-tab-more = 更多标签页

kit-area-cut = 剪切

kit-area-copy = 复制

kit-area-paste = 粘贴

kit-area-select-all = 全选

kit-colour-square = 颜色、饱和度和亮度

kit-colour-swatch = 所选颜色 { $colour }

kit-colour-hue = 色相

kit-colour-hex = 十六进制代码

kit-look-title = 外观

kit-look-lead = 选择这个程序的外观，不受桌面其他部分影响。

kit-look-group = 外观

kit-look-shade = 浅色或深色

kit-look-shade-help = 跟随桌面，或始终使用浅色或深色。

kit-look-desktop = 与桌面相同

kit-look-dark = 深色

kit-look-light = 浅色

kit-look-highlight = 强调色

kit-look-highlight-help = 主要按钮和所选项目的颜色。

kit-look-highlight-desktop = 与桌面相同

kit-date-previous = 上一个

kit-date-next = 下一个

kit-date-month-year = { $year }年{ $month }

kit-date-full = { $year }年{ $month }{ $day }日 { $weekday }

kit-time-hours = 小时

kit-time-minutes = 分钟

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
# A month as it reads inside a whole date, such as "12 November 2026". Some languages change the ending there, so it has its own key.
kit-month-in-date-1 = 1月
kit-month-in-date-2 = 2月
kit-month-in-date-3 = 3月
kit-month-in-date-4 = 4月
kit-month-in-date-5 = 5月
kit-month-in-date-6 = 6月
kit-month-in-date-7 = 7月
kit-month-in-date-8 = 8月
kit-month-in-date-9 = 9月
kit-month-in-date-10 = 10月
kit-month-in-date-11 = 11月
kit-month-in-date-12 = 12月

kit-weekday-1 = 周一

kit-weekday-2 = 周二

kit-weekday-3 = 周三

kit-weekday-4 = 周四

kit-weekday-5 = 周五

kit-weekday-6 = 周六

kit-weekday-7 = 周日

kit-weekday-full-1 = 星期一

kit-weekday-full-2 = 星期二

kit-weekday-full-3 = 星期三

kit-weekday-full-4 = 星期四

kit-weekday-full-5 = 星期五

kit-weekday-full-6 = 星期六

kit-weekday-full-7 = 星期日

kit-key-ctrl = Ctrl

kit-key-shift = Shift

kit-key-alt = Alt

kit-key-logo = Super

kit-key-left = 左

kit-key-right = 右

kit-key-up = 上

kit-key-down = 下

kit-key-page-up = Page Up

kit-key-page-down = Page Down

kit-key-home = Home

kit-key-end = End

kit-key-enter = Enter

kit-key-escape = Esc

kit-key-tab = Tab

kit-key-space = 空格

kit-key-backspace = Backspace

kit-key-delete = Delete

kit-key-insert = Insert

kit-shortcuts = 键盘快捷键

kit-shortcut-kept-window = 此键保留用于在控件之间移动和关闭窗口。

kit-shortcut-kept-typing = 输入时，此键会先交给文本框。

kit-emoji = 表情符号

kit-emoji-search = 按名称查找表情符号

kit-emoji-none = 没有这个名称的表情符号。

kit-emoji-faces = 脸部

kit-emoji-hands = 手势

kit-emoji-animals = 动物

kit-emoji-nature = 自然

kit-emoji-food = 食物和饮料

kit-emoji-travel = 旅行和地点

kit-emoji-objects = 物品

kit-emoji-symbols = 符号

kit-emoji-grin = 露齿笑

kit-emoji-tears-of-joy = 笑出眼泪

kit-emoji-smile = 微笑

kit-emoji-wink = 眨眼

kit-emoji-heart-eyes = 爱心眼

kit-emoji-thinking = 思考

kit-emoji-crying = 哭泣

kit-emoji-surprised = 惊讶

kit-emoji-blush = 脸红

kit-emoji-halo = 天使光环

kit-emoji-cool = 酷

kit-emoji-sleeping = 睡着

kit-emoji-sobbing = 大哭

kit-emoji-angry = 生气

kit-emoji-scream = 尖叫

kit-emoji-eye-roll = 翻白眼

kit-emoji-relieved = 如释重负

kit-emoji-rolling-laugh = 笑得打滚

kit-emoji-blowing-a-kiss = 飞吻

kit-emoji-straight-face = 面无表情

kit-emoji-flushed = 脸涨红

kit-emoji-hug = 拥抱

kit-emoji-unwell = 不舒服

kit-emoji-celebrating = 庆祝

kit-emoji-thumbs-up = 点赞

kit-emoji-thumbs-down = 点踩

kit-emoji-clapping = 鼓掌

kit-emoji-thank-you = 谢谢

kit-emoji-waving = 挥手

kit-emoji-victory = 胜利

kit-emoji-ok = OK

kit-emoji-crossed-fingers = 交叉手指

kit-emoji-strong = 强壮

kit-emoji-pointing-up = 指向上

kit-emoji-pointing-down = 指向下

kit-emoji-pointing-left = 指向左

kit-emoji-pointing-right = 指向右

kit-emoji-raised-hand = 举手

kit-emoji-handshake = 握手

kit-emoji-fist = 拳头

kit-emoji-raised-hands = 举起双手

kit-emoji-call-me = 打电话给我

kit-emoji-writing = 书写

kit-emoji-shrug = 耸肩

kit-emoji-dog = 狗

kit-emoji-cat = 猫

kit-emoji-mouse = 老鼠

kit-emoji-rabbit = 兔子

kit-emoji-fox = 狐狸

kit-emoji-bear = 熊

kit-emoji-panda = 熊猫

kit-emoji-tiger = 老虎

kit-emoji-lion = 狮子

kit-emoji-cow = 奶牛

kit-emoji-pig = 猪

kit-emoji-frog = 青蛙

kit-emoji-monkey = 猴子

kit-emoji-chicken = 鸡

kit-emoji-penguin = 企鹅

kit-emoji-bird = 鸟

kit-emoji-duck = 鸭子

kit-emoji-owl = 猫头鹰

kit-emoji-horse = 马

kit-emoji-bee = 蜜蜂

kit-emoji-butterfly = 蝴蝶

kit-emoji-turtle = 乌龟

kit-emoji-fish = 鱼

kit-emoji-dolphin = 海豚

kit-emoji-blossom = 花朵

kit-emoji-tree = 树

kit-emoji-sun = 太阳

kit-emoji-rain = 雨

kit-emoji-rose = 玫瑰

kit-emoji-sunflower = 向日葵

kit-emoji-tulip = 郁金香

kit-emoji-evergreen = 常青树

kit-emoji-clover = 三叶草

kit-emoji-maple-leaf = 枫叶

kit-emoji-cactus = 仙人掌

kit-emoji-moon = 月亮

kit-emoji-rainbow = 彩虹

kit-emoji-cloud = 云

kit-emoji-snowflake = 雪花

kit-emoji-lightning = 闪电

kit-emoji-wave = 海浪

kit-emoji-earth = 地球

kit-emoji-mountain = 山

kit-emoji-palm-tree = 棕榈树

kit-emoji-apple = 苹果

kit-emoji-bread = 面包

kit-emoji-cake = 蛋糕

kit-emoji-tea = 茶

kit-emoji-banana = 香蕉

kit-emoji-orange = 橙子

kit-emoji-strawberry = 草莓

kit-emoji-grapes = 葡萄

kit-emoji-watermelon = 西瓜

kit-emoji-lemon = 柠檬

kit-emoji-carrot = 胡萝卜

kit-emoji-corn = 玉米

kit-emoji-pizza = 披萨

kit-emoji-burger = 汉堡

kit-emoji-chips = 薯条

kit-emoji-pasta = 意大利面

kit-emoji-fried-egg = 煎蛋

kit-emoji-cheese = 奶酪

kit-emoji-biscuit = 饼干

kit-emoji-chocolate = 巧克力

kit-emoji-ice-cream = 冰淇淋

kit-emoji-beer = 啤酒

kit-emoji-wine = 葡萄酒

kit-emoji-milk = 牛奶

kit-emoji-car = 汽车

kit-emoji-bus = 公共汽车

kit-emoji-bicycle = 自行车

kit-emoji-train = 火车

kit-emoji-aeroplane = 飞机

kit-emoji-rocket = 火箭

kit-emoji-ship = 轮船

kit-emoji-sailing-boat = 帆船

kit-emoji-helicopter = 直升机

kit-emoji-house = 房子

kit-emoji-office = 办公楼

kit-emoji-hospital = 医院

kit-emoji-school = 学校

kit-emoji-church = 教堂

kit-emoji-beach = 海滩

kit-emoji-tower = 塔

kit-emoji-bridge = 桥

kit-emoji-petrol-pump = 加油泵

kit-emoji-traffic-light = 红绿灯

kit-emoji-map = 地图

kit-emoji-phone = 手机

kit-emoji-laptop = 笔记本电脑

kit-emoji-keyboard = 键盘

kit-emoji-camera = 相机

kit-emoji-light-bulb = 灯泡

kit-emoji-key = 钥匙

kit-emoji-lock = 锁

kit-emoji-books = 书籍

kit-emoji-memo = 备忘录

kit-emoji-pencil = 铅笔

kit-emoji-paperclip = 回形针

kit-emoji-calendar = 日历

kit-emoji-email = 电子邮件

kit-emoji-gift = 礼物

kit-emoji-music-note = 音符

kit-emoji-headphones = 耳机

kit-emoji-alarm-clock = 闹钟

kit-emoji-bell = 铃铛

kit-emoji-spanner = 扳手

kit-emoji-money-bag = 钱袋

kit-emoji-shopping-trolley = 购物车

kit-emoji-football = 足球

kit-emoji-heart = 爱心

kit-emoji-star = 星星

kit-emoji-done = 完成

kit-emoji-question = 问号

kit-emoji-party = 派对

kit-emoji-fire = 火

kit-emoji-cross = 叉号

kit-emoji-exclamation = 感叹号

kit-emoji-warning = 警告

kit-emoji-hundred = 一百分

kit-emoji-broken-heart = 心碎

kit-emoji-blue-heart = 蓝心

kit-emoji-green-heart = 绿心

kit-emoji-sparkles = 闪光

kit-emoji-plus = 加号

kit-emoji-minus = 减号

kit-emoji-right-arrow = 右箭头

kit-emoji-left-arrow = 左箭头

kit-emoji-refresh = 刷新

kit-emoji-forbidden = 禁止

kit-emoji-recycling = 回收

kit-emoji-sleep = 睡眠

kit-only-your-own = 只有本人可以按此按钮。

kit-no-layout = Trindle 无法读取键盘布局，所以无法输入。

kit-scene =
    { $count ->
       *[other] 包含 { $count } 个项目的场景
    }

kit-scene-chosen =
    { $count ->
       *[other] 包含 { $count } 个项目的场景，已选择 { $name }
    }

kit-player-play = 播放

kit-player-pause = 暂停

kit-player-back = 后退 5 秒

kit-player-forward = 快进 5 秒

kit-player-seek = 位置

kit-player-mute = 静音

kit-player-unmute = 取消静音

kit-player-volume = 音量

kit-player-time = { $at } / { $length }

kit-crash-title = 出了点问题

kit-crash-lead = { $program } 因出现问题而停止。请复制下面的报告，并随错误报告一起发送，以便解决这个问题。

kit-crash-copy = 复制报告

kit-help-code = 代码

kit-complete-count =
    { $n ->
       *[other] { $n } 条建议
    }

kit-frame-minimise = 最小化

kit-frame-maximise = 最大化

kit-frame-restore = 还原

kit-frame-close = 关闭

kit-frame-move = 移动

kit-frame-resize = 调整大小

kit-numerals-title = 数字样式

kit-numerals-help = 数字、日期和时间的写法。

kit-numerals-auto = 按所在地区的通用写法

kit-numerals-latin = 始终使用 0 到 9

kit-numerals-native = 该语言自己的数字
kit-date-week-short = 周
kit-date-week = 第 { $week } 周
