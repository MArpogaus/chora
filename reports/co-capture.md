# co-capture (co-36 MAJOR, co-37 MINOR)

## r1

Lane claude/co-capture at f4f49783 (unchanged, nothing pushed to it). Result: BLOCKED at step 1, co-36 does not reproduce on f4f49783; co-37 reproduces and is named below. No fix written, no red test committed: a red case that is green is no red case, and a fix without a cause would be a guess.

Step 0: `git fetch --unshallow`, `cargo check --workspace` and `cargo build --workspace` green.

### co-36: the cube reopens after a capture of the flat xp (3 of 3 variants green)

1. Gate cell (tests/gate-construct-cube.sh, the kill and restart cell, co-3), one added line after the flat xp check: `ctl capture "$work/flat-after-kill.png"`, then construct started again. Run once: "a restarted construct reopens the cube with the windows on its faces" ok.
2. Same cell, closer to the acts: a slow `cube.turn {"by": 1, "ms": 3000}`, 1 s, `kill -9` construct mid-turn, three `ctl capture` of the flat xp 0.3 s apart, construct started again. 3 of 3 green (gate, about 2 min each).
3. The real r4 repro: tests/acts of claude/guide-checks 06bb5af4 with `Session.capturing()` returning True (the capture of the flat xp taken again), `tests/acts/run.py --no-selftest construct` against the f4f49783 binaries, 3 of 3: act "construct started again reopens the cube with the windows on its faces" pass, as is "without construct the pager of xp switches the desk" (captures of the flat xp in between). Each run took about 5.5 min.

What r4 saw therefore is not on f4f49783. Candidates, none proven here: (a) the 91615955 merge of guide-checks carried an older construct than f4f49783 (the diff 91615955..f4f49783 in construct, core and kit is four files: kit wm/book.rs "raise and focus a window given in", construct lifecycle.rs a test only, overlay.rs the fade of the list, a test of the give matrix); the first is the only one on a path of the restart. A build of guide-checks 06bb5af4 in its own target dir failed every act with the faces "failed" (the xp manager was not found from that target dir, my setup, not a repro), so (a) is not tested. (b) a timing fault that needs a loaded machine (r4 ran with other lanes). (c) a state of r4's machine.
Reading only: the log line of r4, "Alt+Super+1..4 is grabbed already", is the normal refusal of a give key while another client holds it (crates/chora-kit/src/runtime.rs:365-371 logs the first refusal of a binding once and returns Ok(false); `grab_give_keys` retries from `regrab_give_keys`, runtime.rs:496). While a flat xp is the manager it holds the four give keys, so the line says nothing about a stale grab owner of the dead construct: `ipc_close` drops the grabs of the connection (crates/chora/src/ipc.rs:1204). I did not find a stale owner. `screen.capture` (crates/chora/src/capture.rs:292 `render_shot`) renders into its own texture and restores `frame_left_out` and `frame_dark`; I found no handle to the client there.

Suggested next step for the PM: rerun r4's act on the current guide-checks merge with f4f49783 merged in (capturing() returning True), at --jobs 1 on an idle machine; if it is green, close co-36 as "not reproduced on f4f49783, pin: the two gate variants above". If it is red, hand me the binaries or the sha and the construct-again.log and I continue from step 2. The mid-turn variant of 2 would make a fine pin of the gate cell (about 3 s more); I left it uncommitted as it is green.

### co-37: the capture of the flat xp after the kill shows text at 1.00:1 (reproduced 3 of 3, not fixed)

The acts run of 3 reproduced it, each run: `FAIL: construct I6: in act 'construct killed mid-turn leaves the windows with a flat xp within 3 s': on HEADLESS-1 'My Computer' (xp-icon-1) has the contrast 1.00:1 ((233, 237, 242) on (233, 237, 242)), the guide wants 4.5:1` (3 of 3; evidence of the first run /tmp/acts-run-7n9ka_3v/construct-I6, scratch of this container). A text of exactly the colour of the pixels behind it is no rendering of a label: the capture reads the flat xp before its face drew the text (or before the output of the xp face had its first frame after the move to HEADLESS-1). The act gates on `flat_xp()` (xp state and owners), which holds as soon as the desks are on the screen, not once a frame is on it. Read only, not fixed: with no co-36 on this head the fix would be a wait of the harness for a drawn frame (render.stats frame counter of the output past the flat state), which belongs to the acts harness lane, not to the core. MINOR, stays open.

### Gate

Not run: nothing changed on claude/co-capture.
