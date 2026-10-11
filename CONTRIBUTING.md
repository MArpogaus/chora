# Contributing

## Branches

- `main` holds the root commit `chore: init` and merge commits from `dev`.
  Never commit to `main` directly.
- `dev` is the working branch. Pull requests target `dev`.

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
their equivalents on other distributions.

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
```

The end-to-end test starts chora nested in a headless sway and checks the
exit of milestone M1. It needs sway, foot, grim and jq:

```bash
tests/nested.sh target/debug
```

CI runs `cargo clippy`, `cargo test` and the end-to-end test with `--locked`
in a Fedora 43 container, and `cargo deny` in a separate job.

## Releases

`cz bump` sets the version, writes `CHANGELOG.md` and creates the annotated
tag. A pushed `v*` tag starts `.github/workflows/release.yaml`. The workflow
builds the RPM with `dist/build-rpm.sh` and attaches it with its SHA256 sums
to a GitHub release. Versions below 1.0 are pre-releases.

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
