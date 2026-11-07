# machine translated (Urdu), not yet reviewed by a native speaker.

kit-chart-part = حصہ { $n }

kit-chart-empty = { $name }: بغیر قدروں کا چارٹ۔

kit-chart-line =
    { $n ->
        [one] { $name }: لائن چارٹ، 1 قدر: { $first }۔
       *[other] { $name }: لائن چارٹ، { $n } قدریں، پہلی { $first }، آخری { $last }، سب سے کم { $low }، سب سے زیادہ { $high }۔
    }

kit-chart-area =
    { $n ->
        [one] { $name }: ایریا چارٹ، 1 قدر: { $first }۔
       *[other] { $name }: ایریا چارٹ، { $n } قدریں، پہلی { $first }، آخری { $last }، سب سے کم { $low }، سب سے زیادہ { $high }۔
    }

kit-chart-columns =
    { $n ->
        [one] { $name }: کالم چارٹ، 1 قدر: { $first }۔
       *[other] { $name }: کالم چارٹ، { $n } قدریں، پہلی { $first }، آخری { $last }، سب سے کم { $low }، سب سے زیادہ { $high }۔
    }

kit-chart-spark =
    { $n ->
        [one] { $name }: اسپارک لائن، 1 قدر: { $first }۔
       *[other] { $name }: اسپارک لائن، { $n } قدریں، پہلی { $first }، آخری { $last }، سب سے کم { $low }، سب سے زیادہ { $high }۔
    }

kit-chart-pie =
    { $n ->
        [one] { $name }: پائی چارٹ، 1 حصہ: { $parts }۔
       *[other] { $name }: پائی چارٹ، { $n } حصے: { $parts }۔
    }

kit-chart-ring =
    { $n ->
        [one] { $name }: رنگ چارٹ، 1 حصہ: { $parts }۔
       *[other] { $name }: رنگ چارٹ، { $n } حصے: { $parts }۔
    }
