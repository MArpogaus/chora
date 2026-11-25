# review mc-page-end r1 (rc 1, F7): seeded review of claude/mc-page-end-r1 before its merge

Model: opus, effort high, because the diff picks a page-end design in a look (card band, scroll path, more mark) where a wrong rule cuts lines again; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/mc-page-end-r1 head 23cd2b5a, base 42418379 (9 files, +386 -51). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it in parts; the train runs it whole).
Read briefs/mc-page-end-r1.md (done criteria), reports/mc-page-end.md "## r1" and reports/tour-v9-judge.md "## r2a" (rv9ka-1) on claude/session-log; `git diff 42418379..23cd2b5a`.
Checks, each with its command and files:
1) Red first: 8a0249a2 holds the failing assert and comes before 859cf3fe (`git log --oneline 42418379..23cd2b5a`); run `cargo test -p chora-mc` at 8a0249a2 (red) and at 23cd2b5a (green).
2) No card: the page ends at the footer on every recipe page (build.rs `bottom`, rooms.rs); a card shows: the band is reserved only while it shows and given back on close, scroll clamped (rooms.rs). Read against docs/mc-style.md rule 13 and the new sentence.
3) Remote and keys path: Down/Up/PgDn/PgUp scroll a page longer than its room; the more mark counts wrapped line ends and goes away at the end; no key reaches a client that mc did not mean (main.rs). A second test covers it.
4) No new sleep, no new lint allow, no change to chora-notify or another look (`git diff --stat`); tools/sleep-ratchet, tools/size-ratchet, tools/lint-ratchet on 23cd2b5a.
5) Captures media/2026-11-25/mc-page-end-r1/ m1, m4, m4card before and after: lines whole, mark visible where the page goes on, nothing covers the footer.
Known, not findings: rv9ka-2 (launcher icons, MINOR), rmm3-1..4, mc-sofa volume-wheel flake (FLAKE-SOFA r1), the split gate runs.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rpe-N, severity, file:line, text, fix in one sentence). Append to reports/mc-page-end.md under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
