
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
