# Conventions of the session (owner 11-14 17:02 UTC: agent_skills dev plus our overrides)

Source: repo MArpogaus/agent_skills, branch `dev` (read at 53d7041). setup.sh puts it at
/home/user/agent_skills; read a skill with `git -C /home/user/agent_skills show origin/dev:skills/<name>/SKILL.md`
(and its reference.md where the skill says so). PM and every worker read the skills below once per
container; this file lists only where the session differs. Order: decisions-now.md, rules.md,
this file, the skills, CONTRIBUTING.md.

## Skills that apply
- `dev-conventions`: all of it, with the overrides below.
- `git-conventions`: all of it, with the overrides below.
- `rust-dev`: all of it (chora is a Rust workspace). No overrides.
- `sandbox-env`: for tools a lane adds itself; see override 9.
- `session-finish`: at the end of the session (11-30) and before a long stop; see override 4.
- Not used: `python-dev` (only helper scripts are Python; ruff runs through pre-commit), `gtd`,
  `ntfy-channel` (no channel named), the others (other domains).

## Overrides (agreed; each with its source)
1. Branches (11-09 04:24): code goes to `claude/autonomous-dev-session-3w-i33pg7` and `claude/<lane>`
   branches, not to `dev`. Session files only on `claude/session-log`. `main` and `dev` change only
   in the release: CI PR to `dev`, `cz bump` there, merge to `main`, tag, all by the PM once
   design/rc.md holds (11-11 10:31, rules.md 3).
2. Push (charter 4): workers and the PM push their `claude/*` branches without asking; the cloud
   clone is the checkout, so "never push from a local copy" does not apply.
3. Commits: the owner's identity (Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>,
   11-09 08:20); no attribution lines, whatever a harness reminder says; never --no-verify (11-09 04:20).
4. Records (charter, overrides): working files, handovers and findings are committed on
   `claude/session-log` (nothing else survives a container). Deferred items go to backlog.md, owner
   decisions to state.md "Needs owner"; no gtd agenda, no blocker issues.
5. Reviews (rules.md 7, 8): the commit monitor of dev-conventions 4.1 is replaced by one independent
   review per lane branch before its merge (builder never reviews itself) and the merge gate. The
   final round of 4.4 (two clean rounds in a row) runs on the session branch before the rc tag.
6. Release scope (dev-conventions 1) holds until the rc tag: bugs, security, wrong docs, and the
   BLOCK and MAJOR UX findings of design/rc.md (11-11 10:30: UX/UI clean before the rc).
7. Autonomy (11-12 01:45, 11-14 17:17): decide, ask only what rules.md 3 lists; security
   policy and licensing stay the owner's (charter); outward text stays a draft (skill).
   Watchdog every 2 h plus the heartbeat, not every 30 min.
8. Chat (11-08, 11-10, 11-14 16:47): two lines plus done, running, next, blocked, budget; German
   while the owner writes German. Files, commits and reports stay English. The status page artifact
   is the owner's wish (overrides "no HTML page unless asked").
9. Environment: the cloud container is disposable, not the user's host; setup.sh may use apt and
   cargo install there. Lanes do not install system packages; a missing tool is a blocker.
10. Subagents (rules.md 5, 11-14 16:54): sonnet builds, haiku tests, Fable reviews and hard work
    (at most 2 Fable lanes at once, opus when Fable is gone). Haiku swarms only while they
    find new issues and below 85 % weekly use (owner 11-11 17:29). Each lane owns its worktree and target/.
11. Visual checks (charter): pixel asserts and gate cells replace looking at every screenshot; each
    pixel pass and each keeper gets reviewed captures.

## Known gaps against the skills (to fix, not overrides)
- tests/merge-gate.sh runs cargo build, test and clippy without `--locked` (rust-dev hard rule).
- The rc.4 CHANGELOG entry of claude/docs was written by hand (git-conventions: cz bump writes it).
- README "Use of LLM coding tools" lacks the exact wording of git-conventions "Rules for agents".
