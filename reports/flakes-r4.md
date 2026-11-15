# flakes r4 (rc 3): report

Lane: cloud, branch `claude/flakes-r4` from `d349a6da`, head `ce8f7928` (pushed). Written 11-15 03:01 UTC.
Load: 8 busy shell loops on 4 cores (load average 9 to 11). Logs are under `/home/user/logs` of the
container only.

| Item | Result | Commits (red, fix) |
|---|---|---|
| 1. F17 words alarm | fixed | `915f97b1`, `4a5ddb40` |
| 2. F16 tiling "no bar" | not reproduced; the failure now says why | `16950ea9` |
| 3. GR1 xp | fixed | `0a3955d2`, `67f1c1f2`; cell `94f29634` |
| 3. GR1 tiling (both places) | fixed | `bdeb13b2`, `64088ae2` |
| 3. GR1 mc | fixed | `02360013`, `4372ea29` |
| 3. GR1 construct `scene.rs:185` | no change: see below | none |
| new: two races of the cube gate under load | fixed (gate only) | `243466bb`, `ce8f7928` |
| 4. F7, A5, A17 | not started | none |

**F17.** Raw evidence first: with `drop_caches` before each run the test failed 4 of 65 times, and the
alarm found the handler thread in the state `D` of `/proc` (a wait for the disk, 67 of 455 alarms)
two times in a row. The first backtrace reads the debug info of the 119 MB test binary; with a cold
page cache it waits for the disk, uses no CPU time and is not `R`, so the r1 check took it for
stuck. Fix: the state `D` counts as held up, as `R` does; a wait for a lock is `S` and still ends
the core. No timeout changed. Red: `a_handler_that_waits_for_the_disk_is_not_stuck` fails on
`held_up_in(b'D')`. Green: 0 of 60 cold runs fail.

**F16.** 31 starts of the tiling cell file under load (8 and 16 loops, with disk churn and dropped
caches) and every doors step of the gates below: the bar showed after 1.8 to 4.4 s each time, far
from the 20 s limit, so a longer wait has no reason. waybar wrote to `/dev/null`, so the run of
train 8 left no trace. The failure now gives the answer of the capture and the last lines of
waybar. Read, not shown: `waybar.sh` waits up to 10 times 11 s for `tiling get_tags` when tiling
does not answer (the method timeout of the core is 10 s); a tiling that stalls once at the start
costs the bar 11 s.

**GR1.** xp (`sync_cube_keys`), tiling (`reconfigure` and the claim in `apply_layout`) and mc
(`settings_changed`) start a new set of grabs and end the old one after it. Each has a unit test
on a fake core that was red: every binding went with `input.ungrab` and came back. The cube gate
has a new cell: on the faces, where xp let its cube keys go, Super+d of xp puts the window away
and back, and the window has the keyboard after it. construct `scene.rs:185` (`leave`) lets all
bindings go and grabs none after it, so there is no second set and nothing to keep: unchanged.
voice (`client.rs:684`, `701`) still calls `ungrab_all` and grabs again (not in the brief).

**Cube gate under load (found by the gates of this round).** (1) The cell of the live drag read
the rest width of the front face during the step back (431 or 462 instead of 375 px), and the
release at 49 degrees then "took 5787 ms to settle": 3 of 6 loaded runs. It now waits until the
width stays: 0 of 5. (2) "the card Starting Media center took 337 ms, more than 300": the clock of
the shell timed three programs of the cell. The card is there when `cube.open` answers; the cell
asks then, without a clock: 4 of 4. Both are gate changes, the product is unchanged.

## Gates (full `tests/merge-gate.sh`, 25 steps, on `ce8f7928`)

| Run | Load | Result | Directory |
|---|---|---|---|
| 1 | 9.2 | PASS, 25 of 25 | `/home/user/logs/gates/merge-gate.HZy9Nj` |
| 2 | 10.5 | FAIL at `test` | see `gate-b.log` |
| 3 | | not run | |
| cold | none | PASS, 25 of 25 | `/home/user/logs/gates/merge-gate.GucXRl` |

**Not done: no three loaded gates in a row.** Run 2 failed in the step `test` with a new flake:
`taskbar::tests::the_speaker_has_ink_and_the_no_sign_is_the_palette_red` of chora-xp, "a PNG file:
NotFound" at `plugins/chora-xp/src/sheets.rs:92` (the picture that the theme rendered into its
cache was not there when the test loaded it). 173 of 174 tests passed; no test shares its cache
folder `tray-red`; cause not found. I stopped there at the context limit of the brief. Before the
two gate fixes the loaded gate failed two times at `cube` (the two races above). The cold gate ran
after `rm -rf target`, `sync` and `drop_caches`, without load.

## Open

- New flake (propose F18): the xp tray test above, 1 of 4 runs of the step `test` in this round.
- Two more loaded gates in a row are owed, after F18.
- F16 has no known cause. If it comes back, the doors log names it.
- F7, A5, A17 not started. No independent review of this round yet.
- `docs/cube-modes.md:166` says the start card shows within 300 ms of `cube.open`; the gate now
  asks for more (at the answer). The sentence still holds.
