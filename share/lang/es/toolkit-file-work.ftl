# machine translated (Spanish), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = no estaba allí cuando se revisó, así que se dejó

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = demasiado grande para revisarlo entero, así que se dejó

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = cambió desde que se soltó, así que no se trajo nada

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = mientras tanto otra cosa recibió ese nombre

# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = el que ya estaba allí no se pudo enviar a la Papelera

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = cambió después de copiarse, así que sigue donde estaba

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = la unidad no conservó lo escrito, así que puede que parte no esté allí

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = no se pudo leer todo

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = está dentro de lo que reemplazaría

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = esta copia lo creó

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = la copia no es lo que se leyó

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = cambió después de elegirlo, así que sigue donde estaba

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = no se pudo revisar hasta el final, así que sigue donde estaba

# Reason, in the words of a notice, why something could not be set aside to be removed.
# SAFETY: needs a human review before it ships
kit-proof-not-taken-away = no se pudo quitar

# Reason, in the words of a notice, why something could not be put in the Trash.
# SAFETY: needs a human review before it ships
kit-proof-not-to-trash = no se pudo mover a la papelera

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
        [one] { $count } archivo ilegible
        [many] { $count } archivos ilegibles
       *[other] { $count } archivos ilegibles
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
        [one] { $count } archivo del sistema
        [many] { $count } archivos del sistema
       *[other] { $count } archivos del sistema
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
        [one] { $count } elemento
        [many] { $count } elementos
       *[other] { $count } elementos
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = algunos elementos

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = No se movió: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = no se movió: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } y { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = una tubería

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = un socket

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = un dispositivo

# Why a thing in a dragged folder stayed: this account may not read it.
# SAFETY: needs a human review before it ships
kit-proof-no-permission = sin permiso

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = no se pudo leer

# SAFETY: needs a human review before it ships
# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = la papelera de esta unidad no es segura de usar

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = no hay nada que mover

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = la papelera tiene demasiados elementos con este nombre

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = El otro equipo no responde.
