# notify-body r1 (F7, rc 11; sh-31 MINOR, shared chora-notify, not a look)

Model: sonnet, effort medium (one default size in one plugin, red-first is plain)
Lane: cloud, claude/notify-body at ce866b0d (session head, t124). Cost at most 0.5 %; stop at 150 k context. One area: plugins/chora-notify default card text sizes and their tests; no look plugin, no flavor files beyond the notify config they pass.
Why: findings/shared.md sh-31: the plain chora-notify card (tiling and construct sessions) draws its body at 11 px (notify-card-body), the guide wants 14 px (I6); acts t21, t28 and construct acts with a card are red in `--check`. Not in scope: mc (card.size 19, mc-40), xp (11 px is the replica, keep it), ti-30, co-44.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`, `python3 demo/tour/run.py --check tiling` once (paste the t21 I6 line). A failure: send_message the PM "notify-body BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Cause with file:line: where the 11 px default comes from (`git show 675ebd55`), and how xp gets its own 11 px.
2) Red first (one commit): a unit test that the default card body and title meet the guide size (14 px body, read the constant from the guide or docs/), and that the xp card stays 11 px; it fails on ce866b0d; paste the failing line.
3) Fix: the default size only; the card grows to fit (no clipped text, no overlap of title and body); xp and mc unchanged. Paste the passing line.
4) `python3 demo/tour/run.py --check` after: t21, t28 have no sh-31 I6 line; paste the summary line (14 red on the session head with xp-title merged).
Gate: full `tests/merge-gate.sh --jobs 2` (no --only), detached: `setsid nohup bash -c '(tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll /tmp/gate.log in the same turn; never end a turn to wait. One gate only; a red cell you did not cause: name it with its log line. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: cause line, red then green shas with pasted lines, the --check summary, the gate green or its foreign reds named.
Report: reports/notify-body.md "## r1" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/notify-body and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
