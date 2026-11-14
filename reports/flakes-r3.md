# flakes r3 (rc 3, rc 9): report

Lane: cloud, branch `claude/flakes`, head `17f98f8a` (pushed). Written 11-14 22:30 UTC. Load: 8 busy
shell loops on 4 cores (load average 9). Logs are under `/home/user/logs` of the container only.

| Item | Result | Commits (red, fix) |
|---|---|---|
| 1. R1 refresh during a node grab | fixed | `17d4aad9`, `33653c31` |
| 2. R2 u32 wrap | fixed and documented (docs/api.md, event.rs) | `3bf90064` (test in it) |
| 2. R3 time 0 means unknown | fixed | `611a69db`, `b7710a24` |
| 3. F15 lost first key | cause found: the gate, not the product | `32d00340`, `d80524a7` |
| 4. xp cascade race | fixed | `cd473d24`, `cf84c2a5` |
| 4. layout light-2560x1440@2 | cause found by reading, fixed; not reproduced | `0a67717c`, `f3e3fbce` |
| 5. F7, A5, A17 | not started | none |

**R1.** `State` keeps `pointer_time`, set by each move and button; `refresh_pointer` uses it. Red:
times `[336951, 81234, 336951]` instead of `[81200, 81234, 81234]`.
**R2.** Device input had a 64-bit time, synthesized input a 32-bit one. Both now go out as the 32-bit
count; red `4294967301` instead of `5`. **R3.** The cube times an event with `time` 0 by its own clock;
red: two events without a time both at 0 ns.
**F15.** Raw evidence (WAYLAND_DEBUG of foot): the core delivers every key of `mkdir`, the `m` too.
The cell terminal-keys types a plain Escape into bash; readline holds it as the Meta prefix and takes
the next character (Escape, m is M-m). Shown in a pty: after ESC, `mkdir zz` makes nothing, ` mkdir yy`
works. The cell passes alone without the space and fails in the whole gate. Change: Ctrl+g after that
Escape, no leading space (2 of 2 whole gates, 3 merge gates). New cell in gate-chat-keys.sh: one key
after Esc closes the chat, read back by the probe, with enter, leave and key lines on failure.
**xp.** A window takes its turn at creation and its place from that turn at its first size.
**Deviation:** a window that is maximized before its first commit uses up a turn (a 30 px gap).
**layout.** `regrab` of the construct let all bindings go and grabbed them again; a key in between
goes to the focused window. At the end of a fly-in Alt+Escape reached the terminal, and its ESC then
took the next typed key (the second failure). The kit has `start_grabs` and `end_grabs`; a binding of
both sets stays with the core. Red: `input.ungrab Alt+Escape` before `input.grab Alt+Escape`.
Loaded runs of the cell: 8 of 8 before the fix (not reproduced), 6 of 6 after.

## Gates (full `tests/merge-gate.sh`, 24 steps, on `f3e3fbce`, load 9.2 to 9.6)

| Run | Result | Directory |
|---|---|---|
| 1 | PASS | `/home/user/logs/gates/merge-gate.p07gI3` |
| 2 | PASS | `/home/user/logs/gates/merge-gate.lZq7ey` |
| 3 | PASS | `/home/user/logs/gates/merge-gate.aBhsPm` |

Three of three. After them the session branch moved (11 commits); it is merged as `17f98f8a`, with
build and `cargo test --workspace` green, but no full gate ran on that head.

## Open

- xp, tiling, mc and voice still call `ungrab_all` and grab again: the same gap. Not changed (scope).
- F7 read only: the watchdog does not count time in which its thread did not run, so the test can
  fail on a starved host by design. Not reproduced, not changed. A5 and A17 not started.
- No independent review of this round yet.
