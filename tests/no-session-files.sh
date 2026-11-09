#!/usr/bin/env bash
# pre-commit hook: refuse staged session-log files (plans/autonomy/,
# reports/, reflections/, media/, drafts/, charter.md, state.md, ...).
status=0
while IFS= read -r path; do
  case "$path" in
    plans/autonomy/* | reports/* | reflections/* | media/* | drafts/* | \
      charter.md | state.md | ledger.md | friction.md | answers.md | \
      questions.md | backlog.md | HANDOFF.md)
      echo "$path: session files belong on the branch claude/session-log" >&2
      status=1
      ;;
  esac
done < <(git -c core.quotepath=off diff --cached --name-only --diff-filter=ACMR)
exit "$status"
