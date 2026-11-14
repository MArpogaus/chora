#!/usr/bin/env python3
"""write_settings.py SETTINGS PMDIR: writes the PM hooks into a Claude Code settings file.

Keeps every other key of the file. Each command ends in `|| true`, so a missing script can never
block a prompt or a compaction (friction 44). No UserPromptSubmit hook: it blocked every prompt.
"""
import json
import sys
from pathlib import Path


def command(pm_dir, mode):
    return f"sh {pm_dir}/pm-hook.sh {mode} 2>/dev/null || true"


def hooks(pm_dir):
    def entry(mode):
        return {"type": "command", "command": command(pm_dir, mode), "timeout": 20}

    return {
        "SessionStart": [
            {"matcher": "compact|startup|resume|clear", "hooks": [entry("card"), entry("decisions"), entry("vision")]}
        ],
        "PreCompact": [{"hooks": [entry("precompact")]}],
    }


def main():
    settings, pm_dir = Path(sys.argv[1]), sys.argv[2]
    data = json.loads(settings.read_text()) if settings.exists() else {}
    data["hooks"] = hooks(pm_dir)
    settings.parent.mkdir(parents=True, exist_ok=True)
    settings.write_text(json.dumps(data, indent=2) + "\n")


if __name__ == "__main__":
    main()
