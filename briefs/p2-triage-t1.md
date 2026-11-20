# p2-triage-t1 (RC 2 persona round, plan row 2): triage of P2 round 1 into the ledgers

Model: sonnet, effort high (dedupe and severity by written rules; no code)
Lane: cloud, ledger work on claude/session-log only; the code checkout (session branch) is read-only, for
naming checks. One round, stop at 250 k. Cost about 1.2 %.
Step 0: `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM
"p2-triage BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-plan.md sections 2 (Severity, Dedupe) and 3 (stop rule), findings/README.md,
feedback/README.md, triage.py, the four round reports reports/p2-r1-{xp,tiling,mc,construct}.md "## r1",
the twelve feedback/2026-11-20-p2r1-*.md, the ledgers findings/{xp,tiling,mc,construct,shared,shared-doors}.md.
Items:
1. Run `python3 triage.py` on the twelve files; keep its output in the report.
2. Every finding gets one outcome: duplicate of <ledger id> (hit count +1), new row (next free id of its
   ledger; shared if two or more looks show it), `verify` (check: none and no capture read confirms it),
   `rejected: env` or `rejected: harness` (name the harness fault).
3. Severity by p2-plan.md 2, not the persona's guess: BLOCK = a check red on a shared-bar task or main path,
   or a lost window; MAJOR = check red with a workaround. Every new BLOCK or MAJOR names its check (I<n>,
   contract T<n> or gate cell); without one, name the cell to build in the report table.
4. Known in flight, do not duplicate: launcher row click opens the chat (lane launcher-click), black tiling
   pane (sd-h1, lane black-pane), stray keys after a click (sh-23, sh-24, review stray-grab). Link them as hits.
   Construct C2, T5 and the xp start timeout are harness (lane p2-harness): `rejected: harness`.
5. Rewrite the touched ledger rows in place (rules.md 4: rewritten, not appended).
Done when: each feedback finding has one outcome; ledgers rewritten; reports/p2-triage.md "## T1" with a table
per look (new BLOCK, new MAJOR, duplicates, verify, rejected) and the stop-rule verdict per look (new BLOCK or
MAJOR backed by a check: another round; else stop).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): counts per look,
the new BLOCK and MAJOR ids, stop verdicts.
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus
<38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
