# mc launcher

## r1

Lane mc-launcher-r1 (rc 1, F7; rtr1-3 MAJOR). Branch `claude/mc-launcher-r1`, base 5646b8fd, head 53789bea6.

**Diffstat** (5646b8fd..53789bea6): `crates/chora-kit/src/launcher.rs` +52 -4, `tests/gate-launcher-mc.sh` +141 (new), `tests/merge-gate.sh` +7 -3; 196 insertions, 4 deletions.

**Commits**
- 7ed82ef40 test(mc): launcher click and keys, light and dark (rtr1-3): the red commit (gate step `launcher-mc`, two kit unit tests; `a_config_that_fuzzel_rejects_is_left_out` red on 5646b8fd, the gate cell red with the old fuzzel).
- 80dbd0b48 fix(kit): a launcher whose fuzzel rejects its config is asked plain.
- 53789bea6 test(mc): launcher-mc polls with the lib, no new sleep line (the first full gate stopped at `ratchets`: sleep 782 against 778; the cell took the `until_ok` and `pace` of gate-lib).

**Reproduction.** `run.py --check mc` on 5646b8fd: act 4 red, "the launcher is not open", 21 of 22 green. By hand (persona mc session, `act.py click` on Programs) the launcher opened: the fuzzel of the shell PATH is 1.11.1 (`/root/.local/bin`). The tour PATH (`demo/tour/launch.py` LAUNCHER_PATH, `/usr/bin` first) finds `/usr/bin/fuzzel`, which is **1.9.2** on this host. `fuzzel --config FILE --dmenu` with the file mc writes: `err: config.c:866: FILE:5: [colors].prompt: not a valid option: prompt`, exit 1. The `--check-config` of the PM's check ran on 1.11.1 (1.9.2 has no `--check-config`).

**Bisect** (e39448ad..5646b8fd, by the cause and by the gate cell on a build of each side, chora-mc swapped into the binaries of the lane): `cd277ee4^` (1d9e1f658) green, `cd277ee40` red, so the first bad commit is **cd277ee40** (rpx1-10); 761c0b29 and 798fdc33 (column rects) are not in it.

**Cause.** cd277ee40 starts the launcher as `chora-mc --launch fuzzel --config FILE` with a file that has `prompt`, `placeholder`, `input` and `counter` keys; a fuzzel older than 1.10/1.11 rejects the file and exits at once, so on a PATH whose first fuzzel is 1.9.2 a click on Programs (and Super+Space) ran a process that showed nothing (`chora_kit::launcher::ask` reported it as a notice, which the tour does not show).

**Fix.** `chora_kit::launcher::choose_reporting` asks `ask_or_plain`: when a fuzzel with `--config FILE` (or `--config=FILE`) fails with an err line, the list is asked once more without the config; no config, or a plain failure, is reported once as before.

**Cell.** `tests/gate-launcher-mc.sh`, step `launcher-mc` (also in the fuzzel minimum table): mc light and dark, each with the fuzzel of the machine and with a stand-in that is 1.9.2 to the config of mc; each session opens the launcher by a click on Programs and by Super+Space and closes it by Escape, a fuzzel process and pixels in its box. On 5646b8fd: `FAIL: mc light, old fuzzel: no launcher after mouse` (the two fuzzel of the machine's own: ok); on cd277ee4^: 4 of 4 ok; on the head: 4 of 4 ok.

**Tour.** `run.py mc` on the head: take 1 (recorded, TOUR_OUT fresh) 22/22 green, take 2 (`--check`) 22/22 green. Baseline 5646b8fd in a worktree (recorded): 20/22, act 4 red as in the review, and **act 6 red** ("Mouse path: opens the chat in the column: the chat is not shown: []"), which the `--check` take on 5646b8fd did not show and the two takes on the head did not show: not this diff, not seen again; named for the PM. Captures `media/2026-11-25/mc-launcher-r1/`: `m8-before-5646b8fd.png` (93 KB, no launcher), `m8-after-53789bea.png` (115 KB).

**Open.** (1) On a fuzzel 1.9.2 the fallback launcher is the plain fuzzel: a light box on the dark page in m8.Q (the theme of rpx1-10 applies on fuzzel 1.11+, the minimum of the package); a middle tier with only the keys of 1.9.2 would theme it too. (2) The power menu of mc has the same defect on 1.9.2: `power.sh` runs `CHORA_MENU='fuzzel --config FILE'` and exits 0 on the empty choice, so Power shows nothing; not in this area, not fixed. (3) the persona session of the gate printed `ipc: refused conn client="launcher" code=1` for the notice connection of the launcher (the failure notice of `tell` is refused: no grant); the notice of a failed launcher therefore never shows on a persona or tour session. (4) the column of the dark mc in a persona session shows the HELPERS and CHAT rows overlapped for the first seconds (capture 001); not this lane.

**Gate.** `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no step filter, no --push) on 53789bea6: all steps passed, 67 PASS lines, no SKIP, exit 0; `launcher-mc PASS 32s`, `launcher-click PASS 17s`. Last lines: `tiling-disc PASS 73s`, `e2e PASS 276s`, `all steps passed; no --push, nothing is pushed`. `pgrep -fa 'chora --headless'` empty at the end.
