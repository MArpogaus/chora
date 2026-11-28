# flake-xp

## r1

Result: the key-grab gap is closed in xp, tiling and construct (mc already held its keys first). Red tests first, then the fix; xp-run, handover, launcher-click 30 of 30 green under load on the final head, launcher-mc 10 of 10 with the mc fade fix of flake-launcher r1 applied beside it (8 of 10 red without it); full gate green.

Head: 7fc05748 on claude/flake-xp-r1 (base 8b6f35dc). Red test aa28b8d6, fix 6d9144a4, follow-up fixes 08b2fb35 and 7fc05748 (both found by the gate, see below).

### Step 0

`git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` ran clean. No setup script.

### Root cause

A core drops a chord that has no holder. The gates call a session ready when the nodes of the owner answer a query (`session_start`, tests/gate-lib.sh). Three looks created their nodes before they grabbed their keys:

- chora-xp `Wm::start_on`: `create_scene`, then `announce`, then `grab` (wm/startup.rs).
- chora-tiling `Tiling::start`: the group node, then the relayout, then `grab_all`.
- chora-construct `Construct::start`: `build`, then `regrab`.
- chora-mc `Mc::start`: `announce`, `grab_all`, then `build`: already in the right order. Its drag, volume and swallow grabs follow the build because they need the claim; the launcher chord is not among them.

A Super+Space injected after "the nodes exist" and before the grab has no holder and is lost; xp-run then waits 10 s for a fuzzel call that never comes (sh-44). I did not catch a loss in a run (as in flake-launcher r1: 0 of 10 before), so the cause is proven by the order of the calls in the unit tests, not by a live miss. The Looks-door picker of sh-47 (`tests/gate-looks-move.sh:92`, a click on the waybar word) is a pointer click, not a chord: this gap does not explain it, and it also did not occur in 30 runs on the final head or 20 on the heads before it. sh-47 stays open (see the end).

### Red first

aa28b8d6 adds one test per look, each recording the calls to a fake core and asserting that the first `input.grab` comes before the first `scene.apply` that creates a node: `xp_holds_its_keys_before_its_first_node_exists` (chora-xp, wm/startup.rs), `tiling_holds_its_keys_before_its_first_node_exists` (chora-tiling), `the_construct_holds_its_keys_before_its_first_node_exists` (chora-construct). All three failed on 8b6f35dc with `["create", "grab", "grab", "grab"]` (xp and tiling) and the same for the construct. The test of mc (`mc_holds_its_keys_before_its_first_node_exists`) is green on the base and comes with the fix as a guard.

### Fix

6d9144a4: xp announces and grabs before `create_scene`; tiling grabs before it creates its group (the relayout still grabs again when it gains the claim, D55); the construct calls `regrab` before `build` (`regrab` reads no node). tests/gate-lib.sh says in the doc of `session_start` that ready now holds the keys. No wait or timeout changed; no cell skipped.

Two consequences found by the gate, fixed in their own commits:

- 08b2fb35: the core grabs a pointer button or a media key only for a client with a claim (crates/chora/src/ipc/grabs.rs: Super+BTN_LEFT is shared by claims, media keys need `window_manager`). With the first fix xp grabbed Super+BTN_LEFT before its claim and was refused, so the handover cell failed 10 of 10 ("Super and the button on the xp output turn no cube", run 1 below). xp and tiling now grab what needs no claim before the first node and the buttons and media keys once the scene claims (`grab_without_claim`, then `grab`; tiling keeps its `if claimed.is_none() { grab_all }` fallthrough). Test: `xp_grabs_its_buttons_after_its_scene_claims`.
- 7fc05748: the early xp grab knew no bars, so on virtual outputs alone (the faces of a cube) it took the cube keys: xp-faces `no-cube-on-faces` measured a gap in the first full gate. The early grab reads the cube mode from the outputs. Test: `xp_on_faces_grabs_no_cube_key_before_its_scene`.

Construct: its drag grab has no claim dependence (it holds no claim), so its order change needs no split.

### Miss counts

Load: a loop of `cargo build --workspace` into a separate target dir, followed by `cargo clean` of the chora crates, running beside the cells for the whole run (load average 4.8 and up on 4 cores); the cells one after the other, each as merge-gate.sh calls it (xp-run = gate-xp-run.sh; launcher-mc = gate-launcher-mc.sh; launcher-click = gate-launcher-click.sh; handover = gate-handover.sh then gate-looks-move.sh). 10 runs each.

| cell | run 1, 6d9144a4 (first fix) | run 2, 08b2fb35 | run 3, 7fc05748 (final) |
|---|---|---|---|
| xp-run | 10, 0 misses | 10, 0 | 10, 0 |
| launcher-mc (with the mc fix of flake-launcher r1, 2a90b905 and d06b9be2, in a scratch worktree only) | 10, 8 misses (no mc fix on the branch yet) | 10, 0 | 10, 0 |
| launcher-click | 10, 0 | 10, 0 | 10, 0 |
| handover | 10, 10 misses (cube drag, see above) | 10, 0 | 10, 0 |

The launcher-mc misses of run 1 are all "no launcher after mouse (a fuzzel process; the log of the session says: )", the lost click on the fading mc node set of flake-launcher r1, which this base does not have (d06b9be2 is not merged yet). It is not the key-grab gap: no miss after the keys (Super+Space). The runs 2 and 3 of launcher-mc ran on a scratch worktree of the branch head with 2a90b905 and d06b9be2 cherry-picked (never committed to this branch); with it, 0 of 20. Without it a launcher-mc cell on this head is red about 8 of 10 under load until that lane merges.

xp-run was green before the fix as well (0 of 10 under load in flake-launcher r1; I did not run a baseline of my own), so the "after" count shows no regression, not a before/after difference: the loss needs a chord in a window of a few ms, which the cells do not hit. The unit tests are the red.

### Full gate

`tests/merge-gate.sh --jobs 2`, no --only, on 7fc05748, no load beside it: exit 0, "all steps passed", 67 PASS lines, docs SKIP ("mkdocs is not installed (docs unchanged)", the host gap of the r10 series). Log: media/2026-11-27/flake-xp-r1/full-gate-run3-green.log.

Two earlier full gates did not pass and are not hidden: the first, on 08b2fb35, stopped at xp-faces (`no-cube-on-faces`, fixed by 7fc05748; its log was overwritten); the second, on 7fc05748, stopped at acts with `tiling I14: in act 'the chat of the helper shuts': chora-tiling.launch-27065 disconnected` (media/.../full-gate-run2-acts-flake.log), the known sh-32 / sh-46 flake (findings/shared.md: "acts tiling I14 ... green on rerun"), not touched by this change; the third full run on the same head passed acts and everything else.

### Logs

media/2026-11-27/flake-xp-r1/: run1-summary-first-fix.txt, run2-summary-claim-fix.txt, run3-summary-final-head.txt (cell, run, exit code, seconds); run1-handover-fail-cube-drag.log and run1-launcher-mc-fail-mc-bug.log (the first failing logs); run3-{xp-run,launcher-mc,launcher-click,handover}-logs.txt (the runs of the final head, concatenated); full-gate-run2-acts-flake.log; full-gate-run3-green.log.

### Not fixed, left open

- sh-47 (handover: "the picker of the Looks door did not start", a click at the waybar word of tiling): no reproduction in 30 runs of handover here (10 per head), and the key-grab gap is not its path. A next lane would need a miss to look at; what the click depends on is waybar's door and the pointer route, not a grab.
- launcher-mc stays red under load on a head without d06b9be2 (the mc bug of flake-launcher r1, MERGE-OK, joins the next train).
- A gate-side readiness (a query that lists the held bindings) was not added: with the order fixed, "the nodes of the owner answer" implies the keys are held for these keys, and the core has no query of grabs. A held-bindings query would be a core change.
- Keys that need the claim (buttons, media keys) still follow the first node by design; no gate injects them right after the start.

### Rules kept

Pushed claude/flake-xp-r1 and claude/session-log only; author and committer Marcel Arpogaus; no attribution lines; no --no-verify; no system packages; no bare git stash; the scratch worktree with the mc fix was removed and never pushed. `pgrep -fa 'chora --headless'` is empty at the end.

## Review r1

Review of claude/flake-xp-r1 at 7fc05748, 8b6f35dc..7fc05748 (7 files, +276 -8). One round, opus high.

Verdict: FIX (one MAJOR: rfxr1-1).

### Checks

- Step 0: `git fetch --unshallow`, `cargo check --workspace` clean.
- `cargo test --no-fail-fast -p chora-xp -p chora-tiling -p chora-construct -p chora-mc` on 7fc05748: 371, 303, 230, 223 (+1) passed, 0 failed. `cargo clippy --all-targets` on those four: no warning. `tests/gate-handover.sh`: exit 0 ("ok: 9. tiling ended after the give: xp adopted s1"). `tests/gate-xp-faces.sh`: exit 0 ("every cell as in EXPECTED", 13 s). `pgrep -fa 'chora --headless'` empty after.
- 1) Red first: on aa28b8d6 (8b6f35dc plus the tests only) the three tests fail (`xp_holds_…`, `tiling_holds_…`, `the_construct_holds_…`); on 7fc05748 all pass, with the mc guard and the two xp follow-up tests. The tests record the calls the fake core receives (`core.record` on `input.grab` and `scene.apply` create), so they assert the order the core sees, not a call list of the plugin. They run with no other window manager, so they cannot see a claim refusal (rfxr1-1).
- 2) Claim split: no double grab (`Runtime::grab_as` keeps a held binding and only sends `input.grab` for a new one), and a refused button or media key goes to `waiting` and is asked again on `input.ungrabbed`, as before; xp -> tiling -> xp is green in gate-handover. The rule is one per look but written twice with two parsers (rfxr1-2), and the construct, which also holds a claim, did not get it (rfxr1-1).
- 3) The early xp cube mode reads `rt.outputs()`, the same set `create_scene` makes one bar per output from, so today it agrees with `cube_off`; it is a second copy of the predicate, and an early `Own` grab is never taken back by `grab()` (rfxr1-3).
- 4) Construct `regrab` before `build`: of the plugin state, `regrab` reads `walk_keys` (set by `build`) only through `keys_with_pane`, which needs a focused own window, none at start: no effect. But it depends on core state `build` makes: the claim of the space (panes.rs:339, `claim: Some(..)` unless `--overlay`). The report says the construct "holds no claim"; it does (rfxr1-1).
- 5) Nothing loosened: `git diff 8b6f35dc..7fc05748 -- tests/` is the 4 doc lines of `session_start` in tests/gate-lib.sh; no wait, timeout or cell changed. The doc overclaims slightly (rfxr1-4).
- 6) Honest: the report says the cause is shown by the call order, not by a live miss, and that xp-run was green before as well (0 of 10 in flake-launcher r1). It marks no sh-44 status; findings/shared.md still has sh-44 open (FLAKE-SERIES r10 watches), which is right for an order fix with no live red. sh-47 stays open, correctly, with the reason that it is a click, not a chord.

### Live check of rfxr1-1

A session of two outputs, `chora-xp --output HEADLESS-2`, then after 3 s `chora-construct` (it takes HEADLESS-1); both hold a claim (`claims: construct xp`). The construct's line in the core log:
- 8b6f35dc: `chora-construct: 4 bindings are grabbed already (Alt+Super+1 to Alt+Super+4), asked again when they are free`
- 7fc05748: `chora-construct: 5 bindings are grabbed already (Super+BTN_LEFT to Alt+Super+4), asked again when they are free`

So on the head the construct loses its pane drag (Super+BTN_LEFT, shared by claims, D55) beside a running xp, until xp lets it go or a later regrab (a pane window takes the keyboard). The same failure 08b2fb35 fixed for xp (handover 10 of 10). No gate starts the construct beside a window manager that keeps the button, so the full gate could not see it.

### Findings

- rfxr1-1 MAJOR plugins/chora-construct/src/scene.rs:38 — `regrab` before `build` grabs the drag Super+BTN_LEFT before the space claims, so beside xp (or tiling) the core refuses it and the construct's pane drag is lost (live: base holds it, head refuses it); the report's premise "the construct holds no claim" is wrong (panes.rs:339). Fix: split as in xp, grabbing the keys before `build` and the drag after it (or `regrab` again after `build`), with a fake-core or gate test of the construct beside a window manager that holds the button.
- rfxr1-2 MINOR plugins/chora-tiling/src/main.rs:541, plugins/chora-xp/src/wm.rs:863 — `needs_claim` is written twice with two parsers (`rsplit('+')` and `Binding` parse) and is a loose copy of the core's rule (crates/chora/src/ipc/grabs.rs:28-49: `MEDIA_KEYS` without modifiers, buttons only when shared). Fix: one `Binding::needs_claim` in chora-kit used by xp, tiling and the construct.
- rfxr1-3 MINOR plugins/chora-xp/src/wm/startup.rs:342 — the early cube mode copies `cube_off` (windows.rs:384), and since `grab()` only adds grabs and then sets `cube_keys`, outputs that turn into faces between the two grabs leave the cube keys held for good (`sync_cube_keys` then sees no change). Fix: one `cube_off_of(&outputs)` for both, and `grab()` inside `start_grabs`/`end_grabs` so it lets go what the early grab took.
- rfxr1-4 NIT tests/gate-lib.sh:21 — "holds the keys of the look too" holds only for keys that need no claim (buttons and media keys still follow the first node), and it names sh-47, which the report says this gap does not explain. Fix: say "the keys that need no claim" and drop sh-47.

### Rules kept

Read-only on code; pushed only this section on claude/session-log; author and committer Marcel Arpogaus; no attribution lines; no --no-verify.
