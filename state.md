# State

pm: session_016iS3kQmq6Q6Ce3hxtenYSR. Rules: rules.md (from 11-12). Old sections of this file are in git history (before commit "state: rewrite as Now, Queue, Blocked").

## Now (2026-11-12 04:16 UTC)

budget: 90 % at 2026-11-12 02:17 UTC (user), estimate 90 to 91 %, stop about 06:00 to 08:00 UTC; reset Sat 11-14 03:59 UTC.
session branch: 8f358fa (trains Bangkok 11-12: tiling v2 fe3483f, mc v2 59df16e, train 3 8f358fa = tooling + pkg + xp-v2, full gate 19/19; no train left until 17:00 UTC).
last_reviewed: per-merge reviews only; the per-area review of the whole range a03be22..HEAD is due (Fable, Saturday).

Lanes:
- construct-v2: code fixes done (e558ae9d, full gate 19/19; flake: cube navigate-click 2 of 5 under load). UI fix round M1-M7: the first worker never left PENDING (02:47 to 04:15 UTC, archived; friction 41); retry worker session_016iS3kNNrfvVqfnNJw6akRK (sonnet, started 04:16 UTC; start check 04:22). Tour worker session_016iS3kNcWCmF1YP1YaxYmP6 idle until "go" for tour-v3 after the merge.
- swarm5-fixes: ready for train 4 (9644cc05: merged claude/pkg, both HIGH were covered by pkg, packaging-files.py 0 gaps in 141 files; MEDIUMs fixed; full gate passed on the 2nd run, flake in cube "a client xp runs already" added to the Saturday flake list). Worker archived.
- One lane only on purpose (PM, 04:16 UTC): budget about 92 to 93 %, every other item is done or waits for the 17:00 UTC train; no filler work.
- persona harness: done (claude/persona-harness 8ca9802, smoke red then green on all four looks; bug (a) did not reproduce, the gaps were huds and voice); merges in train 4.
- swarm 6 done (reports/swarm-6.md): 6 new main-path MAJORs (helper never answers and no no-helper state; typed text dropped before picking a helper; mc arrows and Escape; mc no on-screen launcher; tiling bar Keys/Chat buttons dead; xp no volume icon). Kept for week-2 fix lanes by area (PM: no fix now, the next train is after 17:00 UTC and the budget stop is near; they would collide with swarm5-fixes in mc and tiling).
- deep UX xp + cross-look done (3d12b22; reports/ux-deep-xp.md 6 MAJOR: no lock, Log Off silent, Run bare and silent on no match, first F1 leaks "OP" into the terminal, no-helper balloon cut, notify balloon over the paperclip; reports/ux-cross-look.md: lock missing in all looks, mc has no power/volume/screenshot, F1 key list only in xp, launchers lack "Chat with a helper", tiling Chat pill dead on click). Worker archived. Week 2: one shared-doors decision (lock, power, volume, screenshot, F1 = key list in every look) before the fix lanes.

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
