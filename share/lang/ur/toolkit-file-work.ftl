# machine translated (Urdu), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = فولڈر دیکھتے وقت یہ موجود نہیں تھی، اس لیے اسے چھوڑ دیا گیا

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = مکمل جانچ کے لیے بہت بڑا ہے، اس لیے اسے چھوڑ دیا گیا

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = چھوڑے جانے کے بعد یہ بدل گئی، اس لیے کچھ نہیں لایا گیا

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = اس دوران کسی اور چیز کو یہ نام دے دیا گیا

# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = پہلے سے موجود چیز کو ردی دان میں نہیں ڈالا جا سکا

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = نقل کے بعد یہ بدل گئی، اس لیے یہ اب بھی اپنی جگہ پر ہے

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = ڈرائیو نے لکھا ہوا محفوظ کرنے کی تصدیق نہیں کی، اس لیے ہو سکتا ہے کچھ حصہ وہاں نہ ہو

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = اس کا سارا حصہ پڑھا نہیں جا سکا

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = یہ اس چیز کے اندر ہے جسے یہ بدلتی

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = یہ اس نقل نے بنائی تھی

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = نقل اس سے مختلف ہے جو پڑھا گیا تھا

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = اٹھانے کے بعد یہ بدل گئی، اس لیے یہ اب بھی اپنی جگہ پر ہے

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = اسے آخر تک دیکھا نہیں جا سکا، اس لیے یہ اب بھی اپنی جگہ پر ہے

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = اسے ہٹایا نہیں جا سکا

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = اسے ردی دان میں نہیں ڈالا جا سکا

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
        [one] { $count } ناقابل مطالعہ فائل
       *[other] { $count } ناقابل مطالعہ فائلیں
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
        [one] { $count } سسٹم فائل
       *[other] { $count } سسٹم فائلیں
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
        [one] { $count } چیز
       *[other] { $count } چیزیں
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = کچھ چیزیں

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = منتقل نہیں ہوئیں: { $why }۔

# The same in the brackets after a name.
kit-proof-not-moved-reason = منتقل نہیں ہوئیں: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } اور { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = ایک پائپ

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = ایک ساکٹ

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = ایک ڈیوائس

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = اجازت نہیں

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = پڑھی نہیں جا سکی

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = اس ڈرائیو کا ردی دان محفوظ نہیں ہے

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = منتقل کرنے کے لیے کچھ نہیں

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = ردی دان میں اس نام کی بہت زیادہ چیزیں ہیں

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = دوسرا کمپیوٹر جواب نہیں دے رہا۔
