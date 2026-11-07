# machine translated (Turkish), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = bakıldığında orada değildi, bu yüzden bırakıldı

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = tamamen denetlenemeyecek kadar büyük, bu yüzden bırakıldı

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = bırakıldıktan sonra değiştiği için hiçbir şey getirilmedi

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = bu arada başka bir şeye bu ad verildi

# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = zaten orada olan öğe Çöp Kutusu'na taşınamadı

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = kopyalandıktan sonra değiştiği için hâlâ eski yerinde

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = sürücü yazılanları saklamadı, bu yüzden bir kısmı orada olmayabilir

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = tamamı okunamadı

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = yerini alacağı öğenin içinde

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = bu kopya onu oluşturdu

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = kopya, okunanla aynı değil

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = alındıktan sonra değiştiği için hâlâ eski yerinde

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = sonuna kadar taranamadığı için hâlâ eski yerinde

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = kaldırılamadı

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = Çöp Kutusu'na taşınamadı

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
        [one] { $count } okunamayan dosya
       *[other] { $count } okunamayan dosya
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
        [one] { $count } sistem dosyası
       *[other] { $count } sistem dosyası
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
        [one] { $count } öğe
       *[other] { $count } öğe
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = bazı öğeler

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = Taşınmadı: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = taşınmadı: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } ve { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = bir boru

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = bir soket

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = bir aygıt

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = izin yok

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = okunamadı

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = bu sürücünün Çöp Kutusu kullanım için güvenli değil

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = taşınacak bir şey yok

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = Çöp Kutusunda bu adla çok fazla öğe var

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = Diğer bilgisayar yanıt vermiyor.
