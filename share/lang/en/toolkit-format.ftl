# Dates, times, numbers and sizes, in English (lang/format.rs). A language
# writes these its own way: the names of days and months, the patterns that
# order them, the marks inside a number and the unit words of a size.
#
# Patterns use %A (weekday), %a (short weekday), %B (month), %b (short month),
# %d (day, two figures), %-d (day, no leading nought), %m, %Y, %H, %M, %S, %I,
# %p (am or pm), and the letters between them are copied as they stand.

## Weekdays

# Name of a weekday, in a date or a clock line.
kit-date-weekday-sun = Sunday

# Name of a weekday, in a date or a clock line.
kit-date-weekday-mon = Monday

# Name of a weekday, in a date or a clock line.
kit-date-weekday-tue = Tuesday

# Name of a weekday, in a date or a clock line.
kit-date-weekday-wed = Wednesday

# Name of a weekday, in a date or a clock line.
kit-date-weekday-thu = Thursday

# Name of a weekday, in a date or a clock line.
kit-date-weekday-fri = Friday

# Name of a weekday, in a date or a clock line.
kit-date-weekday-sat = Saturday

## Short weekdays

# Short name of a weekday, in a list of times.
kit-date-weekday-short-sun = Sun

# Short name of a weekday, in a list of times.
kit-date-weekday-short-mon = Mon

# Short name of a weekday, in a list of times.
kit-date-weekday-short-tue = Tue

# Short name of a weekday, in a list of times.
kit-date-weekday-short-wed = Wed

# Short name of a weekday, in a list of times.
kit-date-weekday-short-thu = Thu

# Short name of a weekday, in a list of times.
kit-date-weekday-short-fri = Fri

# Short name of a weekday, in a list of times.
kit-date-weekday-short-sat = Sat

## Months

# Name of a month, in a date.
kit-date-month-jan = January

# Name of a month, in a date.
kit-date-month-feb = February

# Name of a month, in a date.
kit-date-month-mar = March

# Name of a month, in a date.
kit-date-month-apr = April

# Name of a month, in a date.
kit-date-month-may = May

# Name of a month, in a date.
kit-date-month-jun = June

# Name of a month, in a date.
kit-date-month-jul = July

# Name of a month, in a date.
kit-date-month-aug = August

# Name of a month, in a date.
kit-date-month-sep = September

# Name of a month, in a date.
kit-date-month-oct = October

# Name of a month, in a date.
kit-date-month-nov = November

# Name of a month, in a date.
kit-date-month-dec = December

## Short months

# Short name of a month, in a date.
kit-date-month-short-jan = Jan

# Short name of a month, in a date.
kit-date-month-short-feb = Feb

# Short name of a month, in a date.
kit-date-month-short-mar = Mar

# Short name of a month, in a date.
kit-date-month-short-apr = Apr

# Short name of a month, in a date.
kit-date-month-short-may = May

# Short name of a month, in a date.
kit-date-month-short-jun = Jun

# Short name of a month, in a date.
kit-date-month-short-jul = Jul

# Short name of a month, in a date.
kit-date-month-short-aug = Aug

# Short name of a month, in a date.
kit-date-month-short-sep = Sep

# Short name of a month, in a date.
kit-date-month-short-oct = Oct

# Short name of a month, in a date.
kit-date-month-short-nov = Nov

# Short name of a month, in a date.
kit-date-month-short-dec = Dec

## Morning, afternoon and the patterns

# Written after a 12 hour clock before midday (am).
kit-date-am = AM
# Written after a 12 hour clock after midday (pm).
kit-date-pm = PM
# How the time of day is written when no other choice is made: hour and minute.
kit-date-pattern-clock = %H:%M
# How a date is written with its weekday on the login screen: weekday, day, month.
kit-date-pattern-day-month = %A %-d %B
# How a date is written with its weekday when it is also spoken: weekday, day, month.
kit-date-pattern-day-month-padded = %A %d %B
# How a whole date is written, such as under a review: day, month, year.
kit-date-pattern-full = %d %B %Y
# How a time from the last week is written under a notification: short weekday, then the time.
kit-date-pattern-weekday-clock = %a %H:%M
# How a date is written in a few letters: short weekday, day, short month.
kit-date-pattern-short-day = %a %-d %b
# How a snapshot or an event is dated in a list: short weekday, day, short month, then the time.
kit-date-pattern-short-day-clock = %a %-d %b, %H:%M
# How a date is written in a few letters without its weekday: day, short month.
kit-date-pattern-day-short = %-d %b
# How a date is written in a few letters with its year: day, short month, year.
kit-date-pattern-day-short-year = %-d %b %Y
# How a date and time are written in a few letters: day, short month, then the time.
kit-date-pattern-day-short-clock = %-d %b, %H:%M
# How a date is written in full without its weekday: day, month.
kit-date-pattern-day-long = %-d %B
# How a date is written in full without its weekday but with its year: day, month, year.
kit-date-pattern-day-long-year = %-d %B %Y
# How a date and time are written in a list of dated entries: short weekday, day, short month, year, then the time. The day is kept two characters wide.
kit-date-pattern-stamp = %a %e %b %Y, %H:%M

## Numbers

# The mark between the whole part of a number and its decimals: 3.5 in English.
kit-number-decimal = .
# The mark that groups the thousands in a long number: 1,234 in English.
kit-number-group = ,
# The language's own ten figures, nought to nine, for a language the toolkit has no region rule for; the rule (CLDR) wins for those it knows.
kit-number-digits = 0123456789
# A percentage. The number is already written in the language's own way.
kit-number-percent = { $n }%

# The day a week starts on in a date picker, as a figure: 1 Monday to 7 Sunday. Not words: write only the figure.
kit-date-first-weekday = 1

## Sizes

# A size under a kilobyte in a file list, such as 1 byte or 812 bytes.
kit-size-bytes =
    { $n ->
        [one] { $n } byte
       *[other] { $n } bytes
    }

# A size in kilobytes, such as under a file. The number is already written in the language's own way.
kit-size-kb = { $size } KB
# A size in megabytes, such as the download of an app.
kit-size-mb = { $size } MB
# A size in gigabytes, such as the room on a drive.
kit-size-gb = { $size } GB
# A size in terabytes, such as the room on a big drive.
kit-size-tb = { $size } TB
