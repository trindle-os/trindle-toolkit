# machine translated (Brazilian Portuguese), not yet reviewed by a native speaker.

kit-chart-part = Parte { $n }
kit-chart-empty = { $name }: este gráfico não tem valores.
kit-chart-line =
    { $n ->
        [one] { $name }: gráfico de linhas, 1 valor: { $first }.
       *[other] { $name }: gráfico de linhas, { $n } valores, primeiro { $first }, último { $last }, menor { $low }, maior { $high }.
    }
kit-chart-area =
    { $n ->
        [one] { $name }: gráfico de área, 1 valor: { $first }.
       *[other] { $name }: gráfico de área, { $n } valores, primeiro { $first }, último { $last }, menor { $low }, maior { $high }.
    }
kit-chart-columns =
    { $n ->
        [one] { $name }: gráfico de colunas, 1 valor: { $first }.
       *[other] { $name }: gráfico de colunas, { $n } valores, primeiro { $first }, último { $last }, menor { $low }, maior { $high }.
    }
kit-chart-spark =
    { $n ->
        [one] { $name }: minigráfico, 1 valor: { $first }.
       *[other] { $name }: minigráfico, { $n } valores, primeiro { $first }, último { $last }, menor { $low }, maior { $high }.
    }
kit-chart-pie =
    { $n ->
        [one] { $name }: gráfico de pizza, 1 parte: { $parts }.
       *[other] { $name }: gráfico de pizza, { $n } partes: { $parts }.
    }
kit-chart-ring =
    { $n ->
        [one] { $name }: gráfico de anel, 1 parte: { $parts }.
       *[other] { $name }: gráfico de anel, { $n } partes: { $parts }.
    }
