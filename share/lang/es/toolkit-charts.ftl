# machine translated (Spanish), not yet reviewed by a native speaker.

kit-chart-part = Parte { $n }
kit-chart-empty = { $name }: gráfico sin valores.
kit-chart-line =
    { $n ->
        [one] { $name }: gráfico de líneas, 1 valor: { $first }.
       *[other] { $name }: gráfico de líneas, { $n } valores, primero { $first }, último { $last }, mínimo { $low }, máximo { $high }.
    }
kit-chart-area =
    { $n ->
        [one] { $name }: gráfico de áreas, 1 valor: { $first }.
       *[other] { $name }: gráfico de áreas, { $n } valores, primero { $first }, último { $last }, mínimo { $low }, máximo { $high }.
    }
kit-chart-columns =
    { $n ->
        [one] { $name }: gráfico de columnas, 1 valor: { $first }.
       *[other] { $name }: gráfico de columnas, { $n } valores, primero { $first }, último { $last }, mínimo { $low }, máximo { $high }.
    }
kit-chart-spark =
    { $n ->
        [one] { $name }: minigráfico, 1 valor: { $first }.
       *[other] { $name }: minigráfico, { $n } valores, primero { $first }, último { $last }, mínimo { $low }, máximo { $high }.
    }
kit-chart-pie =
    { $n ->
        [one] { $name }: gráfico circular, 1 parte: { $parts }.
       *[other] { $name }: gráfico circular, { $n } partes: { $parts }.
    }
kit-chart-ring =
    { $n ->
        [one] { $name }: gráfico de anillo, 1 parte: { $parts }.
       *[other] { $name }: gráfico de anillo, { $n } partes: { $parts }.
    }
