# machine translated (Vietnamese), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = nó không có mặt khi kiểm tra, nên đã được để lại

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = quá lớn để kiểm tra hết, nên đã được để lại

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = nó đã thay đổi sau khi được thả vào, nên không có gì được đưa sang

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = trong lúc đó mục khác đã dùng tên này

# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = mục đã có ở đó không thể chuyển vào Thùng rác

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = nó đã thay đổi sau khi được sao chép, nên vẫn ở chỗ cũ

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = ổ đĩa không giữ lại dữ liệu đã ghi, nên một phần có thể không còn ở đó

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = không thể đọc hết nội dung

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = nó nằm bên trong mục mà nó sẽ thay thế

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = bản sao này đã tạo ra nó

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = bản sao không giống với nội dung đã đọc

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = nó đã thay đổi sau khi được nhấc lên, nên vẫn ở chỗ cũ

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = không thể xem hết nội dung của nó, nên vẫn ở chỗ cũ

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = không thể đưa nó đi

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = không thể chuyển nó vào Thùng rác

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
       *[other] { $count } tệp không đọc được
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
       *[other] { $count } tệp hệ thống
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
       *[other] { $count } mục
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = một số mục

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = Chưa di chuyển: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = chưa di chuyển: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } và { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = một đường ống (pipe)

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = một ổ cắm (socket)

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = một thiết bị

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = không có quyền

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = không đọc được

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = thùng rác của ổ này không an toàn để dùng

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = không có gì để chuyển

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = thùng rác có quá nhiều mục trùng tên này

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = Máy tính kia không trả lời.
