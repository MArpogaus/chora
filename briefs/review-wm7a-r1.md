# review wm7a r1 (F3): seeded review of claude/wm7a before its merge

Model: opus, effort high, because it fixes a cross-look API (cube role in chora_kit::cube and construct IPC)
that every later cube slice and the owner's hosted/flat choice build on; the builder never reviews itself
Lane: cloud, read-only on code: claude/wm7a head 4069e38f, base a17e7858 (17 files, +764 -50). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green).
Read briefs/wm7a-r1.md (done criteria), reports/arch-plan-fable.md section 1.2, design/3d.md and
reports/wm7a.md on claude/session-log; `git diff a17e7858..4069e38f`.
Checks, each with its command and files:
1) Red first: b7afaa13 (acts) before fec3a98a; the new acts fail at b7afaa13 (run only if cheap).
2) API: `cube.list`, `cube.turn {by|to, ms}`, `cube.pull {surface}`, event `construct.cube {front, turning}`
   small, typed, one cube code (construct hosts it), no core change, D71 namespace held; refusals named
   (90956a2b: one refusal without a cube); xp/cube.rs untouched. `cargo test -p chora-kit --features testing`,
   `-p chora-construct`.
3) I1: construct killed mid-turn loses no window (acts; the 8.1 s give-back named in the report); no stray
   key while turning or stood back.
4) Docs: docs/api.md, configuration.md, glossary match the code (names, fields, errors).
5) cube/role.rs split true; tools/copies.toml and size-ratchet counts true (`tools/cut-check`).
6) Report claims each name a cell or capture (rules.md 7): the measurements table (damage at rest,
   29 fps debug, keys at 1:1) and the hosted recommendation follow from them.
Known, not findings: wm7a-a1 (I3 double focus claim on cube open), wm7a-a2 (no pager on a face), wm7a-m1
(start card "Nothing is open yet" after restart): WM7b; direct scanout unmeasured (headless).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/wm7a.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
