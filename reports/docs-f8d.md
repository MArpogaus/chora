## r1

Lane: cloud, sonnet, branch claude/docs-f8d from train 71 head 6aa0bd37, head c8249e2c.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test` green (pre-commit 50 s, build 202 s, test 210 s, ci-check 85 s).
Closed:
- rf-1: D95 status line is "accepted (owner, 2026-11-19 05:59; amends D77)"; `grep -n 'proposed\|Proposed' docs/decisions.md` leaves only the intro lines 126 and 141, no record.
- rf-2: crates/chora-kit/src/wm/mod.rs names all seven parts (book, give, kept, state, focus, open, stack); only desks follow.
- rc-4: lifecycle.rs keys_for_new comment drops the "new window of the same app" child claim (D116 text already said a same-app window without a parent is not a child).
- rc-5: api.md `surface.created` `parent`: as at the first commit, a later set_parent sends no event.
- rd-10: api.md `input.ungrabbed`: a shared button binding (D55) is told while another WM still holds it.
- CONTRIBUTING.md:17 lists all seven wm parts and points to the per-module list for the rest.
Checked, not redone: D116 accepted; `grep -rn D-ungrab` empty (D117).
Not closed: rc-6 (a core cell in ipc.rs:4412 asserting the `parent` of surface.created) is a test, not a doc line, outside this brief.

## r1 PM review (2026-11-19 20:30 UTC)
Diff 6aa0bd37..c8249e2c, 5 files, +12 -12, docs and comments only; builder sonnet lane, reviewer PM. Each closed id checked against its line: D95 status matches decisions-now 11-19 05:59; wm/mod.rs and CONTRIBUTING.md name the seven modules of `pub mod`; api.md parent and ungrabbed lines match D116 and D55; ci-check (rustdoc links) green. No finding. Verdict (computed): MERGE-OK; c8249e2c to train 72. rc-6 (ipc.rs cell for `parent`) to backlog.
