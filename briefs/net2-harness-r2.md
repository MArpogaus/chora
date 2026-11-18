# net2-harness r2 (F3): fix round after review FIX (02:21)

Model: sonnet, effort medium
Lane: cloud, branch claude/net2-harness (head 4052cd82), one round. Stop at 250 k. Cost about 1 %.
Read reports/net2-harness.md and this brief; the review findings are below (file:line approximate).
Red first: each fix gets a failing test commit before its fix.
1. MAJOR harness.py Session.__enter__ (~L255-300): an exception there (wait_env timeout, grant
   failure, looks-connect timeout, queue.Empty) skips __exit__ and leaks the headless process group,
   the core and /tmp/chora-hl.*. Make setup exception-safe (contextlib.ExitStack or __exit__ in an
   except). Test: a forced setup failure leaves no process and no temp dir.
2. MAJOR missing state keys pass silently: busy_of (~L170) treats no `busy` as quiet; i4 (~L96)
   treats an absent `open` as empty; i8 with no `helper`/`rects` passes. Rule: a look that answers
   state.get but lacks a key its invariant needs is a breach "missing key <k>", not a pass. A look
   with no state.get stays "no state" and must be listed in known-red.toml (it already is for xp,
   tiling, construct). Add the row for mc I4 (mc omits `open` until the adapters lane fills it).
   Cells: one per invariant for an absent key.
3. MINOR known-red rows: optional `match` substring checked against the breach text.
4. MINOR __exit__: wait about 8 s before SIGKILL; remove run.py's /tmp/acts-run-* evidence dir
   after a passing run (keep it on failure and print its path).
5. MINOR wait_quiet: two consecutive empty reads about 0.2 s apart instead of one.
6. COSMETIC test_acts.py docstring names live.py; the file is planted.py.
Done when: the new cells red then green, merge gate once on the head (poll it in the same turn).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/net2-harness.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
Run `git fetch --unshallow` first.
