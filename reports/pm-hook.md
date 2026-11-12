# PM hook proposal (11-12; friction 33, 34, 36)

The PM applies this; the subagent did not touch any settings file.

## Findings on the current hook

- Current: one `SessionStart` hook, matcher `compact`, that cats `pm-card.md` and `decisions-now.md`.
- Hook stdout has a cap of 10,000 characters per hook (docs, hooks reference). Above it Claude Code
  saves the text to a file and injects only a 2,000-character preview. The screened `decisions-now.md`
  alone was 11,300 characters in its first draft. With the card added, the old one-command hook would
  have injected a 2,000-character preview, so most of the decisions would have been lost silently.
  Fix: one hook per file, and keep `decisions-now.md` under 9,000 characters (stated in its header;
  now 7,956).
- `compact` covers compaction only. A restart (`startup`), `/resume` or `--continue` (`resume`) and
  `/clear` (`clear`) start without the card. Container replacements happened twice (friction 20).
- Nothing protects the summary itself. Friction 36 came from a paraphrase written into the summary.
  `PreCompact` stdout becomes "instructions for the compaction summary". It fires for `manual` and
  `auto`, gets `trigger`, `custom_instructions` and `transcript_path` on stdin, and has its own
  10,000-character cap. Exit 2 would block compaction. It is not used here, because blocking an auto
  compaction at the context limit fails the request.
- `PostCompact` gets `compact_summary` but cannot inject context, so it is not useful here.
- `UserPromptSubmit` plain stdout is added beside each prompt. A one-line reminder costs about 30
  tokens per user prompt. Whether it also fires on scheduled-task wakes is not verified. It is harmless
  either way.
- Subagents do not fire `SessionStart` or `UserPromptSubmit`, so testers and reviewers do not get the
  card. Another `claude` process started in /home/user would get it. For that case `PM_HOOK_OFF=1`
  silences the script.

## Proposed setup

Helper: `tools/pm_context.py` (stdlib only; 0.03 to 0.15 s; each mode is capped at 9,500 characters).

| Mode | Event | Prints | Size now |
|---|---|---|---|
| `card` | SessionStart (compact, startup, resume, clear) | pm-card.md, the "Now" section of state.md (22 lines), the newest 12 lines of answers.md | 4.7 kB |
| `decisions` | same | decisions-now.md, plus every human message in the transcript newer than its `screened-through:` line, marked "record now" | 8.0 kB |
| `precompact` | PreCompact (manual and auto) | summary rules: quote decisions verbatim, never paraphrase numbers or scope, the files are the source of truth, list pending work and live lanes, and copy unrecorded user messages verbatim | 0.5 kB |
| `prompt` | UserPromptSubmit | one line: record decisions verbatim this turn; reply in 1-2 lines | 117 B |

The full injection happens only at start, resume, clear and compaction. Per prompt the cost is one line.

"Unrecorded" is decided by the `screened-through:` timestamp in the header of decisions-now.md. The
script lists the human-typed messages (origin human, plus queued messages typed during a turn; no
tool results, system reminders or agent messages) after that time. The PM bumps the timestamp when it
records a message. This also gives a cheap staleness check: if the list is long after a compaction,
the PM fell behind on recording.

## Exact JSON for /home/user/.claude/settings.local.json

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "compact|startup|resume|clear",
        "hooks": [
          { "type": "command", "command": "python3 /home/user/wt/log/tools/pm_context.py card", "timeout": 20 },
          { "type": "command", "command": "python3 /home/user/wt/log/tools/pm_context.py decisions", "timeout": 20 },
          { "type": "command", "command": "python3 /home/user/wt/log/tools/pm_context.py decisions2", "timeout": 20 },
          { "type": "command", "command": "python3 /home/user/wt/log/tools/pm_context.py vision", "timeout": 20 }
        ]
      }
    ],
    "PreCompact": [
      {
        "hooks": [
          { "type": "command", "command": "python3 /home/user/wt/log/tools/pm_context.py precompact", "timeout": 20 }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          { "type": "command", "command": "python3 /home/user/wt/log/tools/pm_context.py prompt", "timeout": 5 }
        ]
      }
    ]
  }
}
```

## Follow-ups for the PM

1. Apply the JSON. Then check it with `/hooks`, or run each command by hand: `echo '{}' | python3 .../pm_context.py decisions`.
2. Add one line to pm-card step 3: "bump `screened-through:` in decisions-now.md when you record a user message".
3. Run start step 2 (recreate the hook after a container replacement) should write this JSON, not the old one.
4. Guard (charter 1.15): the session-log pre-commit check can reject a commit when decisions-now.md is
   over 9,000 characters, so it never falls back to the 2,000-character preview.
5. answers.md lacks several 11-11 messages: the construct HUD rework (19:45 Bangkok), voice local models (18:01),
   the reflection after the rc gates (18:03, 18:05), the second Fable reset for next week (17:56), the xp
   balloon (18:26) and "close stale ones" (11:43). They are now in decisions-now.md. Copy them into answers.md.
6. Fix the paraphrase in the answers.md block "11-11 (chat, copied late)": its "cap at 85% weekly use" is the
   wrong reading, and "the rest of this week's heavy work then runs on Fable" contradicts "no more fable for
   this week". Both now carry the corrected reading in decisions-now.md.
