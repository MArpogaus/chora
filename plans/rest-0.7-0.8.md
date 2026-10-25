# Work plan: the rest of 0.7 and 0.8, for a local pi agent

Do the tasks in order, one at a time. Do not invent extra tasks. The scope
and the gates are in `docs/roadmap.md`, sections "0.7" and "0.8"; the gate
issue of 0.7 is #91.

## Rules (read first, follow always)

- Repo: `~/Projekte/chora`, branch `dev`. Never push. Never rewrite
  history (no amend, no rebase, no reset). Never touch `services/voice/`.
- Cargo is not on PATH: run `micromamba run -n chora cargo ...`. There is
  no network: use only crates that `Cargo.lock` already has. If a task
  needs another crate, ask (see the last rule).
- Before each commit run, and fix until all pass:
  1. `micromamba run -n chora cargo fmt --all`
  2. `micromamba run -n chora cargo clippy --workspace --all-targets --locked -- -D warnings`
  3. `micromamba run -n chora cargo test --workspace --locked`
- Tests that render need the Mesa EGL variables of CONTRIBUTING.md. Tests
  that failed before your change for lack of a GPU may stay red; compare
  with a run on the commit before your change.
- Commits: conventional, imperative, at most 68 characters in the subject.
  Stage files by name. No AI attribution lines.
- One task = code + a test that fails without the change + the docs in
  `docs/api.md`, `docs/configuration.md` or `docs/architecture.md`. Docs:
  at most 25 words per sentence, active voice, no em or en dashes.
- Keep every existing test green. If a task must change an old test,
  say why in the commit message.
- If a task needs more than about 300 changed lines (except where a task
  waives it; `Cargo.lock` never counts), or the right behaviour is
  unclear: ask your question as the driver tells you, and change nothing.

## Facts of this machine

- The cargo cache holds `rmcp` 3.7, `tokio` 1 and `cosmic-text` 0.19;
  `Cargo.toml` already lists them (commit `3f5fbf3`).
- Fonts for the text node: `/home/agent/.local/share/fonts/chora-test/`
  (`NotoSansCJK-VF.ttc` for `你好`, `Twemoji.ttf` for `🙂`; prefer
  Twemoji over `Noto-COLRv1.ttf`, which swash may not draw).
- There is no podman, no user systemd, no sway, foot or grim here.
  Scripts that need them answer `SKIP: <why>` and exit 0.

## Decisions of Marcel

- **Plugin and process start (D42):** the core starts no program except
  the startup command. Without a reachable user systemd, `choractl` starts
  and restarts programs itself. That is decided at run time, in one binary.
- **The text node:**
  - The core lays out and rasterizes text.
  - A text node reports its layout size in `rects`, like an image without
    a size.
  - `max_width` only wraps.
  - Text re-rasters at the physical scale of the output.
  - The text node uses the existing `color` prop.
- **Fonts:** test fonts do not go into the repo. Tests find fonts through
  the system fonts plus every folder of `CHORA_FONT_DIRS` (colon
  separated). Gate scripts set `CHORA_FONT_DIRS` to the folder above. A
  test whose fonts are missing answers `SKIP`, never `FAIL`.
- **Earlier questions:** the split of T4 of 0.8 (`8bfb8ce`, `4e1a1d1`)
  stays, `1d4d82b` stays, and the T11 proposal of 0.8 is approved (T13).

## Tasks of 0.7

### T1: a versioned `hello` (D29)

- Goal: `hello` takes an optional `protocol` (an integer) and a list
  `features`. It answers with its own `protocol`, `features` and
  `version`. A client with a higher major protocol gets an error with a
  stable code; a lower one works. Start at protocol 1.
- `chora-scene`'s client library sends its protocol and features, and
  warns on stderr when the core's major version differs.
- Test: an ipc test per case (no protocol, equal, lower, higher).
- Docs: the `hello` row and a short "Protocol versions" part in
  `docs/api.md`.

### T2: stable error codes

- Goal: one stable code for each kind of failure, next to the JSON-RPC
  codes. Today there are `REJECTED = 1` and `GONE_SURFACE = 2` in
  `crates/chora-scene/src/ipc.rs`. Add codes for at least:
  - a missing capability;
  - an unknown node or surface;
  - a limit reached;
  - a name taken;
  - a refusal on a client socket;
  - an unsupported protocol (T1).

  Keep `data` with the index of the failed command.
- Test: one test per code that provokes it (gate check 9 of 0.7).
- Docs: a table "Error codes" in `docs/api.md`, one row per code.
- This touches many places; the 300-line rule is waived for T2.

### T3: the client socket fixes its capabilities (D33, D40)

- Goal: `clients.create(name, capabilities)` and `clients.yaml` give a
  client socket its capabilities. A connection through that socket has
  exactly those. `grants.set` and `grants.revoke` for a name with a
  client socket fail with the code of T2 and say "change clients.yaml and
  run choractl clients up". `choractl clients up` recreates a socket whose
  capabilities changed.
- Look at: `crates/chora/src/ipc/clients.rs`, `crates/chora/src/grants.rs`,
  and `clients up` in `crates/choractl/src/main.rs`.
- Test: a client socket with `screen.capture` captures; one without it
  gets the error code; `grants.set` for its name fails.
- Docs: "Client sockets" in `docs/api.md`, "Clients file" in
  `docs/configuration.md`.
- If this conflicts with how `clients up` works today, ask.
- This touches many places; the 300-line rule is waived for T3.

### T4: `choractl run` (D42, part 1)

- Goal: `choractl run [--] CMD ARGS...` asks the core for an activation
  token, then starts the program. It uses `systemd-run --user --scope`
  with `XDG_ACTIVATION_TOKEN` in the environment when
  `systemctl --user is-system-running` answers. Otherwise it forks the
  program itself, detached, with the same variable. It prints the pid as
  one JSON line.
- Do not remove anything from the core in this task.
- Test: a choractl test that starts `true` through the fallback. A test
  that the `surface.created` of a probe window carries the token
  (`tests/probe` has a mode that maps a toplevel with a token).
- Docs: `choractl run` in `docs/configuration.md` or `docs/api.md`,
  wherever `spawn` is described today.

### T5: `choractl restart` and `choractl supervise` (D42, part 2)

- Goal:
  - `choractl supervise NAME -- CMD...` runs a plugin and starts it again
    when it ends (at most 5 restarts a minute).
  - `choractl restart NAME` restarts it: through
    `systemctl --user restart` when the plugin runs as a unit, else by
    ending the process of `supervise`, which then starts it again.
  - `supervise` writes its pid into `$XDG_RUNTIME_DIR/chora/supervise/NAME.pid`.
- Do not remove anything from the core in this task.
- Test: supervise `sleep 60`, restart it, check that a new pid runs.
- Docs: next to `choractl run`.
- The 300-line rule is waived for T5, retro-actively: the review fixes
  of the first round pushed `7df5de2` past it and history stays.

### T6: move every caller to `choractl run` (D42, part 3)

- Goal: every script, test, example and doc that calls `choractl spawn`,
  `process.spawn`, `process.wait` or `plugins.restart` uses `choractl run`,
  `choractl supervise` or `choractl restart` instead. Find them with
  `git grep -nE 'choractl (--name [a-z-]+ )?(spawn|restart)|process\.(spawn|wait)|plugins\.restart'`.
  The battle workflows in `tests/battle/` and `tests/headless-session.sh`
  are the most important.
- Check: run `tests/battle/w1.sh` and `w2.sh` once in a headless session,
  and paste the result lines into the commit message.
- The 300-line rule is waived for T6.

### T7: the gate scripts of 0.7

- Goal: `tests/gate-0.7-<n>.sh` for the checks 4 to 10 of the 0.7 gate in
  `docs/roadmap.md`, on the pattern of `tests/gate-0.8-*.sh`: `ok:`,
  `SKIP: <why>` or `FAIL: <why>`.
  - Check 4 (a Podman container) needs podman: SKIP here. Also add a CI
    job in `.github/workflows/ci.yaml` that runs it on the ubuntu runner,
    where podman is preinstalled. Do not add a new action. Every action
    stays pinned to its SHA.
  - Check 5 (Quadlet) needs a user systemd: SKIP here.
  - Check 6 (`input.grab-any` with F13 and a letter, and the error code
    without the grant) must run here.
  - Check 7 (no fork in the core) answers `FAIL` until T14 removes
    program start; that is expected.
- Write `plans/gate-0.7.md` with one row per check and its result here.
- The 300-line rule is waived for T7: gate scripts are one file per
  check and `f492b25` stays as it is.

## Tasks of 0.8

### T8: `choractl mcp` on rmcp

Task T6 of `plans/0.8.md`. The 300-line rule is waived.

- Use `rmcp` with the features `server` and `transport-io`, and `tokio`,
  from the workspace dependencies.
- Keep the tool names and the argument names of the current server where
  a tool survives, so that `tests/gate-0.8-1.sh` keeps working.
- Turn `tests/gate-0.8-3.sh` and `tests/gate-0.8-6.sh` from `SKIP` into
  real checks, and run all three before the commit.

### T9: the text node, part a: scene only

- `NodeKind::Text` with the props `text`, `family`, `font_size`,
  `max_width` and `color`;
- `check_kinds`;
- create and update unit tests in `chora-scene`;
- the `text` row in the node kinds table of `docs/api.md`.

No drawing yet; if the core refuses to draw `kind: text`, the docs row
says so.

### T10: the text node, part b: core with cosmic-text

The 300-line rule is waived.

- A text module in the core with the cosmic-text font system: the system
  fonts plus `CHORA_FONT_DIRS`.
- Layout, fallback, the glyph raster into an RGBA buffer, and a cache per
  node that lays out again when a prop changes.
- The render element on the pattern of the cursor
  (`MemoryRenderBufferRenderElement`).
- The exhaustive kind matches in `render.rs`, `capture.rs`, `ipc.rs`,
  `inject.rs` and `mesh.rs`.
- The layout size in `rects`.

Test: a text node `Hi` renders non-transparent pixels inside its rect,
and none outside of it.

### T11: the text node, part c: gate checks 9 and 10 of 0.8

- `tests/gate-0.8-9.sh`: a text node shows `Grüße 你好 🙂` with
  `CHORA_FONT_DIRS` set. The check captures it, and each of the three
  words has non-background pixels in its part of the rect. Do not store a
  reference picture.
- `tests/gate-0.8-10.sh`: an MCP client puts a text node under
  `screen:NAME`, and a capture shows it.
- Update rows 9 and 10 in `plans/0.8-gate.md`.

### T12: the open review findings of the rects of queries

`plans/0.7-review-T2-r3.md` lists five findings:
- mirrored and rotated surfaces;
- shader nodes;
- nodes inside a layer;
- the clip under a perspective matrix;
- the long `rects` row in `docs/api.md`.

Fix each one as it says, with a test per code finding. If one needs more
than 100 lines, document the limit in `docs/api.md` instead and say so in
the commit message.

### T13: close T11 of 0.8 as approved

- Append the no-repro entry from `plans/0.8-questions.md`, section T11, to
  `tests/battle/FRICTION.md` (about 15 lines, no code).
- Set the status of the eight `## core dead` findings to
  `deferred: next battle pass`.

## A prototype for the end

### T14: remove program start from the core (D42, part 4), a prototype

- Goal: remove `process.spawn`, `process.wait` and `plugins.restart` and
  their code from the core. The core keeps only its startup command. The
  grant `process.spawn` goes away. `choractl spawn` becomes an alias of
  `choractl run` that prints one deprecation line on stderr.
- Test: gate check 7 of 0.7 as a test: a test that finds no `fork`,
  `Command::new` or `spawn` of a process in `crates/chora/src` except the
  startup command.
- Docs: remove the rows, update the known gaps of client sockets (the
  `process.spawn` gap is gone), D42 in the roadmap says done.
- Only start this task when T4 to T6 are `done`. If one of them is open,
  write your question and stop.
- This is a prototype. Marcel and Claude finalize it. Commit it as one
  commit with the subject `feat!: remove program start from the core`.
  Do not change other tasks for it.
- The 300-line rule is waived for this task.
