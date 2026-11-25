# mc-minor r4 (rc 1, F7): the page-end follow-ups rpe-1, rpe-2, rpe-3

Model: sonnet, effort medium (routine follow-ups in one look, each with a red test)
Lane: cloud, branch claude/mc-minor-r4 at 23cd2b5a (mc-page-end r1, MERGE-OK, joins train144). Area: plugins/chora-mc and tests/gate-mc-sofa.sh; no other look, no chora-notify. Stop at 250 k. Cost at most 0.6 %.
Why: reports/mc-page-end.md "## Review r1" on claude/session-log (no MAJOR left on mc, rules 6 allows MINOR):
- rpe-1 MINOR keys.rs:181: the key list does not name the new scroll keys; a remote user sees the more mark and is not told which key reaches the lines. Add the row beside Home while the page keys are plain; name the keys in the doc comments of `page_key` (methods.rs) and `PageKeys::Plain`.
- rpe-2 MINOR lifecycle.rs:65: a timer card covers the page's last lines for up to about 1 s until the next rooms round; call `learn_rooms` right after mc's own notify.push (and on the notice server's shown/closed signal if one exists). No new sleep.
- rpe-3 NIT gate-mc-sofa.sh:628: the fail message prints a stale $page; print a fresh node_rect.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`. A failure: send_message the PM "mc-minor-r4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1. Red first, its own commit per finding (rpe-1: a unit test that the key list holds the scroll row while the page keys are plain; rpe-2: a unit test or gate assert that the room is learned in the same round as mc's push, no wait). Paste each red line.
2. Fix commits, KISS and POLS; key list wording in the voice of docs/mc-style.md.
3. One capture of the key list with the new row (media/2026-11-25/mc-minor-r4/, under 500 KB).
Run: the full gate, no step filter: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push) detached with setsid nohup, poll in the same turn; paste the last 20 lines. A red cell you did not cause (mc-sofa light rooms-pill is FLAKE-SOFA r2's): name it with its log line. Never end a turn to wait. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: red commits before fix commits on claude/mc-minor-r4; capture; gate EXIT 0 (or only a named foreign flake); reports/mc-minor.md "## r4" on claude/session-log (base, head, diffstat, each finding closed with its test, gate line).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): head sha, red and fix commits, gate result.
Rules: push claude/mc-minor-r4 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media folder); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
