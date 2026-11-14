# chora session log

The working files of the autonomous session of November 2026. This branch has no
history in common with the code branches and is never merged into them.

- Injected after a compaction (tools/pm_context.py, sizes checked by tools/check_log.py):
  `vision-card.md`, `rules.md`, `state.md`, `decisions-now.md`.
- Working files: `backlog.md`, `design/` (rc criteria in `design/rc.md`), `dev-rules.md`,
  `reports/`, `reflections/`, `feedback/`, `media/`, `status/`.
- Append-only logs: `answers.md`, `friction.md`.
- `archive/`: the charter and other files of week 1, kept for reference; not rules.

## PM down (runbook, friction 44)

Signs: the owner's prompts or `/compact` fail with "blocked by hook", or the PM heartbeat pushes
"PM DOWN". The hooks are fail-open since 11-14, so this should not recur; if a PM dies anyway:

1. Start a new cloud session on MArpogaus/chora in the same environment.
2. Run `bash /home/user/wt/log/tools/takeover.sh` (from a fresh container: clone the log branch
   first, or run `/home/user/.claude/pm/takeover.sh` if it exists). It recreates the worktree,
   runs setup.sh, installs the fail-open hooks and the post-checkout guard, and tests them.
3. Follow the checklist it prints: new id in state.md, triggers (prompts in `tools/triggers.md`),
   workers told, old PM archived, transcript mined for what the log lacks.

Hooks: `tools/install_hooks.sh` writes `/home/user/.claude/settings.local.json`. Every command
ends in `|| true` and runs a stable copy in `/home/user/.claude/pm/`, outside any worktree.
There is no UserPromptSubmit hook. `tests/test_hooks.sh` proves both.
