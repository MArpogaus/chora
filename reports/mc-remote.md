# mc-remote (RC 2, vision "remote is a full second path"; mc-46)

## r1

Lane: cloud, branch claude/mc-remote from 17d9a798. Step 0: `git fetch --unshallow`, `cargo check --workspace` green. The merge of origin/claude/autonomous-dev-session-3w-i33pg7 was already up to date. Gate (`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,chat-keys,doors,mc-paths,mc-sofa,mc-handover,matrix,layout`) on dac835dd: all 13 cells PASS ("partial run: nothing is pushed" as for a subset). Branch pushed at dac835dd.

| item | red | fix | test | gate cell | capture |
|---|---|---|---|---|---|
| mc-46 MAJOR | 7b40a523 | 28a22452 | cell remote-back; unit tests `back_leaves_a_window_for_its_page_and_home_goes_first`, `the_keys_of_a_remote_are_bound_and_need_a_claim` (mc), `only_a_window_manager_grabs_the_listed_media_keys_without_a_modifier` (core, now lists the two keys) | mc-sofa (remote-back) | before-mc-46-remote-back.png, after-mc-46-remote-back.png |

### Cause and fix

The grab API did not allow it: a key without Ctrl, Alt or Super needs `input.grab-any` or has to be one of `MEDIA_KEYS`, which had no XF86Back and no XF86HomePage. So the smallest core change was needed:

1. Core (`crates/chora/src/ipc.rs`): `MEDIA_KEYS` has `XF86Back` and `XF86HomePage`. Same rule as the other media keys: only an unconfined window manager with a claim grabs them without a modifier; any other client and every other plain key still need Super, Ctrl, Alt or `input.grab-any`. The unit test of the list has both keys; `docs/api.md` names them.
2. mc: new action `back` (`Look::Back`). Default bindings `XF86Back: back` and `XF86HomePage: {scene: agenda}` (`mc.yaml`). With a window that holds the keyboard on the shown page, `back` gives the keyboard to the page node (`keyboard_to` with `plain_turn`, as after a plain turn): the page keeps the keyboard, the next Left, Right or Escape turn or go back, Return gives the window the keys again. On a page it is one step back as Escape. `XF86HomePage` is the existing scene binding to the first page, and the page holds the keyboard after it. Both keys are grabbed with the claim (`needs_claim` now looks at the key: `XF86Back`, `XF86HomePage`, besides drag and volume), so mc without `--manage-windows` grabs none, as for the volume keys.
3. No other plain key is taken from a client (POLS): terminal-keys of mc-paths stays green.
4. Docs: `docs/configuration.md` mc `order` item (the two keys) and the actions list (`back`); the key list skips the `back` row (Esc already says "Back one step"); `XF86HomePage` joins the chords of the agenda row; the full-list count test went from -1 to -3 (alias and skipped row).

### Test

Cell `remote-back` of `tests/gate-mc-sofa.sh` (chora-mc `--manage-windows`, a foot window with the app id mcremote): the window page has the keyboard, XF86Back, still on that page, Left turns it; the page of the window again, XF86HomePage goes to agenda, Right then turns to morning (the page holds the keyboard). Red 7b40a523 on the old tree: "after XF86Back the page does not hold the keyboard, Left does not turn it", twice. Green on 28a22452, twice, and in the merge gate.

First tried in gate-mc-paths terminal-back: that gate starts mc without `--manage-windows`, so mc has no claim there and the core refuses the key; its 6-times-Right walk also races the foot window of the terminal page (it takes the keyboard when it maps) and is flaky on this machine, before and after. Not changed; the cell went to the sofa gate, which runs as the persona and the TV do.

### Note

`state.get` names `mc` as holder both when a page node and when a window of mc has the keyboard (`focus.state`), so the cell checks the behavior (Left turns the page) and not the holder.

### Open

- nit: a page reached by a plain turn onto an app scene whose window maps late (the terminal page) hands the keyboard to the window as it maps; a second Right of a remote is then lost. Pre-existing; the XF86Back of this round is the way out now.
- The page of a window does not say how to type into it (Return) nor does the key list name XF86Back; not built (PM to rule, as in mc-p2-fix).

## PM review r1 (2026-11-20, PM is not the builder)
Range: 17d9a798 (mc-p2-fix, in train 85)..dac835dd, 10 files +142 -16. Core: crates/chora/src/ipc.rs adds XF86_Back and XF86_HomePage to MEDIA_KEYS (11), with the unit test listing both.
Checks: red 7b40a523 (gate-mc-sofa remote-back) first; `step_back` takes the keyboard from a window only when the chat is closed, else `back()`; `needs_claim` keeps both keys behind --manage-windows; unit tests back_leaves_a_window..., the_keys_of_a_remote_are_bound...; keys.rs count -3 (two aliases).
- rv-1 MINOR: in mc a laptop's XF86Back now goes to mc, not to the focused browser (no history back in a browser window under mc). Acceptable for the remote path of the vision; known issue line. Backlog MCR-R1.
- rv-2 NIT: the key list does not name XF86Back or XF86HomePage (aliases). Backlog MCR-R2.
- rv-3 NOTE: +1 eprintln (main.rs step_back), house style of mc; the lint-down list must count it (lint-down train after t85).
Core change is two keysyms in a whitelist with a test; a PM review suffices (no new IPC, no security surface beyond the media-key rule D-record already covers).
Verdict (computed): no open MAJOR: MERGE-OK dac835dd (contains 17d9a798); train 85.
