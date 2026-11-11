# Swarm 2 (11-12): mass haiku user tests, data only, no fixes

User call at 71% model use: use the remaining budget to test massively with haiku and gather data; no fixes yet. PM stops at about 90%.

- Build: 06044cf (construct HUD rework merged), binaries snapshotted to scratchpad/swarm-bin, source at /tmp/swarm-src.
- Setup fixes over swarm 1 (friction 31): the PM-owned `swarm.sh` starts and stops each session with CHORA_BIN, PERSONA_KEEP and the injected-press flag; testers only look and act. At most 2 sessions at once (4 CPUs).
- Each tester follows `swarm2-brief.md` and writes `NAME.md` with a fixed findings table; shots and actions go to media/2026-11-12/swarm-2/NAME/.
- Waves of 4. Wave 1: c-novice, c-cube (construct), x-office (xp), t-power (tiling).
- Planned: c-chat, c-drags, x-balloon, x-keys, t-chat, m-tv, m-chat, transitions through the cube, drag across faces, chaos input, habit keys from other desktops.
- At the end one triage pass dedupes, separates environment noise from product bugs, and ranks the findings into findings.md.

## Triage notes while running
- c-cube run 1 is environment noise in its main finding: the cube's xp manager looked for its theme in /usr/share/chora/flavors, which this container lacked; it restarted in a loop ("I come back") and never connected, so the cube stayed black. Fixed by linking /usr/share/chora/flavors to the source checkout; the cube then shows the xp face. Kept as c-cube-run1-env.md. The product finding that remains: a manager that cannot start gives 10 s of black with no message (needs an error state).
- m-tv (mc shows only "1 AGENDA", no key reacts) and x-office (LibreOffice and thunar missing) may be partly environment too; the triage pass checks each against the container.

## Budget rule (user, 11-12)
- Stop testing at 85% weekly model use; keep the rest for major fixes and the review loop.
- Launch a new tester only while the last reports still bring new findings.
- Haiku runs are trimmed: 512 px looks, at most 45 actions and 20 looks, reports of at most 40 lines (brief updated after wave 2 started).
- The weekly reset is Saturday 10:59. After it, the heavy work (fixes, code review, RC) runs on Fable until it is used up, then on a reset.

## Usage estimate (11-11 ~17:00 UTC)
The old cross-check (total_cost_usd of the newest result event) no longer works: the PM session runs as one long turn, so there is no recent result event. New: `usage.py SINCE [UNTIL]` sums the local transcripts of the PM and its local agents into weighted units (opus 1, sonnet 0.6, haiku 0.2; output 5, cache write 1.25, cache read 0.1). Calibrated on the user's readings 68% (15:08 UTC) and 71% (16:42 UTC): 8.35 M units, so about 2.8 M units per percent. Cloud workers are not counted. One haiku tester costs about 0.1 M units, about 0.04%.
