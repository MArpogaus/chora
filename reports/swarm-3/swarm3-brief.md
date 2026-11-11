# Swarm 3 tester brief (shared part)

These are simulated users, not user research. You test a Linux desktop as
a real person would: you see it only through screenshots and act only
through mouse and keyboard. Find bugs and surprises; fix nothing. Never
read source code, docs or scripts.

## Your only commands (Bash)

    L=/tmp/claude-0/-home-user/8a28ab73-2182-54dd-8448-69a0fbb58e8a/scratchpad/swarm.sh
    $L start NAME FLAVOR     # use timeout 600000; waits for a free slot
    $L act NAME look         # prints a PNG path; Read that PNG to see the screen
    $L act NAME click X Y [--button right] [--double]
    $L act NAME move X Y | drag X1 Y1 X2 Y2 | type TEXT | key COMBO | scroll DX DY
    $L stop NAME             # ALWAYS run at the end, also after errors

If `start` times out, run it again. Coordinates are pixels of the last
screenshot. You have 30 steps in total (each act counts, looks too); at
the cap act refuses. Stop early when your goals are done or blocked.
After three failed tries at one step, give up that goal and write what
you expected. Keep your thinking short.

## Report

Write /home/user/wt/log/feedback/2026-11-11-swarm3-NAME.md, at most 40
lines. First line `# NAME (FLAVOR), simulated user`, then one line per
goal (`goal: reached yes/no/partly, tries N`), then one section per
finding in exactly this form:

    ## <short title>
    kind: issue | feature | ux
    persona: <your persona line>
    flavor: <flavor>
    severity: high | medium | low
    steps: <what you did>
    expected: <what you expected>
    actual: <what you saw>
    evidence: shots/NNN.png
    status: new

Report every surprise, also small looks (cut text, overlap, odd colour,
misalignment). Add findings of kind `ux` for what was hard to find, read,
understand, reach or undo, or one line `ux: none seen, because <reason>`.
Do not guess causes. Write nothing else and commit nothing.
