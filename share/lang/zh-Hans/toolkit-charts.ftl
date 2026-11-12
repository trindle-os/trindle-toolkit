# machine translated (Simplified Chinese), not yet reviewed by a native speaker.

kit-chart-part = 第 { $n } 部分

kit-chart-empty = { $name }：此图表没有数值。

kit-chart-line =
    { $n ->
        [1] { $name }：折线图，1 个值：{ $first }。
       *[other] { $name }：折线图，{ $n } 个值，第一个 { $first }，最后一个 { $last }，最低 { $low }，最高 { $high }。
    }

kit-chart-area =
    { $n ->
        [1] { $name }：面积图，1 个值：{ $first }。
       *[other] { $name }：面积图，{ $n } 个值，第一个 { $first }，最后一个 { $last }，最低 { $low }，最高 { $high }。
    }

kit-chart-columns =
    { $n ->
        [1] { $name }：柱状图，1 个值：{ $first }。
       *[other] { $name }：柱状图，{ $n } 个值，第一个 { $first }，最后一个 { $last }，最低 { $low }，最高 { $high }。
    }

kit-chart-spark =
    { $n ->
        [1] { $name }：迷你折线图，1 个值：{ $first }。
       *[other] { $name }：迷你折线图，{ $n } 个值，第一个 { $first }，最后一个 { $last }，最低 { $low }，最高 { $high }。
    }

kit-chart-pie =
    { $n ->
       *[other] { $name }：饼图，{ $n } 个部分：{ $parts }。
    }

kit-chart-ring =
    { $n ->
       *[other] { $name }：环形图，{ $n } 个部分：{ $parts }。
    }
