# sh42 repro

## r1

Lane: cloud, branch claude/sh42-repro-r1 at c3c37c2e (session head). Run only: no product, test or design change.
Status: **BLOCKED** at the hand-run harness. No rates measured, no captures, no sh-42 result either way.

### Step 0 (ok)

- `git fetch --unshallow` ok.
- `cargo check --workspace`: EXIT 0 (1m42s).
- `cargo build --workspace`: EXIT 0.
- `pgrep -fa 'chora --headless'`: empty after the runs.

Note: the r2 fix commits (stray-super-r2, head 1eb30643) are not in c3c37c2, so this is the repro on the unfixed head as the brief asks.

### What the runs showed

- Super+Return in xp opens no window. The Super press is a lone-Super tap that opens the Start menu (`open` shows `xp-menu`), so the chord does not reach a window.
- The xp window for the repro is Notepad from the Start menu: a click at (50,785) opens the menu, then a click on the rect of the `xp-item-2` node (`meta.text` "Notepad") launches the editor. The editor is set with `editor: foot -a notepad sh <script>` in `$XDG_CONFIG_HOME/chora/chora-xp.yaml`; the script is `stty raw -echo; cat > FILE`, so typed bytes land in FILE.
- A debug probe (click on Start, then `ctl query id:xp-menu`) opened the menu 3 out of 3 times. The same click in the repro script did not open it in 3 out of 3 tries (runs 5 to 7, with a retry loop). Cause not identified. The two differ only in the test script's surroundings, the editor config and the retry. Without a visible menu no Notepad, so no foot, so no typed-text test.

### Steps still to settle (next lane)

1. Fix the Start-menu open in the repro (or use `tests/persona/session.sh xp+tiling` with `act.py click`, the driver built for this, rather than my script).
2. Then the matrix: (a) xp with a Notepad foot, tiling joins, type `echo rN`; (b) mc with a foot, tiling joins; control: xp with no window, tiling joins, Super+Return; xp to mc without tiling.
3. Record per run: holder of xp, tiling and mc (`choractl state <look>` `.focus.holder`), owners of `kind:surface`, whether the typed bytes reach the focused foot, and in mc whether a click or Return gives the keyboard back.

Captures: none. Scripts are in the lane scratchpad, not committed.

## r2

Lane: cloud, branch claude/sh42-repro-r2 at c3c37c2e (session head, unfixed). Run only. Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` ok. `pgrep -fa 'chora --headless'` empty at the end.
Harness: `tests/persona/session.sh xp|mc` with `CHORA_TEST_FLAGS=--test-injected-press-is-real`, typing and keys by `tests/persona/act.py`, join by `choractl run ... chora-tiling --no-companions` (as session.sh starts a joiner). `session.sh xp+tiling` starts both at once, so the window could not exist before the join; I started the first look alone, opened the window, then joined tiling. Script and raw per-run lines: media/2026-11-24/sh42-repro/run.sh, results.txt (the Start-menu route was not used).

Window: `foot -a r2 sh -c 'stty raw -echo; cat > FILE'` (xp); `foot --app-id=mcstray ...` plus `Return` (mc page). Typed text is read back from FILE.

### Steps (shortest)
1. `session.sh xp 1280x800` (or `mc 1280x720`).
2. `choractl run foot ...` as above; in mc `act.py key Return`. Typing `echo aN` reaches FILE (5/5 both).
3. `choractl run chora-tiling --no-companions` (in its own dbus session, `CHORA_FLAVOR_DIR=flavors/tiling`); wait 5 s.
4. `act.py type "echo rN"`: nothing reaches FILE.

### Rate table (state after the join; core = surfaces with focused=true)
| Path | runs | core focus after | look holder after | typed text lost | recovered |
|---|---|---|---|---|---|
| xp window, tiling joins | 5 | none 5/5 (before: s1) | xp="xp", tiling=null | **5/5** | click on window 5/5 |
| mc page, tiling joins | 5 | none 5/5 (before: s1) | mc=null, tiling=null | **5/5** | Return 0/5, click 0/5 |
| control: xp, no window, tiling joins | 3 | none (none before) | xp="xp", tiling=null | n/a (no window) | n/a |
| control: mc, no window, tiling joins | 3 | none | mc="mc", tiling=null | n/a | n/a |
| control: xp window, mc joins (no tiling) | 3 | s1 stays focused | xp="xp", mc=null | **0/3** (`echo rN` arrives) | n/a |

Notes: the xp to mc control is mc started beside xp (a join), not a look switch; no switch-without-join command was found in the harness. Holder before the join in all window runs: xp "xp" / mc "mc", core s1. xp holder says "xp" while no surface is focused, so xp's state and the core disagree; in mc the holder itself is null. The mc `Return` and click recovery was tried in that order (Return, then a click at screen centre); the page window text stays `^M`-only, i.e. only the pre-join Return.

Captures (4, under 150 kB each): media/2026-11-24/sh42-repro/{xp,mc}-join-1.png, {xp,mc}-nowin_join-1.png.

sh-42: keys lost 5/5 (xp), 5/5 (mc); recovered by click 5/5 (xp), 0/5 (mc). Reproduces on the session head whenever tiling joins while xp or mc holds a window; a no-window join and an mc join beside xp lose nothing.
