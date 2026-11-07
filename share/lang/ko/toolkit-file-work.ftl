# machine translated (Korean), not yet reviewed by a native speaker.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = 확인할 때 없었던 것이라 남겨 두었습니다

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = 너무 커서 끝까지 확인할 수 없어 남겨 두었습니다

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = 끌어다 놓은 뒤 바뀌어서 아무것도 가져오지 않았습니다

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = 그 사이에 다른 것이 그 이름을 쓰게 되었습니다

# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = 이미 있는 것을 휴지통에 넣을 수 없었습니다

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = 복사한 뒤 바뀌어서 원래 자리에 그대로 있습니다

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = 드라이브가 쓴 내용을 보관하지 않아 일부가 없을 수 있습니다

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = 전부 읽을 수 없었습니다

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = 바꾸려는 것의 안에 있습니다

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = 이 복사에서 만든 것입니다

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = 복사본이 읽은 내용과 다릅니다

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = 집어 든 뒤 바뀌어서 원래 자리에 그대로 있습니다

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = 끝까지 확인할 수 없어 원래 자리에 그대로 있습니다

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = 없앨 수 없었습니다

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = 휴지통에 넣을 수 없었습니다

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
       *[other] 읽을 수 없는 파일 { $count }개
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
       *[other] 시스템 파일 { $count }개
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
       *[other] 항목 { $count }개
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = 일부 항목

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = 옮기지 않음: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = 옮기지 않음: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } 및 { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = 파이프

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = 소켓

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = 장치

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = 권한 없음

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = 읽을 수 없음

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = 이 드라이브의 휴지통은 안전하게 쓸 수 없습니다

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = 옮길 것이 없습니다

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = 휴지통에 이 이름의 항목이 너무 많습니다

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = 상대 컴퓨터가 응답하지 않습니다.
