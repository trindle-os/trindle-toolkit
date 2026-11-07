# machine translated (Japanese), not yet reviewed by a native speaker.

kit-chart-part = 部分 { $n }

kit-chart-empty = { $name }: 値のないグラフ。

kit-chart-line =
    { $n ->
       *[other] { $name }: 折れ線グラフ、値 { $n } 個、最初 { $first }、最後 { $last }、最小 { $low }、最大 { $high }。
    }

kit-chart-area =
    { $n ->
       *[other] { $name }: 面グラフ、値 { $n } 個、最初 { $first }、最後 { $last }、最小 { $low }、最大 { $high }。
    }

kit-chart-columns =
    { $n ->
       *[other] { $name }: 棒グラフ、値 { $n } 個、最初 { $first }、最後 { $last }、最小 { $low }、最大 { $high }。
    }

kit-chart-spark =
    { $n ->
       *[other] { $name }: スパークライン、値 { $n } 個、最初 { $first }、最後 { $last }、最小 { $low }、最大 { $high }。
    }

kit-chart-pie =
    { $n ->
       *[other] { $name }: 円グラフ、{ $n } 個の部分: { $parts }。
    }

kit-chart-ring =
    { $n ->
       *[other] { $name }: リンググラフ、{ $n } 個の部分: { $parts }。
    }
