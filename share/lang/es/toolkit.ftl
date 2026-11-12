# machine translated (Spanish), not yet reviewed by a native speaker.
# machine translated (Spanish), not yet reviewed by a person.

# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = Copiar

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = Seleccionar todo

# Right-click menu on a typing box that has words in it.
kit-cut = Cortar

# Right-click menu on a typing box.
kit-paste = Pegar

# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
kit-show-password = Mostrar la contraseña

# Name of the eye beside a PIN box while the PIN is hidden.
kit-show-pin = Mostrar el PIN

# Name of the eye beside a key box while the key is hidden.
kit-show-key = Mostrar la clave

# Name of the eye beside a password box while the password shows.
kit-hide-password = Ocultar la contraseña

# Name of the eye beside a PIN box while the PIN shows.
kit-hide-pin = Ocultar el PIN

# Name of the eye beside a key box while the key shows.
kit-hide-key = Ocultar la clave

# Said by the screen reader after the eye beside a password box is pressed to show it.
kit-password-shown = Contraseña visible

# Said by the screen reader after the eye beside a password box is pressed to hide it.
kit-password-hidden = Contraseña oculta

# Said by the screen reader after the eye beside a PIN box is pressed to show it.
kit-pin-shown = PIN visible

# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
kit-pin-hidden = PIN oculto

# Said by the screen reader after the eye beside a key box is pressed to show it.
kit-key-shown = Clave visible

# Said by the screen reader after the eye beside a key box is pressed to hide it.
kit-key-hidden = Clave oculta

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = Elige una opción

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = Buscar

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = Nada coincide

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, { $n } de { $total }

# Read out by the screen reader for a spinner that is turning.
kit-loading = Cargando

# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
kit-click-again = Haz clic otra vez para { $action }

# The same, when the button has no name to repeat.
kit-click-again-bare = Haz clic otra vez

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
# SAFETY: needs a human review before it ships
kit-click-again-small = yes

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + más

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
        [one] +{ $n } línea
        [many] +{ $n } líneas
       *[other] +{ $n } líneas
    }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = Trindle no pudo hacer nada con los elementos que soltaste.

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = Lo que soltaste es demasiado grande para arrastrarlo aquí.

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = Lo que soltaste tardó demasiado en llegar, así que Trindle dejó de esperar.

# Said when too many things were dropped at once.
kit-dropped-too-many = Soltaste demasiados elementos a la vez. Suelta menos cada vez.

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = Los elementos que soltaste no son archivos de este equipo.

# Label of a link button whose address is not a web address.
kit-link-not-web = No es una dirección web

# Label of a link button whose address opens a mail program.
kit-link-mailto = Dirección de correo

# Label of a link button whose address opens a file.
kit-link-file = Archivo de este equipo

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = Enlace a { $label }

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = La ventana de selección no respondió. Inténtalo de nuevo.

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = Línea { $line }: { $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = Este archivo de tema es demasiado grande. El tamaño máximo es 64 KB.

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = Este archivo no es una hoja de estilo. Contiene algo más que texto sin formato.

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = un tema no puede importar otro archivo ni una dirección web.

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = El valor de --{ $name } es demasiado largo.

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = Trindle no puede leer --{ $name } como un color.

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = Trindle no puede leer --{ $name } como un número.

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = Trindle no puede leer --{ $name } como una sombra.

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = Trindle no reconoce ningún ajuste de este archivo, así que no cambiaría nada.

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = Progreso

# The name a screen reader gives a menu that opens over a window.
kit-menu = Menú


kit-job-wait = Espera a que termine la tarea.


kit-search-of = { $n } de { $total }

kit-divider = Separador

kit-row-sub =
    { $n ->
        [one] { $words }, submenú, 1 línea
       *[other] { $words }, submenú, { $n } líneas
    }

kit-row-keys = { $words }, { $keys }

kit-drop-target = Zona para soltar

kit-theme-not-step = --{ $name } solo puede ser uno de estos valores: { $steps }. Trindle ignora cualquier otro. Consulta { $guide }.

kit-sheet-not-watched = Reinicia para ver los cambios en la hoja de estilo de { $folder }. { $why }

kit-menu-bar = Barra de menús

kit-cancel = Cancelar

kit-toolbar = Herramientas

kit-more-tools = Más herramientas

kit-status = Estado

kit-path = Ruta

kit-path-more = Más lugares

kit-range-low = extremo inferior

kit-range-high = extremo superior

kit-spin-less = Disminuir { $what }

kit-spin-more = Aumentar { $what }

kit-spin-not-number = Escribe un número de { $min } a { $max }

kit-tab-close = Cerrar

kit-tab-more = Más pestañas

kit-area-cut = Cortar

kit-area-copy = Copiar

kit-area-paste = Pegar

kit-area-select-all = Seleccionar todo

kit-colour-square = Color, saturación y brillo

kit-colour-swatch = Color elegido { $colour }

kit-colour-hue = Tono

kit-colour-hex = Código hexadecimal

kit-look-title = Aspecto

kit-look-lead = Elige cómo se ve este programa, con independencia del resto del escritorio.

kit-look-group = Aspecto

kit-look-shade = Claro u oscuro

kit-look-shade-help = Igual que el escritorio, o siempre claro u oscuro.

kit-look-desktop = Igual que el escritorio

kit-look-dark = Oscuro

kit-look-light = Claro

kit-look-highlight = Color de resalte

kit-look-highlight-help = El color de los botones principales y de los elementos seleccionados.

kit-look-highlight-desktop = Igual que el escritorio

kit-date-previous = Anterior

kit-date-next = Siguiente

kit-date-month-year = { $month } de { $year }

kit-date-full = { $weekday } { $day } de { $month } de { $year }

kit-time-hours = Horas

kit-time-minutes = Minutos

kit-month-1 = enero

kit-month-2 = febrero

kit-month-3 = marzo

kit-month-4 = abril

kit-month-5 = mayo

kit-month-6 = junio

kit-month-7 = julio

kit-month-8 = agosto

kit-month-9 = septiembre

kit-month-10 = octubre

kit-month-11 = noviembre

kit-month-12 = diciembre
# A month as it reads inside a whole date, such as "12 November 2026". Some languages change the ending there, so it has its own key.
kit-month-in-date-1 = enero
kit-month-in-date-2 = febrero
kit-month-in-date-3 = marzo
kit-month-in-date-4 = abril
kit-month-in-date-5 = mayo
kit-month-in-date-6 = junio
kit-month-in-date-7 = julio
kit-month-in-date-8 = agosto
kit-month-in-date-9 = septiembre
kit-month-in-date-10 = octubre
kit-month-in-date-11 = noviembre
kit-month-in-date-12 = diciembre

kit-weekday-1 = lun

kit-weekday-2 = mar

kit-weekday-3 = mié

kit-weekday-4 = jue

kit-weekday-5 = vie

kit-weekday-6 = sáb

kit-weekday-7 = dom

kit-weekday-full-1 = lunes

kit-weekday-full-2 = martes

kit-weekday-full-3 = miércoles

kit-weekday-full-4 = jueves

kit-weekday-full-5 = viernes

kit-weekday-full-6 = sábado

kit-weekday-full-7 = domingo

kit-key-ctrl = Ctrl

kit-key-shift = Mayús

kit-key-alt = Alt

kit-key-logo = Super

kit-key-left = Izquierda

kit-key-right = Derecha

kit-key-up = Arriba

kit-key-down = Abajo

kit-key-page-up = Re Pág

kit-key-page-down = Av Pág

kit-key-home = Inicio

kit-key-end = Fin

kit-key-enter = Intro

kit-key-escape = Esc

kit-key-tab = Tab

kit-key-space = Espacio

kit-key-backspace = Retroceso

kit-key-delete = Supr

kit-key-insert = Insert

kit-shortcuts = Atajos de teclado

kit-shortcut-kept-window = Esta tecla está reservada para moverse entre controles y cerrar la ventana.

kit-shortcut-kept-typing = Mientras escribes, esta tecla va primero al cuadro de texto.

kit-emoji = Emoji

kit-emoji-search = Busca un emoji por su nombre

kit-emoji-none = Ningún emoji tiene ese nombre.

kit-emoji-faces = Caras

kit-emoji-hands = Manos

kit-emoji-animals = Animales

kit-emoji-nature = Naturaleza

kit-emoji-food = Comida y bebida

kit-emoji-travel = Viajes y lugares

kit-emoji-objects = Objetos

kit-emoji-symbols = Símbolos

kit-emoji-grin = Sonrisa amplia

kit-emoji-tears-of-joy = Lágrimas de alegría

kit-emoji-smile = Sonrisa

kit-emoji-wink = Guiño

kit-emoji-heart-eyes = Ojos de corazón

kit-emoji-thinking = Pensando

kit-emoji-crying = Llorando

kit-emoji-surprised = Sorprendido

kit-emoji-blush = Sonrojado

kit-emoji-halo = Aureola

kit-emoji-cool = Genial

kit-emoji-sleeping = Durmiendo

kit-emoji-sobbing = Sollozando

kit-emoji-angry = Enfadado

kit-emoji-scream = Grito

kit-emoji-eye-roll = Poner los ojos en blanco

kit-emoji-relieved = Aliviado

kit-emoji-rolling-laugh = Muerto de risa

kit-emoji-blowing-a-kiss = Lanzando un beso

kit-emoji-straight-face = Cara seria

kit-emoji-flushed = Ruborizado

kit-emoji-hug = Abrazo

kit-emoji-unwell = Indispuesto

kit-emoji-celebrating = Celebrando

kit-emoji-thumbs-up = Pulgar arriba

kit-emoji-thumbs-down = Pulgar abajo

kit-emoji-clapping = Aplaudiendo

kit-emoji-thank-you = Gracias

kit-emoji-waving = Saludando

kit-emoji-victory = Victoria

kit-emoji-ok = OK

kit-emoji-crossed-fingers = Dedos cruzados

kit-emoji-strong = Fuerte

kit-emoji-pointing-up = Señalando arriba

kit-emoji-pointing-down = Señalando abajo

kit-emoji-pointing-left = Señalando a la izquierda

kit-emoji-pointing-right = Señalando a la derecha

kit-emoji-raised-hand = Mano levantada

kit-emoji-handshake = Apretón de manos

kit-emoji-fist = Puño

kit-emoji-raised-hands = Manos levantadas

kit-emoji-call-me = Llámame

kit-emoji-writing = Escribiendo

kit-emoji-shrug = Encogerse de hombros

kit-emoji-dog = Perro

kit-emoji-cat = Gato

kit-emoji-mouse = Ratón

kit-emoji-rabbit = Conejo

kit-emoji-fox = Zorro

kit-emoji-bear = Oso

kit-emoji-panda = Panda

kit-emoji-tiger = Tigre

kit-emoji-lion = León

kit-emoji-cow = Vaca

kit-emoji-pig = Cerdo

kit-emoji-frog = Rana

kit-emoji-monkey = Mono

kit-emoji-chicken = Pollo

kit-emoji-penguin = Pingüino

kit-emoji-bird = Pájaro

kit-emoji-duck = Pato

kit-emoji-owl = Búho

kit-emoji-horse = Caballo

kit-emoji-bee = Abeja

kit-emoji-butterfly = Mariposa

kit-emoji-turtle = Tortuga

kit-emoji-fish = Pez

kit-emoji-dolphin = Delfín

kit-emoji-blossom = Flor

kit-emoji-tree = Árbol

kit-emoji-sun = Sol

kit-emoji-rain = Lluvia

kit-emoji-rose = Rosa

kit-emoji-sunflower = Girasol

kit-emoji-tulip = Tulipán

kit-emoji-evergreen = Árbol de hoja perenne

kit-emoji-clover = Trébol

kit-emoji-maple-leaf = Hoja de arce

kit-emoji-cactus = Cactus

kit-emoji-moon = Luna

kit-emoji-rainbow = Arcoíris

kit-emoji-cloud = Nube

kit-emoji-snowflake = Copo de nieve

kit-emoji-lightning = Rayo

kit-emoji-wave = Ola

kit-emoji-earth = Tierra

kit-emoji-mountain = Montaña

kit-emoji-palm-tree = Palmera

kit-emoji-apple = Manzana

kit-emoji-bread = Pan

kit-emoji-cake = Tarta

kit-emoji-tea = Té

kit-emoji-banana = Plátano

kit-emoji-orange = Naranja

kit-emoji-strawberry = Fresa

kit-emoji-grapes = Uvas

kit-emoji-watermelon = Sandía

kit-emoji-lemon = Limón

kit-emoji-carrot = Zanahoria

kit-emoji-corn = Maíz

kit-emoji-pizza = Pizza

kit-emoji-burger = Hamburguesa

kit-emoji-chips = Patatas fritas

kit-emoji-pasta = Pasta

kit-emoji-fried-egg = Huevo frito

kit-emoji-cheese = Queso

kit-emoji-biscuit = Galleta

kit-emoji-chocolate = Chocolate

kit-emoji-ice-cream = Helado

kit-emoji-beer = Cerveza

kit-emoji-wine = Vino

kit-emoji-milk = Leche

kit-emoji-car = Coche

kit-emoji-bus = Autobús

kit-emoji-bicycle = Bicicleta

kit-emoji-train = Tren

kit-emoji-aeroplane = Avión

kit-emoji-rocket = Cohete

kit-emoji-ship = Barco

kit-emoji-sailing-boat = Velero

kit-emoji-helicopter = Helicóptero

kit-emoji-house = Casa

kit-emoji-office = Oficina

kit-emoji-hospital = Hospital

kit-emoji-school = Escuela

kit-emoji-church = Iglesia

kit-emoji-beach = Playa

kit-emoji-tower = Torre

kit-emoji-bridge = Puente

kit-emoji-petrol-pump = Surtidor de gasolina

kit-emoji-traffic-light = Semáforo

kit-emoji-map = Mapa

kit-emoji-phone = Teléfono

kit-emoji-laptop = Portátil

kit-emoji-keyboard = Teclado

kit-emoji-camera = Cámara

kit-emoji-light-bulb = Bombilla

kit-emoji-key = Llave

kit-emoji-lock = Candado

kit-emoji-books = Libros

kit-emoji-memo = Nota

kit-emoji-pencil = Lápiz

kit-emoji-paperclip = Clip

kit-emoji-calendar = Calendario

kit-emoji-email = Correo electrónico

kit-emoji-gift = Regalo

kit-emoji-music-note = Nota musical

kit-emoji-headphones = Auriculares

kit-emoji-alarm-clock = Despertador

kit-emoji-bell = Campana

kit-emoji-spanner = Llave inglesa

kit-emoji-money-bag = Bolsa de dinero

kit-emoji-shopping-trolley = Carrito de la compra

kit-emoji-football = Fútbol

kit-emoji-heart = Corazón

kit-emoji-star = Estrella

kit-emoji-done = Hecho

kit-emoji-question = Pregunta

kit-emoji-party = Fiesta

kit-emoji-fire = Fuego

kit-emoji-cross = Cruz

kit-emoji-exclamation = Exclamación

kit-emoji-warning = Advertencia

kit-emoji-hundred = Cien

kit-emoji-broken-heart = Corazón roto

kit-emoji-blue-heart = Corazón azul

kit-emoji-green-heart = Corazón verde

kit-emoji-sparkles = Destellos

kit-emoji-plus = Más

kit-emoji-minus = Menos

kit-emoji-right-arrow = Flecha derecha

kit-emoji-left-arrow = Flecha izquierda

kit-emoji-refresh = Actualizar

kit-emoji-forbidden = Prohibido

kit-emoji-recycling = Reciclaje

kit-emoji-sleep = Sueño

kit-only-your-own = Solo tú puedes pulsar este botón.

kit-no-layout = Trindle no pudo leer la distribución del teclado, así que no puedes escribir.

kit-scene =
    { $count ->
        [one] Una escena de 1 elemento
       *[other] Una escena de { $count } elementos
    }

kit-scene-chosen =
    { $count ->
        [one] Una escena de 1 elemento, { $name } elegido
       *[other] Una escena de { $count } elementos, { $name } elegido
    }

kit-player-play = Reproducir

kit-player-pause = Pausar

kit-player-back = Retroceder 5 segundos

kit-player-forward = Avanzar 5 segundos

kit-player-seek = Posición

kit-player-mute = Silenciar

kit-player-unmute = Activar sonido

kit-player-volume = Volumen

kit-player-time = { $at } de { $length }

kit-crash-title = Algo salió mal

kit-crash-lead = { $program } se detuvo porque algo salió mal. Copia el informe de abajo y envíalo con tu informe de error para que se pueda corregir el problema.

kit-crash-copy = Copiar informe

kit-help-code = Código

kit-complete-count =
    { $n ->
        [one] 1 sugerencia
       *[other] { $n } sugerencias
    }

kit-frame-minimise = Minimizar

kit-frame-maximise = Maximizar

kit-frame-restore = Restaurar

kit-frame-close = Cerrar

kit-frame-move = Mover

kit-frame-resize = Cambiar tamaño

kit-numerals-title = Estilo de números

kit-numerals-help = Cómo se escriben los números, las fechas y las horas.

kit-numerals-auto = Lo habitual en tu región

kit-numerals-latin = Siempre de 0 a 9

kit-numerals-native = Los dígitos propios del idioma
kit-date-week-short = Sem.
kit-date-week = Semana { $week }
