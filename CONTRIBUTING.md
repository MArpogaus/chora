# Contributing

## Branches

- `main` holds the root commit `chore: init` and merge commits. Never
  commit to `main` directly.
- In a repository with a single author, `dev` is the working branch.
  Pull requests target `dev`.
- In a repository with several contributors, each contributor works on
  `dev-NAME`. Pull requests from `dev-NAME` target a protected `main` (D47).

## Hooks

Install the hooks once in every fresh clone:

```bash
pre-commit install --install-hooks -t pre-commit -t commit-msg
```

Plain `pre-commit install` activates the `pre-commit` stage only. Check that
`.git/hooks/` contains `pre-commit` and `commit-msg`.

The hooks run:

- `cargo fmt` and `cargo clippy` with `-D warnings`. `clippy.toml` sets the
  complexity guards. Split a function instead of raising a limit.
- `ruff check` and `ruff format` on `services/voice/`. `ruff.toml`
  selects the rules and the complexity guards.
- file hygiene, YAML and TOML formatting, and a spell check
- `actionlint` on the workflows
- `commitizen` on every commit message

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

The unit tests of the core run a core without a backend in the test
process. `crates/chora/src/testing.rs` gives them IPC clients on socket
pairs, a Wayland client, outputs, and a time that the test moves forward.
Pixel tests render with the software driver of Mesa, from
`mesa-dri-drivers` on Fedora.

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
mako, swaybg, swayidle and kanshi.

The other gate scripts `tests/gate-*.sh` take the binaries from
`target/debug`, or from the directory in `BIN`. The list of the scripts
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
`<type>: <description>`, with the description at 68 characters or fewer.
Make one logical change per commit.

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
