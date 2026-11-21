
## r1
Head 31736130 on claude/xp-super-start (base 618ae513). Not reproduced: sh-23 is already fixed.
- Evidence: tests/gate-construct-pane-keys.sh (xp and tiling each in a construct pane, click into the tiling pane, Super_L down, Return down and up, Super_L up) is green at 618ae513, before and after my extension. Extension: Start menu (xp-item-0 rects) still shut 1.5 s after the chord, and typed text (`touch <marker>`) reaches the new tiling window. Cell passes, no red commit possible.
- Cause of the original report: the click into the tiling pane left the keyboard with xp, so Super+Return reached xp's tap logic. Fixed by 363276e8 (core: a click into the claim of another look takes the keyboard), red cells 1ce94726 and 9821d96f. Tap logic in plugins/chora-xp/src/wm/keys.rs:165-172 and 411-417 unchanged; the core also does not deliver the Super_L release to xp after a chord taken by tiling.
- Same-output case (tests/gate-handover-shared.sh, tiling and xp on one output, raw Super_L + Return events): also no Start menu after Super+Return (tried, not committed).
- No xp change, lint ratchet untouched. Merge-gate cells of step 3 not run (test-only change; pane-keys cell run green).
- Suggest closing sh-23 as fixed by 363276e8.
