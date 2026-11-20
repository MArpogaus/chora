# xp-majors r1 (RC, vision bar): paperclip, volume, question card, Super+D, key list by keys

Model: sonnet, effort high (six product faults in one look; each needs a reproduction and a red act)
Lane: cloud, branch claude/xp-majors from 30727c82 (train 77). One round, stop at 250 k. Cost about 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace` (then `cargo build --workspace`); a failure or a missing tool: send_message the PM "xp-majors r1 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/xp.md xp-23, xp-26, xp-27, xp-31, xp-32, xp-33; reports/p2-r1-xp.md "## r1"; feedback/2026-11-20-p2r1-{murat,pixel}.md; findings/README.md (severity); vision-card.md (xp: XP-like UX, Erika and Murat mouse only; shared RC bar).
Each of xp-23 (paperclip at its place on the desktop), xp-26 and xp-32 (one fix: tray speaker click shows the XP level slider with Mute; the level set elsewhere shows in the tray tooltip and the menu tick), xp-27 (question card in the chat's font and frame, does not cover the transcript, Yes/No buttons for the mouse, the hint once), xp-31 (Super+D toggles back; a taskbar button restores a hidden window), xp-33 (Super+F1 opens the key list on xp):
1. Reproduce on the head first (tests/acts/looks/xp.py or a pixel cell). Gone already: a green act that guards it, mark `fixed <guard sha> (gone before)`.
2. Else a red act or cell first (own commit), then the fix (own commit), smallest change, xp plugin only unless the fault is in kit or core (say so). Fallback art only; never commit Microsoft files.
3. findings/xp.md rows `fixed <fix sha> (red <sha>) act:<name>` on claude/session-log.
Poll each run in the same turn; never end a turn to wait.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,acts,xp,xp-faces,xp-paperclip,xp-question,xp-fields,doors,keys`; all PASS.
Done when: each of the six names red commit (or guard), fix commit and passing cell; gate green; reports/xp-majors.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/xp-majors and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
