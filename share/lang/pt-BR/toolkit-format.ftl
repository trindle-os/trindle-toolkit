# machine translated (Brazilian Portuguese), not yet reviewed by a native speaker.

# Name of a weekday, in a date or a clock line.
kit-date-weekday-sun = domingo

# Name of a weekday, in a date or a clock line.
kit-date-weekday-mon = segunda-feira

# Name of a weekday, in a date or a clock line.
kit-date-weekday-tue = terça-feira

# Name of a weekday, in a date or a clock line.
kit-date-weekday-wed = quarta-feira

# Name of a weekday, in a date or a clock line.
kit-date-weekday-thu = quinta-feira

# Name of a weekday, in a date or a clock line.
kit-date-weekday-fri = sexta-feira

# Name of a weekday, in a date or a clock line.
kit-date-weekday-sat = sábado

# Short name of a weekday, in a list of times.
kit-date-weekday-short-sun = dom

# Short name of a weekday, in a list of times.
kit-date-weekday-short-mon = seg

# Short name of a weekday, in a list of times.
kit-date-weekday-short-tue = ter

# Short name of a weekday, in a list of times.
kit-date-weekday-short-wed = qua

# Short name of a weekday, in a list of times.
kit-date-weekday-short-thu = qui

# Short name of a weekday, in a list of times.
kit-date-weekday-short-fri = sex

# Short name of a weekday, in a list of times.
kit-date-weekday-short-sat = sáb

# Name of a month, in a date.
kit-date-month-jan = janeiro

# Name of a month, in a date.
kit-date-month-feb = fevereiro

# Name of a month, in a date.
kit-date-month-mar = março

# Name of a month, in a date.
kit-date-month-apr = abril

# Name of a month, in a date.
kit-date-month-may = maio

# Name of a month, in a date.
kit-date-month-jun = junho

# Name of a month, in a date.
kit-date-month-jul = julho

# Name of a month, in a date.
kit-date-month-aug = agosto

# Name of a month, in a date.
kit-date-month-sep = setembro

# Name of a month, in a date.
kit-date-month-oct = outubro

# Name of a month, in a date.
kit-date-month-nov = novembro

# Name of a month, in a date.
kit-date-month-dec = dezembro

# Short name of a month, in a date.
kit-date-month-short-jan = jan

# Short name of a month, in a date.
kit-date-month-short-feb = fev

# Short name of a month, in a date.
kit-date-month-short-mar = mar

# Short name of a month, in a date.
kit-date-month-short-apr = abr

# Short name of a month, in a date.
kit-date-month-short-may = mai

# Short name of a month, in a date.
kit-date-month-short-jun = jun

# Short name of a month, in a date.
kit-date-month-short-jul = jul

# Short name of a month, in a date.
kit-date-month-short-aug = ago

# Short name of a month, in a date.
kit-date-month-short-sep = set

# Short name of a month, in a date.
kit-date-month-short-oct = out

# Short name of a month, in a date.
kit-date-month-short-nov = nov

# Short name of a month, in a date.
kit-date-month-short-dec = dez

# Written after a 12 hour clock before midday (am).
kit-date-am = AM

# Written after a 12 hour clock after midday (pm).
kit-date-pm = PM

# How the time of day is written when no other choice is made: hour and minute.
kit-date-pattern-clock = %H:%M

# How a date is written with its weekday on the login screen: weekday, day, month.
kit-date-pattern-day-month = %A, %-d de %B

# How a date is written with its weekday when it is also spoken: weekday, day, month.
kit-date-pattern-day-month-padded = %A, %d de %B

# How a whole date is written, such as under a review: day, month, year.
kit-date-pattern-full = %d de %B de %Y

# How a time from the last week is written under a notification: short weekday, then the time.
kit-date-pattern-weekday-clock = %a %H:%M

# How a date is written in a few letters: short weekday, day, short month.
kit-date-pattern-short-day = %a, %-d de %b

# How a snapshot or an event is dated in a list: short weekday, day, short month, then the time.
kit-date-pattern-short-day-clock = %a, %-d de %b, %H:%M

# How a date is written in a few letters without its weekday: day, short month.
kit-date-pattern-day-short = %-d de %b

# How a date is written in a few letters with its year: day, short month, year.
kit-date-pattern-day-short-year = %-d de %b de %Y

# How a date and time are written in a few letters: day, short month, then the time.
kit-date-pattern-day-short-clock = %-d de %b, %H:%M

# How a date is written in full without its weekday: day, month.
kit-date-pattern-day-long = %-d de %B

# How a date is written in full without its weekday but with its year: day, month, year.
kit-date-pattern-day-long-year = %-d de %B de %Y

# How a date and time are written in a list of dated entries: short weekday, day, short month, year, then the time. The day is kept two characters wide.
kit-date-pattern-stamp = %a %e %b %Y, %H:%M

# The mark between the whole part of a number and its decimals: 3.5 in English.
kit-number-decimal = ,

# The mark that groups the thousands in a long number: 1,234 in English.
kit-number-group = .

# A size under a kilobyte in a file list, such as 1 byte or 812 bytes.
kit-size-bytes =
    { $n ->
        [one] { $n } byte
        [many] { $n } de bytes
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

# The day a week starts on in a date picker, as a figure: 1 Monday to 7 Sunday. Not words: write only the figure.
kit-date-first-weekday = 7

kit-number-digits = 0123456789
kit-number-percent = { $n }%
