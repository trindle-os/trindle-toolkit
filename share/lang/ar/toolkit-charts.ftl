# machine translated (Arabic), not yet reviewed by a native speaker.

kit-chart-part = الجزء { $n }

kit-chart-empty = { $name }: لا قيم في هذا المخطط.

kit-chart-line =
    { $n ->
        [one] { $name }: مخطط خطي، قيمة واحدة: { $first }.
        [zero] { $name }: مخطط خطي، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
        [two] { $name }: مخطط خطي، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
        [few] { $name }: مخطط خطي، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
        [many] { $name }: مخطط خطي، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
       *[other] { $name }: مخطط خطي، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
    }

kit-chart-area =
    { $n ->
        [one] { $name }: مخطط مساحي، قيمة واحدة: { $first }.
        [zero] { $name }: مخطط مساحي، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
        [two] { $name }: مخطط مساحي، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
        [few] { $name }: مخطط مساحي، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
        [many] { $name }: مخطط مساحي، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
       *[other] { $name }: مخطط مساحي، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
    }

kit-chart-columns =
    { $n ->
        [one] { $name }: مخطط أعمدة، قيمة واحدة: { $first }.
        [zero] { $name }: مخطط أعمدة، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
        [two] { $name }: مخطط أعمدة، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
        [few] { $name }: مخطط أعمدة، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
        [many] { $name }: مخطط أعمدة، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
       *[other] { $name }: مخطط أعمدة، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
    }

kit-chart-spark =
    { $n ->
        [one] { $name }: مخطط مصغر، قيمة واحدة: { $first }.
        [zero] { $name }: مخطط مصغر، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
        [two] { $name }: مخطط مصغر، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
        [few] { $name }: مخطط مصغر، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
        [many] { $name }: مخطط مصغر، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
       *[other] { $name }: مخطط مصغر، { $n } قيم، الأولى { $first }، الأخيرة { $last }، الأدنى { $low }، الأعلى { $high }.
    }

kit-chart-pie =
    { $n ->
        [one] { $name }: مخطط دائري، جزء واحد: { $parts }.
        [zero] { $name }: مخطط دائري، { $n } أجزاء: { $parts }.
        [two] { $name }: مخطط دائري، { $n } أجزاء: { $parts }.
        [few] { $name }: مخطط دائري، { $n } أجزاء: { $parts }.
        [many] { $name }: مخطط دائري، { $n } أجزاء: { $parts }.
       *[other] { $name }: مخطط دائري، { $n } أجزاء: { $parts }.
    }

kit-chart-ring =
    { $n ->
        [one] { $name }: مخطط حلقي، جزء واحد: { $parts }.
        [zero] { $name }: مخطط حلقي، { $n } أجزاء: { $parts }.
        [two] { $name }: مخطط حلقي، { $n } أجزاء: { $parts }.
        [few] { $name }: مخطط حلقي، { $n } أجزاء: { $parts }.
        [many] { $name }: مخطط حلقي، { $n } أجزاء: { $parts }.
       *[other] { $name }: مخطط حلقي، { $n } أجزاء: { $parts }.
    }
