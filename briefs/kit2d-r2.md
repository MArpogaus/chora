# kit2d r2 (F2): fix review r1 of the xp bindings port

Model: sonnet, effort high (named fixes of a review; no new design)
Lane: cloud, branch claude/kit2d at 888dfb60 (continue it). First `git fetch --unshallow`. When the session branch
moves (trains 60, 61), `git merge origin/claude/autonomous-dev-session-3w-i33pg7`. One round, stop at 200 k. Cost about 1 %.
Read reports/kit2d.md "## Review r1" on claude/session-log (rw-1 to rw-4).
1. rw-1 MAJOR, red first (a commit with the failing cell before the fix): `Super+BTN_RIGHT: close` closes and never
   turns the cube; `Super+x: cube` is refused with a warning. Fix in plugins/chora-xp/src/wm/keys.rs InputButton arm:
   act on `self.rt.action(&binding)`; grab_cube/release_cube only for `Look::Cube`; another action runs on press.
   `refuse`/`check` drops with a warning a BTN_* binding to anything but `cube`, and `cube` on a non-button key.
2. rw-2 MINOR, red first: the Super+Ctrl stand-in applies only while the key keeps its default action
   (plugins/chora-xp/src/wm.rs:815, help.rs:166); `Ctrl+Alt+Left: close` gets no stand-in.
3. rw-3, rw-4 NIT: docs/configuration.md names `{volume: {set: N}}`; docs/api.md:468 one sentence per fact, rewrapped.
No other change. Done when: cells red then green; `cargo test -p chora-xp`, `-p chora-kit --features testing`;
`tests/merge-gate.sh --only xp,keys,acts,hud,cube` green (poll in the same turn). No full gate (the train runs it).
Push claude/kit2d. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines:
head sha, red shas, each rw closed) and append "## r2" to reports/kit2d.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
