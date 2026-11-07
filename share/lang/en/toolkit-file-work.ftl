# Words of the work done on files (copy, move, delete, the Trash): the reasons given in the list of what went wrong, and what is said of what stayed.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = it was not there when this was looked at, so it was left

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = too big to check fully, so it was left

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = it has changed since it was dropped, so nothing was brought

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = something else was given that name meanwhile

# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = the one already there could not go to the Trash

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = it changed after it was copied, so it is still where it was

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = the drive would not keep what was written, so some of it may not be there

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = not all of it could be read

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = it is inside the one it would replace

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = this copy made it

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = the copy is not what was read

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = it changed after it was picked up, so it is still where it was

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = it could not be looked through to its end, so it is still where it was

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = it could not be taken away

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = it could not go to the Trash

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
        [one] { $count } unreadable file
       *[other] { $count } unreadable files
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
        [one] { $count } system file
       *[other] { $count } system files
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
        [one] { $count } item
       *[other] { $count } items
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = some items

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = Not moved: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = not moved: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } and { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = a pipe

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = a socket

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = a device

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = no permission

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = could not be read

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = this drive's Trash is not safe to use

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = nothing to move

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = the Trash has too many items with this name

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = The other computer is not answering.
