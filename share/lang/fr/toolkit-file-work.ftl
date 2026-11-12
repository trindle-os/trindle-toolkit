# machine translated (French), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = il était absent quand Trindle a vérifié, donc il a été ignoré

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = il était trop gros pour être vérifié en entier, donc il a été ignoré

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = il a changé après son dépôt, donc il n’a été ni copié ni déplacé

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = un autre élément porte désormais ce nom
kit-work-drive-no-safe-restore = ce disque ne peut pas restaurer les fichiers en toute sécurité, copiez plutôt le fichier hors de la corbeille

# SAFETY: needs a human review before it ships
# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = l’élément déjà présent n’a pas pu être placé dans la corbeille

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = il a changé après sa copie, donc Trindle l’a laissé où il était

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = le disque n’a pas tout enregistré, il peut donc manquer une partie

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = Trindle n’a pas pu tout lire

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = il se trouve dans le dossier qu’il remplacerait

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = Trindle l’a créé pendant cette copie, donc Trindle ne l’a pas copié de nouveau

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = la copie ne correspond pas à l’original

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = il a changé après que vous l’avez pris, donc il est resté où il était

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = Trindle n’a pas pu tout vérifier, donc l’élément est resté où il était

# SAFETY: needs a human review before it ships
# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = Trindle n’a pas pu le supprimer

# SAFETY: needs a human review before it ships
# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = il n’a pas pu être placé dans la corbeille

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
        [one] { $count } fichier illisible
        [many] { $count } fichiers illisibles
       *[other] { $count } fichiers illisibles
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
        [one] { $count } fichier système
        [many] { $count } fichiers système
       *[other] { $count } fichiers système
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
        [one] { $count } élément
        [many] { $count } éléments
       *[other] { $count } éléments
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = certains éléments

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = Trindle n’a pas pu le déplacer : { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = n’a pas pu être déplacé : { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } et { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = un tube

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = un socket

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = un périphérique

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = pas d’autorisation

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = n’a pas pu être lu

# SAFETY: needs a human review before it ships
# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = il est risqué d’utiliser la corbeille sur ce disque

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = rien à déplacer

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = la corbeille contient trop d’éléments portant ce nom

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = L’autre ordinateur ne répond pas.
