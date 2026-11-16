# review areas a r1 (rc 6): code review per area

Model: sonnet, effort medium (read-only review against a fixed checklist)
Lane: cloud, read-only on code: review the session branch head claude/autonomous-dev-session-3w-i33pg7
as it is at your start (name the sha). One round. Stop at 250 k context and hand over.
Scope: crates/chora-kit, plugins/chora-xp, plugins/chora-tiling, plugins/chora-notify.
Read design/rc.md row 6 (the checklist and charter 11), reports/review-plugins-r1.md and
reports/review-core-r1.md (do not repeat their fixed findings; say which open ones still hold).
Write reports/review-areas-a-r1.md on claude/session-log: per area, one line per finding
(id raa-N, HIGH/MEDIUM/LOW, file:line, text, the fix in one sentence): simplicity and
readability first, then extraction and duplication across plugins, correctness, security
(charter 11), dead code, docs. Push the report early and update it at the end.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines):
counts per area and severity, the HIGH and security MEDIUM list.
Rules: push only claude/session-log (fetch and rebase on reject, no force), only that report;
author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines;
never --no-verify.
