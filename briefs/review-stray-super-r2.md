# review stray-super r2 (rc 1: xp-46, mc-60 MAJOR; rss-1, rss-2): seeded review of claude/stray-super-r2 before its merge

Model: opus, effort high, because the diff changes the shared key path in chora-kit (swallow yield and retry, D117, D118) for all four looks, a core-adjacent review (procedures: opus high for core reviews); the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/stray-super-r2 head 1eb30643, base e1183766 (11 files, +470 -128). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-stray-super-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, run 3, 65 PASS).
Read on claude/session-log: briefs/stray-super-r2.md (done criteria), reports/stray-super.md "## r1", "## Review r1" (rss-1..6) and "## r2", findings/xp.md xp-46, findings/mc.md mc-60, findings/shared.md sh-42; `git diff e1183766..1eb30643`.
Checks, each with its command and files:
1) Red first: 429ce20f (gate-join-keys cells xp-window, mc-first; kit swallow unit tests) before the fixes d88dee68, 160b62fd, 6a8bd3b9, 1eb30643 (`git log --oneline e1183766..1eb30643`). Do 6a8bd3b9 and 1eb30643 have a red test or gate cell before them (ci-check grab-gap-check.py, xp-notepad)? A fix with none is MAJOR.
2) The rule (crates/chora-kit/src/swallow.rs, runtime.rs): a WM of give::KEYS joining makes the look yield its swallow in one batch; it swallows again when the last one went; a refused grab waits and is asked again on input.ungrabbed; a refused bound chord stays bound. Edge cases: two joiners, a joiner leaving while another stays, a look reloading, events arriving out of order. `cargo test -p chora-kit`.
3) rss-1, rss-2 fixed: Super+1..8 and Super+h of tiling act beside mc and xp (gate-join-keys logs or a run of `bash tests/gate-join-keys.sh` cells only if cheap); xp-46, mc-60 stay fixed (Super+x/k/m/q reach no client, Start stays closed).
4) 160b62fd revert of the xp `Chords` state: nothing dead left in plugins/chora-xp (wm.rs, startup.rs, windows.rs); `cargo test -p chora-xp -p chora-mc -p chora-tiling`.
5) The stated cost (about 200 input.ungrab per join, the unbound chords belong to the last joiner): judge against POLS and D118 in docs/decisions.md; severity with reason.
6) No core change (crates/chora-core untouched); 47aad40d polls (until_ok), no new sleep; D118 text matches the code.
Known, not findings: sh-42 (tiling-join keyboard loss, filed); rpx1-* other than xp-46; mc-61; rv9j-1..5.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rss-N from rss-7, severity, file:line, text, fix in one sentence). Append to reports/stray-super.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
