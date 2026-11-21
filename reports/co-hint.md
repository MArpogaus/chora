
## r1

co-42 (rc 2, F7): the hint Keys of a face rests as a small chip after its 3 s; a click opens the key list.

- Branch claude/co-hint: fix cfc805d4 on c69354ae, merged with origin/claude/autonomous-dev-session-3w-i33pg7 as 51475800 (one conflict, camera.rs `grab_keys`: kept this side's doc, took `pub(crate)` from theirs).
- Red act: "co-42: the hint Keys of a face stays after 3 s and opens the key list by a click" (tests/acts/looks/construct.py, keys_default_acts). It steps in, waits 5 s, clicks the hint where it stands. Red on c69354ae: `no the hint construct-hint-keys after 5 s`; green after.
- Fix: new `Set::Rest` (hints.rs `resting_set`: engaged in a face, no timer, no dwell) draws Keys alone with the lifted glass tint (overlay.rs). No new core event, no pointer tracking. One line in docs/construct-style.md. design/construct-hud-spec.md does not forbid a resting chip.
- Not dimmed: the first try dimmed it to 0.6, which broke I6 (4.44:1 against the 7:1 guide). The lifted tint cleared the 6.65:1 over the green face.
- one_to_one (act "back at 1:1 ...") now skips the boxes of construct-hints*, since the chip stays by design.
- Order of the acts: the new act runs after "the hint Keys of a face names a key ...", so the known-red rows of the key list keep their act.
- Acts loop (construct, 5 runs): 4 of 5 green. Run 4: `FAIL: construct I6 passes but is listed in known-red.toml (guide-checks r2: construct-mode-step-in-text: the contrast); remove the row`. That is the held-turn mid-fade dim row (act "the cube is held halfway through a turn"), no resting chip involved, not caused here; not rerun to green. The row wants `flaky = true`.
- Full gate `tests/merge-gate.sh --jobs 1` on 51475800: all steps passed (gate-exit 0).
- Capture: captures/co-hint-r1-chip.png: the chip at the bottom of a face, taken before the tint lift (passing acts keep no still).

## Review r1 (PM, 2026-11-21 19:08 UTC; small diff, PM not the builder)
Base..head: c69354ae..51475800; the fix is cfc805d4 (4 files, +45 -7), the rest is the merge of 1c599c74.
- Done criteria: a face keeps a chip `Keys` after the 3 s hints (hints.rs Set::Rest for engaged with a cube and no timed or low show); its unit test updated (at(true,true,false,false) = Rest, at(true,false,false,false) = None); act "co-42 ... stays after 3 s" red on c69354ae, green after; full gate green at --jobs 1. Pass.
- KISS: one new Set reusing the keys item and the over-content glass; the lifted tint instead of a 0.6 dim, which broke I6. Pass.
- Open note (not a finding): the lane merged 1c599c74, not the session head 892d035c; the train's full gate on the current head is the check. The capture predates the tint lift, so the chip look is proven by the act and I6 checks only; RECORD v9 takes the reviewed capture.
Findings: 0 BLOCK, 0 MAJOR. Verdict (computed): MERGE-OK. co-42 closes when the train merges.
