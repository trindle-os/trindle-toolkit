# machine translated (Russian), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = при просмотре его не было, поэтому оно осталось

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = слишком велико для полной проверки, поэтому оно осталось

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = оно изменилось после того, как его бросили, поэтому ничего не перенесено

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = тем временем это имя заняло что-то другое

# Reason nothing was replaced: the one to make way could not be put in the Trash.
# SAFETY: needs a human review before it ships
kit-work-no-trash = то, что уже лежит на этом месте, не удалось переместить в Корзину

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = оно изменилось после копирования, поэтому осталось на прежнем месте

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = диск не подтвердил запись, поэтому часть данных может отсутствовать

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = не удалось прочитать всё

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = оно находится внутри того, что оно заменило бы

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = это сделала данная копия

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = копия не совпадает с прочитанным

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = оно изменилось после того, как его взяли, поэтому осталось на месте

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = её не удалось просмотреть до конца, поэтому она осталась на месте

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = это не удалось убрать

# Reason, in the words of a notice, why something could not be put in the Trash.
# SAFETY: needs a human review before it ships
kit-proof-not-to-trash = это не удалось переместить в Корзину

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
        [one] { $count } нечитаемый файл
        [few] { $count } нечитаемых файла
        [many] { $count } нечитаемых файлов
       *[other] { $count } нечитаемого файла
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
        [one] { $count } системный файл
        [few] { $count } системных файла
        [many] { $count } системных файлов
       *[other] { $count } системного файла
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
        [one] { $count } элемент
        [few] { $count } элемента
        [many] { $count } элементов
       *[other] { $count } элемента
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = часть элементов

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = Не перемещено: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = не перемещено: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } и { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = канал (pipe)

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = сокет

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = устройство

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = нет разрешения

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = не удалось прочитать

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = Корзина на этом диске небезопасна

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = нечего перемещать

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = в Корзине слишком много элементов с таким именем

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = Другой компьютер не отвечает.
