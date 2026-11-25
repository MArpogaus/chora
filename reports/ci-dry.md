
## PM review r1 (2026-11-25 19:58 UTC; 2 files, +20, under 200 lines)
Base 62a04be0, head e36b6078. tools/mkdocs_hooks.py replaces the README include on index.md only, rewrites `](docs/img/` to `](img/`; mkdocs.yml loads it; strict build stays on. The failing strict build is the red evidence (CI-only job, docs workflow on push to main/dev). No product code, no sleep, no lint allow.
Findings: none BLOCK or MAJOR. rcd1-1 MINOR: the docs workflow does not run on the PR, so a broken site build shows only after the release merge; backlog (run `mkdocs build --strict` in pre-commit or the gate).
NOT RUN at release: vkms, rpm, arch, mike --push; SKIP-only: six gates (missing tools here). They run first in CI on the release PR (rc 4).
Verdict (computed): MERGE-OK, joins the next train.
