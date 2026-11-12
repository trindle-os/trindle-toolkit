# machine translated (German), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = es fehlte, als Trindle nachgesehen hat, deshalb wurde es übersprungen

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = es war zu groß, um es ganz zu prüfen, deshalb wurde es übersprungen

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = es hat sich geändert, nachdem du es abgelegt hast, deshalb wurde es nicht kopiert oder verschoben

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = ein anderes Element hat jetzt diesen Namen
kit-work-drive-no-safe-restore = dieses Laufwerk kann Dateien nicht sicher wiederherstellen; kopiere die Datei stattdessen aus dem Papierkorb

# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = das bereits vorhandene Element konnte nicht in den Papierkorb verschoben werden

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = es hat sich nach dem Kopieren geändert, deshalb hat Trindle es dort gelassen, wo es war

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = das Laufwerk hat nicht alles gespeichert, deshalb fehlt womöglich ein Teil davon

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = Trindle konnte nicht alles davon lesen

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = es liegt in dem Ordner, den es ersetzen würde

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = Trindle hat es bei diesem Kopiervorgang selbst angelegt und deshalb nicht noch einmal kopiert

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = die Kopie stimmt nicht mit dem Original überein

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = es hat sich geändert, nachdem du es aufgenommen hast, deshalb liegt es noch dort, wo es war

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = Trindle konnte es nicht ganz prüfen, deshalb liegt es noch dort, wo es war

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = Trindle konnte es nicht entfernen

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = es konnte nicht in den Papierkorb verschoben werden

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
        [one] { $count } nicht lesbare Datei
       *[other] { $count } nicht lesbare Dateien
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
        [one] { $count } Systemdatei
       *[other] { $count } Systemdateien
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
        [one] { $count } Objekt
       *[other] { $count } Objekte
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = einige Objekte

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = Trindle konnte das nicht verschieben: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = konnte nicht verschoben werden: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } und { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = eine Pipe

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = ein Socket

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = ein Gerät

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = keine Berechtigung

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = konnte nicht gelesen werden

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = der Papierkorb auf diesem Laufwerk lässt sich nicht sicher verwenden

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = nichts zu verschieben

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = der Papierkorb enthält zu viele Einträge mit diesem Namen

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = Der andere Computer reagiert nicht.
