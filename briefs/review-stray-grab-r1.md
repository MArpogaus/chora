# review stray-grab r1 (RC sh-24, sh-23 focus): seeded review of claude/stray-grab before its merge

Model: opus, effort high, because the diff changes the core key path (crates/chora input.rs, place.rs,
state.rs) where a wrong rule sends stray keys to clients; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/stray-grab head 9859071b, base b97369d1 (6 files, +121 -20). One round.
Step 0: `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM
"review-stray-grab BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran 12 cells green).
Read briefs/stray-grab-r1.md (done criteria), reports/stray-grab.md "## r1", findings/shared.md sh-23 and sh-24
on claude/session-log; `git diff b97369d1..9859071b`.
Checks, each with its command and files:
1) sh-24 (cc940fec, input.rs, input/place.rs): an injected key goes to the keyboard place, never the pointer;
   the escape hatch stays above every plugin and every injected key. `cargo test -p chora input::`.
2) sh-23 focus (363276e8, state.rs): a click on another look's empty claim takes the keyboard; no typed key
   reaches the xp client after it; nothing else loses focus (a click on a client still focuses it). Same tests.
3) The two-screen effect the builder names: the core now clears focus on such a click on real outputs too.
   Is that the D55 rule or a regression (a click on an empty area of the same look)? Read D55 and the amendment.
4) Remaining rule: a real key with no keyboard place follows the pointer (documented). Does this contradict
   "no stray keys into clients" (vision card shared bar)? Name the case or clear it.
5) Red first: 25ecc50c, 1ce94726, 9821d96f come before their fixes (`git log --oneline b97369d1..9859071b`).
6) Docs: docs/api.md key rule and docs/decisions.md D55 amendment match the code.
Known, not findings: base had 573f788c (swallow) and 767249d7 (keyboard before pointer).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity,
file:line, text, fix in one sentence). Append to reports/stray-grab.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
