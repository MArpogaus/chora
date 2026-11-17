# Rules for developers of the chora session (read all before starting)

Also binding: /home/user/wt/log/conventions.md (the skills of agent_skills dev and
the session overrides). Read it and the skills it names once per container.

- Work only in your own worktree and on your own branch. Never touch
  /home/user/chora, other worktrees or the branch claude/session-log.
- **Design.** design/looks/ (one file per look) and design/product-briefs.md in
  /home/user/wt/log are the spec, and docs/design-principles.md holds
  the principles (minimal, flawless, flexible). If you find a gap or a
  contradiction, say so in your report; do not invent around it.
- **Tests fail first.** Write the named tests first, run them and record
  the red output, then implement. Report both the red and the green
  evidence.
- **Code.** No inline comments (`//` inside function bodies, `#` in
  shell or Python): names explain the code. Doc comments (`///`, `//!`),
  usage headers and directives are allowed. Keep it simple: no
  over-engineering. Read CONTRIBUTING.md once.
- **Docs.** Docs change in the same branch as the code. Plain technical
  English; no stories, anecdotes or decisions logs (the PM keeps those
  elsewhere). Keep the glossary and the mkdocs nav in sync.
- **Commits.** Conventional type (`feat`, `fix`, `test`, `docs`,
  `refactor`), an imperative subject of at most 68 characters, body
  lines wrapped at 72, one reason per commit. Stage files by name. Never
  add Co-Authored-By, Claude-Session or "Generated with" lines. The
  author is set in the repo config; check that `git config user.email`
  prints 38564291+MArpogaus@users.noreply.github.com. Never use
  `--no-verify` and never skip a hook. If a hook tool is missing, stop
  and report it.
- **Build and disk.** Each worktree builds into its own `target/`.
  Before a build, check `df -h /home/user`; below 8 GB free, stop and
  report. Kill test sessions by pid, never with `pkill -f`.
- **Gate.** Run `tests/merge-gate.sh` from the worktree root (it builds,
  tests, runs clippy, the 2D guard, xp.sh, the handover matrix and the
  HUD gate, each with its own log, and stops at the first failure; see
  its `--help`). Only when it prints that every step passed, push with
  `git push -u origin <your branch>`. Do not merge; the PM merges with
  `tests/merge-gate.sh --push`.
- **Waiting.** Never end a turn to wait for a background job longer than 30 minutes (it
  can be stopped without a wake); run long gates in the foreground or in parts with
  --only. End your last turn only after the report is sent.
- **Restarts.** The container can restart and stop you. Commit and push
  work in progress to your branch at least every hour, through the hooks.
- **Report.** At most 25 lines: the commits, the red and green evidence,
  what you left open, and any deviation from the spec with the reason.

- **11-11 additions.** Gate your branch with `tests/merge-gate.sh --fast`
  once it exists (until then the full gate), and never while another gate
  runs (`pgrep -f tests/merge-gate.sh`). A fix without a red commit is
  sent back. Every new act gets a real-press cell and a
  keyboard-after-the-act cell; every gate script ends with a window
  audit (no window lost). Every refusal or failure shows an honest state
  to the person, never only a log line. Never `--no-verify`.

- **Self-check before you report (11-15, from the review findings of 11-14/15).**
  Go through this list on your diff; each line was a finding more than once.
  1. State: every flag, grab, timer or mode you set is cleared on every exit path
     (done, cancel, error, the client gone); name the test that shows it.
  2. Skips: a SKIP, a missing tool or an `#[ignore]` is not a pass; say which
     checks did not run and why.
  3. Commit types: `fix`/`feat` only with a test in or before it (the commit-msg
     hook checks it); a gate or CI change is `ci`, a test only `test`.
  4. CHANGELOG: never by hand; `cz bump` writes it in the release.
  5. Docs in step: docs, glossary and mkdocs nav change in the same branch.
  6. CI-only checks: run the CI scripts your change touches locally
     (`tests/ci-local.sh`, the grep gates of `.github/workflows/ci.yaml`) before
     you report, not only the merge gate.

- **Refactor proposals (owner 11-17 04:3x).** Every worker report and every review ends with a
  section "Refactor proposals": long-term refactors and code optimisations you saw (duplication,
  a module too large, a weak mechanism, dead code, a missing abstraction in chora-kit), each with
  file:line, the gain and a size S/M/L. Do not build them in your round unless the brief says so;
  the PM ranks them in backlog.md. Code gets leaner and tidier with every release.
  Check the cut (owner 11-17): a plugin uses only chora-kit, chora-scene, chora-draw and the API,
  ships only its own assets and could be packaged and moved to another repo alone; name each breach.
