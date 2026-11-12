# machine translated (Brazilian Portuguese), not yet reviewed by a person.

# One line in the list of what went wrong: the name of the item, then in brackets why.
kit-work-trouble = { $name } ({ $why })

# Reason for something left behind: a hidden thing or a link was not there when the folder was looked through.
kit-work-not-looked = estava ausente quando o Trindle verificou, então foi ignorado

# Reason for something left behind: a hidden thing or a link was in a folder too big to look through to its end.
kit-work-too-big = era grande demais para verificar por inteiro, então foi ignorado

# Reason nothing was brought: something another program dropped changed after it was dropped.
kit-work-changed-since = mudou depois que você o soltou, então não foi copiado nem movido

# Reason a thing was not given its name: something else took the name after the folder was looked at.
kit-work-taken = outro item agora tem esse nome
kit-work-drive-no-safe-restore = esta unidade não consegue restaurar arquivos com segurança, então copie o arquivo para fora da Lixeira

# Reason nothing was replaced: the one to make way could not be put in the Trash.
kit-work-no-trash = o item que já estava lá não pôde ser movido para a Lixeira

# Reason a moved item was not taken away from where it was: it changed after it was copied.
kit-work-stayed = mudou depois de copiado, então o Trindle o deixou onde estava

# Reason given for a folder copied to a drive that would not say everything written had reached it.
kit-work-not-kept = a unidade não salvou tudo, então parte pode estar faltando

# Reason a folder is said to have failed: its list stopped part way.
kit-work-not-all = o Trindle não conseguiu ler tudo

# Reason a thing was not moved or copied: it is inside the one it would replace.
kit-work-inside-replaced = está dentro da pasta que ele substituiria

# Reason a thing was not brought: a copy made it, so it was already there.
kit-work-copy-made-it = o Trindle o criou durante esta cópia, então não o copiou de novo

# Reason a file copy was refused: what was written is not what was read.
kit-work-copy-wrong = a cópia não confere com o original

# Reason, in the words of a notice, why a thing dragged to another program was not moved: it was changed after it was picked up.
kit-proof-not-as-picked = mudou depois que você o pegou, então continua onde estava

# Reason, in the words of a notice, why a folder dragged to another program was not moved: it could not be looked through to its end.
kit-proof-not-looked-through = o Trindle não conseguiu verificar tudo, então continua onde estava

# Reason, in the words of a notice, why something could not be set aside to be removed.
kit-proof-not-taken-away = o Trindle não conseguiu removê-lo

# Reason, in the words of a notice, why something could not be put in the Trash.
kit-proof-not-to-trash = não pôde ser movido para a Lixeira

# Count in a notice of unreadable files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-unreadable =
    { $n ->
        [one] { $count } arquivo ilegível
       *[other] { $count } arquivos ilegíveis
        [many] { $count } arquivos ilegíveis
    }

# Count in a notice of system files that stayed; n picks the form and count is the number as written, such as 1,200.
kit-proof-count-system =
    { $n ->
        [one] { $count } arquivo do sistema
       *[other] { $count } arquivos do sistema
        [many] { $count } arquivos do sistema
    }

# Count in a notice of things that stayed for another reason; n picks the form and count is the number as written.
kit-proof-count-other =
    { $n ->
        [one] { $count } item
       *[other] { $count } itens
        [many] { $count } itens
    }

# Count when a folder moved in part and the number is not known.
kit-proof-count-some = alguns itens

# The sentence in red for everything a move, copy or delete left where it was; why is the counts.
kit-proof-not-moved-said = O Trindle não conseguiu mover isto: { $why }.

# The same in the brackets after a name.
kit-proof-not-moved-reason = não pôde ser movido: { $why }

# Joins the counts in a notice of what stayed: the earlier ones, then the last.
kit-proof-count-both = { $first } e { $second }

# Why a thing in a dragged folder stayed: it is a pipe.
kit-proof-pipe = um pipe

# Why a thing in a dragged folder stayed: it is a socket.
kit-proof-socket = um socket

# Why a thing in a dragged folder stayed: it is a device.
kit-proof-device = um dispositivo

# Why a thing in a dragged folder stayed: this account may not read it.
# SAFETY: needs a human review before it ships
kit-proof-no-permission = sem permissão

# Why a thing in a dragged folder stayed: it could not be read for another reason.
kit-proof-unreadable = não pôde ser lido

# Reason shown when an item cannot be moved to the Trash because the Trash folder on its drive could be used by other people.
kit-trash-drive-unsafe = não é seguro usar a Lixeira desta unidade

# Reason shown when an item to move to the Trash has no name to move.
kit-trash-nothing-to-move = nada para mover

# Reason shown when an item cannot be moved to the Trash because the Trash already holds too many items with the same name.
kit-trash-too-many-names = a Lixeira tem itens demais com este nome

# Note shown when the other computer or a network place does not answer.
kit-reach-not-answering = O outro computador não está respondendo.
