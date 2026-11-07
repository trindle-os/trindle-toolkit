## Small widgets

# The name a screen reader gives a chip that is pressed to take it away. $name is the chip's words.
kit-chip-remove = Remove { $name }

# The name a screen reader gives rating stars. $name is what is rated, $value the stars filled, $max all of them.
kit-rating = { $name }, { $value } of { $max } stars

# The names of the three boxes of a duration entry.
kit-duration-hours = Hours
kit-duration-minutes = Minutes
kit-duration-seconds = Seconds

# Wizard pages. $n is the step you are on and $count how many there are.
kit-step-of = Step { $n } of { $count }
kit-step-back = Back
kit-step-next = Next
kit-step-finish = Finish

# The find bar: the box's name, the buttons' names, and how many matches there are.
kit-find = Find
kit-find-previous = Previous match
kit-find-next = Next match
kit-find-none = No matches
kit-find-count = { $at } of { $count }

# The name a screen reader gives a "+4" box that stands for the people not shown.
kit-more-people =
    { $n ->
        [one] One more person
       *[other] { $n } more people
    }

# The gradient editor. The bar's name has $n, how many stops the gradient has.
kit-gradient-bar = Gradient with { $n } stops
kit-gradient-place = Place of the chosen stop
kit-gradient-add = Add a stop
kit-gradient-remove = Remove the stop

# The "i" button that shows and hides a card's help.
kit-help-show = Show help
kit-help-hide = Hide help
