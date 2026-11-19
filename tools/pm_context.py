#!/usr/bin/env python3
"""Print PM context for Claude Code hooks; each mode stays under the 10,000-character hook cap.

Modes:
  card       rules.md, routines.md and state.md, whole
  vision     vision-card.md
  conventions conventions.md (agent_skills dev plus the session overrides) and procedures.md
  decisions  decisions-now.md, plus user messages newer than its screened-through line
  decisions2 nothing (kept for the installed hooks)
  decisions3 nothing (kept for the installed hooks)
  precompact instructions for the compaction summary (PreCompact stdout)
  prompt     one reminder line (UserPromptSubmit)

Hook input JSON on stdin is optional; its transcript_path is used to find unrecorded user messages.
PM_HOOK_OFF=1 in the environment silences every mode (for other claude processes started in /home/user).
PM_LOG overrides the log worktree (tests). Installed hooks run a stable copy via pm-hook.sh, which never blocks.
"""
import json
import os
import re
import select
import sys
from pathlib import Path

LOG = Path(os.environ.get("PM_LOG", "/home/user/wt/log"))
CAP = 9500
DEFAULT_TRANSCRIPT = Path("/root/.claude/projects/-home-user/8a28ab73-2182-54dd-8448-69a0fbb58e8a.jsonl")


def read(name):
    try:
        return (LOG / name).read_text()
    except OSError:
        return f"[{name} missing]\n"


def hook_input():
    try:
        if select.select([sys.stdin], [], [], 0.2)[0]:
            return json.loads(sys.stdin.read() or "{}")
    except (ValueError, OSError):
        pass
    return {}


def screened_through():
    match = re.search(r"^screened-through:\s*(\S+)", read("decisions-now.md"), re.M)
    return match.group(1) if match else ""


def message_text(event):
    if event.get("type") == "attachment":
        attachment = event.get("attachment") or {}
        if attachment.get("type") != "queued_command":
            return ""
        prompt = attachment.get("prompt")
    else:
        if event.get("type") != "user" or event.get("isMeta") or event.get("isCompactSummary"):
            return ""
        if event.get("toolUseResult") is not None or event.get("isSidechain"):
            return ""
        if (event.get("origin") or {}).get("kind", "human") != "human":
            return ""
        prompt = (event.get("message") or {}).get("content")
    if isinstance(prompt, list):
        if any(isinstance(b, dict) and b.get("type") == "tool_result" for b in prompt):
            return ""
        prompt = " ".join(b.get("text", "") for b in prompt if isinstance(b, dict) and b.get("type") == "text")
    text = (prompt or "").strip()
    if not text or text.startswith(("<", "/", "Caveat:", "[Request interrupted", "This session is being continued")):
        return ""
    return text


def unrecorded_messages(transcript, since, limit=15):
    path = Path(transcript) if transcript else DEFAULT_TRANSCRIPT
    found = []
    try:
        with path.open() as handle:
            for line in handle:
                if '"type":"user"' not in line and "queued_command" not in line:
                    continue
                try:
                    event = json.loads(line)
                except ValueError:
                    continue
                stamp = event.get("timestamp", "")
                if stamp <= since:
                    continue
                text = message_text(event)
                if text and (not found or found[-1][1] != text):
                    found.append((stamp[:16].replace("T", " "), text))
    except OSError:
        return []
    return found[-limit:]


def format_messages(messages):
    return "\n".join(f'- {stamp} UTC "{text[:400]}"' for stamp, text in messages)


def capped(text):
    if len(text) > CAP:
        text = text[:CAP] + "\n[cut at the hook cap; read the file itself]"
    return text


def mode_card():
    return "PM RULES (hook; read before acting):\n" + read("rules.md") + "\nROUTINES:\n" + read("routines.md") + "\nSTATE:\n" + read("state.md")


def mode_decisions(transcript):
    text = read("decisions-now.md")
    messages = unrecorded_messages(transcript, screened_through())
    if messages:
        text = (
            "\nUSER MESSAGES AFTER screened-through, not yet screened (record each decision verbatim in"
            " decisions-now.md and answers.md now, then bump screened-through):\n" + format_messages(messages)
            + "\n\n" + text
        )
    return text


def mode_precompact(transcript):
    messages = unrecorded_messages(transcript, screened_through())
    text = (
        "Summary rules for this PM session: quote every user decision, rule, limit or correction word for word,"
        " never paraphrase numbers or their scope (friction 36: '85 % for the haiku swarms while they find new"
        " issues' became 'this week's cap'). Name decisions-now.md and rules.md as the source of truth for"
        " standing decisions and rules; the summary is not the plan. Keep the budget line (reset Sat 03:59 UTC, daily pace, model per lane)"
        " and the PM main duties of the prompt hook word for word. List the pending items as open work to"
        " resume at once, with the running lanes (session ids, background commands) and the last pushed commit."
    )
    if messages:
        text += "\nUser messages not yet in decisions-now.md; copy them verbatim into the summary:\n" + format_messages(messages)
    return text


def main():
    if os.environ.get("PM_HOOK_OFF"):
        return
    mode = sys.argv[1] if len(sys.argv) > 1 else "card"
    if mode == "prompt":
        print(
            "PM main duties (owner 11-15 09:27, 09:35: 'This is your main job'): 1) budget: pace line in state.md,"
            " spend the week by Sat 03:59 UTC (10:59 Bangkok); model and effort per lane (owner 11-19 06:56): haiku tests,"
            " sonnet routine, opus only complex and planning, Fable only by named need; 2) RC first (design/rc.md), every Queue line names its criterion;"
            " 3) vision card: four looks, KISS, POLS, stability and looks over new features. A user decision in this"
            " prompt goes verbatim into decisions-now.md and answers.md this turn; reply in 1-2 lines."
        )
        return
    if mode in ("decisions2", "decisions3"):
        return
    transcript = hook_input().get("transcript_path", "")
    builders = {
        "card": mode_card,
        "vision": lambda: "VISION CARD (hook):\n" + read("vision-card.md"),
        "conventions": lambda: "CONVENTIONS (hook; skills of agent_skills dev plus overrides):\n" + read("conventions.md")
        + "\nPROCEDURES (hook):\n" + read("procedures.md"),
        "decisions": lambda: mode_decisions(transcript),
        "precompact": lambda: mode_precompact(transcript),
    }
    if mode not in builders:
        sys.exit(f"unknown mode {mode}; use card, vision, decisions, decisions3, decisions2, precompact or prompt")
    print(capped(builders[mode]()))


if __name__ == "__main__":
    main()
