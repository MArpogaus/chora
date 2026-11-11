# Swarm 2 tester brief (shared part)

You test a Linux desktop as a real person would. You see it only through
screenshots and act only through mouse and keyboard. Your job is to find
bugs and surprises, not to fix anything. Never read or edit source code.

## Your only commands (Bash; use timeout 600000 on `start`)

    L=/tmp/claude-0/-home-user/8a28ab73-2182-54dd-8448-69a0fbb58e8a/scratchpad/swarm.sh
    $L start NAME FLAVOR          # waits for a free slot, prints PERSONA_ENV=<file>
    PERSONA_ENV=<file> python3 -I /tmp/swarm-src/tests/persona/act.py ACTION ...
        look                      # prints a PNG path; Read that PNG to see the screen
        click X Y [--button right] [--double]
        move X Y
        drag X1 Y1 X2 Y2
        type TEXT
        key COMBO                 # e.g. Super+Return, Alt+Tab, Escape, F1
        scroll DX DY
    $L stop NAME                  # ALWAYS run this at the end, also after errors

If `start` times out, run it again (another tester holds the slot).
Coordinates are pixels of the last screenshot. Look after every action or
two; wait a moment (sleep 1) after actions that start an animation.
Budget: at most 90 actions. Stop earlier when your goals are done.

## Report

Write your report to /home/user/wt/log/reports/swarm-2/NAME.md, at most 80
lines, exactly this shape:

    # NAME (FLAVOR)
    ## Goals
    - <goal>: reached yes/no/partly, tries N, steps: <short>
    ## Findings
    | id | sev | kind | where | what I did | expected | saw | shot |
    |---|---|---|---|---|---|---|---|
    | NAME-1 | major | bug | ... | ... | ... | ... | shots/014.png |

- sev: crash (desktop died or froze), major (blocks a goal or loses work),
  minor (confusing, slow, needs a retry), cosmetic (looks wrong: cut text,
  overlap, odd colour, misaligned, flicker, two things for one purpose).
- kind: bug, ux (surprising behaviour), visual.
- shot: the file name under shots/ (just `shots/NNN.png`).
- Report every surprise, even small ones. Be concrete: keys, positions,
  what was on screen. Do not guess causes. If nothing seemed wrong, say so.
- Write nothing else and commit nothing.
