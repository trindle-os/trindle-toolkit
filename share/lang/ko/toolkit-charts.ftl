# machine translated (Korean), not yet reviewed by a native speaker.

kit-chart-part = 부분 { $n }
kit-chart-empty = { $name }: 이 차트에는 값이 없습니다.
kit-chart-line =
    { $n ->
       *[other] { $name }: 선 차트, 값 { $n }개, 처음 { $first }, 마지막 { $last }, 최저 { $low }, 최고 { $high }.
    }
kit-chart-area =
    { $n ->
       *[other] { $name }: 영역 차트, 값 { $n }개, 처음 { $first }, 마지막 { $last }, 최저 { $low }, 최고 { $high }.
    }
kit-chart-columns =
    { $n ->
       *[other] { $name }: 막대 차트, 값 { $n }개, 처음 { $first }, 마지막 { $last }, 최저 { $low }, 최고 { $high }.
    }
kit-chart-spark =
    { $n ->
       *[other] { $name }: 스파크라인, 값 { $n }개, 처음 { $first }, 마지막 { $last }, 최저 { $low }, 최고 { $high }.
    }
kit-chart-pie =
    { $n ->
       *[other] { $name }: 원형 차트, 부분 { $n }개: { $parts }.
    }
kit-chart-ring =
    { $n ->
       *[other] { $name }: 도넛 차트, 부분 { $n }개: { $parts }.
    }
