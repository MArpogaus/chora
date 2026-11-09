# Guards

Every rule with the check that enforces it. Status: **guarded** (runs on
its own), **to build** (a guard is possible and planned), **manual**
(no automatic check possible; reason given). Inventory of 2026-11-09.

| Rule | Guard | Status |
|---|---|---|
| No attribution lines in commits | `no-agent-trailers` commit-msg hook (code branches and session log) | guarded |
| Conventional commit type | `commitizen` commit-msg hook | guarded |
| Commit subject at most 68 characters, body at 72 | commit-msg hook | to build |
| Format, lints, complexity | `cargo fmt`, `cargo clippy -D warnings` hooks | guarded |
| Shell scripts lint | `shellcheck` hook | guarded |
| Docs prose style | `vale` hook on `docs/*.md` | guarded |
| Links in Markdown | `lychee` hook (installed 11-09) | guarded |
| Spelling | `typos` hook | guarded |
| No GPL-3.0-only code in the core | local hook | guarded |
| No inline comments in code bodies (added lines) | pre-commit hook over the staged diff of Rust, shell and Python files | to build |
| No session files on code branches (`plans/autonomy/`, reports, logs) | pre-commit hook that rejects those paths on code branches | to build |
| No dashes as punctuation in docs | vale rule | to build (check whether `.vale/` has it) |
| Hooks installed in every clone | run start step 2 checks `.git/hooks` | guarded by the run start |
| Never `--no-verify` | CI pre-commit job on every push of `dev` and `main`; locally not checkable | manual until CI runs on the branch |
| Tests pass, docs build, deny, machete | CI jobs; locally the gate of charter section 3 | guarded in CI, manual on `claude/*` (CI does not run there) |
| 2D unchanged by 3D work | `tests/2d-guard.sh` | guarded when run; to build: run it in a hook or CI job on renderer changes |
| A new test fails without the fix | review evidence | manual (needs judgement of what "the fix" is) |
| Stage files by name, no `git add -A` | none possible in git | manual |
| Review round when due (weighted size, critical paths, age) | `review_due.py` at run start | guarded (suggests; the round itself is judgement) |
| Clean review round before a merge or tag | the merge step of the PM | manual (a review is judgement) |
| Budget rules | `tokens.py` estimate at run start | guarded by the run start |
| Never push to `dev` or `main` | branch protection on GitHub (D47, after 1.0) | manual until protection exists |
| English replies, no overselling in reports | daily reflection | manual |
| Code duplicated across plugins | a copy-paste detector over `plugins/` (for example `jscpd`) in the weekly round | to build |
| POLS and KISS in every implementation | a view of every review round | manual (judgement) |
| Split review once a plugin or service reaches 5,000 lines | `review_due.py` (plugin sizes) at run start | guarded |
