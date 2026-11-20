# flake-h

Brief: briefs/flake-h-r1.md. Lane: cloud, branch claude/flake-h from the train 72 merge 219d5afe. Test files only; no production code changed.

## r1

Setup: step 0 tools present (cargo, sccache, foot, python3), setup.sh not run. `git fetch --unshallow`, `cargo check --workspace` exit 0, `cargo build --workspace --locked` for the binaries.

Command: `tests/merge-gate.sh --jobs 3 --only matrix,layout,doors,pull`. The brief's `--list` does not exist in merge-gate.sh and the step of gate-construct-pull.sh is named `pull`, not `construct-pull`.

### Reproduction

Without extra load, 5 of 5 green on 219d5afe (each run 5m40 to 5m45; layout 199 s, pull 64 s, matrix 288 s, doors 340 s). Nothing red, so I added the load of the earlier reds: `tests/under-load.sh --runs 5 --cpus 2` around the same command. 3 of 5 green:

| load run | red cell | line | waited |
|---|---|---|---|
| 1 | layout | `FAIL: light-1280x720: Super+T in the walk started no third terminal`, then `the hints with one pane are: ` and `a window is lost` | `until_ok 'panes_are 3'`, 40 rounds of 0.25 s, about 10 s |
| 3 | doors | table cell `helper-door` of xp `missing` (the cell function returned false) | cell has one tooltip read and `until_ok chat_open` of 10 s |

The pull cell (pane 169 px for 200, wm5 r3) did not go red in 10 runs here (5 plain, 5 loaded of which 4 reached it); no run of it was red, so the cause below is from reading the gate, not from a red run of mine.

### Causes and fixes (commits 745def72, bc3f1662)

- layout (gate-construct-layout.sh): `until_ok` counted 40 rounds, so under load the budget was a round count and short for the start of a foot terminal; and the wait for the pane of the sunset ended in `break` or fell through without a check (silent). Fix: `until_ok` takes a wall-clock budget in seconds (twice the nominal rounds time: 40 rounds is 20 s); the sunset wait fails with a line if it runs out.
- pull (gate-construct-pull.sh:165): `shown_width` was read once, after `sleep 1` from the drop, while the pane still eases to its size (169 px for 200). Fix: `until_ok '[ "$(shown_width)" -ge 200 ]' 40` before the assert, so a pane that never reaches 200 px still fails. The same `until_ok` change.
- doors (tests/doors/lib.sh, xp.sh): `until_ok` the same change; `cell_helper_door` read the helper word once and returned false on an empty word or "No helper"; it now hovers again until a state word shows (20 s budget in seconds, `helper_word_shows`). Cause of the red run 3 not isolated: 6 runs of the cell alone under 4 busy loops were green, so this is a hardening of the two single reads and the wait, not a confirmed single cause.
- FF1 rw-1 (gate-tiling-audit.sh `fullscreen_rests`) and rw-4 (under-load.sh `timeout -k`) were already fixed on 219d5afe by c227d433 (flakes-g); no change.

Side finding (fixed in bc3f1662): tests/doors/xp.sh `settle` calls `menu_close` in every cell and `menu_open` (text "Turn Off Computer") is true with no menu shown, so the Escape wait `! menu_open` always runs out: about 10 s per cell, 19 cells. My first fix doubled it (xp doors 413 s, doors step 513 s on all 5 runs of the first set, green but slow); bc3f1662 keeps the old 24 rounds (xp 253 s alone, doors step 357 s). The cause (`menu_open` true when closed) is left: backlog.

### 5 runs after the fix (bc3f1662, same command, no extra load)

All 5 exit 0, "partial run: nothing is pushed".

| run | start (UTC) | end | layout | pull | matrix | doors |
|---|---|---|---|---|---|---|
| 1 | 22:56:07 | 23:02:04 | PASS 197 s | PASS 64 s | PASS 285 s | PASS 357 s |
| 2 | 23:02:04 | 23:08:02 | PASS 197 s | PASS 64 s | PASS 281 s | PASS 357 s |
| 3 | 23:08:02 | 23:13:56 | PASS 197 s | PASS 62 s | PASS 279 s | PASS 354 s |
| 4 | 23:13:56 | 23:19:51 | PASS 195 s | PASS 62 s | PASS 278 s | PASS 354 s |
| 5 | 23:19:51 | 23:25:48 | PASS 194 s | PASS 63 s | PASS 282 s | PASS 357 s |

An earlier set of 5 on 745def72 (before bc3f1662) was also 5 of 5 green (doors step about 513 s).

Not done: the fix was not run under `under-load.sh`. The reds only showed under load (2 of 5), and the 5 green runs are without load, so they show no regression, not that the load reds are gone. Say so if a loaded set is wanted.

Head: bc3f1662e4cb329f97267df6cb6a2991d323c721 on claude/flake-h.

## PM review r1 (23:30)
Diff 219d5afe...bc3f1662: tests/doors/lib.sh, xp.sh, gate-construct-layout.sh, gate-construct-pull.sh (+19 -16). Read in full.
- rw-1 MAJOR: the reds showed only under `under-load.sh --cpus 2` (3 of 5 green); the fix was run 5 times without load. No passing cell under the load that failed, so the claim "load reds gone" is open (rules.md 7).
- rw-2 MINOR: `until_ok`'s second argument keeps the old round count and halves it to seconds (`${2:-40} / 2`); a caller reads "40" as rounds. Name the unit in a comment at the function.
- rw-3 NIT: `menu_close` passes 24 to keep the old 10 s because `menu_open` is true with no menu shown; the root cause goes to the backlog (lane's side find).
Good: the sunset wait now fails instead of falling through; pull waits for 200 px instead of one read.
Verdict: FIX (rw-1 open MAJOR). r2: run under load, fix what is red, rw-2.

## r2

Lane: cloud, branch claude/flake-h from bc3f1662. setup.sh was not run (the auto-mode classifier denied it; tools cargo, sccache, foot, sway, python3, jq present as in r1). `cargo build --workspace --locked` ok. The brief's step `construct-pull` is named `pull` in merge-gate.sh (the first try said `unknown step construct-pull`, 0 of 5, no test ran).

Command: `tests/under-load.sh --runs 5 --cpus 2 -- tests/merge-gate.sh --jobs 3 --only matrix,layout,doors,pull`.

| set | head | result | run lines |
|---|---|---|---|
| A | bc3f1662 | 5 of 5 | PASS 360, 358, 358, 354, 357 s |
| B | 9e1aa126 | 4 of 5 | PASS 358, 357, 358; run 4 FAIL 391 s; PASS 352 s |
| C | b0564df2 | 5 of 5 | PASS 374, 361, 359, 359, 363 s |

Red cell (set B run 4): doors, cell `helper-door` of xp `missing`; same cell as the r1 red. The log showed no step, so the cause is not isolated: the cell had three single steps (helper word, one click, `chat_open`). A click injected under load can be lost. Fix (b0564df2): the cell clicks up to 3 times until the chat opens, and each exit prints a `FAIL: helper-door: ...` line naming its step. Set C is the 5 loaded runs after that fix.

rw-2: the repo hook `no-inline-comments` rejects an added comment, and `--no-verify` is barred. The unit is named in code instead (9e1aa126): `local rounds=${2:-40}`, `end=$((SECONDS + rounds / 2))` in tests/doors/lib.sh, gate-construct-layout.sh, gate-construct-pull.sh.

rw-3: no change.

Commits: 9e1aa126 (rw-2), b0564df2 (helper-door). Head: b0564df2.

## PM review r2 (2026-11-20 01:10 UTC)
Diff bc3f1662..b0564df2 on top of the session branch: tests only (doors/lib.sh, doors/xp.sh, gate-construct-layout.sh, gate-construct-pull.sh); no product code. Done criterion holds: set C 5 of 5 under `--cpus 2` on b0564df2.
- MINOR-1: the helper-door retry (up to 3 clicks) rests on an unproven cause ("a click injected under load can be lost"). If the first click opens the chat after 10 s, the second click may toggle it shut. Not a MAJOR: the cell still fails when no click opens the chat. Backlog: print the click count in the cell, and a product check if a count above 1 shows up.
- No BLOCK or MAJOR. Verdict (computed): MERGE-OK b0564df2, train 80.
