# Translating Trindle

Every word Trindle shows is in a plain text file with the ending `.ftl`. A language is a folder of these files, and the folder is named after the language. English is the folder `en`, and it is the master. You do not need any programs except a text editor.

## Add a language

1. Copy the folder `en` and name the copy after the language: `fr` for French, `pt-BR` for Brazilian Portuguese, `zh-Hans` for simplified Chinese.
2. Inside the copy, each file holds the words of one part of Trindle (`toolkit.ftl`, `report.ftl`, `ring.ftl` and so on). Translate only the right-hand side, the words after the `=`. Leave the key on the left alone, and leave anything inside `{ }` alone too (see "Values" below). Keep the file names; you may translate some files and leave out the rest.
3. Lines that start with `#` are notes for you. They say where each word is shown. You can delete them in your copy.
4. Put the folder in `~/.config/trindle/lang/` (so the files are in `~/.config/trindle/lang/fr/`) and log out and in. Trindle reads every `.ftl` file in the folder, in the order of their names, as one list. A word you have not translated shows in English, so you can send a folder that is half done.
5. Run `trindle-lang-check` to see what is missing or wrong; `trindle-lang-check --missing fr` lists every word French still lacks, and its share of English done.

If Trindle is installed in the usual way there are two English folders: this one's (the toolkit's words, keys that start `kit-`) and Trindle's own, `share/lang/en/` in the main folder. Translate both, into two folders with the same language name.

A key may be in only one file of a folder. The check says so if it is in two.

## Values

Some words have a place for something Trindle fills in, written like `{ $name }` or `{ $n }`. Keep every one of them, spelled the same, but put it wherever it reads best in your language.

    report-saved-as = Saved as Downloads/{ $name }.
    report-saved-as = Gespeichert als Downloads/{ $name }.

## Plurals

When a word changes with a number, the file lists a line for each form. Write the forms your language needs, and always keep the line that starts with `*`: it is the one used when no other fits.

English has two forms, `one` and `other`:

    kit-more-lines =
        { $n ->
            [one] +{ $n } line
           *[other] +{ $n } lines
        }

Russian has `one` (1, 21, 101), `few` (2 to 4, 22 to 24), `many` (5 to 20, 25 to 30) and `other` (for numbers with a decimal point):

    kit-more-lines =
        { $n ->
            [one] +{ $n } строка
            [few] +{ $n } строки
            [many] +{ $n } строк
           *[other] +{ $n } строки
        }

The forms a language uses are the ones in the Unicode plural rules for it (search for "CLDR plural rules"). Chinese, Japanese and Korean have only `other`. You can also match an exact number, such as `[0] No lines`, and that line wins over the form.

A few words choose between named forms instead of numbers, such as the battery's state. They look the same, with a name in the brackets (`[charging]`, `[onbattery]`). Keep the names and translate what follows them.

## Gender

Some languages change a word with the gender of the person it speaks of. The program hands over the gender (`male`, `female` or anything else), and the file chooses a line the same way it does for a plural, with a named line for each and a `*[other]` line used for all the rest:

    kit-invited =
        { $gender ->
            [male] Он пригласил вас
            [female] Она пригласила вас
           *[other] Они пригласили вас
        }

A gender choice can sit inside a plural choice, or the other way round. A language with no gendered words leaves the choice out and writes one plain line. A language that needs it where English has none can add it, and the program still gives the gender.

## Rules for the words

- Keep the English meaning, and keep each one short. They go on small buttons.
- Use the way your language writes a button or a menu: sentence case where your language has capitals (only the first word starts with a capital).
- Do not add or remove a key. If a key is not needed in your language, still translate it.
- Do not put a link, a tag or anything else in the words. They are plain text.
- A message may run over several lines. Keep the lines indented under the key, as in English.

## What the check says

`trindle-lang-check` lists, for each language folder: the keys it lacks (a note, not a fault), keys English does not have, a key in two files, and values that do not match English. It also fails when English and Trindle's code disagree, which only matters to the people building Trindle.
