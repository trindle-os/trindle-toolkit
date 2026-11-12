# machine translated (French), not yet reviewed by a native speaker.


# Right-click menu on selected words, a typing box or words that can only be copied.
kit-copy = Copier

# Right-click menu on a typing box or on words that can only be copied.
kit-select-all = Tout sélectionner

# Right-click menu on a typing box that has words in it.
kit-cut = Couper

# Right-click menu on a typing box.
kit-paste = Coller

# SAFETY: needs a human review before it ships
# Name of the eye beside a password box while the password is hidden. Also read out by the screen reader.
kit-show-password = Afficher le mot de passe

# SAFETY: needs a human review before it ships
# Name of the eye beside a PIN box while the PIN is hidden.
kit-show-pin = Afficher le code PIN

# SAFETY: needs a human review before it ships
# Name of the eye beside a key box while the key is hidden.
kit-show-key = Afficher la clé

# SAFETY: needs a human review before it ships
# Name of the eye beside a password box while the password shows.
kit-hide-password = Masquer le mot de passe

# SAFETY: needs a human review before it ships
# Name of the eye beside a PIN box while the PIN shows.
kit-hide-pin = Masquer le code PIN

# SAFETY: needs a human review before it ships
# Name of the eye beside a key box while the key shows.
kit-hide-key = Masquer la clé

# SAFETY: needs a human review before it ships
# Said by the screen reader after the eye beside a password box is pressed to show it.
kit-password-shown = Mot de passe affiché

# SAFETY: needs a human review before it ships
# Said by the screen reader after the eye beside a password box is pressed to hide it.
kit-password-hidden = Mot de passe masqué

# SAFETY: needs a human review before it ships
# Said by the screen reader after the eye beside a PIN box is pressed to show it.
kit-pin-shown = Code PIN affiché

# SAFETY: needs a human review before it ships
# Said by the screen reader after the eye beside a PIN box is pressed to hide it.
kit-pin-hidden = Code PIN masqué

# SAFETY: needs a human review before it ships
# Said by the screen reader after the eye beside a key box is pressed to show it.
kit-key-shown = Clé affichée

# SAFETY: needs a human review before it ships
# Said by the screen reader after the eye beside a key box is pressed to hide it.
kit-key-hidden = Clé masquée

# Shown in a dropdown's closed box while none of its choices is the one kept.
kit-choose-one = Choisir

# Shown, quietly, in the search box at the top of a long dropdown list while nothing is typed.
kit-search = Rechercher

# Shown in a dropdown's list when what was typed in its search box matches nothing. Also read out by the screen reader.
kit-nothing-matches = Aucun résultat

# Read out by the screen reader while a dropdown is being searched: the lit row, then how many rows match.
kit-search-count = { $row }, { $n } sur { $total }

# Read out by the screen reader for a spinner that is turning.
kit-loading = Chargement

# SAFETY: needs a human review before it ships
# The name of a button that has been pressed once and must be pressed again to do what it says. $action is the button's own name, such as "delete these files", with its first letter made small in English.
kit-click-again = Cliquez de nouveau pour { $action }

# SAFETY: needs a human review before it ships
# The same, when the button has no name to repeat.
kit-click-again-bare = Cliquez de nouveau

# Write yes if the button's name has its first letter made small where it follows on in kit-click-again, or no if it keeps it: a language whose words start with a capital after the colon, or one with no capitals. A name that starts with a capital word such as USB is always kept.
# SAFETY: needs a human review before it ships
kit-click-again-small = yes

# The last line of a label on pointing that is cut short, when what is left is the rest of one line.
kit-more = + d'autres

# The last line of a label on pointing that is cut short: how many more lines there are.
kit-more-lines =
    { $n ->
        [one] +{ $n } ligne
       *[other] +{ $n } lignes
    }

# Said under whatever was dropped on, when nothing came of it.
kit-dropped-empty = Trindle n’a rien pu faire des éléments que vous avez déposés.

# Said when what was dropped is too big to take by dropping.
kit-dropped-too-big = Ce que vous avez déposé est trop gros pour être glissé ici.

# Said when what was dropped took too long to arrive.
kit-dropped-too-slow = Ce que vous avez déposé a mis trop de temps à arriver, donc Trindle a arrêté d’attendre.

# Said when too many things were dropped at once.
kit-dropped-too-many = Vous avez déposé trop d’éléments d’un coup. Déposez-en moins à la fois.

# Said when what was dropped was a list of files that are not on this computer.
kit-dropped-not-files = Les éléments que vous avez déposés ne sont pas des fichiers de cet ordinateur.

# Label of a link button whose address is not a web address.
kit-link-not-web = Pas une adresse web

# Label of a link button whose address opens a mail program.
kit-link-mailto = Adresse e-mail

# Label of a link button whose address opens a file.
kit-link-file = Fichier sur cet ordinateur

# Read out by the screen reader for a link button. $label is the shown address.
kit-link-to = Lien vers { $label }

# Said under whatever asked for a file or folder, when the window for choosing did not answer.
kit-chooser-no-answer = La fenêtre de sélection n’a pas répondu. Réessayez.

# Starts a fault found in a theme file the person is trying. $line is the line number in the file, $why what is wrong.
kit-theme-line = Ligne { $line } : { $why }

# Fault in a theme file the person is trying: it is bigger than 64 KB.
kit-theme-too-big = Ce fichier de thème est trop gros. La taille maximale est de 64 Ko.

# Fault in a theme file the person is trying: it holds unreadable characters.
kit-theme-not-sheet = Ce fichier n’est pas une feuille de style. Il contient autre chose que du texte simple.

# Fault in a theme file the person is trying: it brings in another file.
kit-theme-imports = un thème ne peut pas importer d’autre fichier ni d’adresse web.

# Fault in a theme file the person is trying: a value is too long. $name is the setting's name.
kit-theme-too-long = La valeur de --{ $name } est trop longue.

# Fault in a theme file the person is trying: a setting that should be a colour is not one Trindle can read. $name is the setting's name.
kit-theme-not-colour = Trindle ne peut pas lire --{ $name } comme une couleur.

# Fault in a theme file the person is trying: a setting that should be a number is not one Trindle can read.
kit-theme-not-number = Trindle ne peut pas lire --{ $name } comme un nombre.

# Fault in a theme file the person is trying: a setting that should be a shadow is not one Trindle can read.
kit-theme-not-shadow = Trindle ne peut pas lire --{ $name } comme une ombre.

# Fault in a theme file the person is trying: nothing in it is a setting Trindle knows.
kit-theme-nothing-known = Trindle ne reconnaît aucun réglage dans ce fichier, donc rien ne changerait.

# The name a screen reader gives a ring that shows how far a job has got.
kit-progress = Progression

# The name a screen reader gives a menu that opens over a window.
kit-menu = Menu


kit-job-wait = Attendez la fin de la tâche.

kit-search-of = { $n } sur { $total }

kit-divider = Séparateur

kit-row-sub =
    { $n ->
        [one] { $words }, sous-menu, 1 ligne
       *[other] { $words }, sous-menu, { $n } lignes
    }

kit-row-keys = { $words }, { $keys }

kit-drop-target = Zone de dépôt

kit-theme-not-step = --{ $name } ne peut prendre que l’une de ces valeurs : { $steps }. Trindle ignore toute autre valeur. Voir { $guide }.

kit-sheet-not-watched = Redémarrez pour voir les changements de la feuille de style dans { $folder }. { $why }

kit-menu-bar = Barre de menus

kit-cancel = Annuler

kit-toolbar = Outils

kit-more-tools = Plus d'outils

kit-status = État

kit-path = Chemin

kit-path-more = Autres emplacements

kit-range-low = extrémité inférieure

kit-range-high = extrémité supérieure

kit-spin-less = Diminuer { $what }

kit-spin-more = Augmenter { $what }

kit-spin-not-number = Saisissez un nombre de { $min } à { $max }

kit-tab-close = Fermer

kit-tab-more = Autres onglets

kit-area-cut = Couper

kit-area-copy = Copier

kit-area-paste = Coller

kit-area-select-all = Tout sélectionner

kit-colour-square = Couleur, saturation et luminosité

kit-colour-swatch = Couleur choisie { $colour }

kit-colour-hue = Teinte

kit-colour-hex = Code hexadécimal

kit-look-title = Apparence

kit-look-lead = Choisissez l’apparence de ce programme, indépendamment du reste du bureau.

kit-look-group = Apparence

kit-look-shade = Clair ou sombre

kit-look-shade-help = Suivre le bureau, ou toujours clair ou sombre.

kit-look-desktop = Identique au bureau

kit-look-dark = Sombre

kit-look-light = Clair

kit-look-highlight = Couleur d'accent

kit-look-highlight-help = La couleur des boutons principaux et des éléments sélectionnés.

kit-look-highlight-desktop = Identique au bureau

kit-date-previous = Précédent

kit-date-next = Suivant

kit-date-month-year = { $month } { $year }

kit-date-full = { $weekday } { $day } { $month } { $year }

kit-time-hours = Heures

kit-time-minutes = Minutes

kit-month-1 = janvier

kit-month-2 = février

kit-month-3 = mars

kit-month-4 = avril

kit-month-5 = mai

kit-month-6 = juin

kit-month-7 = juillet

kit-month-8 = août

kit-month-9 = septembre

kit-month-10 = octobre

kit-month-11 = novembre

kit-month-12 = décembre
# A month as it reads inside a whole date, such as "12 November 2026". Some languages change the ending there, so it has its own key.
kit-month-in-date-1 = janvier
kit-month-in-date-2 = février
kit-month-in-date-3 = mars
kit-month-in-date-4 = avril
kit-month-in-date-5 = mai
kit-month-in-date-6 = juin
kit-month-in-date-7 = juillet
kit-month-in-date-8 = août
kit-month-in-date-9 = septembre
kit-month-in-date-10 = octobre
kit-month-in-date-11 = novembre
kit-month-in-date-12 = décembre

kit-weekday-1 = lun.

kit-weekday-2 = mar.

kit-weekday-3 = mer.

kit-weekday-4 = jeu.

kit-weekday-5 = ven.

kit-weekday-6 = sam.

kit-weekday-7 = dim.

kit-weekday-full-1 = lundi

kit-weekday-full-2 = mardi

kit-weekday-full-3 = mercredi

kit-weekday-full-4 = jeudi

kit-weekday-full-5 = vendredi

kit-weekday-full-6 = samedi

kit-weekday-full-7 = dimanche

kit-key-ctrl = Ctrl

kit-key-shift = Maj

kit-key-alt = Alt

kit-key-logo = Super

kit-key-left = Gauche

kit-key-right = Droite

kit-key-up = Haut

kit-key-down = Bas

kit-key-page-up = Page précédente

kit-key-page-down = Page suivante

kit-key-home = Début

kit-key-end = Fin

kit-key-enter = Entrée

kit-key-escape = Échap

kit-key-tab = Tab

kit-key-space = Espace

kit-key-backspace = Retour arrière

kit-key-delete = Suppr

kit-key-insert = Inser

kit-shortcuts = Raccourcis clavier

kit-shortcut-kept-window = Cette touche est réservée au passage d’un contrôle à l’autre et à la fermeture de la fenêtre.

kit-shortcut-kept-typing = Pendant que vous tapez, cette touche va d’abord à la zone de texte.

kit-emoji = Émojis

kit-emoji-search = Trouver un émoji par son nom

kit-emoji-none = Aucun émoji ne porte ce nom.

kit-emoji-faces = Visages

kit-emoji-hands = Mains

kit-emoji-animals = Animaux

kit-emoji-nature = Nature

kit-emoji-food = Nourriture et boissons

kit-emoji-travel = Voyages et lieux

kit-emoji-objects = Objets

kit-emoji-symbols = Symboles

kit-emoji-grin = Grand sourire

kit-emoji-tears-of-joy = Larmes de joie

kit-emoji-smile = Sourire

kit-emoji-wink = Clin d'œil

kit-emoji-heart-eyes = Yeux en cœur

kit-emoji-thinking = Réflexion

kit-emoji-crying = Pleurs

kit-emoji-surprised = Surpris

kit-emoji-blush = Rougissant

kit-emoji-halo = Auréole

kit-emoji-cool = Cool

kit-emoji-sleeping = Endormi

kit-emoji-sobbing = Sanglots

kit-emoji-angry = En colère

kit-emoji-scream = Cri

kit-emoji-eye-roll = Yeux au ciel

kit-emoji-relieved = Soulagé

kit-emoji-rolling-laugh = Mort de rire

kit-emoji-blowing-a-kiss = Envoi d'un baiser

kit-emoji-straight-face = Visage neutre

kit-emoji-flushed = Gêné

kit-emoji-hug = Câlin

kit-emoji-unwell = Malade

kit-emoji-celebrating = Fête

kit-emoji-thumbs-up = Pouce en l'air

kit-emoji-thumbs-down = Pouce vers le bas

kit-emoji-clapping = Applaudissements

kit-emoji-thank-you = Merci

kit-emoji-waving = Salut de la main

kit-emoji-victory = Victoire

kit-emoji-ok = OK

kit-emoji-crossed-fingers = Doigts croisés

kit-emoji-strong = Fort

kit-emoji-pointing-up = Index vers le haut

kit-emoji-pointing-down = Index vers le bas

kit-emoji-pointing-left = Index vers la gauche

kit-emoji-pointing-right = Index vers la droite

kit-emoji-raised-hand = Main levée

kit-emoji-handshake = Poignée de main

kit-emoji-fist = Poing

kit-emoji-raised-hands = Mains levées

kit-emoji-call-me = Appelle-moi

kit-emoji-writing = Écriture

kit-emoji-shrug = Haussement d'épaules

kit-emoji-dog = Chien

kit-emoji-cat = Chat

kit-emoji-mouse = Souris

kit-emoji-rabbit = Lapin

kit-emoji-fox = Renard

kit-emoji-bear = Ours

kit-emoji-panda = Panda

kit-emoji-tiger = Tigre

kit-emoji-lion = Lion

kit-emoji-cow = Vache

kit-emoji-pig = Cochon

kit-emoji-frog = Grenouille

kit-emoji-monkey = Singe

kit-emoji-chicken = Poulet

kit-emoji-penguin = Manchot

kit-emoji-bird = Oiseau

kit-emoji-duck = Canard

kit-emoji-owl = Hibou

kit-emoji-horse = Cheval

kit-emoji-bee = Abeille

kit-emoji-butterfly = Papillon

kit-emoji-turtle = Tortue

kit-emoji-fish = Poisson

kit-emoji-dolphin = Dauphin

kit-emoji-blossom = Fleur

kit-emoji-tree = Arbre

kit-emoji-sun = Soleil

kit-emoji-rain = Pluie

kit-emoji-rose = Rose

kit-emoji-sunflower = Tournesol

kit-emoji-tulip = Tulipe

kit-emoji-evergreen = Conifère

kit-emoji-clover = Trèfle

kit-emoji-maple-leaf = Feuille d'érable

kit-emoji-cactus = Cactus

kit-emoji-moon = Lune

kit-emoji-rainbow = Arc-en-ciel

kit-emoji-cloud = Nuage

kit-emoji-snowflake = Flocon de neige

kit-emoji-lightning = Éclair

kit-emoji-wave = Vague

kit-emoji-earth = Terre

kit-emoji-mountain = Montagne

kit-emoji-palm-tree = Palmier

kit-emoji-apple = Pomme

kit-emoji-bread = Pain

kit-emoji-cake = Gâteau

kit-emoji-tea = Thé

kit-emoji-banana = Banane

kit-emoji-orange = Orange

kit-emoji-strawberry = Fraise

kit-emoji-grapes = Raisin

kit-emoji-watermelon = Pastèque

kit-emoji-lemon = Citron

kit-emoji-carrot = Carotte

kit-emoji-corn = Maïs

kit-emoji-pizza = Pizza

kit-emoji-burger = Hamburger

kit-emoji-chips = Frites

kit-emoji-pasta = Pâtes

kit-emoji-fried-egg = Œuf au plat

kit-emoji-cheese = Fromage

kit-emoji-biscuit = Biscuit

kit-emoji-chocolate = Chocolat

kit-emoji-ice-cream = Glace

kit-emoji-beer = Bière

kit-emoji-wine = Vin

kit-emoji-milk = Lait

kit-emoji-car = Voiture

kit-emoji-bus = Bus

kit-emoji-bicycle = Vélo

kit-emoji-train = Train

kit-emoji-aeroplane = Avion

kit-emoji-rocket = Fusée

kit-emoji-ship = Navire

kit-emoji-sailing-boat = Voilier

kit-emoji-helicopter = Hélicoptère

kit-emoji-house = Maison

kit-emoji-office = Bureau

kit-emoji-hospital = Hôpital

kit-emoji-school = École

kit-emoji-church = Église

kit-emoji-beach = Plage

kit-emoji-tower = Tour

kit-emoji-bridge = Pont

kit-emoji-petrol-pump = Pompe à essence

kit-emoji-traffic-light = Feu tricolore

kit-emoji-map = Carte

kit-emoji-phone = Téléphone

kit-emoji-laptop = Ordinateur portable

kit-emoji-keyboard = Clavier

kit-emoji-camera = Appareil photo

kit-emoji-light-bulb = Ampoule

kit-emoji-key = Clé

kit-emoji-lock = Cadenas

kit-emoji-books = Livres

kit-emoji-memo = Mémo

kit-emoji-pencil = Crayon

kit-emoji-paperclip = Trombone

kit-emoji-calendar = Calendrier

kit-emoji-email = E-mail

kit-emoji-gift = Cadeau

kit-emoji-music-note = Note de musique

kit-emoji-headphones = Casque audio

kit-emoji-alarm-clock = Réveil

kit-emoji-bell = Cloche

kit-emoji-spanner = Clé à molette

kit-emoji-money-bag = Sac d'argent

kit-emoji-shopping-trolley = Chariot

kit-emoji-football = Football

kit-emoji-heart = Cœur

kit-emoji-star = Étoile

kit-emoji-done = Terminé

kit-emoji-question = Question

kit-emoji-party = Fête

kit-emoji-fire = Feu

kit-emoji-cross = Croix

kit-emoji-exclamation = Exclamation

kit-emoji-warning = Avertissement

kit-emoji-hundred = Cent

kit-emoji-broken-heart = Cœur brisé

kit-emoji-blue-heart = Cœur bleu

kit-emoji-green-heart = Cœur vert

kit-emoji-sparkles = Étincelles

kit-emoji-plus = Plus

kit-emoji-minus = Moins

kit-emoji-right-arrow = Flèche droite

kit-emoji-left-arrow = Flèche gauche

kit-emoji-refresh = Actualiser

kit-emoji-forbidden = Interdit

kit-emoji-recycling = Recyclage

kit-emoji-sleep = Sommeil

kit-only-your-own = Ce bouton ne peut être actionné que par vous.

kit-no-layout = Trindle n’a pas pu lire la disposition du clavier, donc vous ne pouvez pas taper.

kit-scene =
    { $count ->
        [one] Une scène de 1 élément
       *[other] Une scène de { $count } éléments
    }

kit-scene-chosen =
    { $count ->
        [one] Une scène de 1 élément, { $name } choisi
       *[other] Une scène de { $count } éléments, { $name } choisi
    }

kit-player-play = Lecture

kit-player-pause = Pause

kit-player-back = Reculer de 5 secondes

kit-player-forward = Avancer de 5 secondes

kit-player-seek = Position

kit-player-mute = Couper le son

kit-player-unmute = Réactiver le son

kit-player-volume = Volume

kit-player-time = { $at } sur { $length }

kit-crash-title = Un problème est survenu

kit-crash-lead = { $program } s’est arrêté parce qu’un problème est survenu. Copiez le rapport ci-dessous et envoyez-le avec votre signalement de bogue afin que le problème puisse être corrigé.

kit-crash-copy = Copier le rapport

kit-help-code = Code

kit-complete-count =
    { $n ->
        [one] 1 suggestion
       *[other] { $n } suggestions
    }

kit-frame-minimise = Réduire

kit-frame-maximise = Agrandir

kit-frame-restore = Restaurer

kit-frame-close = Fermer

kit-frame-move = Déplacer

kit-frame-resize = Redimensionner

kit-numerals-title = Style des chiffres

kit-numerals-help = La façon d'écrire les nombres, les dates et les heures.

kit-numerals-auto = Habituel pour votre région

kit-numerals-latin = Toujours de 0 à 9

kit-numerals-native = Les chiffres propres à la langue
kit-date-week-short = Sem.
kit-date-week = Semaine { $week }
