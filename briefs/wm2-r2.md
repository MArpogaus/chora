# wm2 r2 (F3): fix the review findings of wm2 r1

Model: sonnet, effort high (named fixes with a given cause and fix; review r1 did the hard part)
Lane: cloud, branch claude/wm2 at 751dfbcb (continue it). One round, stop at 200 k. Cost about 1 %.
Run `git fetch --unshallow` first. Read reports/wm2.md "## Review r1" on claude/session-log.
1. Red first, one commit: a cell per finding that fails on 751dfbcb:
   rw-1 (plugins/chora-mc/src/manage.rs:204): click s1, `focus.changed` to another look, click s1: mc sends Focus;
   rw-2 (plugins/chora-mc/src/main.rs:1017): after `take_free_keyboard`, state.get holder is the page node;
   rw-3 (crates/chora/src/input/place.rs:54): while `layer_focus` holds the keyboard, the key place is not the
   owner of the window that had it before.
2. Fixes as the review says: rw-1 `self.focus.take()` and send when `self.focus.sends(Some(node))`; rw-2
   `self.focus.to_node(Some(page_id(&name)))`; rw-3 skip the window owner while `state.layer_focus` is set.
   The safety layer stays above every plugin; no other change.
3. NITs of the review only if one line each.
Done when: the cells red then green; `cargo test -p chora input::`, `-p chora-mc`, `-p chora-tiling`,
`-p chora-kit --features testing` green; `tests/merge-gate.sh --only keys,pane-keys,wm-pane,acts` green
(poll in the same turn). No full gate (the train runs it).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append "## r2" to reports/wm2.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
