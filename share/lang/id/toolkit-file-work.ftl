# machine translated (Indonesian), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = tidak ada saat diperiksa, jadi dibiarkan

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = terlalu besar untuk diperiksa sepenuhnya, jadi dibiarkan

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = berubah setelah dijatuhkan, jadi tidak ada yang dibawa

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = nama itu sementara itu sudah dipakai oleh yang lain

# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = yang sudah ada tidak dapat dimasukkan ke Tempat Sampah

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = berubah setelah disalin, jadi masih di tempat semula

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = drive tidak menyimpan yang ditulis, jadi sebagian mungkin tidak ada

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = tidak semuanya dapat dibaca

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = berada di dalam yang akan digantikannya

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = salinan ini yang membuatnya

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = salinan tidak sama dengan yang dibaca

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = berubah setelah diambil, jadi masih di tempat semula

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = tidak dapat diperiksa sampai tuntas, jadi masih di tempat semula

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = tidak dapat disingkirkan

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = tidak dapat dimasukkan ke Tempat Sampah

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
       *[other] { $count } file tidak terbaca
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
       *[other] { $count } file sistem
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
       *[other] { $count } item
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = beberapa item

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = Tidak dipindahkan: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = tidak dipindahkan: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } dan { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = sebuah pipa

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = sebuah soket

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = sebuah perangkat

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = tidak ada izin

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = tidak dapat dibaca

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = Tempat Sampah drive ini tidak aman digunakan

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = tidak ada yang dipindahkan

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = Tempat Sampah memiliki terlalu banyak item dengan nama ini

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = Komputer lain tidak menjawab.
