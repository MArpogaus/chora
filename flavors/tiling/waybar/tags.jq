# The tags module of the panel: turns the answer of `tiling.get_tags` into
# the JSON of a custom module of waybar. It takes the output of the bar
# from WAYBAR_OUTPUT_NAME, else the focused output, and lists the shown and
# the occupied tags: shown ones underlined, focused ones bold, urgent ones
# red.
.outputs
| (map(select(.name == env.WAYBAR_OUTPUT_NAME)) + map(select(.focused)))[0]
| [.tags[]
   | select(.shown or .occupied)
   | (.tag | tostring) as $n
   | if .urgent then "<span color='#bf616a'>\($n)</span>"
     elif .focused then "<b><u>\($n)</u></b>"
     elif .shown then "<u>\($n)</u>"
     else $n end]
| {text: join(" ")}
