# machine translated (Arabic), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = كان مفقودًا عندما فحص Trindle، فتم تجاوزه

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = كان أكبر من أن يُفحص كاملًا، فتم تجاوزه

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = تغيّر بعد أن أسقطته، فلم يُنسخ أو يُنقل

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = يوجد الآن عنصر آخر بهذا الاسم
kit-work-drive-no-safe-restore = لا يستطيع هذا القرص استعادة الملفات بأمان، لذا انسخ الملف من سلة المحذوفات بدلًا من ذلك

# Reason nothing was replaced: the one to make way could not be put in the Trash.
# SAFETY: needs a human review before it ships
kit-work-no-trash = تعذّر نقل العنصر الموجود بالفعل إلى سلة المحذوفات

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = تغيّر بعد نسخه، لذا تركه Trindle حيث كان

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = لم يحفظ القرص كل شيء، فقد يكون بعضه مفقودًا

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = تعذّر على Trindle قراءته كله

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = هو داخل المجلد الذي سيحل محله

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = أنشأه Trindle أثناء هذا النسخ، لذا لم ينسخه مرة أخرى

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = النسخة لا تطابق الأصل

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = تغيّر بعد أن التقطته، فهو لا يزال حيث كان

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = تعذّر على Trindle فحصه كله، فهو لا يزال حيث كان

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = تعذّر على Trindle إزالته

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = تعذّر نقله إلى سلة المحذوفات

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
kit-proof-not-moved-said = تعذّر على Trindle نقل هذا: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = تعذّر نقله: { $why }

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
kit-trash-drive-unsafe = ليس من الآمن استخدام سلة المحذوفات في هذا القرص

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = لا شيء لنقله

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = في سلة المهملات عناصر كثيرة جدًا بهذا الاسم

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = الحاسوب الآخر لا يستجيب.
