# machine translated (Swahili), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = haikuwepo wakati huu ulipokaguliwa, kwa hiyo iliachwa

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = ni kubwa mno kukaguliwa kikamilifu, kwa hiyo iliachwa

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = ilibadilika baada ya kuachwa hapa, kwa hiyo hakuna kilicholetwa

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = kitu kingine kilipewa jina hilo kwa wakati huo

# Reason nothing was replaced: the one to make way could not be put in the Trash.
# SAFETY: needs a human review before it ships
kit-work-no-trash = ile iliyokuwepo haikuweza kwenda Tupio

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = ilibadilika baada ya kunakiliwa, kwa hiyo bado iko mahali pake

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = diski haikuhifadhi kilichoandikwa, kwa hiyo sehemu yake huenda haipo

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = haikuweza kusomwa yote

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = iko ndani ya ile ingebadilisha

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = nakala hii iliifanya, kwa hiyo ilikuwepo tayari

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = nakala si sawa na kilichosomwa

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = ilibadilika baada ya kuchaguliwa, kwa hiyo bado iko mahali pake

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = haikuweza kukaguliwa hadi mwisho, kwa hiyo bado iko mahali pake

# Reason, in the words of a notice, why something could not be set aside to be removed.
# SAFETY: needs a human review before it ships
kit-proof-not-taken-away = haikuweza kuondolewa

# Reason, in the words of a notice, why something could not be put in the Trash.
# SAFETY: needs a human review before it ships
kit-proof-not-to-trash = haikuweza kwenda Tupio

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
        [one] faili { $count } isiyosomeka
       *[other] faili { $count } zisizosomeka
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
        [one] faili { $count } ya mfumo
       *[other] faili { $count } za mfumo
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
        [one] kitu { $count }
       *[other] vitu { $count }
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = baadhi ya vitu

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = Havikuhamishwa: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = haikuhamishwa: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } na { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = bomba (pipe)

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = soketi

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = kifaa

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = hakuna ruhusa

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = haikuweza kusomwa

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = Tupio la diski hii si salama kutumia

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = hakuna cha kuhamisha

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = Tupio lina vitu vingi mno vyenye jina hili

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = Kompyuta nyingine haijibu.
