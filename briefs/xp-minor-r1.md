# xp-minor r1 (RC 2 known issues, vision: looks): five xp MINOR findings (backlog P2R3-MINOR)

Model: sonnet, effort medium (five small fixes in one look, each with its act or cell)
Lane: cloud, branch claude/xp-minor from fee9c695. One round, stop at 250 k. Cost about 1.5 %. xp has no open BLOCK or MAJOR (xp-35, xp-36 fixed), so rule 6 allows MINOR work.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure or missing tool: send_message the PM "xp-minor BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: backlog.md row P2R3-MINOR, findings/xp.md rows xp-34..xp-36, reports/xp-36.md, feedback/2026-11-20-p2r3-murat.md, feedback/2026-11-20-p2r1-erika.md. Not yours: other looks, the core, the gate script.
Items, in order (KISS, POLS: what XP SP2 did; no new feature):
1. xp-36 rv-1: Run... with a warning on stderr then Escape shows a false "Run could not start": exit 1 = closed, quiet; 2 and up = the failure card.
2. Screenshot gives no visible sign: a short XP-style tray balloon "Screenshot saved" naming the file (chora-notify, placed as xp places it).
3. Volume menu: the tray speaker shows the XP level slider with the current level, not only a tick.
4. Super+E notice names a config file path: say what to do in plain words (the path only as a second line).
5. A small yellow square after clicks (Erika): find what draws it (capture first); remove it or name it in the report if it is the tray balloon anchor.
Each item: a red act or cell commit first, then the fix; name the cell and a capture at 1280x800.
Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,xp,xp-faces,xp-question,xp-notepad,doors,acts`; poll it in the same turn; never end a turn to wait.
Done when: per item red sha, fix sha, cell, capture (or why not reached); reports/xp-minor.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/xp-minor and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages; never commit Microsoft files.
