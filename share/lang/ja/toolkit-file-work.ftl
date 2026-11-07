# machine translated (Japanese), not yet reviewed by a native speaker.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = 確認したときには存在しなかったため、残しました

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = 大きすぎて完全に確認できないため、残しました

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = ドロップされた後に変更されたため、何も取り込みませんでした

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = その間に別のものがその名前になりました

# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = 既にあるものをごみ箱に入れられませんでした

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = コピーの後に変更されたため、元の場所に残っています

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = ドライブが書き込まれた内容を保持しなかったため、一部が存在しない可能性があります

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = すべてを読み取れませんでした

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = 置き換えようとしているものの中にあります

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = このコピーが作成したものです

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = コピーが読み取った内容と一致しません

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = 手に取った後に変更されたため、元の場所に残っています

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = 最後まで確認できなかったため、元の場所に残っています

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = 取り除けませんでした

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = ごみ箱に入れられませんでした

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
       *[other] { $count } 個の読み取れないファイル
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
       *[other] { $count } 個のシステムファイル
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
       *[other] { $count } 個の項目
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = いくつかの項目

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = 移動していません: { $why }。

# The same in the brackets after a name.
kit-proof-not-moved-reason = 移動していません: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } と { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = パイプ

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = ソケット

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = デバイス

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = 権限がありません

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = 読み取れませんでした

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = このドライブのごみ箱は安全に使えません

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = 移動するものがありません

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = この名前の項目がごみ箱に多すぎます

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = 相手のコンピューターが応答していません。
