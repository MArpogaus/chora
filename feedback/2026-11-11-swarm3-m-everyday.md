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
status: duplicate of Key list and help exist only in construct

## Terminal opens with a wall of colour-config errors
kind: issue
persona: Lea, 24, student, tech affinity 3, keyboard first, uses a tiling WM at home.
flavor: mc
severity: medium
steps: pressed Ctrl+Alt+T (Super+Return had shown nothing).
expected: a clean prompt.
actual: about 20 red "invalid section name: colors-dark" lines and a locale warning fill the terminal above the prompt.
evidence: shots/005.png
status: rejected: env, mc.yaml says the colors-dark section needs foot 1.26 or later and the container has foot 1.16.2 (shot 005)

## Notifications panel shows labels but no content
kind: ux
persona: Lea, 24, student, tech affinity 3, keyboard first, uses a tiling WM at home.
flavor: mc
severity: medium
steps: pressed Super+N; clicked the NOTIFICATIONS label; clicked the clock at top right.
expected: a notifications list or clock popup opens.
actual: only faint NOTES and NOTIFICATIONS labels appear at the right edge, with no content. Clicks do nothing.
evidence: shots/012.png
status: rejected: by design, nothing was sent so the notify text is empty, and mc binds no click on a region (shot 012)

## Super+slash types a slash into the terminal
kind: issue
persona: Lea, 24, student, tech affinity 3, keyboard first, uses a tiling WM at home.
flavor: mc
severity: medium
steps: Pressed Super+slash with the terminal focused.
expected: The key is eaten by the desktop or shows help.
actual: A / appears at the prompt (shot 007); mc.yaml binds no Super+slash. Same class as Unbound Super chords type into the focused terminal.
evidence: media/2026-11-12/swarm-3/m-everyday/shots/007.png
status: verified

## Super+Return opens no terminal
kind: ux
persona: Lea, 24, student, tech affinity 3, keyboard first, uses a tiling WM at home.
flavor: mc
severity: low
steps: Pressed Super+Return expecting a terminal.
expected: A terminal, as in tiling.
actual: Nothing visible (shot 004). mc.yaml binds Super+Return to toggle the drawer, not a terminal; Ctrl+Alt+T opened one.
evidence: media/2026-11-12/swarm-3/m-everyday/shots/004.png
status: rejected: by design, Super+Return toggles the drawer in mc.yaml; the gap is the missing key list, see Key list and help exist only in construct
