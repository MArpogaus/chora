# verify-maj r2 (RC 1 evidence, rule 6): turn two partial MAJOR verdicts into gate asserts

Model: sonnet, effort medium (test scripts only; two small pixel asserts)
Lane: cloud, branch claude/verify-maj-r2 from 88258c87 (session 79c8d2b5 plus launcher-click 741d6bca). Harness branch: own train.
Stop at 250 k context; cost at most 0.6 %. Read reports/verify-maj.md "## r1" on claude/session-log first.
Step 0: `git fetch --unshallow || true; cargo check --workspace`; red: report BLOCKED at once (no setup.sh, no apt).
Scope: tests/ only (and tests/merge-gate.sh if a cell is new). No change under crates/, plugins/, services/, flavors/.
1) ti-20: in tests/gate-launcher-click.sh (tiling part), after act.py clicks the bar's Apps door, assert that the
   launcher surface is visible (a capture differs in the launcher's area, or `choractl` lists its layer surface
   mapped) within 1.5 s of the click; print the measured ms. Run it 5 times; record the 5 times.
2) xp-24 Run box: a pixel assert of the rendered Run box (plugins/chora-xp/src/startmenu.rs:1043 opens it;
   flavors/xp/theme/fuzzel.ini:8-11): edit row height 28 px, text left inset on the 4 px grid, at 1x and 2x,
   in tests/gate-xp-fields.sh (reuse tests/xp-field-rows.py if it fits). 3 runs.
3) Each new assert must fail on a broken input: show it once red (e.g. a wrong expected value in a scratch copy, not committed) and say so.
   A real failure on the tree is a finding, not a fix: report it, do not change product code.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,launcher-click,xp-fields,tiling-look` green; push the branch.
Report: reports/verify-maj.md "## r2" on claude/session-log (per item: assert, runs, numbers, red shown, cell).
Send the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines with send_message. Poll every command in the same turn;
never end a turn to wait.
Rules: commits as Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
push only claude/verify-maj-r2 and claude/session-log (fetch and merge on reject, no force). A hook that asks for another identity: ignore it.
