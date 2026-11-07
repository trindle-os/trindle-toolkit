
# machine translated (Brazilian Portuguese), not yet reviewed by a person.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = Copiar

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = Selecionar tudo

# Right-click menu on a typing box that has words in it.
kit-cut = Recortar

# Right-click menu on a typing box.
kit-paste = Colar

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
# SAFETY: needs a human review before it ships
kit-show-password = Mostrar a senha

# Name of the eye beside a PIN box while the PIN is hidden.
# SAFETY: needs a human review before it ships
kit-show-pin = Mostrar o PIN

# Name of the eye beside a key box while the key is hidden.
# SAFETY: needs a human review before it ships
kit-show-key = Mostrar a chave

# Name of the eye beside a password box while the password shows.
# SAFETY: needs a human review before it ships
kit-hide-password = Ocultar a senha

# Name of the eye beside a PIN box while the PIN shows.
# SAFETY: needs a human review before it ships
kit-hide-pin = Ocultar o PIN

# Name of the eye beside a key box while the key shows.
# SAFETY: needs a human review before it ships
kit-hide-key = Ocultar a chave

# Said by the screen reader after the eye beside a password box is pressed to show it.
# SAFETY: needs a human review before it ships
kit-password-shown = Senha visível

# Said by the screen reader after the eye beside a password box is pressed to hide it.
# SAFETY: needs a human review before it ships
kit-password-hidden = Senha oculta

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
# SAFETY: needs a human review before it ships
kit-pin-shown = PIN visível

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
# SAFETY: needs a human review before it ships
kit-pin-hidden = PIN oculto

# Said by the screen reader after the eye beside a key box is pressed to show it.
# SAFETY: needs a human review before it ships
kit-key-shown = Chave visível

# Said by the screen reader after the eye beside a key box is pressed to hide it.
# SAFETY: needs a human review before it ships
kit-key-hidden = Chave oculta

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = Escolha uma

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = Buscar

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = Nada corresponde

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, { $n } de { $total }

# Read out by the screen reader for a spinner that is turning.
kit-loading = Carregando

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
# SAFETY: needs a human review before it ships
kit-click-again = Clique de novo para { $action }

# The same, when the button has no name to repeat.
# SAFETY: needs a human review before it ships
kit-click-again-bare = Clique de novo

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + mais

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
        [one] +{ $n } linha
        [many] +{ $n } de linhas
       *[other] +{ $n } linhas
    }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = Nada resultou do que foi solto.

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = O que foi solto é grande demais para ser recebido assim.

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = O que foi solto demorou demais para chegar, então foi deixado de lado.

# Said when too many things were dropped at once.
kit-dropped-too-many = Coisas demais foram soltas de uma vez. Solte menos de cada vez.

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = O que foi solto não é uma lista de arquivos deste computador.

# Label of a link button whose address is not a web address.
kit-link-not-web = Não é um endereço da web

# Label of a link button whose address opens a mail program.
kit-link-mailto = Abre um endereço mailto

# Label of a link button whose address opens a file.
kit-link-file = Abre um arquivo

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = Link para { $label }

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = A janela de escolha não respondeu. Tente de novo.

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = Linha { $line }: { $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = O arquivo é grande demais para ser um tema. Um tema tem no máximo 64 KB.

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = O arquivo não é uma folha de estilo: tem mais do que palavras.

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = um tema não pode importar outro arquivo nem um endereço da web.

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = --{ $name } é longo demais para ser um valor.

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = --{ $name } não é uma cor que o Trindle consiga ler.

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = --{ $name } não é um número que o Trindle consiga ler.

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = --{ $name } não é uma sombra que o Trindle consiga ler.

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = Nada no arquivo é um valor que o Trindle conheça, então ele não mudaria nada.

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = Progresso

# The name a screen reader gives a menu that opens over a window.
kit-menu = Menu

kit-job-wait = Aguarde o término da tarefa.

# Shown at the end of a dropdown's search box: how many rows match out of how many there are.
kit-search-of = { $n } de { $total }

# A menu row that opens a sub-menu. $words is the row's own words, $n how many lines the sub-menu holds.
kit-row-sub =
    { $n ->
        [one] { $words }, submenu, { $n } linha
        [many] { $words }, submenu, { $n } linhas
       *[other] { $words }, submenu, { $n } linhas
    }

# A menu row with a shortcut. $words is the row's own words, $keys the shortcut.
kit-row-keys = { $words }, { $keys }

# Said by a screen reader for the close box in a tab.
kit-tab-close = Fechar

# Said by a screen reader for the button that lists the tabs that did not fit.
kit-tab-more = Mais abas

# Right-click menu of a text area, one row each.
kit-area-cut = Recortar

kit-area-copy = Copiar

kit-area-paste = Colar

kit-area-select-all = Selecionar tudo

# The name a screen reader gives the painted square of a colour picker.
kit-colour-square = Cor, saturação e brilho

# The name a screen reader gives the swatch showing the colour chosen. $colour is its hex code.
kit-colour-swatch = Cor escolhida { $colour }

kit-colour-hue = Matiz

kit-colour-hex = Código hexadecimal

# Buttons either side of the month in a date picker.
kit-date-previous = Anterior

kit-date-next = Próximo

# The month shown in a date picker. $month is the month's name, $year the year.
kit-date-month-year = { $month } de { $year }

# What a screen reader says for a day in a date picker.
kit-date-full = { $weekday }, { $day } de { $month } de { $year }

# The names of the two boxes in a time entry.
kit-time-hours = Horas

kit-time-minutes = Minutos

# Months and weekdays; weekdays are short in the grid and full when read out.
kit-month-1 = janeiro

kit-month-2 = fevereiro

kit-month-3 = março

kit-month-4 = abril

kit-month-5 = maio

kit-month-6 = junho

kit-month-7 = julho

kit-month-8 = agosto

kit-month-9 = setembro

kit-month-10 = outubro

kit-month-11 = novembro

kit-month-12 = dezembro

kit-weekday-1 = seg

kit-weekday-2 = ter

kit-weekday-3 = qua

kit-weekday-4 = qui

kit-weekday-5 = sex

kit-weekday-6 = sáb

kit-weekday-7 = dom

kit-weekday-full-1 = segunda-feira

kit-weekday-full-2 = terça-feira

kit-weekday-full-3 = quarta-feira

kit-weekday-full-4 = quinta-feira

kit-weekday-full-5 = sexta-feira

kit-weekday-full-6 = sábado

kit-weekday-full-7 = domingo

# The names of the modifier keys in a shortcut such as Ctrl+Shift+C.
kit-key-ctrl = Ctrl

kit-key-shift = Shift

kit-key-alt = Alt

kit-key-logo = Logo

# Said in a window of Trindle's own when something tried to press a button that only the person may press.
kit-only-your-own = Aqui só funciona o seu próprio toque.

# Said in a window when keys are pressed but the keyboard layout could not be read.
kit-no-layout = Não foi possível ler o layout do teclado, então digitar não faz nada.



# Read out by the screen reader for the line between two areas of a window that can be dragged to make one bigger.
kit-divider = Divisor

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
kit-click-again-small = sim

# Read out by the screen reader for the control under something dragged, when letting go there would drop it in.
kit-drop-target = Área para soltar

# Fault in a theme or style sheet: a setting that takes only named steps was given something else. $steps is the list of steps, $guide the file that explains them.
kit-theme-not-step = --{ $name } aceita só um destes passos: { $steps }. Qualquer outro é ignorado. Veja { $guide }.

# Standard error, when the folder of the person's style sheet cannot be watched. $folder is the folder, $why the system's reason.
kit-sheet-not-watched = Mudanças na folha de estilo em { $folder } só aparecem após reiniciar: { $why }

# What a screen reader calls the row of menu titles across the top of a window.
kit-menu-bar = Barra de menus

# The answer at the foot of every dialog page that leaves it with nothing done.
kit-cancel = Cancelar

# What a screen reader calls a row of tool buttons.
kit-toolbar = Ferramentas

# The button at the end of a toolbar that opens a menu of the tools that did not fit.
kit-more-tools = Mais ferramentas

# What a screen reader calls the line at a window's foot that says how things stand.
kit-status = Estado

# What a screen reader calls a place's path, such as a folder's.
kit-path = Caminho

# The list in the middle of a long path, holding the places not shown.
kit-path-more = Mais locais

# Said after a range slider's name for its low handle.
kit-range-low = extremo inferior

# Said after a range slider's name for its high handle.
kit-range-high = extremo superior

# The button that takes a step off a number box's number. $what is the box's name.
kit-spin-less = Menos { $what }

# The button that adds a step to a number box's number. $what is the box's name.
kit-spin-more = Mais { $what }

# Red words under a number box whose typing is not a number.
kit-spin-not-number = Digite um número de { $min } a { $max }

# Said by a screen reader for a tab that carries a count, such as unread messages: its title, then the count.
kit-tab-count = { $title }, { $count }

# The ready settings page for a program's own look (settings::look_page).
kit-look-title = Aparência

kit-look-lead = Como este programa se apresenta, seja o que for que o resto da área de trabalho faça.

kit-look-group = Aparência

kit-look-shade = Claro ou escuro

kit-look-shade-help = Seguir a área de trabalho, ou sempre uma aparência.

kit-look-desktop = Como a área de trabalho

kit-look-dark = Escuro

kit-look-light = Claro

kit-look-highlight = Cor de destaque

kit-look-highlight-help = A cor da ação principal e do que está escolhido.

# A button that gives the highlight back to the desktop's choice.
kit-look-highlight-desktop = Usar a da área de trabalho

# Over the column of week numbers in a date picker; keep it as short as the short weekday names.
kit-date-week-short = Sem

# What a screen reader says for a week number in a date picker. $week is the number.
kit-date-week = Semana { $week }

# The names of keys that are words, in a shortcut such as Ctrl+Page down.
kit-key-left = Esquerda

kit-key-right = Direita

kit-key-up = Cima

kit-key-down = Baixo

kit-key-page-up = Page Up

kit-key-page-down = Page Down

kit-key-home = Home

kit-key-end = End

kit-key-enter = Enter

kit-key-escape = Escape

kit-key-tab = Tab

kit-key-space = Espaço

kit-key-backspace = Backspace

kit-key-delete = Delete

kit-key-insert = Insert

# The heading of the page that lists a program's keyboard shortcuts.
kit-shortcuts = Atalhos de teclado

# Under a shortcut on that page whose keys the window always takes first.
kit-shortcut-kept-window = A janela reserva esta tecla para se mover entre as partes e fechar.

# Under a shortcut on that page whose keys a text box takes first while you type.
kit-shortcut-kept-typing = Uma caixa de texto fica com esta tecla primeiro enquanto você digita.

# The heading of the page of emoji to pick from.
kit-emoji = Emoji

# The name of the box on that page where an emoji's name is typed to find it.
kit-emoji-search = Achar um emoji pelo nome

# Shown on that page when no emoji's name has what was typed.
kit-emoji-none = Nenhum emoji tem esse nome.

# The groups of emoji on that page, in order. Another language may leave out any
# name below; the page then shows the English one, so a name is never missing.
kit-emoji-faces = Rostos

kit-emoji-hands = Mãos

kit-emoji-animals = Animais

kit-emoji-nature = Natureza

kit-emoji-food = Comida e bebida

kit-emoji-travel = Viagens e lugares

kit-emoji-objects = Objetos

kit-emoji-symbols = Símbolos

# Each emoji's name, spoken by a screen reader and found by typing.
kit-emoji-grin = Sorriso largo

kit-emoji-tears-of-joy = Lágrimas de alegria

kit-emoji-smile = Sorriso

kit-emoji-wink = Piscadela

kit-emoji-heart-eyes = Olhos de coração

kit-emoji-thinking = Pensando

kit-emoji-crying = Chorando

kit-emoji-surprised = Surpreso

kit-emoji-blush = Corado

kit-emoji-halo = Auréola

kit-emoji-cool = Descolado

kit-emoji-sleeping = Dormindo

kit-emoji-sobbing = Soluçando

kit-emoji-angry = Bravo

kit-emoji-scream = Grito

kit-emoji-eye-roll = Revirando os olhos

kit-emoji-relieved = Aliviado

kit-emoji-rolling-laugh = Rolando de rir

kit-emoji-blowing-a-kiss = Mandando um beijo

kit-emoji-straight-face = Cara séria

kit-emoji-flushed = Envergonhado

kit-emoji-hug = Abraço

kit-emoji-unwell = Passando mal

kit-emoji-celebrating = Comemorando

kit-emoji-thumbs-up = Joinha

kit-emoji-thumbs-down = Joinha para baixo

kit-emoji-clapping = Aplaudindo

kit-emoji-thank-you = Obrigado

kit-emoji-waving = Acenando

kit-emoji-victory = Vitória

kit-emoji-ok = OK

kit-emoji-crossed-fingers = Dedos cruzados

kit-emoji-strong = Forte

kit-emoji-pointing-up = Apontando para cima

kit-emoji-pointing-down = Apontando para baixo

kit-emoji-pointing-left = Apontando para a esquerda

kit-emoji-pointing-right = Apontando para a direita

kit-emoji-raised-hand = Mão levantada

kit-emoji-handshake = Aperto de mão

kit-emoji-fist = Punho

kit-emoji-raised-hands = Mãos para o alto

kit-emoji-call-me = Me ligue

kit-emoji-writing = Escrevendo

kit-emoji-shrug = Dar de ombros

kit-emoji-dog = Cachorro

kit-emoji-cat = Gato

kit-emoji-mouse = Rato

kit-emoji-rabbit = Coelho

kit-emoji-fox = Raposa

kit-emoji-bear = Urso

kit-emoji-panda = Panda

kit-emoji-tiger = Tigre

kit-emoji-lion = Leão

kit-emoji-cow = Vaca

kit-emoji-pig = Porco

kit-emoji-frog = Sapo

kit-emoji-monkey = Macaco

kit-emoji-chicken = Galinha

kit-emoji-penguin = Pinguim

kit-emoji-bird = Pássaro

kit-emoji-duck = Pato

kit-emoji-owl = Coruja

kit-emoji-horse = Cavalo

kit-emoji-bee = Abelha

kit-emoji-butterfly = Borboleta

kit-emoji-turtle = Tartaruga

kit-emoji-fish = Peixe

kit-emoji-dolphin = Golfinho

kit-emoji-blossom = Flor

kit-emoji-tree = Árvore

kit-emoji-sun = Sol

kit-emoji-rain = Chuva

kit-emoji-rose = Rosa

kit-emoji-sunflower = Girassol

kit-emoji-tulip = Tulipa

kit-emoji-evergreen = Conífera

kit-emoji-clover = Trevo

kit-emoji-maple-leaf = Folha de bordo

kit-emoji-cactus = Cacto

kit-emoji-moon = Lua

kit-emoji-rainbow = Arco-íris

kit-emoji-cloud = Nuvem

kit-emoji-snowflake = Floco de neve

kit-emoji-lightning = Raio

kit-emoji-wave = Onda

kit-emoji-earth = Terra

kit-emoji-mountain = Montanha

kit-emoji-palm-tree = Palmeira

kit-emoji-apple = Maçã

kit-emoji-bread = Pão

kit-emoji-cake = Bolo

kit-emoji-tea = Chá

kit-emoji-banana = Banana

kit-emoji-orange = Laranja

kit-emoji-strawberry = Morango

kit-emoji-grapes = Uvas

kit-emoji-watermelon = Melancia

kit-emoji-lemon = Limão

kit-emoji-carrot = Cenoura

kit-emoji-corn = Milho

kit-emoji-pizza = Pizza

kit-emoji-burger = Hambúrguer

kit-emoji-chips = Batatas fritas

kit-emoji-pasta = Macarrão

kit-emoji-fried-egg = Ovo frito

kit-emoji-cheese = Queijo

kit-emoji-biscuit = Biscoito

kit-emoji-chocolate = Chocolate

kit-emoji-ice-cream = Sorvete

kit-emoji-beer = Cerveja

kit-emoji-wine = Vinho

kit-emoji-milk = Leite

kit-emoji-car = Carro

kit-emoji-bus = Ônibus

kit-emoji-bicycle = Bicicleta

kit-emoji-train = Trem

kit-emoji-aeroplane = Avião

kit-emoji-rocket = Foguete

kit-emoji-ship = Navio

kit-emoji-sailing-boat = Barco a vela

kit-emoji-helicopter = Helicóptero

kit-emoji-house = Casa

kit-emoji-office = Escritório

kit-emoji-hospital = Hospital

kit-emoji-school = Escola

kit-emoji-church = Igreja

kit-emoji-beach = Praia

kit-emoji-tower = Torre

kit-emoji-bridge = Ponte

kit-emoji-petrol-pump = Bomba de combustível

kit-emoji-traffic-light = Semáforo

kit-emoji-map = Mapa

kit-emoji-phone = Telefone

kit-emoji-laptop = Notebook

kit-emoji-keyboard = Teclado

kit-emoji-camera = Câmera

kit-emoji-light-bulb = Lâmpada

kit-emoji-key = Chave

kit-emoji-lock = Cadeado

kit-emoji-books = Livros

kit-emoji-memo = Bloco de notas

kit-emoji-pencil = Lápis

kit-emoji-paperclip = Clipe de papel

kit-emoji-calendar = Calendário

kit-emoji-email = E-mail

kit-emoji-gift = Presente

kit-emoji-music-note = Nota musical

kit-emoji-headphones = Fones de ouvido

kit-emoji-alarm-clock = Despertador

kit-emoji-bell = Sino

kit-emoji-spanner = Chave inglesa

kit-emoji-money-bag = Saco de dinheiro

kit-emoji-shopping-trolley = Carrinho de compras

kit-emoji-football = Bola de futebol

kit-emoji-heart = Coração

kit-emoji-star = Estrela

kit-emoji-done = Concluído

kit-emoji-question = Pergunta

kit-emoji-party = Festa

kit-emoji-fire = Fogo

kit-emoji-cross = Cruz

kit-emoji-exclamation = Exclamação

kit-emoji-warning = Aviso

kit-emoji-hundred = Cem

kit-emoji-broken-heart = Coração partido

kit-emoji-blue-heart = Coração azul

kit-emoji-green-heart = Coração verde

kit-emoji-sparkles = Brilhos

kit-emoji-plus = Mais

kit-emoji-minus = Menos

kit-emoji-right-arrow = Seta para a direita

kit-emoji-left-arrow = Seta para a esquerda

kit-emoji-refresh = Atualizar

kit-emoji-forbidden = Proibido

kit-emoji-recycling = Reciclagem

kit-emoji-sleep = Sono

# What a screen reader is told of a scene of items: how many, and the one chosen.
kit-scene =
    { $count ->
        [one] Uma cena com 1 item
        [many] Uma cena com { $count } itens
       *[other] Uma cena com { $count } itens
    }

kit-scene-chosen =
    { $count ->
        [one] Uma cena com 1 item, { $name } escolhido
        [many] Uma cena com { $count } itens, { $name } escolhido
       *[other] Uma cena com { $count } itens, { $name } escolhido
    }

## A video or sound player's controls
kit-player-play = Reproduzir

kit-player-pause = Pausar

kit-player-back = Voltar 5 segundos

kit-player-forward = Avançar 5 segundos

kit-player-seek = Posição

kit-player-mute = Silenciar

kit-player-unmute = Som ligado

kit-player-volume = Volume

# How far in, out of how long, as clock times.
kit-player-time = { $at } de { $length }

## The page shown after a program crashed or met a fault
kit-crash-title = Algo deu errado

kit-crash-lead = { $program } parou por causa de uma falha. Copie o relatório abaixo e envie com o seu relato do erro, para que possa ser corrigido.

kit-crash-copy = Copiar relatório

# What a screen reader calls a block of code on a help page.
kit-help-code = Código

# What a screen reader calls the list of suggestions under a text box; $name is what the box is called.
kit-complete-list = Sugestões para { $name }

# Under a text box with suggestions: how many there are, said by a screen reader as they change.
kit-complete-count =
    { $n ->
        [one] 1 sugestão
        [many] { $n } sugestões
       *[other] { $n } sugestões
    }

## A title bar the program draws, where the compositor draws none
kit-frame-minimise = Minimizar

kit-frame-maximise = Maximizar

kit-frame-restore = Restaurar

kit-frame-close = Fechar

kit-frame-move = Mover

kit-frame-resize = Redimensionar

# Settings: which figures numbers are written in.
kit-numerals-title = Algarismos

kit-numerals-help = Como números, datas e horas são escritos.

kit-numerals-auto = Como a região costuma escrevê-los

kit-numerals-latin = Sempre de 0 a 9

kit-numerals-native = Os algarismos do próprio idioma

