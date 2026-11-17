# The module of one tag of the panel: turns the answer of
# `tiling.get_tags` into the JSON of a custom module of waybar. It takes
# the output of the bar from WAYBAR_OUTPUT_NAME, else the focused output,
# and the tag from the variable $n. The text is the number; the class is
# one of current (shown, also when empty), occupied, empty, and urgent
# besides. A tag that the output does not have gives no text, so the
# module hides. The look is style.css, not markup.
# Usage: jq -c --argjson n 3 -f tags.jq ANSWER
.outputs
| (map(select(.name == env.WAYBAR_OUTPUT_NAME)) + map(select(.focused)))[0]
| (.tags[] | select(.tag == $n)) as $t
| {text: ($t.tag | tostring),
   class: ([if $t.shown then "current" elif $t.occupied then "occupied" else "empty" end]
           + [if $t.urgent then "urgent" else empty end])}
