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

Each bug fix comes with a test that fails without the fix (D48).

CI runs `cargo clippy`, `cargo test` and the end-to-end test with `--locked`
in a Fedora 44 container, and `cargo deny` in a separate job. A further job
checks the build with the minimum Rust version.

## Releases

A release ends a milestone, once its gate holds (see
[Roadmap](https://github.com/MArpogaus/chora/blob/main/docs/roadmap.md#rules)).

1. On `dev`, `cz bump` sets the version, writes `CHANGELOG.md` and makes a
   tag. Never edit the changelog by hand.
2. Delete the local tag, because the tag belongs on the merge commit:

   ```bash
   git tag -d vX.Y.Z
   ```

3. Push `dev` and wait for green CI.
4. Merge `dev` into `main` with a merge commit. Tag the merge commit with an
   annotated tag. Push the tag first, so that the release finds it, then
   `main`:

   ```bash
   git switch main
   git merge --no-ff dev
   git tag -a vX.Y.Z -m "vX.Y.Z"
   git push --no-verify origin refs/tags/vX.Y.Z
   git push origin main
   ```

5. A new version in `Cargo.toml` on `main` starts
   `.github/workflows/release.yaml`. It builds the RPM with
   `dist/build-rpm.sh` and the Arch package with `dist/build-arch.sh`, and
   attaches both with their SHA256 sums to a GitHub release. Versions below
   1.0 are pre-releases.

Milestone 0.6 makes the tag start the release and lets the release wait for
green CI of the same commit (D47). Then `main` accepts pull requests only.

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
