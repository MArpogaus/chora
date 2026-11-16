# tiling look r2 (rc 1, 11): the open MAJOR of chora-tiling from journey 1

Model: sonnet, effort high (three UI faults across bar, tooltip and helper pill)
Lane: cloud, branch claude/tiling-look, continue from 47fef21b (r1, review MERGE-OK; merge the
session branch first). One round. Stop at 250 k context. Read findings/tiling.md ti-22..ti-24,
reports/journey-1.md (T1, T2, T4), reports/review-tiling-look-r1.md, and the stills named there
(media/2026-11-16/tour-v4/stills on claude/session-log).

1. ti-22 MAJOR: Super+Print gives an honest notice in tiling: saved (with the path) or failed.
2. ti-23 MAJOR: no stale tooltip: the 'Run a program (Super+Space)' tooltip leaves when the
   pointer or focus leaves, and never stays over a client.
3. ti-24 MAJOR: the bar has the power and lock entry the key list promises (mouse path to lock and
   power, shared RC bar); the 'Chat ...' pill never floats over client content and matches the
   helper state the bar shows (no "No helper" beside a live chat pill).
4. review-tiling-look-r1 MINOR: docs/xp-split.md:282 still says wobbly is off by default.
For each: a red gate cell or unit test first (its own commit), then the fix; no tolerance or retry.
Captures Ink and Paper. Done when: the new cells pass 10 times, tests/merge-gate.sh passes once.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 8 lines) and as
reports/tiling-look-r2.md on claude/session-log, each claim with its gate cell and capture. Rules
as briefs/flakes-r2.md; push only claude/tiling-look and that report; never end a turn to wait for
a long background job. If the shallow clone breaks 2d-guard: git fetch --unshallow.
