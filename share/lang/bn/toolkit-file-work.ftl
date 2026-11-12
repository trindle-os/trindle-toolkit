# machine translated (Bengali), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = Trindle যখন দেখেছিল তখন এটি ছিল না, তাই বাদ দেওয়া হয়েছে

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = পুরোপুরি যাচাই করার জন্য এটি অনেক বড় ছিল, তাই বাদ দেওয়া হয়েছে

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = আপনি ফেলার পর এটি বদলে গেছে, তাই কপি বা সরানো হয়নি

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = এখন অন্য একটি আইটেমের সেই নাম আছে
kit-work-drive-no-safe-restore = এই ড্রাইভ নিরাপদে ফাইল ফিরিয়ে আনতে পারে না, তাই ফাইলটি ট্র্যাশ থেকে কপি করে নিন

# Reason nothing was replaced: the one to make way could not be put in the Trash.
# SAFETY: needs a human review before it ships
kit-work-no-trash = আগে থেকে থাকা আইটেমটি ট্র্যাশে সরানো যায়নি

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = কপির পর এটি বদলে গেছে, তাই Trindle এটি আগের জায়গাতেই রেখে দিয়েছে

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = ড্রাইভ সবকিছু সংরক্ষণ করেনি, তাই কিছু অংশ না-ও থাকতে পারে

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = Trindle সবটা পড়তে পারেনি

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = এটি যে ফোল্ডারটি বদলে দিত, তারই ভেতরে আছে

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = এই কপির সময় Trindle এটি বানিয়েছে, তাই আর কপি করেনি

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = কপিটি মূল ফাইলের সঙ্গে মেলে না

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = আপনি তোলার পর এটি বদলে গেছে, তাই এটি আগের জায়গাতেই আছে

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = Trindle পুরোটা যাচাই করতে পারেনি, তাই এটি আগের জায়গাতেই আছে

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = Trindle এটি সরাতে পারেনি

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = এটি ট্র্যাশে সরানো যায়নি

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
        [one] { $count }টি পড়া-অযোগ্য ফাইল
       *[other] { $count }টি পড়া-অযোগ্য ফাইল
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
        [one] { $count }টি সিস্টেম ফাইল
       *[other] { $count }টি সিস্টেম ফাইল
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
        [one] { $count }টি জিনিস
       *[other] { $count }টি জিনিস
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = কয়েকটি জিনিস

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = Trindle এটি সরাতে পারেনি: { $why }।

# The same in the brackets after a name.
kit-proof-not-moved-reason = সরানো যায়নি: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } এবং { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = একটি পাইপ

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = একটি সকেট

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = একটি ডিভাইস

# Why a thing in a dragged folder stayed: this account may not read it.
# SAFETY: needs a human review before it ships
kit-proof-no-permission = অনুমতি নেই

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = পড়া যায়নি

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = এই ড্রাইভে ট্র্যাশ ব্যবহার করা নিরাপদ নয়

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = সরানোর মতো কিছু নেই

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = ট্র্যাশে এই নামের অনেক বেশি জিনিস আছে

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = অন্য কম্পিউটারটি সাড়া দিচ্ছে না।
