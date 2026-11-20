# core-minors

## r1

Branch `claude/core-minors` from 79c8d2b5, pushed at d23b03ae (one commit).

| item | open or closed | red | fix | test | cell |
|---|---|---|---|---|---|
| 1 rd-9b move filter in tell_ungrabbed | closed on the head (grabs.rs:85; ipc_close and the revoke path both call it) | 7e56ae02 | already in | `tell_ungrabbed_does_not_tell_a_move_binding`, `a_move_grab_that_ends_with_its_connection_is_not_told` | none new |
| 2 rc-6 surface.created parent at first commit | closed (ipc.rs `surface_created_carries_the_parent_of_the_first_commit`: parent at the first commit, none from a later set_parent) | already in | none needed | that test | none new |
| 3 rc-4, rc-5 docs | closed: api.md `surface.created` says "as it is at the first commit ... a later `set_parent` sends no event"; D116 says a new window of the same app without a parent is not a child | none (docs) | already in | none | none |
| 4 rw-3 focus.changed {surface:null} on node focus | open; decided by docs: the focused surface stays while a node has the keys and the keyboard returns to it ("Keyboard focus of nodes"), so no event is right | none: pinning test, green by design (the code already matches) | doc line on the `focus.changed` row of api.md | `a_node_that_takes_the_keyboard_sends_no_focus_changed` (d23b03ae) | none |
| 5 rw-2 kept nodes hidden during the grace | open, NOT changed: needs a PM decision | none | none | none | none |

Item 4: the rw-3 proposal (emit null when a node takes the keyboard) would need a second event on the way back (`focus_node {}` returns to the old surface), so xp would learn both ways; that widens the event contract. Not done. If the PM wants xp to see node focus, the event needs its own design (D107 `Focus::lost` is the kit side).

Item 5: docs/api.md (Window managers, resume, ~l.965) says the core keeps tree, stacking, properties, surfaces, cameras and assets for the grace time, and says nothing about hiding. Hiding would be new behaviour: a quick restart (the point of `resume`, "without a loss") would flicker, `scene.query visible` would change, and the plugin would have to un-hide on resume. No documented behaviour to test red, so I made no change. Options for the PM: (a) leave it and add a doc line that kept nodes show during the grace, with a pinning test; (b) hide on disconnect with `resume` and show on the resume `hello`, doc and red test first.

Gate (`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,matrix,acts,wm-pane`): pre-commit, build, test, clippy, ci-check, wm-pane, matrix PASS. One flake each, green on the re-run alone: keys (`the program notify-false of fuzzel is no pane`) and acts (selftest persona sound, mc I5 overlap). The diff is a test and a doc line, so no binary changes; both are timing flakes outside this lane.

Notes: the commit hooks of the repo refuse `Co-Authored-By`/`Claude-Session` trailers and a subject over 68 characters; commits carry neither. Commit author is Marcel Arpogaus (git config).
