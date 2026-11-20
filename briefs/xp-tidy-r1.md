# xp-tidy-r1 (shared bar, known issues): open xp MINORs from the last five xp reviews

Model: sonnet, effort medium (small xp fixes and missing cells; no design work)
Lane: cloud, branch claude/xp-tidy at 22fead94 (xp-give r1 on session ce2c436b). One round, stop at 250 k. Cost about 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "xp-tidy r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Step 1: `git fetch origin claude/xp-minor && git merge --no-edit fca4f8b5` (xp-minor r2, MERGE-OK, not yet on the session branch); a conflict you cannot settle in 10 minutes: BLOCKED.
Read on claude/session-log: reports/xp-give.md "## PM review r1", reports/xp-minor.md "## PM review r2", backlog.md lines DB8, XPF-R1, XP-MAJORS rv-1..4, XP-24-RV. Not yours: other looks, the core, the gate script, art or pixel rounds (rules.md 6).
First check each item is still open on your head; a closed one: one line with the sha that closed it, no work.
Items (each: red cell first, then the fix; a test-only item: the cell must fail on a broken copy, say how you checked):
1. xp-give pr1-1: looks_running must not block the menu-open path on clients.list; cache it, refresh on client events.
2. xp-give pr1-2: a failed clients.list logs one warn line (honest state) and keeps the rows gray.
3. xp-minor pr2-1: strip fuzzel's "<file>.c:<n>: " prefix from the Run notice (crates/chora-kit/src/launcher.rs).
4. XPF-R1 rv-1: the kit launch check ends the Starting message only on a window without a token or with this start's token.
5. XP-MAJORS rv-1..3: question-card gate asserts placement and font; paperclip gate asserts its art drew; volume popup closes on an outside click and leaks no key to the client.
6. XP-24-RV rv-1, rv-2: pad-6 case matches its message; the gap comes from geometry, not the font.
7. DB8 MINOR: startmenu.rs `contains("(")` and the terminal prefix match; the no-editor notice names xp.yaml.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,xp,xp-faces,xp-paperclip,xp-fields,xp-question,xp-notepad,launcher-click,matrix`; poll it in the same turn; never end a turn to wait.
Done when: commits pushed to claude/xp-tidy; the gate list passes; reports/xp-tidy.md "## r1" on claude/session-log names per item the red and fix shas and the cell.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/xp-tidy and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
