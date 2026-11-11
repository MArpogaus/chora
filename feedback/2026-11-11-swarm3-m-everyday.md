# m-everyday (mc), simulated user
goal: (1) move to another scene (Super+Right) reached yes, tries 1
goal: (2) terminal runs `date` reached yes, tries 2 (Ctrl+Alt+T; Super+Return did nothing)
goal: (3) find list of keys or help reached no, tries 3 (Super+slash, then typed help)
goal: (4) find notifications or clock partly, tries 3 (clock top right; notifications panel empty)

## No key list or help found
kind: feature
persona: Lea, 24, student, tech affinity 3, keyboard first, uses a tiling WM at home.
flavor: mc
severity: high
steps: pressed Super+slash; then typed help and Enter in the terminal.
expected: a discoverable list of shortcuts or a help command.
actual: no overlay appeared. The terminal got "/help" and printed "No such file or directory".
evidence: shots/007.png
status: new

## Terminal opens with a wall of colour-config errors
kind: issue
persona: Lea, 24, student, tech affinity 3, keyboard first, uses a tiling WM at home.
flavor: mc
severity: medium
steps: pressed Ctrl+Alt+T (Super+Return had shown nothing).
expected: a clean prompt.
actual: about 20 red "invalid section name: colors-dark" lines and a locale warning fill the terminal above the prompt.
evidence: shots/005.png
status: new

## Notifications panel shows labels but no content
kind: ux
persona: Lea, 24, student, tech affinity 3, keyboard first, uses a tiling WM at home.
flavor: mc
severity: medium
steps: pressed Super+N; clicked the NOTIFICATIONS label; clicked the clock at top right.
expected: a notifications list or clock popup opens.
actual: only faint NOTES and NOTIFICATIONS labels appear at the right edge, with no content. Clicks do nothing.
evidence: shots/012.png
status: new
