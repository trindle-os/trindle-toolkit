# machine translated (Bengali), not yet reviewed by a native speaker.

kit-chart-part = অংশ { $n }
kit-chart-empty = { $name }: কোনো মান নেই এমন চার্ট।
kit-chart-line =
    { $n ->
        [one] { $name }: লাইন চার্ট, ১টি মান: { $first }।
       *[other] { $name }: লাইন চার্ট, { $n }টি মান, প্রথম { $first }, শেষ { $last }, সর্বনিম্ন { $low }, সর্বোচ্চ { $high }।
    }
kit-chart-area =
    { $n ->
        [one] { $name }: এরিয়া চার্ট, ১টি মান: { $first }।
       *[other] { $name }: এরিয়া চার্ট, { $n }টি মান, প্রথম { $first }, শেষ { $last }, সর্বনিম্ন { $low }, সর্বোচ্চ { $high }।
    }
kit-chart-columns =
    { $n ->
        [one] { $name }: কলাম চার্ট, ১টি মান: { $first }।
       *[other] { $name }: কলাম চার্ট, { $n }টি মান, প্রথম { $first }, শেষ { $last }, সর্বনিম্ন { $low }, সর্বোচ্চ { $high }।
    }
kit-chart-spark =
    { $n ->
        [one] { $name }: স্পার্কলাইন, ১টি মান: { $first }।
       *[other] { $name }: স্পার্কলাইন, { $n }টি মান, প্রথম { $first }, শেষ { $last }, সর্বনিম্ন { $low }, সর্বোচ্চ { $high }।
    }
kit-chart-pie =
    { $n ->
        [one] { $name }: পাই চার্ট, ১টি অংশ: { $parts }।
       *[other] { $name }: পাই চার্ট, { $n }টি অংশ: { $parts }।
    }
kit-chart-ring =
    { $n ->
        [one] { $name }: রিং চার্ট, ১টি অংশ: { $parts }।
       *[other] { $name }: রিং চার্ট, { $n }টি অংশ: { $parts }।
    }
