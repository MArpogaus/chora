# Final review loop over 88cd086..HEAD — CLOSED clean

Date: 2026-10-25. Repo: /home/marcel/Projekte/chora, branch dev,
tip a0d7f9c, 47 commits in the range. Nothing pushed.

## Verdict

Two consecutive audited rounds with `open: 0`: R10 and R11.
All rounds ran as pi sessions with the gufo model; every report is
in this directory (review-r1.md .. review-r11.md), each with its
session and an audit.py check.

| Round | Verdict | Open findings | Resolved in |
|---|---|---|---|
| R1 | 9 open | T3, T12, T13, T14 missing; gate 8; run double start; XDG_RUNTIME_DIR; 300-line; text docs | 6ca4245, 77550e5, d660de2, 50ed2e9, 859a7e5, 99d867b |
| R2 | 3 open | spawn deprecation, fork needle, gate 8 wording | 18a4810 |
| R3 | 1 open | my 75 char commit subject | d34905d (accepted, history not rewritten) |
| R4 | void audit + 3 | emoji never inked; 0.8 gate rows; dependencies.md | a4b9ee9, 4642586 |
| R5 | 2 open | space warned and scanned; awk \\# warning | 05f5cdc |
| R6 | void audit + 1 | w2 read places during the xp lamp | 2ac21f1 (harness) |
| R7 | 2 open | gate 4 violated the T3 socket rule; 4 prose lines over 80 | 907c839 |
| R8 | 1 open | the run to window token chain of T4 untested | 437b404, c8a3326 |
| R9 | 2 open | em dash in rustdoc; run docs said pid only | a0d7f9c |
| R10 | clean | open: 0 | — |
| R11 | clean | open: 0 | — |

The two void rounds cost nothing: R4's items were fixed anyway, and
the audit false positive (a quoted /tmp mkdir counted as a mutation)
is fixed in the skill (local-pi-session 7a3f012).

## What the loop caught that the nightly run missed

- Color emoji were laid out and never drawn (swash does not decode
  CBDT PNGs); gates 0.8-9 and 0.8-10 are green for real now.
- Every space in a text node warned and triggered a full fallback
  scan of the font database.
- The T3 client socket rule broke gate 0.7-4 on its only runner.
- The T4 token chain (run exports the token, the window carries it)
  existed only compositor-side; the probe now prefers
  XDG_ACTIVATION_TOKEN and agent.sh step 21 follows the whole trip.
- w2 sampled window places during the magic lamp animation, where
  the clipped rect query legitimately has no resting box.

## Verification state on this box

- fmt, clippy -D warnings, workspace tests: green with the EGL and
  CHORA_FONT_DIRS environment.
- Gates green here: 0.7 checks 1, 2, 5, 7, 8, 9, 10; 0.8 checks 1, 2,
  3, 5, 6, 7, 9, 10.
- Not runnable here, CI or user only: podman container part of 0.7-4,
  systemd user part of 0.7-5 (0.7 gate row 4 and 5), foot and sway
  scripts (0.7-8, 0.8-11, tests/agent.sh), the push itself.

## Questions for Marcel (open since the nightly run)

1. Activation token lifetime. `choractl run` now prints the token on
   stdout next to the pid so that tests can follow it to the window.
   The token is short lived, but on a multi user machine the stdout
   of `run` is readable by whoever started it. Keep it, or print the
   token only with a flag like `--show-token`?
2. Recreate on changed capabilities (T3, confirmed by the loop):
   `clients up` no longer keeps the socket when the capabilities
   change; the connected Wayland clients of that name die with it.
   This is intended (D40) and documented, but it is a promise that
   `clients up` used to give. Sign it off for the release notes?
3. 300 line waivers. T2, T6 got a waiver line in the plan; T5 got a
   retro active waiver in 859a7e5. Next crossing: split before
   commit, or one waiver per task decided at review time? The rule
   as written ("ask, do not commit") never fired because the run
   committed first; pick the handling you want for 0.9.
4. Release shape. The range contains breaking changes (D42: the core
   starts nothing; `process.spawn` and `process.wait` are gone; T3:
   `grants.set` refuses names with a client socket). Tag
   v0.7.0-rc.1 and drive the remaining gate rows (4, 5, 8, 11) on a
   machine with podman, systemd user, sway, foot and grim?
