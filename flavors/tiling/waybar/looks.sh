#!/bin/sh
# The Looks module of the panel: prints the word of the Looks door as the
# JSON of a custom module of waybar. The word names where it leads: "Looks"
# when no other look runs, "Looks: xp" (or "Looks: xp, mc") with the ones
# that do. Without chora-tiling on the path it is "Looks".
# Usage: sh looks.sh
chora-tiling --launch looks-word 2>/dev/null || echo '{"text": "Looks"}'
