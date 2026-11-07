# machine translated (Arabic), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = لم يكن موجودًا عند النظر فيه، فتُرك

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = أكبر من أن يُفحص كاملًا، فتُرك

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = تغيّر منذ أن أُسقط، فلم يُجلب شيء

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = أُعطي شيء آخر هذا الاسم في هذه الأثناء

# Reason nothing was replaced: the one to make way could not be put in the Trash.
# SAFETY: needs a human review before it ships
kit-work-no-trash = تعذّر نقل الموجود هناك إلى سلة المهملات

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = تغيّر بعد نسخه، فهو ما زال حيث كان

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = لم يحتفظ القرص بما كُتب، وقد يكون بعضه غير موجود

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = تعذّرت قراءته كله

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = هو داخل الذي سيستبدله

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = صنعته هذه النسخة

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = النسخة ليست ما قُرئ

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = تغيّر بعد التقاطه، لذلك لا يزال حيث كان

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = تعذّر فحصه حتى نهايته، لذلك لا يزال حيث كان

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = تعذّرت إزالته

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = تعذّر وضعه في سلة المحذوفات

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
        [zero] لا ملفات غير مقروءة
        [one] ملف واحد غير مقروء
        [two] ملفان غير مقروءين
        [few] { $count } ملفات غير مقروءة
        [many] { $count } ملفاً غير مقروء
       *[other] { $count } ملف غير مقروء
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
        [zero] لا ملفات نظام
        [one] ملف نظام واحد
        [two] ملفا نظام
        [few] { $count } ملفات نظام
        [many] { $count } ملف نظام
       *[other] { $count } ملف نظام
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
        [zero] لا عناصر
        [one] عنصر واحد
        [two] عنصران
        [few] { $count } عناصر
        [many] { $count } عنصراً
       *[other] { $count } عنصر
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = بعض العناصر

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = لم يُنقل: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = لم يُنقل: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } و{ $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = أنبوب

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = مقبس

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = جهاز

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = لا إذن

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = تعذّرت قراءته

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = سلة المهملات في هذا القرص غير آمنة للاستخدام

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = لا شيء لنقله

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = في سلة المهملات عناصر كثيرة جدًا بهذا الاسم

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = الحاسوب الآخر لا يستجيب.
