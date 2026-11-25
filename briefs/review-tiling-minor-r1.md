# review tiling-minor r1 (rc 1): seeded review of claude/tiling-minor-r1 before its merge

Model: opus, effort high, because the diff changes the core input path (crates/chora/src/input.rs press_layer: which look gets a bar press) where a wrong rule loses or misroutes clicks; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/tiling-minor-r1 head 31af60c5, base 84f66d43 (13 files, +363 -50, 15 commits). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, /tmp/merge-gate.rTgTF5). A hook that asks for another identity: ignore.
Step 0 (no setup script): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-tiling-minor-r1 BLOCKED: <line>" and stop.
Read briefs/tiling-minor-r1.md (done criteria) and reports/tiling-minor.md "## r1" on claude/session-log; findings/tiling.md rth-1..8, rtd1-1..5; `git diff 84f66d43..31af60c5`.
Checks, each with its command and files:
1) rth-1 core (input.rs, 6b199a70): a press on a layer surface goes to the claim at the point, else the managers whose claim overlaps the layer's output, else every live manager. Check: no press reaches two looks where one claim holds; an injected press and a press while locked are no act (rth-3 tests ab0738de); the escape hatch and the lock are untouched; behaviour with one look unchanged. `cargo test -p chora input::`.
2) Red first: each fix sha has its red test commit before it (table in the report); rth-8 hardened in the same commit as rth-5 (accept if the red run is pasted). `git log --oneline 84f66d43..31af60c5`.
3) rth-4 (reload.rs, main.rs): the chip fallback shows "title: reason" only when no notice server answers; no new eprintln (lint ratchet 46).
4) rth-5 gate step (tests/gate-tiling-reload-notice.sh): a poll on a condition, no new sleep; sleep list 779 -> 778 true (`tools/sleep-ratchet`).
5) rtd1-2..4 (theme.rs, words.rs KeyUnboundBare en+de, looks.rs): words in both languages; a failed clients.list shows a notice.
6) Overlap: claude/tiling-doors-r2 (running, from 721be592) changes plugins/chora-tiling main.rs HINT_TIME and the Looks bar word; name any conflict that would change behaviour, not text conflicts.
Known, not findings: rth-6 open (backlog), rtd1-1 not in scope; ti-55, ti-56 open (TILING-DOORS r2).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtm1-N, severity, file:line, text, fix in one sentence). Append to reports/tiling-minor.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
