# flakes-g r1 (F5, F26, F28, FF1), 11-19

Lane: cloud, opus, branch claude/flakes-g from claude/flakes-f 847f0964, merged
origin/claude/autonomous-dev-session-3w-i33pg7 (daf83391, clean) at start: 2513bee7.
Host: 4 CPUs, 15 G.

## Blocker: the lane container has no setup

The setup script of this container (119 characters, ran 12:08-12:14 UTC) installed none of the
build or gate dependencies: no libudev, libinput, xkbcommon, gbm, libseat, wayland-server, egl,
libdrm (pkg-config finds only pixman-1), no foot, fuzzel, waybar, grim; the Rust 1.99 toolchain
was not installed either (rustup fetched it on first use). `cargo check -p chora --lib` fails:
"failed to run custom build command for `libudev-sys v0.1.4` ... Package libudev was not found".
The brief forbids system packages, so no compositor, no gate script and no Rust cell of
crates/chora ran here. Owner fix: the environment's setup script (cloud environment menu, Edit,
Setup script) must again run the repository setup (apt build deps, foot, fuzzel 1.11.1, waybar);
new sessions then run it. Items 1-3 and rw-2 need it.

## Fixed and proven

| item | cause (file:line at 847f0964) | red | fix | proof |
|---|---|---|---|---|
| FF1 rw-4 under-load.sh no time limit | tests/under-load.sh:44 ran each command without a limit, so a hung gate hung the tool and its busy loops | 8d555fc5 tests/under-load-selftest.sh (new): `sleep 60` with --timeout 2 must end as a failed run within 10 s saying "timed out after 2 s", runs of `true` pass, no busy loop left in the tool's session; red: "unknown argument --timeout" | f0c9e640 each run under `timeout -k 10 S` (--timeout S, default 1800); a run at its limit fails with "timed out after S s"; the gate-lib step of merge-gate.sh runs the self-test | self-test green 4/4 (2 s each); tests/merge-gate-selftest.sh ok |
| FF1 rw-1 tiling-audit wait counted rounds | tests/gate-tiling-audit.sh:86 `fullscreen_rests` looped 80 x 0.25 s, each round two ctl calls plus jq | no red commit: the fix is in a gate script (a test for red-first); evidence: the scratch probe below | c227d433 loops to a deadline of 20 s by SECONDS | probe (fullscreen_state/fullscreen_rests cut out of the script, ctl stub answers in 1 s, bar never hides): before "rests=1 after 181 s", after "rests=1 after 20 s". Not run under the gate (blocker) |

## Not reached (blocker)

- 1 cube.open over 300 ms (tests/gate-construct-cube.sh:656, plugins/chora-construct/src/cube.rs:1142
  `open_cube`): needs the build and the cube gate; no red, no change. The proposal of flakes-f
  (pipeline the four output.add, or one output.add with a count, a small kit Conn API in docs/api.md)
  stands.
- 2 F26 doors "missing" and 3 F28 chat-keys "text typed before the pick did not reach the picked
  helper" (tests/gate-chat-keys.sh:103): need the compositor, foot and fuzzel; no reproducer, no
  cause. Leads unchanged (F26: train 22 dump, click on a hit node over mc's surface s1).
- FF1 rw-2 (crates/chora/src/words.rs:101 short-write cell): crates/chora does not build here.
- FF1 rw-3 (NIT, drag_pane pace) was not in this brief.

## Merge gate on the head

Not run: `tests/merge-gate.sh` starts with build, which fails here (libudev). Head c227d433 has
three commits over the merge 2513bee7, all shell (tests/under-load.sh, tests/under-load-selftest.sh,
tests/merge-gate.sh step gate-lib, tests/gate-tiling-audit.sh); pre-commit hooks passed on each.
The merge of the session branch must be merged again before the gate (brief).

Commits on claude/flakes-g: 2513bee7 (merge), 8d555fc5 (red), f0c9e640, c227d433.
