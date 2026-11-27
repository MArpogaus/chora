# flake-launcher

Brief: briefs/flake-launcher-r1.md (F5, rc 3: the launcher-start flake family). Lane: cloud, branch claude/flake-launcher-r1 at 0e18f14c, one area (the mouse path to the launcher of mc).

## r1

Result: one root cause found and fixed for the launcher-mc misses (9 of 10 runs red before, 0 of 10 after, all four cells 40 of 40 green after, full gate green). The xp-run (sh-44) and handover (sh-47) misses did not reproduce here, before or after, and stay open.

Head after the fix: d06b9be2 on claude/flake-launcher-r1 (red test 2a90b905, then fix d06b9be2).

### Step 0

`git fetch --unshallow`, `cargo check --workspace` and `cargo build --workspace` ran clean. No setup script.

### Miss counts, same load before and after

Load: six busy bash loops (`while :; do :; done`) beside the cells on 4 cores (load average 6 to 7.5); the cells run one after the other (the gate lock serialises them). Each cell is its own script as merge-gate.sh calls it: xp-run = gate-xp-run.sh; launcher-mc = gate-launcher-mc.sh; launcher-click = gate-launcher-click.sh; handover = gate-handover.sh then gate-looks-move.sh.

| cell | before (0e18f14c): runs, misses | after (d06b9be2): runs, misses |
|---|---|---|
| xp-run | 10, 0 | 10, 0 |
| launcher-mc | 10, 9 | 10, 0 |
| launcher-click | 10, 0 | 10, 0 |
| handover | 10, 0 | 10, 0 |

All nine before-misses are the same line in launcher-mc: "no launcher after mouse (a fuzzel process; the log of the session says: )", in both themes and with both fuzzel (light new run 2, 4, 5; light old run 1, 8; dark new run 7, 9, 10; dark old run 6). No miss after the keys (Super+Space), no "launcher process runs but nothing is drawn". The line "the log of the session says:" is empty: mc logs nothing at the click, so the gate never says what happened.

Logs: media/2026-11-26/flake-launcher-r1/ : before-summary.txt, after-summary.txt (cell, run, exit code, seconds), before-launcher-mc-run1.log, before-launcher-mc-run2.log (the first failing logs), before-run2-session-chora.log, before-run2-session-actions.jsonl, before-run2-base.png (the screen at the miss: the Programs door is at 888..953, 741; the click went to 920, 742), after-logs/ (all 40 runs of the four cells, concatenated per cell), before-chora.fail.log (a second miss), after-full-gate.log.

### Root cause

mc draws its regions in two alternating sets of nodes, `mc-r0-...` and `mc-r1-...` (plugins/chora-mc/src/region.rs `render`, `prepare`, `set_id`). A render that changes the items (a new key list, the pill, the chat) builds the other set, and the old set fades out for 250 ms (FADE) and goes invisible; during the fade its hits still take the pointer. A press on a hit arrives as `PointerPressed { node: "mc-r0-right-doors-programs-hit" }`; `hit_token` (plugins/chora-mc/src/region/items.rs) looked up the item with the id `mc-r0-...` among the items of the current set, `mc-r1-...`, found none, `click_hit` fell through to `click_row`, which returned false, and the press did nothing: no `Host::run`, no `choractl run`, no fuzzel. The miss is therefore a lost click on the set that fades, not a slow or failing spawn: the gate's `launcher_up` waits 40 s and no fuzzel is ever started.

Evidence at a miss (live session, after the 40 s wait, before the session stopped): `scene.query owner:mc` lists both sets: `mc-r1-right-doors-programs-hit` visible with rect [888, 728, 65, 28] (the current set) and `mc-r0-right-doors-programs-hit` with no rect (the old set), the fake `calls.log` has no fuzzel call and no `choractl run` ran, the persona log has the click at 920, 742 with ok. In passing runs the same flip is seen from mc itself, 650 ms after the press (a diagnostic build of mc wrote its events to a file: `PointerPressed node=mc-r0-right-doors-programs-hit`, then `PointerLeft mc-r0-...`, `PointerEntered mc-r1-...`). There is no log line for the lost press (mc logs nothing there); the proof is the unit test below, red on 0e18f14c without the fix and green with it, and the cell going from 9 misses in 10 to 0 in 10 with no other change. I did not catch the event order of one miss itself: every diagnostic build that logged in mc shifted the timing and passed (16 of 16), so that part is inferred.

Why the load matters: a render of the set takes longer, the flip comes closer to the click of the gate (`programs_door` reads the rect, the persona approaches the door and presses 0.3 s later), and under the real gate the CPUs are busy. The same loss is real for a person who clicks a door in the 250 ms of a fade.

### Fix

Red test commit first (2a90b905): `a_click_on_a_hit_of_the_set_that_fades_out_still_acts` in plugins/chora-mc/src/main.rs: page "morning", the hit of the footer line to the previous page in the set before, the chat opens (a new set; the old hit is still a node), `click_hit(old)` must be a hit and turn the page. It fails on 0e18f14c at "a hit of the old set is a hit".

Fix commit (d06b9be2): `hit_token` reads a node of the set before the current one as the same item of the current set (swaps the `mc-r<N>-` prefix). No timeout changed, no cell touched, nothing skipped. All 222 chora-mc tests pass.

### Full gate

`tests/merge-gate.sh --jobs 2`, no --only, on d06b9be2, no load beside it: exit 0, "all steps passed", 67 PASS lines, 0 FAIL; docs SKIP ("mkdocs is not installed (docs unchanged)", the host gap of the r10 series). Log: media/2026-11-26/flake-launcher-r1/after-full-gate.log. `pgrep -fa 'chora --headless'` empty at the end.

### What is not fixed

- sh-44 (xp-run: "Super+Space did not run the launcher", t154, t155) and sh-47 (handover: "the picker of the Looks door did not start", the waybar Looks word) did not reproduce: 0 of 10 each before the fix and 0 of 10 after, under the load above, on this host. They are not the mc bug (xp has no hits of mc, tiling's door is waybar). The launcher-mc misses of t156 and kit-swallow r1 run 2 are the mc bug.
- One unproven lead for xp-run: `Wm::start_on` of chora-xp creates its scene (`create_scene`) and only then announces and grabs its keys (`announce`, `grab`), and `session_start` of the gates is ready when the nodes of the owner exist, so a Super+space injected in that gap has no holder and is lost, and the cell waits 10 s for a fuzzel call that never comes. I did not reproduce a loss this way, so I changed nothing; a gate that waits for the grab (no query shows the held bindings yet) would be the test side of it. Left for the next flake round with those two sightings.
- `Mc::mark` on a node of a set that is already removed returns the core's error, which `dispatch` passes up: not seen here, noted only.

### Rules kept

Pushed claude/flake-launcher-r1 and claude/session-log only; author and committer Marcel Arpogaus; no attribution lines; no --no-verify; no system packages; no bare git stash. The diagnostic copy of the gate and the diagnostic mc builds were never committed.

## Review r1 (PM, 2026-11-27, 0e18f14c..d06b9be2, 2 files +43 -1)

PM review (under 200 lines, procedures). Checks:
1) Red first: 2a90b905 adds `a_click_on_a_hit_of_the_set_that_fades_out_still_acts` before the fix d06b9be2 (lane: red on 0e18f14c). Pass.
2) Root cause, not timeout: the fix maps a hit of the fading set `mc-r{set^1}-` to the same item of the current set in `hit_token`; no wait changed. Pass.
3) POLS: an old-set hit whose item no longer exists in the current set still finds no item, so the press does nothing (no wrong action). The two-set swap is the existing design (`self.set ^ 1`). Pass.
4) Evidence: launcher-mc 9/10 red before, 0/10 after; four cells 40/40; full gate 67 PASS on d06b9be2. Pass.
Findings: none. sh-44 and sh-47 stay open (not reproduced), lead to the next flake round.
Verdict (computed): MERGE-OK, joins the next train.
