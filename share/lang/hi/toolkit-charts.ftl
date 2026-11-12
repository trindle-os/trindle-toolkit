# machine translated (Hindi), not yet reviewed by a native speaker.

kit-chart-part = भाग { $n }
kit-chart-empty = { $name }: इस चार्ट में कोई मान नहीं है।
kit-chart-line =
    { $n ->
        [one] { $name }: रेखा चार्ट, 1 मान: { $first }।
       *[other] { $name }: रेखा चार्ट, { $n } मान, पहला { $first }, आख़िरी { $last }, सबसे कम { $low }, सबसे ज़्यादा { $high }।
    }
kit-chart-area =
    { $n ->
        [one] { $name }: क्षेत्र चार्ट, 1 मान: { $first }।
       *[other] { $name }: क्षेत्र चार्ट, { $n } मान, पहला { $first }, आख़िरी { $last }, सबसे कम { $low }, सबसे ज़्यादा { $high }।
    }
kit-chart-columns =
    { $n ->
        [one] { $name }: स्तंभ चार्ट, 1 मान: { $first }।
       *[other] { $name }: स्तंभ चार्ट, { $n } मान, पहला { $first }, आख़िरी { $last }, सबसे कम { $low }, सबसे ज़्यादा { $high }।
    }
kit-chart-spark =
    { $n ->
        [one] { $name }: स्पार्कलाइन, 1 मान: { $first }।
       *[other] { $name }: स्पार्कलाइन, { $n } मान, पहला { $first }, आख़िरी { $last }, सबसे कम { $low }, सबसे ज़्यादा { $high }।
    }
kit-chart-pie =
    { $n ->
        [one] { $name }: पाई चार्ट, 1 भाग: { $parts }।
       *[other] { $name }: पाई चार्ट, { $n } भाग: { $parts }।
    }
kit-chart-ring =
    { $n ->
        [one] { $name }: रिंग चार्ट, 1 भाग: { $parts }।
       *[other] { $name }: रिंग चार्ट, { $n } भाग: { $parts }।
    }
