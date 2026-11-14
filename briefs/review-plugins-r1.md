# review plugins r1 (rc 6 per area, rc 9a logging)

Lane: Fable, cloud, read-only on code. One round.
Scope: plugins chora-xp, chora-tiling, chora-mc, chora-construct, chora-assistant,
chora-notify, and services/voice. Five fix lanes are changing these right now, so
you write no code: review the session branch head as it is.
Write reports/review-plugins-r1.md on branch claude/session-log: per area, one line
per finding (id, HIGH/MEDIUM/LOW, file:line, text): simplicity and readability first,
then correctness, security, dead code, docs, logging (every error path logs once with
context, no secrets). Push the report early and update it at the end.
Report: counts per area and severity; the HIGH list.
Rules: push only claude/session-log (fetch and rebase on reject, no force), only that
report file; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no
AI attribution lines; never --no-verify. Stop at 250 k context and hand over.
