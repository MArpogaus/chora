# Split of the repository

The plan to split this repository into four repositories after 1.0. Nothing
is split now. Section 5 prepares the split on the session branch, so that
section 6 is mechanical. Evidence from 2026-11-09, version `0.9.0-rc.3`.

## 1. Criteria and evidence

**Release coupling.** One version for all members (`Cargo.toml:130`), one
release workflow, one RPM with every binary (`crates/chora/Cargo.toml:37-87`).
The cadences differ already: the core has gated milestones (D46), the XP
showroom has no release per milestone (D39), mc and construct change without
notice (`crates/chora/Cargo.toml:103`), voice has its steps V1 to V3.

**API boundary.** `cargo metadata`: `chora-scene` depends on no member.
`chora`, `choractl` and `chora-kit` depend on `chora-scene` only. Every
plugin and `chora-voice` depend on `chora-kit` and `chora-scene` only. No
plugin depends on another plugin, on `chora` or on `choractl`. The protocol
is versioned (`crates/chora-scene/src/ipc.rs:69`, `PROTOCOL = 1`, no
features) but not frozen before 1.0 (D3).

**Who changes what together.** `git log --name-only`: 477 commits without
merges, 2026-10-13 to 2026-11-09 (older history is on `origin/archive/main`).
Commits per group: docs 157, voice 112, core 88, scene 40, tiling 37,
choractl 35, mc 30, xp 29, construct 19, kit 16. Strong pairs: core and
scene 33, scene and tiling 22, core and tiling 21, core and kit 13. Voice and
mc: below 10. With the split below, 29 of 375 code commits touch two
repositories (8 %), mostly API changes before 1.0 (`44d9727`, `343189d`,
`01959ea`) and chores. Moving `chora-tiling` or `chora-kit` out of the core
raises the crossings with the plugins from 14 to 17.

**Build and CI time.** The core group needs 264 crates. The plugins add 57
(resvg, zbus, chrono). `chora-voice` adds 112 that nothing else needs
(tract-onnx, reqwest, rustls, tokio-tungstenite). Plugins and voice link no
Smithay and no native library of `dist/fedora-build-requires.txt`. CI builds
the whole workspace in every job (`.github/workflows/ci.yaml:32-42,76,101,116`).
`cargo check -p` passes alone for `chora-kit`, `chora-tiling`, `chora-xp`,
`chora-mc`, `chora-construct` and `chora-voice`.

**Packaging.** One RPM and one Arch package (`dist/build-rpm.sh:20`,
`dist/arch/PKGBUILD:47`). D49 asks for a subpackage per plugin;
`cargo generate-rpm` reads its metadata per crate.

**Licenses.** Code is GPL-3.0-or-later. The XP theme adds GPL-3.0-only
artwork (Exposé, modernXP) and GPL-2.0-or-later, LGPL-2.1-or-later, OFL-1.1
and CC-BY-SA-4.0 files (`flavors/xp/theme/README.md`). Without the theme the
core is GPL-3.0-or-later and MIT (`LICENSES/MIT-Smithay.txt`), which fits D8.
The Silero model (MIT) belongs to voice.

**Contributor entry.** Plugin and voice unit tests need no Smithay, GPU
libraries or sway. A smaller repository with its own issues is easier to enter.

## 2. Proposed repositories

All in the organization `chora-desktop` (D36).

**`chora-desktop/chora`: core and API.**
- Paths: `crates/` (chora, chora-scene, chora-kit, choractl),
  `plugins/chora-tiling/`, `flavors/tiling/`, `tests/` without
  `tests/plugins/`, `dist/`, `docs/` without `docs/plugins/` and
  `docs/voice.md`, `examples/`, `plans/`, root files.
- Releases: tag `vX.Y.Z` gives the RPMs `chora` and `chora-tiling`, the Arch
  package, and `chora-scene` and `chora-kit` on crates.io.
- CI: check, coverage, msrv, core e2e with tiling, vkms, podman, deny, the
  schema job of 1.0, docs.
- Docs: the reference site: vision, glossary, scenarios, architecture, API,
  configuration (views, clients, tiling, flavors, sessions, services),
  project structure, dependencies, alternatives, the one decision log, the
  core roadmap.
- Why alone: it defines the API. `chora-tiling` stays as the reference
  plugin and the daily driver of 1.0 (D51); 14 core test scripts and 6 battle
  workflows start it (`tests/nested.sh:57`, `tests/agent.sh:68`). `chora-kit`
  stays because tiling uses it and it changes with `chora-scene`.

**`chora-desktop/chora-plugins`: the desktops beyond the reference.**
- Paths: `plugins/chora-xp/`, `plugins/chora-mc/`, `plugins/chora-construct/`,
  `flavors/xp/` (theme and session files), `flavors/construct/`,
  `tests/plugins/`, `docs/plugins/`. Same layout as today.
- Releases: own tag; RPMs `chora-xp`, `chora-mc`, `chora-construct`, each
  with `Requires: chora >= 1.0`.
- CI: unit tests on a plain runner; e2e that builds `chora`, `choractl`,
  `chora-tiling` and `chora-probe` from the core tag in `core.version`; a
  nightly job against core `main`.
- Docs: configuration of XP, mc and construct, both style guides, the XP
  split design (X1), the XP, split and construct tracks.
- Why alone: a showroom and experiments with their own pace, and all
  non-code licenses. The XP split blocks (D73) and later plugins land here,
  so there is no repository per plugin.

**`chora-desktop/chora-voice`: the voice agent.**
- Paths: `services/voice/`, `docs/voice.md`, `ruff.toml`.
- Releases: own tag; RPM `chora-voice` with unit, quadlets and PipeWire
  file, `Requires: choractl`.
- CI: unit tests, ruff, e2e with the core tag in `core.version`.
- Docs: `docs/voice.md`, `services/voice/README.md`.
- Why alone: a product of its own (D53), the largest dependency tree,
  Python tooling, a model under MIT. It reaches mc only through client
  events (D65).

**`chora-desktop/chora-website`: the website.**
- Content: the five USPs, captures, getting started (COPR, choose a
  session), contributing, links to the three reference sites.
- Now: the orphan branch `claude/website` (charter section 9).
- CI: lychee, site build, GitHub Pages.
- Why alone: no code, no shared history, changes with releases.

The organization also gets `.github` (profile, issue templates). There is no
`chora-packaging`: each repository builds into the one COPR project.

## 3. Shared code and versioning

- `chora-scene` and `chora-kit` go to crates.io with 1.0. Both names are
  free; `chora` is taken by an unrelated crate (checked 2026-11-09). Both
  depend on crates.io crates only. `chora` needs Smithay from git and
  stays unpublished, as `choractl`.
- The major version of `chora-scene` is the protocol version. An addition
  is a feature in `FEATURES` and a minor release. A break is protocol 2 and
  `chora-scene` 2.0. `chora-kit` gets its own semver at the split.
- `chora-plugins` and `chora-voice` use `chora-scene = "1"` and
  `chora-kit = "1"`. Their e2e tests pin the core tag in `core.version`.
- Fallback if crates.io is declined: `chora-scene = { git =
  "https://github.com/chora-desktop/chora", tag = "v1.0.0" }`.
- Must hold before the split: `v1.0.0` is tagged; the committed JSON schema
  and its CI job exist (roadmap 1.0); every plugin speaks protocol 1 with a
  CI check (1.0 gate 5); `voice.state`, `voice.confirm` and
  `voice.confirm.answer` have an example each in `docs/voice.md`, and
  `tests/fake-voice.py` follows them.

## 4. Coupling that blocks a clean split today

- Manifests: `Cargo.toml:62` members include `plugins/*` and
  `services/voice`; `:68-69` path dependencies without version.
- Core RPM: `crates/chora/Cargo.toml:41-45,50-59,61-65,78-87` ships plugin and
  voice binaries, the XP theme, mc collectors, voice files and XP and
  construct session entries; `:46` the mc font file; `:51` the Silero
  license; `:93` one license field. Same in `dist/build-rpm.sh:14-17` and
  `dist/arch/PKGBUILD:47,61-71,80-92`.
- Tests: `tests/gate-0.7-10.sh:13,23` runs battle W2, which starts chora-xp
  (`tests/battle/w2.sh:7`); `tests/gate-0.9-session-units.sh:14,16,23` needs
  chora-xp and all flavor units; `tests/settings-docs.sh:28-43` checks every
  plugin and voice; `tests/battle/lib.sh:26,188` name xp and mc;
  `tests/voice-events.sh:41,54`, `tests/voice-acp.sh:48,62` and
  `tests/battle/w16.sh` start chora-mc with chora-voice;
  `tests/gate-handover.sh:33` and `tests/gate-construct-*.sh` need tiling.
  Every e2e script uses `tests/gate-lib.sh` or `tests/headless-session.sh`
  and `chora-probe`.
- CI: `ci.yaml:116` builds the workspace for e2e; `:126,167-170` run XP tests
  in the core job; `:131` the battle gate; `release.yaml:96,125` one package.
- Docs: `docs/configuration.md:277` (mc), `:632` (construct), `:1055` (XP)
  are plugin sections; `:443,685,875` link to `mc-style.md`,
  `construct-style.md`, `voice.md`; `mkdocs.yml:30-33` lists them;
  `mkdocs.yml:3-4`, `Cargo.toml:128`, `README.md:24,133,143` name
  `MArpogaus/chora`. `crates/choractl/src/docs.rs:17-20` and `mcp.rs:1037`
  embed `docs/api.md` and `docs/configuration.md` in tests; all 12 parsed
  examples are in core sections, so this holds.
- Decisions: `docs/decisions.md:65` (D36) plans a repository per plugin and
  `chora-packaging`; `docs/project-structure.md:7-8`, `docs/roadmap.md:580-582`.
- Tooling: `.pre-commit-config.yaml:45-47,54` (ruff, voice model), `:64,67`
  (XP theme), `:84` (session scripts); `CONTRIBUTING.md:29,175`.
- Holds if the layout stays: `plugins/chora-xp/src/main.rs:129`,
  `theme.rs:804`, `cascade.rs:313`, `dialog.rs:1043`, `text.rs:373,392` use
  `../../flavors/xp/theme`. Runtime: `flavors/xp/chora-xp-xwayland.service`
  runs `/usr/libexec/chora/chora-xwayland` of the core, so `Requires: chora`.
- No source file names a crate of another group; `crates/chora/src/backend/winit.rs:346`
  and `crates/chora-kit/src/settings.rs:658` are a test string and a comment.

## 5. Preparation on the session branch

Each step is small, passes the local checks, and changes nothing for users
unless marked.

1. **Group lists.** Add `tools/split/plugins.paths`, `voice.paths`,
   `shared.paths` (section 6). `tools/split/check-paths.sh` fails when a
   tracked file is in two groups. Test: it passes.
2. **Each group alone.** `tools/split/test-groups.sh` runs `cargo test` with
   `-p chora -p chora-scene -p chora-kit -p choractl -p chora-tiling -p chora-probe`,
   then `-p chora-xp -p chora-mc -p chora-construct`, then `-p chora-voice`.
   Fix any reliance on feature unification. Test: three green runs.
3. **Versioned path dependencies.** Add `version = "0.9.0-rc.3"` to the
   `chora-scene` and `chora-kit` entries. Test: `cargo metadata` is the same
   graph.
4. **Test library by variable.** Plugin and voice scripts source
   `${CHORA_TESTLIB:-$repo/tests}/gate-lib.sh` and `headless-session.sh`.
   Test: `tests/xp.sh` passes with and without the variable.
5. **Tests to their group.** `tests/xp.sh`, `gate-0.8-xp-*.sh`,
   `gate-construct-*.sh`, `gate-handover.sh`, `fake-voice.py` and battle W2,
   W3, W10 to W15 go to `tests/plugins/`. `voice-acp.sh`, `voice-events.sh`,
   `w16.sh`, `w16-fakes.py` go to `services/voice/tests/`. CI and
   `CONTRIBUTING.md` follow. Test: the same CI steps are green.
6. **Cut the crossings.** `gate-0.7-10.sh` runs W1; the plugins job runs W2.
   `gate-0.9-session-units.sh` and `settings-docs.sh` take a group.
   `battle/lib.sh:26,188` take the plugin from a variable. The voice tests
   watch `voice.*` with `choractl events` instead of chora-mc; mc stays
   covered by `fake-voice.py`. Test: each script runs with its group and the
   core binaries only.
7. **Files to their group.** Session entries go to `flavors/xp/session/`
   and `flavors/construct/session/` (narrow the RPM glob `flavors/construct/*`
   to `chora-construct*`); `LICENSES/MIT-silero-vad.txt` goes to
   `services/voice/LICENSES/`. Test: `rpm -qlp` is unchanged.
8. **A package per crate (D49).** Move the assets of tiling, xp, mc,
   construct and voice into `[package.metadata.generate-rpm]` of their crate,
   each with its license field. `build-rpm.sh` builds six RPMs; `chora`
   requires the other five until the split. The PKGBUILD becomes a split
   package. Test: the union of file lists equals the old list; CI installs
   all six. User-visible: new package names, the same files.
9. **Docs per group.** Move the mc, construct and XP sections of
   `configuration.md`, both style guides and the XP, split and construct
   tracks to `docs/plugins/`. One site with the sections Core, Plugins,
   Voice. Test: `mkdocs build --strict`, lychee, `settings-docs.sh`.
   User-visible: some page URLs of the site.
10. **One CI job set per group.** `core`, `plugins`, `voice`; each builds
    its group with `-p` plus the core binaries it starts. Test: no foreign
    crate in the build lines; all green.
11. **Stage the new roots.** `tools/split/plugins/` and `tools/split/voice/`
    hold `Cargo.toml`, `core.version`, `README.md`, `mkdocs.yml`,
    `.github/workflows/`, `.packit.yaml`, `dist/build-rpm.sh` and a
    `CHANGELOG.md` that points to the core changelog. Test: step 12.
12. **Dry run.** `tools/split/extract.sh GROUP OUTDIR` clones with
    `--no-local`, runs `git filter-repo --paths-from-file`, copies the staged
    root, points `chora-scene` and `chora-kit` to paths in an extracted core,
    builds, tests, runs pre-commit; weekly in CI. Test: green, and
    `git log --follow tests/plugins/xp.sh` reaches its first commit.
13. **Website** on the orphan branch `claude/website`, any time. Test:
    lychee and the build.
14. **Decision draft for the owner.** Replace D36 with this split; update
    `project-structure.md`. The owner decides.

## 6. The split procedure after 1.0

Preconditions: section 3 holds, steps 1 to 12 are done, the dry run was
green for a week, the owner accepted step 14.

`tools/split/shared.paths` is kept in every repository, one path per line:
`LICENSE`, `CONTRIBUTING.md`, `.config/`, `.cz.toml`, `.gitignore`,
`.pre-commit-config.yaml`, `.shellcheckrc`, `.vale/`, `.vale.ini`,
`_typos.toml`, `clippy.toml`, `deny.toml`, `lychee.toml`, `rust-toolchain.toml`.

`tools/split/plugins.paths` (rename lines keep the history before steps 5,
7 and 9):

```text
plugins/chora-xp/
plugins/chora-mc/
plugins/chora-construct/
flavors/xp/
flavors/construct/
tests/plugins/
docs/plugins/
tests/xp.sh==>tests/plugins/xp.sh
tests/gate-0.8-xp-settings.sh==>tests/plugins/gate-0.8-xp-settings.sh
tests/gate-0.8-xp-token.sh==>tests/plugins/gate-0.8-xp-token.sh
tests/gate-handover.sh==>tests/plugins/gate-handover.sh
tests/fake-voice.py==>tests/plugins/fake-voice.py
regex:^tests/(gate-construct-[a-z0-9]+\.sh)$==>tests/plugins/\1
regex:^tests/battle/(w(2|3|1[0-5])\.sh)$==>tests/plugins/battle/\1
regex:^dist/session/(chora-xp(-uwsm)?\.desktop)$==>flavors/xp/session/\1
regex:^dist/session/(chora-construct(-uwsm)?\.desktop)$==>flavors/construct/session/\1
docs/mc-style.md==>docs/plugins/mc-style.md
docs/construct-style.md==>docs/plugins/construct-style.md
```

`tools/split/voice.paths`:

```text
services/voice/
docs/voice.md
ruff.toml
LICENSES/MIT-silero-vad.txt==>services/voice/LICENSES/MIT-silero-vad.txt
tests/voice-acp.sh==>services/voice/tests/voice-acp.sh
tests/voice-events.sh==>services/voice/tests/voice-events.sh
tests/battle/w16.sh==>services/voice/tests/w16.sh
tests/battle/w16-fakes.py==>services/voice/tests/w16-fakes.py
```

The dry run checks that a rename line keeps its old path; if not, the old
path gets a line of its own.

1. Create `chora-desktop` if it does not exist. Transfer `MArpogaus/chora`
   to `chora-desktop/chora`; GitHub redirects old URLs and keeps issues,
   pull requests, releases and stars.
2. For `plugins` and `voice`, from a core checkout at the tag:
   `cat tools/split/GROUP.paths tools/split/shared.paths >/tmp/GROUP.paths`,
   `git clone --branch main https://github.com/chora-desktop/chora chora-GROUP`,
   then in it `git filter-repo --paths-from-file /tmp/GROUP.paths --tag-rename v:core-v`.
   Copy `tools/split/GROUP/` of the core checkout into the root; commit
   `chore: split from chora at SHA`. Push `main` and `dev` to the new
   repository and protect `main` (D47).
3. In the core, one commit on `dev` runs `git rm -r` on the paths of both
   lists, sets `members = ["crates/*", "plugins/chora-tiling", "tests/probe"]`,
   and drops the moved assets and the `Requires` of step 8; `chora` then
   recommends `chora-tiling`. Core hashes stay, so links stay valid.
4. Publish `chora-scene` and `chora-kit` 1.0.0; switch the new repositories
   to `"1"`; tag `v1.0.0` in each after green CI.
5. Issues: label `area:plugins` and `area:voice` before the split, then
   `gh issue transfer N chora-desktop/chora-plugins` (or `chora-voice`).
   Gate issues stay in the core.
6. Releases: existing releases and tags stay in the core. The new
   repositories keep old tags only as `core-v*` and start at `v1.0.0`. The
   tag `archive/xp-scene-programs` stays in both.
7. Packaging: a `.packit.yaml` per repository builds into the COPR project
   `chora-desktop/chora`. `secureblue-images` installs the packages it wants.
8. Docs: each repository publishes with mike on its own `gh-pages`. Push
   `claude/website` as `main` of the website repository. Rewrite the cross
   links of section 4 with `tools/split/links.sed`.

## 7. Risks and open questions

1. D36 plans a repository per plugin and `chora-packaging`. Default: amend
   D36 to these four repositories, packaging per repository.
2. Home of `chora-tiling`. Default: core, as reference plugin and window
   manager of the core tests.
3. Home of `chora-kit`. Default: core, with its own version; move it only if
   it stops changing with `chora-scene`.
4. crates.io or git tags. Default: crates.io for `chora-scene` and
   `chora-kit` from 1.0; git tags if the owner declines a public crate.
5. Core changes break plugins late. Default: a nightly job of
   `chora-plugins` and `chora-voice` against core `main` opens an issue.
6. The staged roots drift. Default: the weekly dry run of step 12.
7. Two Rust copies of the voice events (`services/voice/src/mc.rs`,
   `plugins/chora-mc/src/agent.rs`). Default: document them and test both
   sides with fakes; shared types in `chora-kit` only after two breaks.
8. The battle suite spans every plugin. Default: split by group (step 5); a
   full cross run stays a manual round on the reference hardware.
9. Decision log and roadmap. Default: one decision log in the core, so
   numbers stay unique; each repository keeps the roadmap of its tracks.
10. Website repository name. Default: `chora-desktop.github.io`, the
    organization root without a custom domain; `chora-website` with a domain.
11. Rewritten hashes in the new repositories. Default: the first commit
    names the source hash; old tags stay as `core-v*`.
12. The organization name can be taken before 1.0. Default: the owner
    creates the empty organization now.
13. Package names change in step 8. Default: `chora` requires the
    subpackages until the split, so an update installs the same files.
