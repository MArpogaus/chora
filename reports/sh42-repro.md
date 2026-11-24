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
