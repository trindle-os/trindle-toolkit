# machine translated (Simplified Chinese), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name }（{ $why }）

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = Trindle 检查时它已经不在了，所以跳过了它

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = 它太大，无法完整检查，所以跳过了

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = 它在拖入后发生了变化，所以没有复制或移动

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = 现在已有其他项目使用这个名称
kit-work-drive-no-safe-restore = 此磁盘无法安全还原文件，请改为把文件从回收站中复制出来

# SAFETY: needs a human review before it ships
# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = 已有的那个项目无法移到回收站

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = 它在复制后发生了变化，所以 Trindle 把它留在了原处

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = 磁盘没有保存全部内容，所以其中一部分可能丢失

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = Trindle 无法读取全部内容

# SAFETY: needs a human review before it ships
# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = 它位于将被替换的文件夹里面

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = Trindle 在这次复制中创建了它，所以没有再次复制

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = 复制的内容与原件不一致

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = 它在被选取后发生了变化，所以仍留在原处

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = Trindle 无法检查它的全部内容，所以它仍留在原处

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = Trindle 无法移除它

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = 无法将它移到回收站

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
       *[other] { $count } 个无法读取的文件
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
       *[other] { $count } 个系统文件
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
       *[other] { $count } 项
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = 部分项目

# SAFETY: needs a human review before it ships
# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = Trindle 无法移动它：{ $why }。

# SAFETY: needs a human review before it ships
# The same in the brackets after a name.
kit-proof-not-moved-reason = 无法移动：{ $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first }和{ $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = 管道

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = 套接字

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = 设备

# SAFETY: needs a human review before it ships
# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = 没有权限

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = 无法读取

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = 在此磁盘上使用回收站不安全

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = 没有可移动的内容

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = 回收站里同名的项目太多了

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = 另一台电脑没有响应。
