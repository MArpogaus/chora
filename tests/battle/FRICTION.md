# Friction and unexpected behaviour — battle test W1–W7

Living document. Every point: what blocked, whether it needs an API/design change.

## Environment (sandbox) — no code change needed, but the workflows assume more

- **No `foot`.** The spec's window client is missing. Substitute:
  `target/debug/chora-probe window R G B` (app_id `chora-probe`). All W1–W7
  use probe windows. Queries must use `app_id:chora-probe`, not `app_id:foot`.
- **No cargo on PATH**; used the `chora` mamba env
  (`/home/agent/.local/share/mamba/envs/chora/bin`). Debug build of the
  workspace filled the disk to 60 GB once.
- Session mechanics (heartbeat without cron, driver traps, sub-agent
  audit) are documented in the `local-pi-session` skill, not here.

## Review process (phase 1)

Harness-side only (`--no-session` gives no evidence, CLEAN detection,
empty-diff rounds). All of it moved to the `local-pi-session` skill;
the concrete runs are in `tests/battle/reviews/`.

## choractl / API — candidates for change

- **No machine-readable exit or wait primitive.** Health check must be
  `timeout 5 choractl query >/dev/null` in a loop; there is no
  `choractl wait <selector>` and `query` has no `--watch`. All W scripts wrap
  every call in `timeout`.
- **Tiling exports only `set_layout`, `get_layout`, `give`.** Tags (view/move
  window to tag), masters/columns/share are binding-only. Automating "move
  windows between tags" (W1) needs injected `Super+<n>` / `Super+Shift+<n>`
  shortcuts and depends on which surface has focus at that moment — racy in a
  headless loop. *API candidate:* export these tiling methods (awesome does:
  `awesome.client.tags`).
- **chora-xp exports no methods at all** (only the scrollbar thing in
  explorer.rs). Start menu, minimize/restore, Alt+Tab and "XP Restart" must
  be driven by pixel clicks derived from `query` — brittle across theme and
  title-bar changes. *API candidate:* export `xp.<action>` methods like tiling
  does, at least for testing.
- `choractl restart <name>` needs the plugin to have been started through the
  core (view/startup command), fine via `headless-session.sh -- <plugin>`,
  but the error when not is only in the log, not in a code. Minor.
- **`input.inject` shortcut names must exist on the keymap**; a wrong keysym
  rejects the whole batch (documented, good), but there is no dry-run flag to
  validate a batch without applying it.

## Test helpers

- **`tests/ipc-bench.sh` hardcoded sway** (`WLR_BACKENDS=headless sway`) even
  though the repo has `tests/headless-session.sh` with the headless core
  backend — the bench could not run in a sway-free sandbox, and "use a
  release build" conflicts with the battle spec (`cargo build --workspace`
  = debug). *Fixed:* the bench runs on `headless-session.sh` in 8142636.
- `examples/showcase/snake.py` needs `PYTHONPATH=examples/showcase`
  (`choraclient`), undocumented in the battle TODO.
- `headless-session.sh` prints KEY=VALUES only once the socket is up; a
  failed start prints only to its log — a caller waiting on `hl.env` waits
  blind. Minor: a timeout with `tail` of CHORA_LOG is the workaround
  (done in `lib.sh`).

## Unexpected behaviour (battle, appended during runs)

(none yet — appended below by the runs)

## Findings of the W smoke runs (facts, not yet design proposals)

- `choractl query` node JSON has **no `pos`/`size` for surfaces**: geometry
  lives in `transform.matrix`, **column-major** (translation at [12..14]),
  relative to `parent`. Nothing in `docs/api.md` states the order. The
  harness composes matrices up the parent chain (`geom.py`).
  *Docs candidate:* say "column-major" next to "a 4x4 matrix".
  *Superseded:* `scene.query` answers with `rects` per output, so the
  harness reads places and grab points from the core; `geom.py`, `pick.py`
  and `positions.py` are deleted.
- `chora-probe window` ignored `xdg_toplevel.close_request`: it was a test
  client that only exited with its process. So "close all windows" in W1
  tested process death, not the close path. *Fixed:* probe flag
  `--exit-on-close` in 0a1b8b5, loop wake-up in b78d7d1.
- SIGUSR1 emergency reset **revokes all grants** (documented in
  `docs/api.md:478`) — the harness has to grant again after W7's reset;
  the TODO's W7 text does not mention it.
- `choractl spawn` answered pretty-printed JSON; scripting wants one line
  (harness parses with sed). *Fixed:* compact one-line output in 498e4ea.
- The grid after SIGUSR1 is the work of the core, not of tiling: the
  emergency reset disconnects every client, and tiling ends with "the core
  closed the connection" (verified 2026-10-21). W7's "windows in a grid"
  holds without any window manager.

## Findings of the W2 smoke runs

- `chora-probe` never sends `xdg_toplevel.set_window_mode` (server-side
  decoration), and chora-xp draws its frame (title bar, min/max/close) only
  on decorated surfaces. So W2 cannot test the XP title bar at all with the
  current probe. *Tooling candidate:* `chora-probe window --server-deco`.
- Node queries right after session start answer with an empty scene for a
  moment (2 nodes vs 37 two seconds later). Nothing tells the caller the
  plugin is "ready". *API candidate:* a `ready` flag or a `choractl wait`
  (same gap as the missing `wait` above).
- `query` has no field that says which surface holds keyboard focus, so
  "Alt+Tab switches" cannot be verified — only that the core survives it.
  *API candidate:* a `focused: true` marker on the surface node.
- The injected click (`pointer_move` + `click`) works: start menu opens and
  taskbar minimize/restore toggle `xp-win-<s>.visible`.
- Taskbar button of the **active** window: the first click does nothing, the
  second minimizes. *Checked in 0.7 (T12):* not a bug, and `is_active` stays
  in sync. A click on the button of the active window minimizes on the first
  click. A click on an inactive window only activates it. For a visible
  window this looks like "nothing". The smoke harness guessed the active
  window from query order and usually missed, so its first click was an
  activation. It could not know: focus was not in the query back then (see
  the `focused` finding above). Verified headless with the `focused` marker
  of 16a7f90. The first click on the active window's button minimizes after
  focus came from spawn, from an injected Alt+Tab (3/3) and after `choractl
  restart xp`. The click on an inactive window activates, the next one
  minimizes. Real XP behaves the same. *Docs candidate:* state the click
  semantics in `docs/configuration.md`.

## Findings of the W3 smoke runs

- `input.inject` lifts buttons that a request leaves down **at the end of
  that request** (documented). A drag spread over several inject requests
  therefore never grabs: press, all moves and release must be one request,
  which also means no sleeps between the steps. *API candidate:* a
  `pointer.grab`-style method or a `hold: true` flag on `button`.
- The WM handoff by dragging over the output edge works (3/3 to tiling),
  but a dragged-away window keeps answering under its old `owner` for a
  moment; selecting the next victim needs `owner:xp` at query time.
- `choractl restart <name>` takes the short client name (`xp`, `tiling`),
  `plugins.md` names the binaries `chora-xp`/`chora-tiling`. *Docs
  candidate:* state the client names in one table.

## Findings of the W5 smoke runs

- There is no **node budget**: 14.6k rects under `root` where fine, then the
  watchdog disconnected the offender ("frames are slow … blames a client",
  core stays healthy). So the limit of 0.6 is a performance limit, not a
  count. Fine as behaviour, *docs candidate:* say so in `docs/api.md`.
- A name with a client socket is refused on the main socket ("has a client
  socket and connects only through it") — probing both sockets needs two
  names. Documented, but it bit the harness twice.
- After a refusal line the core closes the connection; writing again gives
  `BrokenPipeError`. Harness must stop sending.
- `clients create` on an already taken name answers with an error and
  `choractl` exits 1 — parsing an empty path silently loses the reason.
- snake stress: `screen.capture` needs `choractl grant snake screen.capture`
  (its `grant-snake.sh`); the 16.4 tick/s run itself is stable.

## Limits of this sandbox (not product defects)

- The cgroup holds ~3 GB and the host sits near 100 % RAM: with software
  rendering, 100 mapped windows make the probes die in the swap storm.
  The battle tests now wait for >= 2 GB `MemAvailable` before every
  session and W8 runs 40 windows.
- PID 1 does not reap here (zombies are cosmetic); process killing by
  exact name and the `pkill -f` trap live in the `local-pi-session` skill.

## Pass 1 storm (environment, not product)

- Pass 1 at 20:55 ran into the memory storm again: every workflow reported
  0 windows / missing grants because the probes died on shm allocation.
  Those 21 findings were dropped; workflows pass standalone at 21:2x.

## 0.7 backlog (judged "open (not small)", carried to 0.8)

- Silent core death after the budgets probe (W5, load-only). The core
  vanished right after `frames are slow; the watchdog blames a client
  client=budget`: no panic, no OOM record, no exit path in `safety.rs`.
  Forensics 2026-10-24, part two (plan task T11): ten full W5
  iterations on the rebuilt binaries, plus both battle passes, all
  clean, while the same blame-and-disconnect sequence ran every
  iteration. The core now writes a backtrace on SIGSEGV, SIGBUS, SIGILL
  and SIGABRT (`5b41d48`), and the finding records State, VmRSS and
  wchan (`4444484`). Classified environmental: host memory pressure.
  Reopen only where it bites again, the last words decide.
- SIGUSR1 emergency reset revokes all grants (documented), and the WM
  plugins died with the connection: no window manager until a human
  restarted one. Fixed on `prep-0.8` in `b3a6d84`: both plugins
  reconnect (half second tries, 30 second episode) and re-adopt via
  the existing grace-time machinery.
- Watchdog tolerance while the budget probe churns nodes: fixed in 0.8
  by D57 (`2e4934a`): an owner budget decides which owner the watchdog
  may cut, and stress alone no longer blames the wrong client.
- Blocked here, not in the product (sandbox has no network): rmcp and
  cosmic-text are not in the cargo cache, so `choractl mcp` on rmcp
  (T6) and the text node (T9) stay open; checks 3, 6, 9 and 10 of the
  0.8 gate skip. Run `plans/0.8.md` T6 and T9 where crates.io answers,
  after seeding CJK and emoji fonts.

## Core dead: ten clean W5 iterations after the rebuild (T13 of the rest-plan)

Rebuilt at dd941b0, so the binaries carry the last words handler
(5b41d48), the process facts in findings (4444484) and the owner budget
(2e4934a). Ran tests/battle/w5.sh ten times in a row on 2026-10-24,
14:09 to 14:16 UTC: ten clean iterations, zero findings. Each iteration
churned to 13600 nodes until the watchdog fired, hit the 16 connection
limit, and ended with a capture; the core logged the exact blame-then-
disconnect sequence every earlier core dead finding names as its last
lines. Memory stood at 13 GB MemAvailable of 128 GB. No core died
outside the harness; the kernel side stays blind here (no dmesg,
core_pattern pipes to /bin/false). The eight core dead findings in
findings.md are set to "deferred: next battle pass": if one bites
again, the last words and the process facts decide.
