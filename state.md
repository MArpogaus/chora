# State

pm: session_016iS3kQmq6Q6Ce3hxtenYSR. Rules: rules.md (from 11-12). Old sections of this file are in git history (before commit "state: rewrite as Now, Queue, Blocked").

## Now (2026-11-12 02:29 UTC)

budget: 90 % at 2026-11-12 02:17 UTC (user), estimate 90 to 91 %, stop about 06:00 to 08:00 UTC; reset Sat 11-14 03:59 UTC.
session branch: 59df16e (merge trains today, Bangkok 11-12: tiling v2 fe3483f, mc v2 59df16e; one train left).
last_reviewed: per-merge reviews only; the per-area review of the whole range a03be22..HEAD is due (Fable, Saturday).

Lanes:
- construct-v2: tour worker session_016iS3kNcWCmF1YP1YaxYmP6 (sonnet, cloud): fixing the code review (3 MEDIUM, 9 LOW), XP face labels, then records tour-v3.
- swarm5-fixes: cloud worker session_016iS3kM51byn28e4KkoLSee (sonnet): mc mouse way back, AGENDA heading, tiling light/dark switch, key list mouse section, fuzzel version.
- xp-v2: local sonnet agent in wt/xp-v2: steps 2, 4, 10, 13, tray icon, 14 px sweep, Help as a window.
- tooling: log tools live (check_log.py hook, lanes.py, 18 tests); claude/tooling code review FIX (3 MEDIUM: ci-local skip path, pre-commit step skips, red-first order and test paths): cloud worker session_016iS3kMhTbAjyJBj4Q5zpNB fixing. The xp gate goes red on the paperclip size (real flaw): sent to the xp agent.
- week-1 report: local sonnet agent, reports/week-1.md.
- ready, not merged: claude/pkg (80f31a7, reviewed, python3 fix in) waits for train 3 with construct-v2.

Triggers: watchdog trig_01VxAcAFayNMtGiWDVuzJuRz every 2 h at :15; self-check trig_01X5NqMSsvxMSpYFJxo5Wxsg at 00:50, 06:50, 12:50, 18:50 UTC (routines.md); resume after reset trig_01Cbw7Sgz7WRVpTnjsN1zdwq (Sat 04:05 UTC); the old three-runs task trig_012VGJoVzsJ81hFa6AufBc9S disabled (replaced by the watchdog).

## Queue

Deep UX tiling (reports/ux-deep-tiling.md): 4 MAJOR for next week: no visible chat entry; helper surfaces (pill, Listening dot, card, 400 px chat panel) white in a system font, not Ink, 1 px under the bar (this is the "floating chat button" the user saw: confirmed real, not old); no notification daemon in the tiling flavor; launcher preselects Screenshot. Harness bug before swarm 6: persona session.sh never starts chora-assistant (command -v before PATH) and the assistant needs an installed huds folder.


End of week (user 02:17 UTC): finish the running lanes and merge (construct, pkg, tooling, swarm5-fixes; xp if ready), then intensive UX/UI reviews of all four looks with fresh captures at 1x and 2x (one opus reviewer per look plus a cross-look consistency reviewer against design/looks/shared.md) and swarm 6 on the merged build: data for next week, fixes only for main-path blockers.


1. construct-v2: code fixes by the tour worker (then it waits); UI review 1 says FIX (7 main-path flaws, reports/review-construct-v2.md): a fresh cloud worker fixes them after the code fixes; UI review 2 confirm-only; then train 3 with claude/pkg; then the tour worker records tour-v3.
2. tooling lane: review (sonnet, bugs and simplicity), merge in train 3 if ready, else first train Saturday; then the session-log hook goes live.
3. swarm5-fixes: review, merge (next train).
4. xp-v2: layout audit green, UI review 1 (taste and the look document only), fix round fresh, review 2 confirm-only, merge.
5. tour-v3 screencast and 6 to 10 stills of the new looks onto the status page (user 11-12).
6. Reflection round 2 (confirm-only on the applied changes, about 1 M units); stop when only LOW points come.
6b. Stale merged branches: the PM's delete was refused by the permission check (2026-11-12); list and command in reports/stale-branches.md; low priority, for the user on return.
7. Before the stop: week-1 report checked by the PM; Saturday briefs written below.
Saturday 11-14 from 04:05 UTC (Fable): per-area code review of a03be22..HEAD (core and security paths first, read-only, then HIGH fixes); construct Host refactor so each cube face runs its own manager (D4); rc reflection loop on Fable. Sonnet: fix lanes by area; swarm 6 on the four v2 looks with launcher fixes (start tests/fake-voice.py, a notify-send stub, a fake loginctl running swaylock). Then flakes (core SIGSEGV tests under load), three greens in a row, CI draft PR, packaging smoke, persona round 4; rc 11-18 to 11-20.

## Blocked

- Deleting 46 merged claude/* branches on origin: refused by the permission check; reports/stale-branches.md.
