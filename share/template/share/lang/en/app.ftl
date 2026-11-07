## The program's words in English. Translations go in share/lang/<code>/; check them with trindle-lang-check.

# The window's title.
app-title = New program
# The words above the button. $n is how many times it was pressed.
app-pressed =
    { $n ->
        [0] Not pressed yet
        [one] Pressed once
       *[other] Pressed { $n } times
    }
# The button.
app-button = Press me
