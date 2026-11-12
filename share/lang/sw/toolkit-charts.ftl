# machine translated (Swahili), not yet reviewed by a native speaker.

kit-chart-part = Sehemu { $n }
kit-chart-empty = { $name }: chati hii haina thamani.
kit-chart-line =
    { $n ->
        [one] { $name }: chati ya mstari, thamani 1: { $first }.
       *[other] { $name }: chati ya mstari, thamani { $n }, ya kwanza { $first }, ya mwisho { $last }, ya chini { $low }, ya juu { $high }.
    }
kit-chart-area =
    { $n ->
        [one] { $name }: chati ya eneo, thamani 1: { $first }.
       *[other] { $name }: chati ya eneo, thamani { $n }, ya kwanza { $first }, ya mwisho { $last }, ya chini { $low }, ya juu { $high }.
    }
kit-chart-columns =
    { $n ->
        [one] { $name }: chati ya safu wima, thamani 1: { $first }.
       *[other] { $name }: chati ya safu wima, thamani { $n }, ya kwanza { $first }, ya mwisho { $last }, ya chini { $low }, ya juu { $high }.
    }
kit-chart-spark =
    { $n ->
        [one] { $name }: mstari mdogo, thamani 1: { $first }.
       *[other] { $name }: mstari mdogo, thamani { $n }, ya kwanza { $first }, ya mwisho { $last }, ya chini { $low }, ya juu { $high }.
    }
kit-chart-pie =
    { $n ->
        [one] { $name }: chati ya duara, sehemu 1: { $parts }.
       *[other] { $name }: chati ya duara, sehemu { $n }: { $parts }.
    }
kit-chart-ring =
    { $n ->
        [one] { $name }: chati ya pete, sehemu 1: { $parts }.
       *[other] { $name }: chati ya pete, sehemu { $n }: { $parts }.
    }
