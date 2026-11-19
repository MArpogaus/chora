# docs-f8b r1 (F8)

Branch claude/docs-f8b, commit 2d0d0da1 on a17e7858. Gate steps pre-commit and gate-lib green; lychee offline ok.
- docs/decisions.md D65: amendment line (look.theme, look.view; core stamps data.look; subscriber trusts it only on those two names; rw-1 open).
- D104: reviewer wording (wrapper subshell, status file, exit 0 waits 30 s, a sh beside each app).
- Not done on purpose: kit::launch, cut2 and kit::focus are NOT on the session branch (claude/kit6, claude/cut2 unmerged), so no file:line claims, and both records say "merge pending". D99 (book), D103 (cut gate) already exist, so no new records.
- CONTRIBUTING.md: where shared code lives and copies.toml already existed; added a cut gate paragraph (D103). dev-rules.md lives on session-log, not in the repo: unchanged.
- Follow-up when kit6 and cut2 merge: set D104 and D65 status to the merge commits.

## PM check r1 (2026-11-19, PM is not the builder; diff a17e7858..2d0d0da1, 2 files)
- rw-1 MAJOR docs/decisions.md D104: "The wrapper runs the command in a subshell, so `exit N` and a failed
  `exec` still write the status" is false on claude/kit6 (crates/chora-kit/src/launch.rs:202 uses `{ }`; kit6
  review rw-1 open, KIT2c fixes it). Fix: state today's form and name the open point (kit6 rw-1, KIT2c), as D65
  does for its open points.
- rw-2 NIT D104 leaves out "the status file stays" of the review wording; add it.
- D65 amendment, CONTRIBUTING cut gate paragraph: match cut2 review and D103; ok.
Verdict (computed): FIX (rw-1 MAJOR open).
