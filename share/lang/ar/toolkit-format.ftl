# machine translated (Arabic), not yet reviewed by a native speaker.

# Name of a weekday, in a date or a clock line.
kit-date-weekday-sun = الأحد

# Name of a weekday, in a date or a clock line.
kit-date-weekday-mon = الاثنين

# Name of a weekday, in a date or a clock line.
kit-date-weekday-tue = الثلاثاء

# Name of a weekday, in a date or a clock line.
kit-date-weekday-wed = الأربعاء

# Name of a weekday, in a date or a clock line.
kit-date-weekday-thu = الخميس

# Name of a weekday, in a date or a clock line.
kit-date-weekday-fri = الجمعة

# Name of a weekday, in a date or a clock line.
kit-date-weekday-sat = السبت

# Short name of a weekday, in a list of times.
kit-date-weekday-short-sun = أحد

# Short name of a weekday, in a list of times.
kit-date-weekday-short-mon = اثنين

# Short name of a weekday, in a list of times.
kit-date-weekday-short-tue = ثلاثاء

# Short name of a weekday, in a list of times.
kit-date-weekday-short-wed = أربعاء

# Short name of a weekday, in a list of times.
kit-date-weekday-short-thu = خميس

# Short name of a weekday, in a list of times.
kit-date-weekday-short-fri = جمعة

# Short name of a weekday, in a list of times.
kit-date-weekday-short-sat = سبت

# Name of a month, in a date.
kit-date-month-jan = يناير

# Name of a month, in a date.
kit-date-month-feb = فبراير

# Name of a month, in a date.
kit-date-month-mar = مارس

# Name of a month, in a date.
kit-date-month-apr = أبريل

# Name of a month, in a date.
kit-date-month-may = مايو

# Name of a month, in a date.
kit-date-month-jun = يونيو

# Name of a month, in a date.
kit-date-month-jul = يوليو

# Name of a month, in a date.
kit-date-month-aug = أغسطس

# Name of a month, in a date.
kit-date-month-sep = سبتمبر

# Name of a month, in a date.
kit-date-month-oct = أكتوبر

# Name of a month, in a date.
kit-date-month-nov = نوفمبر

# Name of a month, in a date.
kit-date-month-dec = ديسمبر

# Short name of a month, in a date.
kit-date-month-short-jan = يناير

# Short name of a month, in a date.
kit-date-month-short-feb = فبراير

# Short name of a month, in a date.
kit-date-month-short-mar = مارس

# Short name of a month, in a date.
kit-date-month-short-apr = أبريل

# Short name of a month, in a date.
kit-date-month-short-may = مايو

# Short name of a month, in a date.
kit-date-month-short-jun = يونيو

# Short name of a month, in a date.
kit-date-month-short-jul = يوليو

# Short name of a month, in a date.
kit-date-month-short-aug = أغسطس

# Short name of a month, in a date.
kit-date-month-short-sep = سبتمبر

# Short name of a month, in a date.
kit-date-month-short-oct = أكتوبر

# Short name of a month, in a date.
kit-date-month-short-nov = نوفمبر

# Short name of a month, in a date.
kit-date-month-short-dec = ديسمبر

# Written after a 12 hour clock before midday (am).
kit-date-am = ص

# Written after a 12 hour clock after midday (pm).
kit-date-pm = م

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
kit-date-pattern-short-day-clock = %a %-d %b، %H:%M

# How a date is written in a few letters without its weekday: day, short month.
kit-date-pattern-day-short = %-d %b

# How a date is written in a few letters with its year: day, short month, year.
kit-date-pattern-day-short-year = %-d %b %Y

# How a date and time are written in a few letters: day, short month, then the time.
kit-date-pattern-day-short-clock = %-d %b، %H:%M

# How a date is written in full without its weekday: day, month.
kit-date-pattern-day-long = %-d %B

# How a date is written in full without its weekday but with its year: day, month, year.
kit-date-pattern-day-long-year = %-d %B %Y

# How a date and time are written in a list of dated entries: short weekday, day, short month, year, then the time. The day is kept two characters wide.
kit-date-pattern-stamp = %a %e %b %Y، %H:%M

# The mark between the whole part of a number and its decimals: 3.5 in English.
kit-number-decimal = ٫

# The mark that groups the thousands in a long number: 1,234 in English.
kit-number-group = ٬

# The language's own ten figures, nought to nine, as numbers are written in it.
kit-number-digits = ٠١٢٣٤٥٦٧٨٩

# A percentage. The number is already written in the language's own way.
kit-number-percent = { $n }٪

# A size under a kilobyte in a file list, such as 1 byte or 812 bytes.
kit-size-bytes =
    { $n ->
        [zero] { $n } بايت
        [one] بايت واحد
        [two] بايتان
        [few] { $n } بايتات
        [many] { $n } بايتًا
       *[other] { $n } بايت
    }

# A size in kilobytes, such as under a file. The number is already written in the language's own way.
kit-size-kb = { $size } KB

# A size in megabytes, such as the download of an app.
kit-size-mb = { $size } MB

# A size in gigabytes, such as the room on a drive.
kit-size-gb = { $size } GB

# A size in terabytes, such as the room on a big drive.
kit-size-tb = { $size } TB

kit-date-first-weekday = 7

