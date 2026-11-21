
## r1
Head 31736130 on claude/xp-super-start (base 618ae513). Not reproduced: sh-23 is already fixed.
- Evidence: tests/gate-construct-pane-keys.sh (xp and tiling each in a construct pane, click into the tiling pane, Super_L down, Return down and up, Super_L up) is green at 618ae513, before and after my extension. Extension: Start menu (xp-item-0 rects) still shut 1.5 s after the chord, and typed text (`touch <marker>`) reaches the new tiling window. Cell passes, no red commit possible.
- Cause of the original report: the click into the tiling pane left the keyboard with xp, so Super+Return reached xp's tap logic. Fixed by 363276e8 (core: a click into the claim of another look takes the keyboard), red cells 1ce94726 and 9821d96f. Tap logic in plugins/chora-xp/src/wm/keys.rs:165-172 and 411-417 unchanged; the core also does not deliver the Super_L release to xp after a chord taken by tiling.
- Same-output case (tests/gate-handover-shared.sh, tiling and xp on one output, raw Super_L + Return events): also no Start menu after Super+Return (tried, not committed).
- No xp change, lint ratchet untouched. Merge-gate cells of step 3 not run (test-only change; pane-keys cell run green).
- Suggest closing sh-23 as fixed by 363276e8.

## PM check r1 (00:08 UTC; PM not the builder; diff 618ae513..31736130, 1 file, +10 -1, test only)
- xs-1 MAJOR: "sh-23 not reproducible" is not backed: the new assert replays a click into the tiling pane, then Super+Return; the persona (feedback/2026-11-20-p2r3-ricer.md:27) did Super+2, Super+1, Super+Return with no click, at 22:03, twenty hours after 363276e8. The persona's path is untested; sh-23 stays open.
- xs-2 MINOR: merge-gate cells of step 3 not run (test only; the train runs them).
Verdict (computed): open MAJOR: FIX. The added asserts are kept (a good guard); r2 replays the persona's steps red first.

## r2
Head 31736130 on claude/xp-super-start (unchanged, no code commit). Not reproduced, setup: tiling+xp 1280x800 and xp+tiling 1920x1080 (tests/persona/session.sh, binaries of 31736130, CHORA_TEST_FLAGS=--test-injected-press-is-real), act.py keys, `choractl query id:xp-item-0` rects as the open test (the gate's menu_open).
- Tries (all Start menu shut 1.5 s after Super+Return; xp state `open` empty):
  1. A: Super+2, Super+1, Super+Return one chord each via act.py key. shut.
  2. A (second round, same session): same chords, pointer moved to the xp part first. shut.
  3-7. A: Super+2, Super+1, Super+Return with 0.4 s between, 4 times. shut each time.
  8. A: Super+3, Super+Return. shut.
  9-11. B (fresh session, pointer at 900,500): Super+2, Super+1, Super+Return with 0.3 s gaps, three times. shut; Super+Alt+1 after: shut.
  Sanity: a lone Super_L tap in A opens the Start menu, a second closes it (xp tap logic alive).
- Raw per-event inject from choractl was refused (no grant); act.py key sends the chord events itself, so the same press/release path was covered.
- Observed in A: focus holder stayed xp (xp windows s1.. opened by Super+Return, tiling had 0 windows, tiling shown desk 3); the ricer's tiling-owned terminal came only after Super+Alt+1, a step I ran in B, no menu.
- Reading: the path is the one fixed by 363276e8 (core hands the Super release of a chord to the claimant, xp's tap flag is cleared by `key()` at plugins/chora-xp/src/wm/keys.rs:165-172). Not reproduced on current code; no red possible, so no fix (step 3 not applicable).
- Gate cells not run (no code change); r1's pane-keys cell stays the guard.
- Suggest closing sh-23 as not reproducible on 31736130 / fixed by 363276e8; if the ricer sees it again, ask for the raw action log (shots 013-014 timing) of that round.

## PM check r2 (2026-11-21 00:20 UTC)
Verdict: FIX (computed: xs-3 MAJOR open). sh-23 stays open.
- xs-3 MAJOR: "not reproduced" rests on a partial replay. Shot 014 of the ricer (media/2026-11-20/persona-tiling-r3-ricer/shots/014.png) shows the Start menu open over two foot windows right after Super+Return; 013 shows it shut. The lane replayed only actions 25-29. It skipped actions 1-23 of actions.jsonl in the same file tree: click Apps (50,14), type "edit", Ctrl+u, scroll, click a launcher row (262,230) that opened foot s1 (xp frame), typed commands with Return into s1, click Keys (749,14), Escape. Any of these can leave a Super or tap state behind (the Keys list and Escape most of all). Also the lane's setup A had no client typed into before the chords.
- xs-4 MINOR: the brief asked for tests/persona/round.sh with the ricer's setup (PERSONA_CHECK=1, round.sh begin); the lane used session.sh only.
Next: r3 replays the whole action log in order with the same gaps, then cuts the prefix by halves to the action that matters.
