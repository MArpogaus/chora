# review wm7b r2 (F3): seeded review of claude/wm7b before its merge

Model: opus, effort high, because wm7b moves the face turn into construct (hosted, owner 11-19 10:52) with a flat
fallback and crash recovery; a wrong rule loses a window in a handover (shared RC bar)
Lane: cloud, read-only on code: claude/wm7b head 31f04c07, session base daf83391; wm7a f4e4f4d9 is merged in and
already reviewed. Own commits: `git log --oneline --no-merges daf83391..31f04c07 ^f4e4f4d9` (17); diff of own work:
`git diff $(git merge-tree --write-tree daf83391 f4e4f4d9) 31f04c07` or per commit. One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run.
Read briefs/wm7b-r1.md, briefs/wm7b-r2.md (done criteria) and reports/wm7b.md on claude/session-log.
Checks, each with its command and files:
1) Items 1-6 of wm7b-r2 closed: a1 I3 (xp drops its claim on focus.changed {surface:null}, row wm7a-a1 out of
   known-red), flat fallback to desk 2, m4 (the worker reads it as "client of cube_manager's program": sound or a
   finding), gate-construct-cube.sh section 6, I1 act, docs api.md, configuration.md, decisions.md line.
2) Red first: each fix has its red commit before it (rules.md 8); name the shas.
3) Extra fixes 6e79fb7 (restart act I3), 31f04c07 (xp outlives cube.close after reopen), e973e15 (pane-host xp: no
   pager or turn keys): real fixes or weakened checks.
4) The worker's MAJOR candidate: a crashed construct's resume nodes cover the screen for the 10 s grace. Confirm or
   refute with file:line; severity by the vision card (never covers work).
5) ci-check and cut-check (the lane's gate list lacked them): `tests/merge-gate.sh --only ci-check` may be skipped
   if too slow, then `cargo clippy --workspace --all-targets -- -D warnings` and `python3 tools/cut-check`.
6) `cargo test -p chora-construct -p chora-xp -p chora-kit --features testing`; no file over 1,500 lines.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/wm7b.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
