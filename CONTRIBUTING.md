# Contributing

## Branches

- `main` holds the root commit `chore: init` and merge commits. Never
  commit to `main` directly.
- In a repository with a single author, `dev` is the working branch.
  Pull requests target `dev`.
- In a repository with several contributors, each contributor works on
  `dev-NAME`. Pull requests from `dev-NAME` target a protected `main` (D47).

## Where shared code lives

A mechanism that two looks (mc, xp, tiling, construct) need lives in a shared crate, not in a look:

- `crates/chora-scene`: the scene model, IPC types and client, and `paths` (every XDG and runtime directory, `private_dir`; D101).
- `crates/chora-kit`: the plugin runtime, `actions` and `settings::Common` (D98), `ask` (D100), `words` and `Language` (D102), `wm` (`book`, `focus`, `give`, `kept`, `open`, `stack`, `state`; D99), `helper`, `runtime_file`, and `testing` with `FakeCore` (D96); the rest is in the list below, one line per module.
- `crates/chora-draw`: drawing blocks.

One line per module of `crates/chora-kit/src`: what lives there, and when to use it. A new module adds its line in the commit that adds it.

- `actions`: the actions every look shares (`System`), `Action<L>` for a look's own, and `actions/perform.rs` that performs them; use it for a key binding.
- `ask`: the question card model of `voice.confirm`; use it for any yes or no question to the person.
- `chat`: the transcript of a helper, the text sanitizer and the wrap into lines; use it where a host draws a chat.
- `clock`: the monotonic or stepped clock of animations; a recording steps its time.
- `conn`: the connection skeleton (`hello`, calls, node queries, tolerant transactions); the runtime owns the connection.
- `cube`: the cube role (`cube.list`, `cube.turn`, `cube.pull`, `NAME.cube`) and `exporter`; use it to host a cube or to reach one (D113).
- `desktop_entry`: the applications of the XDG desktop files; use it for a menu or launcher.
- `effects`: the driver of wobbly windows and the magic lamp for a window manager.
- `float`: floating windows off a face of a shape, the method `windows.float`.
- `geometry`: the slots of panes in a 3D context, the cube and its faces (D76).
- `give`: the claim under a point and the method `window.give` that every window manager exports (D55).
- `glass`: the tokens of the construct HUD (material, type, space) that the construct and `chora-assistant` share.
- `helper`: the state machine of the helper from the events of voice; use it for the word and the door of a helper (D111).
- `hud`: the model of a HUD folder (version 1) that `chora-assistant` draws.
- `icons`: the icons of the installed icon themes, by name.
- `keysheet`: the rows of a key list from the bindings; use it, not words and caps of your own (D114).
- `lamp`: the frames of the magic lamp of minimize and restore.
- `launch`: start a program and follow it until its window shows (D104); never track a child by hand.
- `launcher`: the rows of `fuzzel --dmenu` for the installed programs, and what a typed text runs.
- `modes`: the use and navigate modes of a view on a shape of panes, such as the cube.
- `notice`: show, replace and dismiss a message through `chora-notify`; never draw a notice or call D-Bus yourself (D110).
- `runtime`: the plugin runtime: connect, subscribe, export methods, grab bindings, `settings.*`, reconnect; also `runtime_file` and `report(name, err)`, the one log line of a failed request that a look keeps running through.
- `swallow`: the Super chords that a face or pane manager binds no action for; use `Runtime::grab_swallowed` after the manager's own grabs and before `end_grabs`, so that a window gets no `Super+q` or `Super+Tab` (D118).
- `settings`: the typed YAML settings of a plugin and `settings::Common` (D62, D98).
- `shared`: the files every look shares in the folder `shared` (`power.sh`, `screenshot.sh`, launcher entries).
- `testing`: `FakeCore` and the script of a test over it (D96); use it in every plugin test.
- `volume`: the volume of the default output through `wpctl`, on a thread of its own.
- `wobbly`: the model of wobbly windows, a spring grid.
- `words`: the word tables and `Language` (D102, D105); use them for every word on the screen.
- `xdg`: the XDG base directories, where empty and relative values do not count; the runtime and cache directories are `chora_scene::paths` (D101).

One line per module of `crates/chora-kit/src/wm`, the window-management core without IO (D99):

- `book`: which windows are the look's: the claim filter, adopt and forget as proposals, outputs, phase events.
- `focus`: the keyboard focus of a look and its `Policy` for the next focus (D107).
- `give`: a window that goes to another window manager, and the pass-on of a reserved surface.
- `kept`: what a restart keeps in node meta under the versioned key `kept.1`.
- `open`: the list of open menus, popups, dialogs, sheets and launchers; use it for Escape and for what a drag shuts (D108).
- `stack`: the two stacking rules: a modal above its parent, the held window on top (D109).
- `state`: the typed answer of `state.get`; every look answers the same keys.

The decision records are in [docs/decisions.md](docs/decisions.md#decisions-of-the-shared-core); the layout is in [docs/project-structure.md](docs/project-structure.md).

**A look uses the kit; it keeps no copy.** When a mechanism moves to the kit, a row in `tools/copies.toml` counts what remains of it in the looks. To add a row:

1. Write the pattern as a regex of the line that marks the copy (for example `to != NAME`), and `allow` the crate prefixes that may hold it (usually `crates/chora-kit/`).
2. Set `max` to the count that exists today and give the row a `name`; `paths` limits the scan (default: crates, plugins, services).
3. Port the looks, and lower `max` with each port; at `max = 0` the row blocks, and `tools/cut-check` fails in `ci-check` and the merge gate when a copy comes back.

A count only falls: `tools/cut-check` prints "lower max" when the real count is lower. Write the red test of the shared module first (D96 `FakeCore`), then the move, then the row.

The cut gate (D103) runs `tools/cut-check` in `ci-check`, the merge gate and CI: a plugin or service may not depend on another one, and a row of `tools/copies.toml` at `max = 0` blocks a copy that comes back. A merge that moves code into the kit lowers the row in the same commit. Details are further down, next to the size ratchet.

## Hooks

Install the hooks once in every fresh clone:

```bash
pre-commit install --install-hooks -t pre-commit -t commit-msg -t pre-push
```

Plain `pre-commit install` activates the `pre-commit` stage only. Check that
`.git/hooks/` contains `pre-commit`, `commit-msg` and `pre-push`. CI also
runs `cargo machete`, which fails on an unused dependency.

The hooks run:

- `cargo fmt` and `cargo clippy` with `-D warnings`. `clippy.toml` sets the
  complexity guards. Split a function instead of raising a limit. See
  [Lints](#lints).
- `ruff check` and `ruff format` on `services/voice/`. `ruff.toml`
  selects the rules and the complexity guards.
- file hygiene, YAML and TOML formatting, and `typos` (allowed
  words are in `_typos.toml`)
- `shellcheck` on all shell scripts (`.shellcheckrc` sets the source path).
  Fix a finding, or add `# shellcheck disable=SCxxxx` with a reason
- `actionlint` on the workflows
- `commitizen` on every commit message
- no inline comments: a comment on an added line of Rust, shell or Python
  fails, except doc comments, `// SAFETY:`, tool directives and the usage
  header of a script
- `mkdir -p` under `$XDG_RUNTIME_DIR` in a shell script of `flavors/` or
  `plugins/` needs `-m 700`
- session files: logs, reports and media paths of the autonomous sessions
  fail
- commit message length: the subject at 68 characters, body lines at 72
- red first: a `fix` or `feat` commit needs a test change in it or in a
  commit before it on the branch, or a body line `No test: <reason>`;
  `tests/red-first.sh` holds the rule that the red-first step of the merge
  gate also runs, and `tests/merge-gate-selftest.sh` proves both; the
  hook refuses only when the base ref resolves, else (a shallow or
  single-branch clone, a first commit, a commit on the base branch
  itself) it prints a note and the merge gate checks the range
- no process by its name alone: a test script that uses `pkill`,
  `killall`, `pidof` or `pgrep` without a parent (`-P`), a session
  (`-s`), a process group (`-g`) or a pid file (`-F`) fails, because
  other sessions run cores on the same machine; text in comments,
  quotes, `echo`, `for` lists and here-documents is not a command, but a
  command substitution is, quoted or not; a `pgrep` filtered by
  `/proc/PID/environ` (`session_pids`) passes, and a first argument in a
  variable (`pgrep $scope -x foot`) only warns
- the guard test: `tests/guards.sh` runs when a guard or its test changes;
  a commit that only adds `*_case` lines to it passes while it is red (the
  red commit of a test-first change), any other change to a red one does not

Run all hooks on all files before a push:

```bash
pre-commit run --all-files
```

## Build and test

The build needs the system libraries in `dist/fedora-build-requires.txt`, or
their equivalents on other distributions. With rustup, `rust-toolchain.toml`
selects the Rust version of development and CI. The `rust-version` in
`Cargo.toml` is the minimum version.

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
```

### Lints

`[workspace.lints]` in `Cargo.toml` sets the lints of all crates. Clippy
runs with `pedantic` and these further lints:

- `unwrap_used` and `expect_used`. Tests can use both (`clippy.toml`).
- `print_stdout` and `print_stderr`. The core logs with `tracing`.
- `dbg_macro`, `todo` and `unimplemented`.

The plugins have their own `plugins/clippy.toml`, because clippy reads
only the nearest one: it repeats every setting of `clippy.toml` (the
`clippy-config-sync` hook checks that) and adds `disallowed-methods` and
`disallowed-types`, so that a plugin that opens its own connection to the
core (`Conn::connect`, `Client::connect`, `Client::from_env`,
`Client::from_stream` or the type `Client`) fails clippy at once. The
runtime of chora-kit owns the connection; `tests/gate-0.8-plugin-runtime.sh`
stays as the grep backstop.

`rustdoc::broken_intra_doc_links` is `deny`. Clippy does not run it, so
CI also runs `cargo doc --workspace --no-deps`.

Handle an error with `?`, or log it and continue. Allow a lint only where
the code cannot fail or must print. Put the `allow` on the smallest item,
and give a reason:

```rust
#[allow(clippy::expect_used, reason = "new_seat adds the keyboard")]
pub fn keyboard(&self) -> KeyboardHandle<State> {
    self.seat.get_keyboard().expect("the seat has a keyboard")
}
```

A program that writes its output or its log to the terminal allows the
print lints for its crate or module, for example `choractl` and the
plugins, which log to stderr. Use `unwrap_or_else(PoisonError::into_inner)`
for a lock.

`services/voice` and `choractl` still allow `unwrap_used` and `expect_used`
for the whole crate, until a later change removes the remaining calls.

### Coverage

The `coverage` job of CI runs the tests with
[cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov). The job
summary shows the line coverage per crate, and the `lcov` artifact holds
`lcov.info`. There is no threshold. To measure locally, with the
`llvm-tools` component of rustup:

```bash
cargo llvm-cov nextest --workspace --no-report
cargo llvm-cov report --json --summary-only --output-path coverage.json
tests/coverage-summary.sh coverage.json  # needs jq
cargo llvm-cov report --html             # target/llvm-cov/html
```

### Tests

The unit tests of the core run a core without a backend in the test
process. `crates/chora/src/testing.rs` gives them IPC clients on socket
pairs, a Wayland client, outputs, and a time that the test moves forward.
Pixel tests render with the software driver of Mesa, from
`mesa-dri-drivers` on Fedora. Text tests measure the system fonts Inter
and DejaVu Sans, Serif and Sans Mono: `rsms-inter-fonts`,
`dejavu-sans-fonts`, `dejavu-serif-fonts` and `dejavu-sans-mono-fonts` on
Fedora, `fonts-inter` and `fonts-dejavu` on Debian. chora-mc finds the
face files with the tools of `fontconfig` (`fc-list`, `fc-scan`,
`fc-match`), which the font packages of Fedora do not pull in. Without
them, these tests fail.

All testing runs headless: `chora --headless OUTPUTS -- STARTUP` runs a
whole session on virtual outputs, without a host display, GPU or seat.
OUTPUTS is `WxH[@SCALE][+X+Y]`, comma separated, for example
`1280x800@1.5,1920x1080+1280+0`; the outputs are `HEADLESS-1`,
`HEADLESS-2` and so on. Input comes only from `input.inject`. Captures
render the same scene as the frame loop, in a separate pass. `choractl
add-output 1024x768` and `choractl remove-output HEADLESS-2` plug outputs
in and out for hotplug tests; `choractl configure-output` changes their
scale and transform.

`tests/headless-session.sh` starts such a session in a private
`XDG_RUNTIME_DIR` under `/tmp`, without the `WAYLAND_DISPLAY` and
`DISPLAY` of the host. It prints `XDG_RUNTIME_DIR`, `WAYLAND_DISPLAY` and
`CHORA_SOCKET` as lines `KEY=VALUE`, waits, and on a signal or at the end
of the core stops the process group of the core and removes the
directory. An agent or a CI job uses it so:

```bash
export PATH=$PWD/target/debug:$PATH  # plugins start apps with choractl run
unset DBUS_SESSION_BUS_ADDRESS       # no units in the user manager of the host
tests/headless-session.sh 1280x800 -- target/debug/chora-tiling >session.env &
session=$!
until grep -q CHORA_SOCKET session.env; do sleep 0.1; done
set -a; . ./session.env; set +a
choractl grant agent screen.capture input.inject
choractl run foot
choractl --name agent capture shot.png
kill $session
```

`CHORA_BIN` sets the directory of `chora` (default `target/debug`), and
`CHORA_LOG` the log file of the core. Programs of `choractl run` get
the variables that the sourced lines provide (see
[Programs](docs/api.md#programs)). Without
system Mesa, the software EGL driver and the clients can come from Fedora
RPMs unpacked into the home directory:

```bash
export __EGL_VENDOR_LIBRARY_DIRS=$HOME/.local/opt/mesa-egl/usr/share/glvnd/egl_vendor.d
export LD_LIBRARY_PATH=$HOME/.local/opt/mesa-egl/usr/lib64
# a client, for example foot, with its own libraries and fonts
choractl run env \
    LD_LIBRARY_PATH=$HOME/.local/opt/clients/usr/lib64:$LD_LIBRARY_PATH \
    FONTCONFIG_FILE=$HOME/.local/opt/clients/fonts.conf \
    $HOME/.local/opt/clients/usr/bin/foot -f "DejaVu Sans Mono:size=10"
```

`dnf download --resolve PACKAGE` and `rpm2cpio FILE | cpio -idm` in
`~/.local/opt/clients` add further programs. The same variables let the
pixel unit tests run.

The end-to-end test starts chora nested in a headless sway. It checks the
scene, transactions, layer shell, the session lock, effects, the safety
bindings and the watchdog. It needs
sway, foot, grim, jq, python3, pgrep and pkill, a Mesa driver, waybar with
dbus-run-session, fuzzel, wlrctl, swaylock and wlr-randr. The Wayland test client
`tests/probe` checks what no packaged client shows: the lock against other
clients and presentation times.

```bash
tests/nested.sh target/debug
```

`tests/xp.sh` checks the XP flavor in the same way: the taskbar, title bars,
the drag of a window, minimize, maximize, close and the start menu. It needs
sway, foot, grim, jq and pgrep.

```bash
tests/xp.sh target/debug
```

`tests/agent.sh` checks the agent interface: grants, captures, input
injection, screencopy, views, assets, reserves, programs and the MCP tools
through `tests/mcp-client.py`. It needs swaylock and python3 in addition.

```bash
tests/agent.sh target/debug
```

`tests/gate-0.9-session-units.sh` checks the units of the session and of
the flavors with `systemd-analyze --user verify`, and the session entries
with `desktop-file-validate`. It needs systemd, desktop-file-utils, waybar,
swaybg, swayidle and kanshi.

A lane passes the output of `tests/cells-for.sh` to the `only` option of `tests/merge-gate.sh`: it prints the cells that the diff against the base touches (the cells that run a changed test script or a lib it sources, the cells that name the binary of a changed crate, every cell for a change to the core, the kit or the scene) and always holds `ci-check`, `pre-commit`, `build`, `test` and `clippy`. `tests/cells-for-selftest.sh` proves it from the `gate-lib` step. The pick is advisory: a plugin that a cell starts only through the session config is not picked, so the train runs the full gate.

`tests/merge-gate.sh` decides whether a merge into the session branch may
be pushed. It runs `red-first`, `identity` (every commit of the range,
merges included, has the author and the committer in `expected_identity`
at the top of the script; each failure names the sha and the identity
found; `--list` prints the step names), build, test, clippy, the 2D guard, `tests/xp.sh`, the
handover matrix, the HUD gate, the doors gate (`tests/gate-doors.sh`: the
shared keys and one visible door per action in every look), the act
contracts (`tests/gate-acts.sh`: acts per look with the invariants of
`tests/acts/invariants.py`, failures known today listed in
`tests/acts/known-red.toml`),
`tests/last-words.sh` and the cube gate
of the construct, one log per step
in a run directory. `tests/gate-watchdog-starved.sh` is not a step: it
loads every CPU for half a minute, so run it alone. A
step fails on a non-zero exit code, on a line that starts with `FAIL:` or
`MISMATCH`, or on `test result: FAILED`. `--only STEP,...` runs some steps
and never pushes; `--push BRANCH` pushes `HEAD` after every step passed
and only from a clean work tree. `--jobs N` runs up to N steps after
2d-guard at once. The gate scripts mostly wait, so `--jobs 3` on four
CPUs keeps their times and cuts the gate from about 27 to about 10
minutes. A gate script judges its programs by pid (`session_pids` of
`tests/gate-lib.sh`), never by name, since the steps beside it start the
same programs. `tests/merge-gate-selftest.sh` proves this with fake steps
and needs no build. One gate runs at a time on a
machine: the merge gate and every script with `tests/gate-lib.sh` take the
lock of `tests/gate-lock.sh`, `${XDG_RUNTIME_DIR:-/tmp}/chora-gate.lock`,
and a second gate prints `waits for` and waits. The scripts that the
gate runs share its lock.

```bash
tests/merge-gate.sh --push claude/session
```

The other gate scripts `tests/gate-*.sh` take the binaries from
`target/debug`, or from the directory in `BIN`. Their EXIT trap ends
every program that they started, also one that a gate stopped with
`SIGSTOP`; `tests/gate-lib-selftest.sh`, the first step of the merge
gate, proves it. The list of the scripts
that CI runs is `.github/workflows/ci.yaml`, and its `dnf install` line
names the packages that they need. A script without its tools answers
`SKIP`. `tests/gate-0.9-daily.sh` and `tests/gate-0.9-hardware.sh` run by
hand inside the chora session on the reference hardware.

Each bug fix comes with a test that fails without the fix (D48).

CI runs `cargo clippy`, `cargo test` and the end-to-end test with `--locked`
in a Fedora 44 container, and `cargo deny` in a separate job. A further job
checks the build with the minimum Rust version. The `check` job runs
gate check 9 of 0.7, and `cargo test` covers check 7. The `e2e` job runs
the gate checks 6, 8 and 10. Two jobs on an Ubuntu runner run the gate checks that
need root or podman. `vkms` starts chora on the tty backend, and
`podman` runs a client in a container. A gate check that needs a user
manager runs by hand on the reference machine.

### Size and cut checks

`tools/size-ratchet` lists the Rust files over 1,500 production lines (the
lines before the `#[cfg(test)] mod` of a file; test files count 0) in
`tools/size-ratchet.list`. A listed file may not have more lines than at
the merge base (the lower of the list and the base counts holds, so a
stale list entry after a split does not block), and a new file over the
limit fails; run `tools/size-ratchet --update` after an extraction to
lower the list. The list is sorted by path and `--update` edits only the
changed lines, so parallel split branches merge cleanly. The test cut
needs the rustfmt form: `#[cfg(test)]` on one line, then `mod x {` on the
next; any other form counts the tests as production lines.

`tools/cut-check` asks whether a plugin could be packaged and moved to
another repository alone. These block: a dependency of one crate under
`plugins/` or `services/` on another (a dev or build dependency too), a
workspace dependency beyond chora-kit, chora-scene and chora-draw, an
`include_str!` or `include_bytes!` path or a link that leaves the crate,
an entry of a `huds` folder other than `neutral` (a look ships its own
costume in its own `hud` folder), a name of another plugin beyond the
count of `tools/cut-known.toml`, and a copy row of `tools/copies.toml` at
`max = 0` that still matches. The counts of `cut-known.toml` only fall:
the tool prints "lower max" when the real count is lower, and a new
breach is fixed, not listed. With `--base REF` it also reports the match
arms on a chora-kit enum that the plugins gained since `REF`, so a port
that starts to act on a shared variant shows before review. Both run in
`ci-check` of the merge gate and in CI, each with a self-test under
`tests/`.

## Demo media

`demo/record.sh OUT_DIR` makes the demo media of `chora-construct`,
each clip as a GIF and an MP4: a walk through a construct, a pane that
wobbles in 3D, the cube turned once around with its windows floating
off the faces, a window pulled off a face of the cube into the space,
and a still of the cube in a turn. `demo/scene-construct.sh` starts the
scene of the first two: a headless session with five panes.
`demo/scene-cube.sh` starts the scene of the others: the cube with two
windows on each of its four faces. `demo/content.py` generates the
content, so no clip holds an asset of a third party. `choractl record`
steps the scene clock by 1/30 s per frame, so the clips show the same
motion on any renderer. In the clips the cube turns in 1.5 s a quarter,
through `cube.turn`, slower than the 0.5 s of its keys and chips, and
the pull-out runs with `--test-injected-press-is-real`, as the core gives
no injected drag away otherwise. `tests/gate-demo.sh` checks the scenes
and the clips. A caption of a clip names the renderer, for example
software GL in a container.

## Releases

A release ends a milestone, once its gate holds (see
[Roadmap](https://github.com/MArpogaus/chora/blob/main/docs/roadmap.md#rules)).

1. On `dev`, `cz bump` sets the version, writes `CHANGELOG.md` and makes a
   tag. Never edit the changelog by hand. `cz bump --prerelease rc --increment MINOR`
   makes a release candidate such as `0.6.0-rc.1`. A later `cz bump` makes
   `0.6.0`.
2. Delete the local tag, because the tag belongs on the merge commit:

   ```bash
   git tag -d vX.Y.Z
   ```

3. Push `dev` and wait for green CI.
4. Merge `dev` into `main` with a merge commit, and push `main`:

   ```bash
   git switch main
   git merge --no-ff dev
   git push origin main
   ```

5. Tag the merge commit with an annotated tag, and push the tag:

   ```bash
   git tag -a vX.Y.Z -m "vX.Y.Z"
   git push --no-verify origin refs/tags/vX.Y.Z
   ```

6. The tag starts `.github/workflows/release.yaml`. The workflow checks
   that the tag is annotated, names the version of `Cargo.toml` and sits on
   a merge commit of `main`. It waits for green CI of that commit. Then it
   builds the RPM with `dist/build-rpm.sh` and the Arch package with
   `dist/build-arch.sh`. It attaches both with their SHA256 sums to a
   GitHub release. Versions below 1.0 and release candidates are
   pre-releases.

From the move to the organization after 1.0, `main` accepts pull
requests only (D47).

## Commits

Use [conventional commits](https://www.conventionalcommits.org/):
`<type>(<scope>): <description>`, with the scope optional. The subject
line has 68 characters or fewer, counting the whole line including
`type(scope): `. Body lines have 72 characters or fewer. Make one logical
change per commit.

## Documentation

The docs are Markdown in `docs/`, built with MkDocs Material:

```bash
uvx --with-requirements docs/requirements.txt mkdocs serve
```

Write in Simplified Technical English:

- sentences of 25 words or fewer
- active voice, one idea per sentence
- one word for one thing, as defined in the glossary
- no em or en dashes, no first person

Keep each fact in one place and link to it.

## Copied code

The core is GPL-3.0-or-later, so it can reuse code from GPL projects such as
Pinnacle and niri.

- Keep the copyright notice of the source file.
- Add one line with the source: `// Adapted from <project>, <path>`.
- Never copy code under GPL-3.0-only. A hook rejects files with a
  GPL-3.0-only header.
