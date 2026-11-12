# machine translated (French), not yet reviewed by a native speaker.

kit-chart-part = Partie { $n }

kit-chart-empty = { $name } : ce graphique n’a aucune valeur.

kit-chart-line =
    { $n ->
        [one] { $name } : graphique en courbe, 1 valeur : { $first }.
       *[other] { $name } : graphique en courbe, { $n } valeurs, première { $first }, dernière { $last }, plus basse { $low }, plus haute { $high }.
    }

kit-chart-area =
    { $n ->
        [one] { $name } : graphique en aires, 1 valeur : { $first }.
       *[other] { $name } : graphique en aires, { $n } valeurs, première { $first }, dernière { $last }, plus basse { $low }, plus haute { $high }.
    }

kit-chart-columns =
    { $n ->
        [one] { $name } : graphique en colonnes, 1 valeur : { $first }.
       *[other] { $name } : graphique en colonnes, { $n } valeurs, première { $first }, dernière { $last }, plus basse { $low }, plus haute { $high }.
    }

kit-chart-spark =
    { $n ->
        [one] { $name } : mini-courbe, 1 valeur : { $first }.
       *[other] { $name } : mini-courbe, { $n } valeurs, première { $first }, dernière { $last }, plus basse { $low }, plus haute { $high }.
    }

kit-chart-pie =
    { $n ->
        [one] { $name } : graphique en secteurs, 1 part : { $parts }.
       *[other] { $name } : graphique en secteurs, { $n } parts : { $parts }.
    }

kit-chart-ring =
    { $n ->
        [one] { $name } : graphique en anneau, 1 part : { $parts }.
       *[other] { $name } : graphique en anneau, { $n } parts : { $parts }.
    }
