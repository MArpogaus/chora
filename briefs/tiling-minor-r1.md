# tiling-minor r1 (rc 1b, F7: tiling MINOR and COSMETIC after the pass confirmed rc 1)

Model: sonnet, effort high (routine: words, a sync and two sizes, causes named in the report)
Red on head: rpt2-2, rpt3-1, rpt3-2 MINOR and rpt3-3, rpt3-4 COSMETIC open on 7822a66c (reports/rc1-pass.md "## r2 tiling", "## r3 tiling"); no BLOCK or MAJOR open in tiling (r3 confirms rc 1), so rules.md 6 allows this round.
Lane: cloud, branch claude/tiling-minor-r1 at 7822a66c8f6a78080fd416f03ef0420b65d172e7 (session head with kit-swallow r2, train 159). Area: the tiling look (chora-tiling, its bar, chip, picker and fuzzel config). One look. Stop at 250 k. Cost at most 0.6 %.
Why: owner 11-17 10:59 "Be perfectionist and very critical about the met targets."; vision card: POLS, honest states.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`. A failure: send_message the PM "tiling-minor-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1. rpt3-1: the unbound-chord chip spells chords as the key list does ("Super+Alt+X", "Super+[", "Page Down"/"Page Up"); one formatter shared with the key list, not a second table.
2. rpt3-2: the bar helper word and the chat header show the same state at the same moment (one source of the state).
3. rpt2-2: the Looks picker offers "Move window ..." only when a window has focus; no tool name in a notice.
4. rpt3-3: the volume tooltip does not cover the key list or tiles (hide on a key list open, or no hover tooltip).
5. rpt3-4: the Looks picker box fits its rows (pass --lines like the power menu).
Each fix: a red test commit first (a cell or unit test failing on 7822a66c), then the fix; a capture before and after under media/2026-11-27/tiling-minor-r1/ (each under 500 KB). Use 0.15 s between a chord and text in one inject (rpt3 harness note).
Run: the full gate, no step filter: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 --allow-skip docs` (no --push) detached with setsid nohup, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line, rerun that step alone and then the steps after it. Never end a turn to wait. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: five ids closed each with red commit, fix commit and captures (or one id left with a reason); gate EXIT 0 (or a named foreign flake and every later step run); reports/tiling-minor.md "## r1" on claude/session-log (base, head, diffstat, per id).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): head sha, ids closed, gate result.
Rules: push claude/tiling-minor-r1 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media folder); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
