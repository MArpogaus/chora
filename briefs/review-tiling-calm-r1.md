# review tiling-calm r1 (rc 2, 8, tiling): seeded review of claude/tiling-calm r1 before its merge

Model: sonnet, effort high (UI fixes in one plugin, scripts and one kit file; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/tiling-calm head e9387fb2, base ad0093ba (12 files, +283 -22). One round. Stop at 120 k; cost at most 0.5 %; at most 8 findings. No merge gate run (lane gate: all PASS on e9387fb2).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `git fetch origin claude/session-log`, `cargo check --workspace`, `cargo test -p chora-tiling -p chora-kit`. A failure: send_message the PM "review-tiling-calm-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/tiling-calm-r1.md (done criteria), reports/tiling-calm.md "## r1", findings/tiling.md ti-27, ti-50, ti-51 and the rtr rows; `git diff ad0093ba..e9387fb2`; captures media/2026-11-24/tiling-calm-r1/.
Checks, each with its command and files:
1) Red first: 1f722eb3 (ti-27), 6b526318 (ti-50), 4a284cdc (rtr-1/3) each before its fix; each red cell fails at its red commit (paste the line) and passes at head. A fix without a red before it is MAJOR.
2) ti-27: the power menu, the Apps list and the helper panel close the key list (hide_keys); no other path closes it wrongly (the key list still opens and stays with Super+? alone); no lost focus after it closes.
3) ti-50: the launcher row "Chat with a helper (NAME)" starts the same chora-chat as Super+a; typing the helper's name or "help" finds it; the ASCII choice (fuzzel C locale) is stated; the doors test change (e9387fb2) asserts no weaker than before (a dropped assert is MAJOR).
4) rtr-1/3: waybar keeps no lock fd (9>&-), a missing flock prints one line; 9af3321f does not raise a lint allow count without a `ratchet-raise:` line.
5) ti-51: the lane says the check read hidden rows; is that a harness fault to file (name file:line), not a product one? No fix expected.
6) KISS, POLS, the tiling look: no new mode or setting; no other look touched (crates/chora-kit change: name the callers and the cells that keep them).
Known, not findings: ti-47, ti-48, sh-39, sh-40, co-*, xp-*.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtc1-N, severity, file:line, text, fix in one sentence). Append to reports/tiling-calm.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
