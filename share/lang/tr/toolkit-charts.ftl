# machine translated (Turkish), not yet reviewed by a native speaker.

kit-chart-part = Bölüm { $n }
kit-chart-empty = { $name }: bu grafikte hiç değer yok.
kit-chart-line =
    { $n ->
        [one] { $name }: çizgi grafik, 1 değer: { $first }.
       *[other] { $name }: çizgi grafik, { $n } değer, ilk { $first }, son { $last }, en düşük { $low }, en yüksek { $high }.
    }
kit-chart-area =
    { $n ->
        [one] { $name }: alan grafiği, 1 değer: { $first }.
       *[other] { $name }: alan grafiği, { $n } değer, ilk { $first }, son { $last }, en düşük { $low }, en yüksek { $high }.
    }
kit-chart-columns =
    { $n ->
        [one] { $name }: sütun grafiği, 1 değer: { $first }.
       *[other] { $name }: sütun grafiği, { $n } değer, ilk { $first }, son { $last }, en düşük { $low }, en yüksek { $high }.
    }
kit-chart-spark =
    { $n ->
        [one] { $name }: mini çizgi grafik, 1 değer: { $first }.
       *[other] { $name }: mini çizgi grafik, { $n } değer, ilk { $first }, son { $last }, en düşük { $low }, en yüksek { $high }.
    }
kit-chart-pie =
    { $n ->
        [one] { $name }: pasta grafiği, 1 bölüm: { $parts }.
       *[other] { $name }: pasta grafiği, { $n } bölüm: { $parts }.
    }
kit-chart-ring =
    { $n ->
        [one] { $name }: halka grafiği, 1 bölüm: { $parts }.
       *[other] { $name }: halka grafiği, { $n } bölüm: { $parts }.
    }
