# machine translated (Hindi), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = देखते समय यह वहाँ नहीं था, इसलिए इसे छोड़ दिया गया

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = पूरी तरह जाँचने के लिए बहुत बड़ा है, इसलिए इसे छोड़ दिया गया

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = छोड़े जाने के बाद यह बदल गया, इसलिए कुछ नहीं लाया गया

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = इस बीच किसी और चीज़ को यही नाम दे दिया गया

# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = वहाँ पहले से मौजूद चीज़ ट्रैश में नहीं जा सकी

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = कॉपी होने के बाद यह बदल गया, इसलिए यह अब भी अपनी जगह पर है

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = ड्राइव ने लिखी गई चीज़ें नहीं रखीं, इसलिए हो सकता है उनमें से कुछ वहाँ न हों

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = सब कुछ पढ़ा नहीं जा सका

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = यह उसी के अंदर है जिसकी जगह यह लेती

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = यह कॉपी इसी ने बनाई

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = कॉपी वैसी नहीं है जैसी पढ़ी गई थी

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = उठाए जाने के बाद यह बदल गया, इसलिए यह अब भी अपनी जगह पर है

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = इसे अंत तक देखा नहीं जा सका, इसलिए यह अब भी अपनी जगह पर है

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = इसे हटाया नहीं जा सका

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = यह ट्रैश में नहीं जा सका

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
        [one] { $count } न पढ़ी जा सकने वाली फ़ाइल
       *[other] { $count } न पढ़ी जा सकने वाली फ़ाइलें
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
        [one] { $count } सिस्टम फ़ाइल
       *[other] { $count } सिस्टम फ़ाइलें
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
        [one] { $count } चीज़
       *[other] { $count } चीज़ें
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = कुछ चीज़ें

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = हटाया नहीं गया: { $why }।

# The same in the brackets after a name.
kit-proof-not-moved-reason = हटाया नहीं गया: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } और { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = एक पाइप

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = एक सॉकेट

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = एक डिवाइस

# Why a thing in a dragged folder stayed: this account may not read it.
kit-proof-no-permission = अनुमति नहीं है

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = पढ़ा नहीं जा सका

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = इस ड्राइव का ट्रैश इस्तेमाल करना सुरक्षित नहीं है

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = हटाने के लिए कुछ नहीं है

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = ट्रैश में इस नाम की बहुत सारी चीज़ें हैं

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = दूसरा कंप्यूटर जवाब नहीं दे रहा।
