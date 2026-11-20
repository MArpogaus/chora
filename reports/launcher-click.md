# launcher-click (RC shared bar: a visible way to start a program)

Lane: cloud, branch claude/launcher-click from b97369d1 (nothing to merge: the session branch was still at b97369d1). Step 0: unshallow, cargo check and build green.

## r1
Root cause: (a), the harness. `tests/persona/act.py` click sent one `pointer_move` and the `click`. The launcher is fuzzel 1.11.1: it takes the first motion of an entered surface as its start and selects the row under the pointer from the second motion on, and a move that enters its surface is an enter, not a motion (the same finding as tests/gate-tiling-apps-click.sh, commit f11eb6ce). So after the click on Apps the first row (Chat with a helper in the rounds, Alpha with my entries) was still highlighted, and the click started it. Not a product fault: a hand makes many motions, and the click with three or more moves starts the right row on both looks.

Repro (persona session, entries Alpha, Mid, Zebra, launcher open, Mid at 400,277 on tiling and 400,196 on construct, capture 800x500):
- `act.py click 400 277` alone: `alpha: 1 mid: 0` (tiling), `alpha: 1 mid: 0` (construct, row 400,196). The log line is a single click action; before the fix it injected `[pointer_move, click]`.
- `act.py move 400 265; act.py move 400 272; act.py click 400 277`: `alpha: 0 mid: 1` on tiling, and the same with 400 190, 400 193, 400 196 on construct.
So the launcher is right and one move is too few. Both rounds' findings (beginner: Apps > Foot 3 times, explorer: Programs > Foot) are this and not a product defect. "Chat with a helper" is the first row of the list only when nothing sorts before it.

Second finding of the lane (harness, fixed in the cell): fuzzel keeps a history in `$XDG_CACHE_HOME/fuzzel` and puts a started row first in the next list, so a cell that clicks a fixed y is flaky when the cache is the machine's (the Mid row had moved to the top: trials gave mid, none, alpha). The cell gives its session a cache folder of its own.

Commits on claude/launcher-click:
- red: ce194597 `test(persona): a click by act.py on a launcher row starts that row` (alone: tests/test_act_pointer.py, tests/gate-launcher-click.sh, the cell `launcher-click` in tests/merge-gate.sh). Red at that commit: `FAIL: tiling: a click on the Mid row of the launcher did not start mid (alpha, the first row: 1, the chat: 0)`; the selftest of act.py fails 3 of 6 (click, double click, drag: 1 move before the press, a hand makes at least 3).
- fix: 741d6bca `fix(persona): act.py moves the pointer as a hand does before a click`: act.py `approach()` sends three small moves of 3, 2 and 1 pixel before the target, each as its own inject with a pause of 50 ms, then the target; click and the press of a drag use it. Moves sent in one inject (four in one batch) were not enough on the construct: the cell was flaky there until each move went alone.
- the same fix also covers item 3: drag moved once before its press and now approaches the same way; `move` stays one move (a plain hover, the caller makes the hand); `scroll` takes no position and sends no move by design (it acts where the pointer is). tests/test_act_pointer.py pins all four.

Test: tests/gate-launcher-click.sh = the selftest of act.py (events of click, double click, right click, drag, move, scroll through a stand-in choractl) + a persona session each for tiling (Apps door at 35,15) and construct (Programs door at 96,32) that clicks the Mid row by act.py and asks for app_id mid and no alpha. Red at ce194597, green at 741d6bca, 3 of 3 reruns green on both looks, and red again with the old act.py and the final script.

Gate cell: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,doors,keys,tiling-look,launcher-click` on 741d6bca: all nine PASS (pre-commit 1 s, build 24 s, test 209 s, clippy 16 s, ci-check 89 s, keys 18 s, launcher-click 9 s, doors 355 s, tiling-look 577 s). Log: media/2026-11-20/launcher-click/merge-gate.log.

Capture: media/2026-11-20/launcher-click/after-door-launcher-open.png (Apps list, Alpha highlighted) and after-fix-click-mid.png (after `act.py click` on Mid: the Mid window opened, Alpha not); actions-fixed.jsonl.

Not touched: crates/chora input.rs, flavors, the launcher config. Open for the PM: whether a compositor that sends an enter and then a motion for an injected `pointer_move` that enters a surface is wanted, so that a one-move click works by itself (input.rs, the stray-grab lane); not needed for the personas now.

## Review r1 (PM, 2026-11-20 02:42 UTC; PM not the builder; small harness diff)
Base b97369d1..741d6bca, 4 files, +269 -6 (tests/persona/act.py, tests/test_act_pointer.py, tests/gate-launcher-click.sh, tests/merge-gate.sh).
1) Red first: ce194597 (test) before 741d6bca (fix). Pass.
2) approach(): 3 moves at offsets 3, 2, 1 px then the target, each injected alone with 50 ms; click and drag press use it, move and scroll unchanged; pinned by test_act_pointer.py. Pass.
3) Cell isolation: own XDG_CACHE_HOME and XDG_DATA_DIRS; session stopped in a trap. Pass.
Findings: lc-1 NIT act.py approach: a target at x or y below 3 gets a negative start point (clamped by the core, untested). lc-2 NIT gate-launcher-click.sh: fixed `sleep 2` before the row click (wait on the launcher's first frame instead). lc-3 NIT: the four door and row positions are hard-coded capture pixels.
Open question of the lane (one-move click in input.rs): backlog, not RC.
Verdict (computed): no BLOCK or MAJOR: MERGE-OK. Train 82 alone (harness branch first); merge conflict in merge-gate.sh step list with mc-column resolved (both kept).
