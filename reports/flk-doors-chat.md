# flk-doors-chat

## r1

Head: ec8de7b7 on claude/flk-doors-chat (one commit on 32a505db; origin/claude/autonomous-dev-session-3w-i33pg7 at 1c599c74 is already in it).

Repro on 32a505db: `merge-gate.sh --jobs 1 --only build,doors` 5 times, launcher-chat tiling missing 1 of 5. The tiling cells alone (`tests/doors/tiling.sh`, 21 s) missing 11 of 22 runs. The launcher cells alone (ONLY=launcher-key|launcher-door|launcher-chat) 0 of 20. Bases 1c599c74 and 80f6ae67 were not measured: the cause below is in the stand-in and the waybar poll, and the cell fails by timing, not by a merge, so naming the merge was not needed to fix it.

Cause: tests/doors/lib.sh fuzzel_stand_in (`sed 's/^/row /' >>"$marks"`). The launcher (crates/chora-kit/src/launcher.rs run_ordered, helper_last puts the row Chat with a helper last) gives fuzzel its rows without a trailing newline. sed keeps that, so the last row `row Chat with a helper` ends the marks file without a newline. waybar polls wpctl all the time, and the stand-in wpctl appends `wpctl get-volume ...` to the same marks file; when its line lands first, it joins the row: `row Chat with a helperwpctl get-volume ...` (captured in a failing run). cell_launcher_chat (tests/doors/tiling.sh) greps `^row Chat with a helper$` and never matches. Only tiling has waybar, so only tiling shows it. Not a product fault (real fuzzel does not need the newline), not a timeout, not the ti-29 panel.

Fix: the stand-in ends its last row with a newline (`sed 's/^/row /; $a\'`). A test fix, so no red test commit. After it: tiling.sh 26 of 26 runs without launcher-chat missing (was 11 of 22).

Gate: `tests/merge-gate.sh --jobs 1` full on ec8de7b7: all steps passed (doors 369 s, e2e 286 s). Doors alone `--only build,doors`: 5 of 5 PASS (369-374 s). Not pushed to the train branch.

### PM review r1 (2026-11-21 16:01 UTC)
- ec8de7b7 (tests/doors/lib.sh:204-206): `sed 's/^/row /; $a\'` ends the last dmenu row with a newline only if it lacks one (GNU sed); a waybar wpctl line can no longer join `row Chat with a helper`. Test-only, 3 lines, no product change; the cause and repro (tiling.sh 11/22 red before, 26/26 after) are named; full gate green on the fix.
- MINOR (no action): a concurrent writer could still split a line if writes interleave mid-line; not seen in 26 runs.
- Verdict (computed): no open MAJOR -> MERGE-OK at ec8de7b7; joins t101 after t100.
