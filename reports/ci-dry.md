# ci-dry

## r1

Base 62a04be03 (session head, t140 green). Head e36b60781 on claude/ci-dry-r1.
Diffstat: mkdocs.yml +2, tools/mkdocs_hooks.py +18 (2 files, 20 insertions).
Run in the cloud container with the toolchains of the repo, not Fedora 44, so
the system libraries and the container options differ from CI. All steps ran
locally in the order of the workflows; install steps and `uses:` actions
skipped. Seconds are wall time.

| job | step | command | result | s |
|---|---|---|---|---|
| check | clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS | 39 |
| check | agent sandbox | `tests/agent-sandbox.sh` | PASS | 0 |
| check | nextest | `cargo nextest run --workspace --locked --profile ci` (cargo-nextest 0.9.152 installed with cargo install --locked) | PASS 2996/2996 (10 leaky) | 348 |
| check | doctests | `cargo test --doc --workspace --locked` | PASS | 100 |
| check | rustdoc | `cargo doc --workspace --no-deps --locked` | PASS | 5 |
| check | gate-0.7-9 / 0.8-5 / 0.8-8 / 0.8-plugin-runtime | `tests/gate-*.sh` | PASS | 87 / 1 / 1 / 1 |
| check | packaging-files, flavor-notify | `tests/*.py` | PASS | 0 / 0 |
| check | screenshot-notice, tiling-scripts-selftest | `tests/*.sh` | PASS | 0 / 13 |
| check | size, ratchet-raise, lint, sleep, cut ratchets and selftests | `tools/*-ratchet`, `tools/cut-check` and selftests | PASS | 1 / 1 / 0 / 1 / 3 |
| coverage | llvm-cov nextest, lcov, json, summary | `cargo llvm-cov nextest ... --no-report`, `report`, `tests/coverage-summary.sh` (llvm-tools added with rustup) | PASS | 259 / 4 / 3 / 0 |
| msrv | check 1.89 | `cargo +1.89 check --workspace --all-targets --locked` | PASS | 305 |
| e2e | build | `cargo build --workspace --locked` | PASS | 28 |
| e2e | settings-docs, session-units, shader-hang, input | `tests/settings-docs.sh`, `tests/gate-0.9-*.sh` | PASS (session-units: SKIP desktop-file-validate missing, passes with SKIP) | 0 / 0 / 12 / 1 |
| e2e | nested, xp, agent | `tests/nested.sh`, `tests/xp.sh`, `tests/agent.sh` | PASS | 40 / 96 / 36 |
| e2e | gate-0.7-6 | `tests/gate-0.7-6.sh` | PASS as SKIP: xkeyboard-config here has no fkeys option | 1 |
| e2e | gate-0.7-10, 0.7-8, e2e-fonts | `tests/*.sh` | PASS | 45 / 2 / 0 |
| e2e | gate-0.8-1 .. 0.8-10 (1,2,3,4,6,7,9,10) | `tests/gate-0.8-*.sh` | PASS | 4/1/7/4/1/3/1/1 |
| e2e | tiling-style, tiling-look, tiling-tags | `tests/gate-*.sh` | PASS | 1 / 2 / 2 |
| e2e | tiling-apps, apps-click, bar-early-signal | `tests/gate-tiling-*.sh` | PASS | 14 / 6 / 2 |
| e2e | gate-0.9-screencast | `tests/gate-0.9-screencast.sh` | PASS as SKIP: pipewire missing here | 1 |
| e2e | gate-0.9-x11 | `tests/gate-0.9-x11.sh` | PASS as SKIP: xwayland-satellite missing here | 1 |
| e2e | gate-0.8-xp-token | `tests/gate-0.8-xp-token.sh e2e/xp` | PASS as SKIP: thunar missing here | 1 |
| e2e | gate-0.8-xp-settings | `tests/gate-0.8-xp-settings.sh` | PASS | 5 |
| podman | build, gate-0.7-4 | `cargo build --locked -p chora -p choractl && tests/gate-0.7-4.sh` | PASS as SKIP: podman missing here (build ran) | 63 |
| vkms | build, modprobe, seatd, gate-0.7-3, vkms-hotplug | `sudo modprobe vkms; seatd; tests/gate-0.7-3.sh` | NOT RUN: no /dev/dri, no vkms, no seatd here, no system packages | - |
| deny | cargo-deny action | `cargo deny check` | PASS (advisories, bans, licenses, sources ok; warning: unmatched license allowance CC0-1.0) | 4 |
| deny | cargo machete | `cargo machete` | PASS | 0 |
| pre-commit | pre-commit (SKIP=cargo-clippy,lychee,vale) | `pre-commit run --all-files` | PASS | 164 |
| pre-commit | lychee | `lychee --config lychee.toml docs README.md plans` | PASS (119 OK, 0 errors, 43 excluded) | 0 |
| pre-commit | vale | `vale --minAlertLevel=error docs` | PASS (0 errors in 23 files) | 1 |
| docs | mike deploy (build part) | `mkdocs build` strict | FAIL, then fixed: `WARNING - Doc file 'index.md' contains a link 'docs/img/xp.png', but the target is not found` (4 warnings, strict) | 4 |
| docs | rerun after fix e36b60781 | `mkdocs build -d /tmp/site-dry` | PASS | 1 |
| docs | mike deploy --push, set-default | git push to gh-pages | NOT RUN: pushes the site branch | - |
| packages | rpm: cargo install cargo-generate-rpm@0.21.0 | `cargo install --locked` | PASS | - |
| packages | rpm: build-rpm.sh | `dist/build-rpm.sh target/rpm` | NOT RUN: `rpm: command not found` (release build itself passed, 8m48s; the script stops at `rpm --eval`) | 529 |
| packages | rpm: package-deps, install once | `tests/gate-package-deps.sh --rpm`, `dnf install` | NOT RUN: no rpm file, no dnf | - |
| packages | arch | `makepkg` in the Arch container | NOT RUN: makepkg and pacman missing | - |

Counts: PASS 43 steps (6 of them pass as a SKIP of the script), FAIL 1 (fixed),
NOT RUN 6. Every other run step of the four workflows is in the table.

## Fix

- e36b60781 fix(docs): docs/index.md includes README.md (pymdownx snippets); its
  images link docs/img/... from the repo root, so the strict mkdocs build, which
  the docs workflow runs through `mike deploy`, aborted with 4 warnings. A hook
  (tools/mkdocs_hooks.py, `hooks:` in mkdocs.yml) includes the README with the
  site paths. The images load in the built site. No Rust change, so no gate run.

## For the PM to decide

- The docs workflow runs only on push to main, dev, dev-*, not on the pull
  request; this red would have shown only after the merge. The fix is on
  claude/ci-dry-r1.
- Six cells pass as a SKIP here only because tools are missing (desktop-file-validate,
  pipewire, xwayland-satellite, thunar, podman, xkeyboard-config fkeys). CI
  has them: those gates run for the first time on the pull request. Same for
  vkms, rpm and arch.
- cargo-deny prints "unmatched license allowance CC0-1.0" (a warning, exit 0).
- Not changed: release.yaml was out of scope.
