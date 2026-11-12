# State

pm: session_016iS3kQmq6Q6Ce3hxtenYSR. Rules: rules.md (from 11-12). Old sections of this file are in git history (before commit "state: rewrite as Now, Queue, Blocked").

## Now (2026-11-12 02:44 UTC)

budget: 90 % at 2026-11-12 02:17 UTC (user), estimate 90 to 91 %, stop about 06:00 to 08:00 UTC; reset Sat 11-14 03:59 UTC.
session branch: 59df16e (merge trains today, Bangkok 11-12: tiling v2 fe3483f, mc v2 59df16e; one train left).
last_reviewed: per-merge reviews only; the per-area review of the whole range a03be22..HEAD is due (Fable, Saturday).

Lanes:
- construct-v2: tour worker session_016iS3kNcWCmF1YP1YaxYmP6 (sonnet, cloud): fixing the code review (3 MEDIUM, 9 LOW), XP face labels, then records tour-v3.
- swarm5-fixes: cloud worker session_016iS3kM51byn28e4KkoLSee (sonnet): mc mouse way back, AGENDA heading, tiling light/dark switch, key list mouse section, fuzzel version.
- xp-v2: round 2 done (52e7044); UI review FIX (4 MAJOR: a Windows flag in three places, Help table cut off, Run unthemed, false launch balloons; reports/review-xp-v2.md); cloud worker session_016iS3kNoQYkfu5FnAnT5HM8 fixing; review 2 confirms M1 to M4 only.
- ready, not merged: claude/tooling (5f15153, review fixes in; xp gate red until the xp paperclip fix merges).
- ready, not merged: claude/pkg (80f31a7, reviewed, python3 fix in) waits for train 3 with construct-v2.

Triggers: watchdog trig_01VxAcAFayNMtGiWDVuzJuRz every 2 h at :15; self-check trig_01X5NqMSsvxMSpYFJxo5Wxsg at 00:50, 06:50, 12:50, 18:50 UTC (routines.md); resume after reset trig_01Cbw7Sgz7WRVpTnjsN1zdwq (Sat 04:05 UTC); the old three-runs task trig_012VGJoVzsJ81hFa6AufBc9S disabled (replaced by the watchdog).

## Queue

Deep UX tiling (reports/ux-deep-tiling.md): 4 MAJOR for next week: no visible chat entry; helper surfaces (pill, Listening dot, card, 400 px chat panel) white in a system font, not Ink, 1 px under the bar (this is the "floating chat button" the user saw: confirmed real, not old); no notification daemon in the tiling flavor; launcher preselects Screenshot. Deep UX mc (reports/ux-deep-mc.md): 6 MAJOR for next week: the Terminal scene drops the column (boxed chat, orphaned Stop and card on page turn); no visible Mute; all-keys list overflows the column; Super+F1 vs F1 in docs; terminal text 11 px; critical notification only a red line with a dead 7 px close. Harness bug before swarm 6: persona session.sh never starts chora-assistant (command -v before PATH) and the assistant needs an installed huds folder.


End of week (user 02:17 UTC): finish the running lanes and merge (construct, pkg, tooling, swarm5-fixes; xp if ready), then intensive UX/UI reviews of all four looks with fresh captures at 1x and 2x (one opus reviewer per look plus a cross-look consistency reviewer against design/looks/shared.md) and swarm 6 on the merged build: data for next week, fixes only for main-path blockers.


1. construct-v2: code fixes by the tour worker (then it waits); UI review 1 says FIX (7 main-path flaws, reports/review-construct-v2.md): a fresh cloud worker fixes them after the code fixes; UI review 2 confirm-only; then train 3 with claude/pkg; then the tour worker records tour-v3.
2. tooling: merge in train 3 with xp-v2 (its gate needs the xp paperclip fix).
3. swarm5-fixes: review, merge (next train).
4. xp-v2: layout audit green, UI review 1 (taste and the look document only), fix round fresh, review 2 confirm-only, merge.
5. tour-v3 screencast and 6 to 10 stills of the new looks onto the status page (user 11-12).
6. Dropped today (PM push-back, user 02:43): no further reflection round this week; folded into the Saturday Fable reflection.
6b. Stale merged branches: the PM's delete was refused by the permission check (2026-11-12); list and command in reports/stale-branches.md; low priority, for the user on return.
7. Before the stop: week-1 report checked by the PM; Saturday briefs written below.
Saturday 11-14 from 04:05 UTC (Fable): first a big, very critical reflection on setup and progress (backlog W2-1, user 02:43); then per-area code review of a03be22..HEAD (core and security paths first, read-only, then HIGH fixes); construct Host refactor so each cube face runs its own manager (D4); rc reflection loop on Fable. Sonnet: fix lanes by area; swarm 6 on the four v2 looks with launcher fixes (start tests/fake-voice.py, a notify-send stub, a fake loginctl running swaylock). Then flakes (core SIGSEGV tests under load), three greens in a row, CI draft PR, packaging smoke, persona round 4; rc 11-18 to 11-20.

## Blocked

- Deleting 46 merged claude/* branches on origin: refused by the permission check; reports/stale-branches.md.
