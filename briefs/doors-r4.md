# doors r4 (rc 1): one helper word rule

Lane: same worker, branch claude/doors (at cb203c68). One round; setup and rules as briefs/doors-r3.md.
Items from reports/review-doors-r3.md, red first:
1. dr3-1 MAJOR: the construct island says "No helper" whenever no helper runs, the same rule as xp,
   mc and opener.rs, and follows helpers that exit (no stale Some(false)); a cell where the last
   helper exits and the island says "No helper" without a voice reconnect.
2. dr3-2: no repeated blocking ask on an Io error (ask once per connection, or not on the event loop).
3. dr3-3: entry_dirs uses the shared helper.
4. Merge the session branch (4e3c019e, with ci r2's fuzzel change) into claude/doors and click ti-1
   with the real fuzzel 1.9.2: say whether a program starts by click; if not, record the cause.
Done when: tests/merge-gate.sh passes as one run. Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN
(at most 10 lines). If your context is above 200 k, stop instead and say so.
