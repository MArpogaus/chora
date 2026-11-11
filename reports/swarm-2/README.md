# Swarm 2 (11-12): mass haiku user tests, data only, no fixes

User call at 71% model use: use the remaining budget to test massively with haiku and gather data; no fixes yet. PM stops at about 90%.

- Build: 06044cf (construct HUD rework merged), binaries snapshotted to scratchpad/swarm-bin, source at /tmp/swarm-src.
- Setup fixes over swarm 1 (friction 31): the PM-owned `swarm.sh` starts and stops each session with CHORA_BIN, PERSONA_KEEP and the injected-press flag; testers only look and act. At most 2 sessions at once (4 CPUs).
- Each tester follows `swarm2-brief.md` and writes `NAME.md` with a fixed findings table; shots and actions go to media/2026-11-12/swarm-2/NAME/.
- Waves of 4. Wave 1: c-novice, c-cube (construct), x-office (xp), t-power (tiling).
- Planned: c-chat, c-drags, x-balloon, x-keys, t-chat, m-tv, m-chat, transitions through the cube, drag across faces, chaos input, habit keys from other desktops.
- At the end one triage pass dedupes, separates environment noise from product bugs, and ranks the findings into findings.md.
