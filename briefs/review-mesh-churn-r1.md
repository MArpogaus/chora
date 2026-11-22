# review mesh-churn r1 (rc 9: sl-1): seeded review of claude/mesh-churn before its merge

Model: opus, effort high, because the diff changes the core render cache (layer texture lifetime) and adds an unsafe libc call to the core process; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/mesh-churn head dc6a266d, base 96e5c804, 3 files +109 -3 (crates/chora/src/mesh.rs, render/cost.rs, tools/lint-ratchet.list). One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane gate was green, 49/49).
Step 0: `git fetch --unshallow || true`, `cargo check --workspace`; a failure: send_message the PM "review-mesh-churn BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/mesh-churn-r1.md (done criteria) and reports/mesh-churn.md "## r1" on claude/session-log; `git diff 96e5c804..dc6a266d`.
Checks, each with its command and files:
1) drop_left (Layers::slot, mesh.rs): a texture still in use this frame is never dropped (two views, two scales, the same (node, slot) used twice in one frame, a view that skips a frame); no use-after-drop of a texture handle; the per-view frame counter cannot wrap or grow without bound. `cargo test -p chora mesh:: render::`.
2) trim_heap: malloc_trim(0) only after textures went, at most once per retain; cfg(target_env = "gnu") keeps musl and other targets building; SAFETY line true; the unsafe allow is local to that fn; cost per frame (a call on every retain with drops during a resize drag: name the worst case).
3) Red first: ed1bc1b2 holds the test before 0d9e7de8; the bound 2x10 (report: a rounded rect makes 2 slots per size) is exact, not loosened to hide churn.
4) Ratchet: lint-ratchet allow 360 to 361 has its `ratchet-raise:` line. claude/core-diag 788605f4 (MERGE-OK, also in train 110) raises the same count for its mallopt: say the merged value both need (362) and whether one shared unsafe helper would be simpler (a finding, not a fix).
5) sl-1 is still RED by 29 MB after the fix (report): with core-diag's mallopt, is the rest expected to close; name what the sim-long recheck must measure.
Known, not findings: the hint-click act flake (lane flake-hint); co-41/co-42.
Severity: BLOCK, MAJOR, MEDIUM, MINOR. Verdict computed: an open BLOCK or MAJOR, or a failed Done-when line, is FIX or INCOMPLETE, else MERGE-OK. Findings one line each (mch-N, severity, file:line, text, fix in one sentence).
Report: reports/review-mesh-churn.md "## r1" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines: verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages.
