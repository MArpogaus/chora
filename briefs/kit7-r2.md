# kit7 r2 (F2): fix the review r1 findings of the one helper machine

Model: opus, effort high, because rw-2 and rw-3 are the state rule of the helper machine across four looks and
the presenter (vision card: honest states; never "working" without a helper)
Lane: cloud, branch claude/kit7 from c1dbc47b. One round, stop at 250 k. Cost about 3 %.
Start: merge origin/claude/autonomous-dev-session-3w-i33pg7 (train 62 may have moved it); again before the gate.
Read reports/kit7.md "## Review r1" on claude/session-log. Each item: red cell first (rules.md 8), then the fix.
1) rw-1 BLOCK: `tools/cut-check` exits 0 (names of chora-assistant in construct config.rs:239, mc config.rs:322,
   services/voice/chat/src/main.rs:45: say "the HUD host"; no new allowance in tools/copies.toml).
2) rw-2 MAJOR: helper.rs:141 state()/door() from all live reporters, not the latest: working wins while any helper
   works; the door names the working helper (opener.rs:66 follows). Cells: coder working then voice listening
   gives working; atlas working then coder idle gives a door naming atlas.
3) rw-3 MAJOR: helper.rs:227 voice thinking with no helper is never "working" with door "No helper": choose one
   honest pair (no_helper, or the voice agent named in the door), change step 2 of the red cell, docs/api.md.
4) MINOR rw-4 seed each look's machine from voice state at start and on connect, hear voice.say in xp and
   tiling; rw-5 a cell for Tab right after a new window in construct, known red as kit7-a2 under WM2 (keep the
   flight only where the act needs it); rw-6 Expect(helper="working") and Expect(helper="answered") in
   helper_acts; rw-7 restore the dropped construct-hud checks that still apply to the assistant HUD host
   (name each kept or dropped with its reason in the report).
Done when: items 1-4 closed with red and green shas; `tests/merge-gate.sh --only build,ci-check,acts,xp,hud,chat-keys,construct-hud,mc-handover,keys`
green (ci-check included: rw-1 passed the r1 lane only because it was left out); `cargo test -p chora-kit
--features testing -p chora-assistant -p chora-construct -p chora-mc -p chora-xp -p chora-tiling`; clippy clean.
Report: append "## r2" to reports/kit7.md on claude/session-log (items, red/green shas, gate lines, open).
Then send_message to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines). Poll every command in the
same turn; never end a turn to wait.
Rules: push claude/kit7 and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
no system packages (a missing tool is a blocker).
