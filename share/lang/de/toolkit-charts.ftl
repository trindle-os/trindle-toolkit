# machine translated (German), not yet reviewed by a native speaker.

kit-chart-part = Teil { $n }
kit-chart-empty = { $name }: Diagramm ohne Werte.
kit-chart-line =
    { $n ->
        [one] { $name }: Liniendiagramm, 1 Wert: { $first }.
       *[other] { $name }: Liniendiagramm, { $n } Werte, erster { $first }, letzter { $last }, niedrigster { $low }, höchster { $high }.
    }
kit-chart-area =
    { $n ->
        [one] { $name }: Flächendiagramm, 1 Wert: { $first }.
       *[other] { $name }: Flächendiagramm, { $n } Werte, erster { $first }, letzter { $last }, niedrigster { $low }, höchster { $high }.
    }
kit-chart-columns =
    { $n ->
        [one] { $name }: Säulendiagramm, 1 Wert: { $first }.
       *[other] { $name }: Säulendiagramm, { $n } Werte, erster { $first }, letzter { $last }, niedrigster { $low }, höchster { $high }.
    }
kit-chart-spark =
    { $n ->
        [one] { $name }: Sparkline, 1 Wert: { $first }.
       *[other] { $name }: Sparkline, { $n } Werte, erster { $first }, letzter { $last }, niedrigster { $low }, höchster { $high }.
    }
kit-chart-pie =
    { $n ->
        [one] { $name }: Kreisdiagramm, 1 Teil: { $parts }.
       *[other] { $name }: Kreisdiagramm, { $n } Teile: { $parts }.
    }
kit-chart-ring =
    { $n ->
        [one] { $name }: Ringdiagramm, 1 Teil: { $parts }.
       *[other] { $name }: Ringdiagramm, { $n } Teile: { $parts }.
    }
