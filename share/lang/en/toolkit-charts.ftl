## What a screen reader says of a chart

# The name of a part of a pie or ring chart that was not given a label. $n is its place, from 1.
kit-chart-part = Part { $n }
# A chart with no values. $name is what the chart shows.
kit-chart-empty = { $name }: chart with no values.
# A line chart. $name is what it shows, $n how many values; the rest are figures.
kit-chart-line =
    { $n ->
        [one] { $name }: line chart, 1 value: { $first }.
       *[other] { $name }: line chart, { $n } values, first { $first }, last { $last }, lowest { $low }, highest { $high }.
    }
# An area chart: a line with the room under it filled.
kit-chart-area =
    { $n ->
        [one] { $name }: area chart, 1 value: { $first }.
       *[other] { $name }: area chart, { $n } values, first { $first }, last { $last }, lowest { $low }, highest { $high }.
    }
# A column chart.
kit-chart-columns =
    { $n ->
        [one] { $name }: column chart, 1 value: { $first }.
       *[other] { $name }: column chart, { $n } values, first { $first }, last { $last }, lowest { $low }, highest { $high }.
    }
# A small line chart with no axes.
kit-chart-spark =
    { $n ->
        [one] { $name }: sparkline, 1 value: { $first }.
       *[other] { $name }: sparkline, { $n } values, first { $first }, last { $last }, lowest { $low }, highest { $high }.
    }
# A pie chart. $parts lists each part and its share, such as "Rent 50%, Food 30%".
kit-chart-pie =
    { $n ->
        [one] { $name }: pie chart, 1 part: { $parts }.
       *[other] { $name }: pie chart, { $n } parts: { $parts }.
    }
# A ring chart of parts, laid out as the pie chart is.
kit-chart-ring =
    { $n ->
        [one] { $name }: ring chart, 1 part: { $parts }.
       *[other] { $name }: ring chart, { $n } parts: { $parts }.
    }
