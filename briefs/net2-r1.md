# net2 r1 (F3): tests/acts with the first invariants (two lanes)

Model: opus, effort high, because the harness API and invariant semantics carry every later slice (complex)
Lanes: cloud, two branches from the session branch after NET1 merges:
- claude/net2-harness: `tests/acts` (`Session`, `act`, `Expect`, quiet = every `busy` empty or timeout),
  I1, I3, I4, I7, I8, I14, `tests/acts/known-red.toml` (a listed pass fails), gate step `acts`.
- claude/net2-adapters: `state.rs` adapter per look (xp, tiling, construct) filling the 3.1 shape via
  `Plugin::state`; the door acts per look. Merges after the harness.
One round each. Stop at 250 k. Cost about 7 % together.
Read reports/arch-plan-fable.md (3.1-3.3, 4: NET2), design/foundation.md (F3), reports/net1.md.
1. Selftest first: a cell per invariant that breaks it on purpose and must fail.
2. The door acts run on the head; every failure goes to known-red.toml with its finding id and slice.
3. No look logic changes; an adapter only reads. The adapters lane also fixes mc state.rs (net1
   review): focus `holder` only when mc owns the focused surface, else null; omit `open` until filled.
4. A shallow clone fails 2d-guard: run `git fetch --unshallow` first.
Done when: selftests red-then-green, acts step in tests/merge-gate.sh, merge gate once on the head
(poll it in the same turn).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/net2-<lane>.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
