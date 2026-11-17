# xp-polish r7 (rc 1): Notepad is never a terminal editor

Model: sonnet, effort medium (one launcher rule, review-found)
Lane: cloud, branch claude/xp-polish; merge the session branch aa8aa8c0 (or newer) first. One round. Stop at 250 k.
Read reports/review-tour-v6.md (j1-2/N1, N-6), reports/xp-polish-r6.md, design/looks/xp.md, dev-rules.md.

PM decision (06:27): the container has no GUI editor and lanes install nothing. KISS and POLS: "Notepad" opens
only a GUI editor window; it never opens nano, vim or another terminal editor in a frame titled Notepad.
1. j1-2/N1 MAJOR: drop terminal editors from the Notepad chain by default (a user may still set one in xp.yaml).
   No GUI editor found: the honest notice "Cannot open Notepad: no text editor installed (for example mousepad)"
   and no window. Red cells first: only nano on PATH -> the notice, no window; a fake GUI editor on PATH -> its window.
2. Packaging: list mousepad (or the distro's small GUI editor) as a recommended dependency of chora-xp in the
   arch, rpm and podman files; docs/configuration.md and the known-issues line say so.
3. N-6 (same menu, same fix): the Start menu does not pin Vim or ImageMagick next to Notepad.
No other MINOR work (rules.md 6). Fakes on PATH only, no packages.
Done when: the cells pass, cargo test and clippy of chora-xp pass, tests/merge-gate.sh once on the head (one run, no wrapper).
Refactor proposals: list up to 3 in the report (file, why, size), do not do them.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/xp-polish-r7.md on claude/session-log, captures named with look, build and date.
Rules as briefs/flakes-r2.md, push only claude/xp-polish and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
