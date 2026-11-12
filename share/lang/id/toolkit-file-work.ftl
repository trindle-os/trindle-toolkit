# machine translated (Indonesian), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = tidak ada saat Trindle memeriksa, jadi dilewati

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = terlalu besar untuk diperiksa penuh, jadi dilewati

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = berubah setelah Anda melepasnya, jadi tidak disalin atau dipindahkan

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = item lain kini memakai nama itu
kit-work-drive-no-safe-restore = drive ini tidak dapat memulihkan file dengan aman, jadi salin file dari Tempat Sampah sebagai gantinya

# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = item yang sudah ada di sana tidak dapat dipindahkan ke Tempat Sampah

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = berubah setelah disalin, jadi Trindle membiarkannya di tempat semula

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = drive tidak menyimpan semuanya, jadi sebagian mungkin hilang

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = Trindle tidak dapat membaca semuanya

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = berada di dalam folder yang akan diganti olehnya

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = Trindle membuatnya selama penyalinan ini, jadi tidak menyalinnya lagi

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = salinan tidak sama dengan aslinya

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = berubah setelah Anda mengambilnya, jadi masih di tempat semula

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = Trindle tidak dapat memeriksa semuanya, jadi masih di tempat semula

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = Trindle tidak dapat menghapusnya

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = tidak dapat dipindahkan ke Tempat Sampah

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
kit-proof-not-moved-said = Trindle tidak dapat memindahkan ini: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = tidak dapat dipindahkan: { $why }

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
kit-trash-drive-unsafe = tidak aman memakai Tempat Sampah di drive ini

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = tidak ada yang dipindahkan

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = Tempat Sampah memiliki terlalu banyak item dengan nama ini

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = Komputer lain tidak merespons.
