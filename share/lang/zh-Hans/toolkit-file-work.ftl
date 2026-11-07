# machine translated (Simplified Chinese), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name }（{ $why }）

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = 查看时它不在那里，所以已保留不动

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = 太大，无法完整检查，所以已保留不动

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = 它在被拖放之后发生了变化，所以没有带入任何内容

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = 在此期间，别的东西已使用了这个名称

# SAFETY: needs a human review before it ships
# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = 已在那里的这一项无法移到回收站

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = 它在被复制之后发生了变化，所以仍留在原处

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = 驱动器没有保存所写入的内容，所以有些内容可能不在那里

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = 无法读取全部内容

# SAFETY: needs a human review before it ships
# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = 它位于将被它替换的那一项里面

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = 这个副本创建了它

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = 副本与读取到的内容不一致

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = 它在被拿起后发生了变化，所以仍留在原处

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = 无法查看到它的末尾，所以仍留在原处

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = 无法移走

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = 无法移到回收站

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
kit-proof-not-moved-said = 未移动：{ $why }。

# SAFETY: needs a human review before it ships
# The same in the brackets after a name.
kit-proof-not-moved-reason = 未移动：{ $why }

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
kit-trash-drive-unsafe = 这个驱动器的回收站使用起来不安全

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = 没有可移动的内容

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = 回收站里同名的项目太多了

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = 另一台电脑没有响应。
