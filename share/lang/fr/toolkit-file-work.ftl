# machine translated (French), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = il n’était pas là quand on a regardé, il a donc été laissé

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = trop gros pour être vérifié en entier, il a donc été laissé

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = il a changé depuis son dépôt, donc rien n’a été apporté

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = autre chose a reçu ce nom entre-temps

# SAFETY: needs a human review before it ships
# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = celui qui est déjà là n’a pas pu aller à la corbeille

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = il a changé après avoir été copié, il est donc resté où il était

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = le disque n’a pas conservé ce qui a été écrit, donc une partie peut manquer

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = tout n’a pas pu être lu

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = il est à l’intérieur de celui qu’il remplacerait

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = cette copie l’a créé

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = la copie n’est pas identique à ce qui a été lu

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = il a changé après avoir été saisi, il est donc resté où il était

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = il n’a pas pu être examiné jusqu’au bout, il est donc resté où il était

# SAFETY: needs a human review before it ships
# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = il n’a pas pu être retiré

# SAFETY: needs a human review before it ships
# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = il n’a pas pu aller à la corbeille

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
kit-proof-not-moved-said = Non déplacé : { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = non déplacé : { $why }

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
kit-trash-drive-unsafe = la corbeille de ce disque n’est pas sûre

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = rien à déplacer

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = la corbeille contient trop d’éléments portant ce nom

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = L’autre ordinateur ne répond pas.
