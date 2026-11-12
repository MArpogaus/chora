# Week 1 report (budget week of Mon 11-09 to Fri 11-13)

Written Thu 11-12 08:55 Asia/Bangkok (01:55 UTC), by a sub-agent of the PM from the files of the
session log and from git, so Friday 11-13 is not in it. Times are Bangkok (UTC+7) unless marked
UTC; git times are UTC. The PM checks this file before the limit stops the session (state.md,
Queue 7). Where a number is an estimate, the text says so. Where I could not verify something, it
says "not verified".

## 1. In short

- Code: 75 merges (first parent) into the session branch since 11-09 00:00 UTC, 738 commits,
  408 files changed, +80,823 and -7,536 lines against the commit before the 2D guard (0153f0d).
  Tests: 846 at the baseline (11-09) to 1751 at the last merge gate (11-11 23:00 UTC).
- Product: 3D base, construct on 3D, virtual outputs and the cube, HUD host and chat, and the v2
  looks of tiling and mc are merged. The xp v2 and construct v2 looks are on branches, not merged.
- Not reached: the foundation gate of charter 1.20 is not recorded as met, there is no rc, CI never
  ran on this branch, the per-area code review is not done, nothing was tested on real hardware.
- Budget: the user read 87 % of the weekly limit at 11-12 08:26. About 13 points are left
  until the reset on Sat 11-14 10:59.
- Process: the user steered the PM about 30 times in 24 h on 11-11 and 11-12; the PM lost context
  after compactions (frictions 33, 34, 36); reviews passed visible flaws (friction 28, judge 11-12
  point 4). This is the main weakness of the week, more than any single bug.

## 2. Goals against results

| Goal (source) | Result at 11-12 08:55 | Verdict |
|---|---|---|
| Security fix (a) to (c) verified (charter 1.11, 11-08) | Found already built; four live attacks refused; seven smaller findings fixed in 11-09 commits (fcf70e5, 9967c60, 59c4bd2, 1788a06, d443fe7) | met |
| Foundation first: stability, 3D, plugin separation (charter 1.20) | 11-09 state: gate "not met" (3D 2 of 5, handover 69 of 85 cells, XP split 1 of 4). Since: 3D steps 2 to 6 merged 11-10, core-death lane (last words, watchdog, W12 ruled out), handover matrix "complete" (f6c09e5), flake fixes. No recorded line says the gate is met; state.md no longer carries the table | not shown as met |
| 3D finished this week (user 11-09) | Steps 0 to 6 merged 11-09 to 11-10 (3d-step0 45d85f2 to 3d-step6 a03be22), wobble in 3D (3dc1eb6), construct on the 3D base (55383af). Headless software GL only; the Fable judge of 11-10 called it "merged, unverified on hardware" | merged, not verified on a real GPU |
| Virtual outputs, cube, pull out, floating windows (D69, D76, user 11-11) | Merged 11-10 to 11-11 (7ad15de, 67d51c0, e2cff6b, a4e5459, 0fa6c16, a6b8444). All four cube faces run xp; a different manager per face needs a Host refactor, planned after the reset | partly (faces are xp, labelled "XP 1" to "XP 4") |
| HUD and ACP chat sprint (user 11-09) | H0 to H7 merged (gate table 5e92531 to the chat in the HUD df3da16), voice chat methods, grants step 1 and 2, HUD host per look (59df16e) | met in code; chat per look still weak (section 6) |
| XP split X1 to X5 (charter 1.20, user 11-09) | X1 (4a13267) and X2 (e5391bd) merged 11-09. I found no X3 to X5 merge in the first-parent log | not met |
| Impressive demos as features land (user 11-10, 11-11) | GIFs of the construct walk and the wobble (11-10), tour-v2 screencast of all looks (8 chapters, 163 stills), design mocks of four looks, status page with images | met for media; the tour was recorded on 06044cf, before the v2 looks, tour-v3 is queued |
| UI/UX polish toward perfection, four looks (user 11-11) | tiling v2 merged (fe3483f), mc v2 merged (59df16e), construct HUD rework merged (06044cf). construct v2 14 of 15 steps, xp v2 8 of 15 steps, both on branches | 2 of 4 merged; "perfection" not reached (section 6) |
| Release candidate, then user test round (user 11-11) | rc criteria in design/rc.md: none of the 10 closed. No CI run, no packaging smoke, no code review per area, no three greens in a row, no persona round 4 | not started; estimate 11-18, range 11-17 to 11-20 |
| Reports and status page (user 11-09, 11-11) | Run reports 11-08 and 11-09 only; the PM wrote no run report for 11-10 and 11-11 (not verified: no file in reports/). Status page reworked into tabs 11-11 | weak |
| Budget 19 % a day in week 0, then 14 % (charter 1.1) | Readings: 15 % end of Mon, 38 % Tue 15:20, 58 % Wed 05:59, 87 % Thu 08:26. About 26 % a day Tue to Wed, not 19 % | overspent by choice (user: "do not be too paranoid", 11-10 15:20) |

## 3. Decisions of the week

"User" means the user said it in chat or answers.md (verbatim in decisions-now.md). "PM" means a PM
default that the user can reverse.

| Decision | Options | Reason | Who | Reversible |
|---|---|---|---|---|
| 3D by sort and split of planes, no depth buffer (D75, 11-09) | depth-tested world | translucent pane edges; Smithay framebuffers have no depth attachment; two opus reviews | PM (question 1, default kept, unanswered) | yes, before code depends on it |
| Mark `agent.set` and `notify.push` privileged (83f9b6c) | leave open | other clients on a socket should need a grant | PM (question 2, unanswered) | yes |
| Repository split into four repos after 1.0, prepare only (design/split.md) | keep D36 | fewer repos, clearer ownership | user (question 3, 11-09) | yes, nothing split yet |
| Grants configurable, raisable temporarily or permanently (question 6) | keep the confinement fixed, or loosen | the user wants control | user | yes |
| Hybrid workers: cloud sessions for long lanes and testers, reviewers local, merges only in the PM session (11-10) | all local, or many workers | disk and CPU local; parallel lanes | user ("Agreed, go with the hybrid") | yes |
| Drop the fresh-PM-per-run switch (11-10) | switch (Fable check: NO-GO with five blockers, all fixed; dry run still refused by the permission check) | an unattended run would block | user ("Drop the session switch") | yes |
| After the Fable judge (11-10 12:45): cut effects pack, Nix flake and windowed playground; window manager bindings ignore injected keys | keep them | focus on foundation, fail closed | user | yes |
| Fable only for heavy steps and judges; no new Fable lane after 11-11 13:13 until the reset | Fable for everything | the pool is small; the user resets it when spent | user | yes |
| Packaging before 1.0: Fedora/COPR, Arch, distrobox/OCI image | Nix flake, AUR, other distros | scope | user (chat answer 11-10) | yes |
| HUD chat lives in each look, drawn by the host (7.1), then design per look, no generalized HUD (11-11 19:30) | one shared HUD model | the shared model made two surfaces where people expect one (friction 29) | user, confirmed by PM | yes |
| Defaults of the HUD sprint questions 7.2 to 7.8 and the HUD mockups | other answers listed in questions.md | user: "Go with your defaults for the HUD mockups" (11-11 20:17) | PM, user approved the mockups | yes |
| Taste of tiling (islands, gradient border, Ink by default), mc (hairline, key block, serif chat), construct and xp (design/decisions-taste.md) | the B and C options of each mock | user delegated taste until Saturday ("You work autonomously until sat") | PM | yes |
| Super+Space opens the launcher in every look; tiling layout cycle moves to Super+Shift+Space | per-look launcher keys | swarm finding "keys differ per look" (POLS) | PM | yes |
| Construct cube faces stay xp with honest labels until the Host refactor (11-12) | ship wrong labels, or delay construct v2 | one manager per face needs a Host change | PM | yes |
| Fix before feature sprints: UX loop, foundation, code review, then the rc; the PM may release the rc when all criteria hold (11-11 17:20) | features first | user wants a clean base | user | yes |
| Test swarms may run to 85 % weekly use only while they find new issues; they stopped after swarm 4 found 0 new (11-11) | a fixed cap, or more swarms | user correction 11-12 00:29: not a week cap | user rule, PM applied it | yes |
| PM autonomy: "Don't rely on me" (11-12 03:00 UTC); budget estimate = last reading plus usage.py units over 3.8 M per point | wait for readings | the user should not need to steer | user | yes |

## 4. Resources per model

Source: `reports/swarm-2/usage.py` over the transcripts of the PM session and its local agents.
Weighted units: input + 1.25 cache write + 0.1 cache read + 5 output, times a model weight
(haiku 0.2, sonnet 0.6, opus and Fable 1.0). The weights are my arithmetic, not the bill. Run for
Bangkok days (17:00 UTC to 17:00 UTC):

| Day | opus | sonnet | Fable | haiku | all (M units) |
|---|---|---|---|---|---|
| Mon 11-09 | 44.36 | 27.26 | 0 | 0.94 | 72.56 |
| Tue 11-10 | 74.27 | 32.28 | 5.97 | 0 | 112.52 |
| Wed 11-11 | 88.32 | 0.16 | 2.07 | 1.18 | 91.73 |
| Thu 11-12 to 08:55 | 24.32 | 42.95 | 0 | 0.71 | 67.98 |
| Week to now | 227.03 | 100.51 | 8.04 | 2.82 | 338.39 |

What is measured and what is estimated:

- Measured: the token counts in the transcripts. Not covered: cloud worker sessions (the tour and
  construct worker alone shows 321 M cache read and $61 in the 11-12 judge) and the Fable
  workers of 11-10 and 11-11. The Fable column is therefore a floor.
- Estimated: the share of the weekly limit. The user's readings are the only real meter:
  3 % (11-09 10:33), 15 % (11-09 18:04), 24 % (11-10 11:12), 38 % (11-10 15:20), 52 % (11-10 17:52),
  58 % (11-11 05:59), 75 % (11-11 11:10), 83 % (11-11 12:40), then 54 % at 17:10 after the Fable
  reset (the "all models" figure included Fable), 71 % (11-11 23:42), 87 % (11-12 08:26), 87 % again at 09:50.
- Calibration moved: 32 M cache read per point on 11-09, then 2.8 M units per point, which
  overstated (usage.py gave about 95 % when the user read 87 %); now 3.8 M units per point
  (61.0 M units from 71 % to 87 %). 338 M units over 3.8 gives about 89 %, in line with 87 %,
  but the earlier days were calibrated differently, so treat it as plus or minus 3 points.
- Expensive items (judge 11-12): the PM's own turns 42.1 M units over 828 calls in 24 h, about
  as much as the builders of two looks; the construct HUD developer on opus 35.2 M units (560
  calls, context to 856 k), a third of that day's subagent spend; tiling v2 about 16 M, mc v2 about
  21 M (build 3.9 M, fix rounds 11.1 M). Sonnet builders ran to 391 to 541 k context.
- Pace: 1.8 points an hour from 11-11 23:42 to 11-12 08:26 on the user's readings. At that
  rate the 5-hour or weekly limit stops the work before the reset (state.md estimates the stop at
  08:00 to 09:00 UTC, 15:00 to 16:00 Bangkok today). The 5-hour limit was hit on 11-10 before 16:20 and
  again about 18:25 that evening (container restart, resumed 11-11 06:00); five heavy lanes used it
  in about 3 h.

## 5. Quality numbers

Tests and gates:

- Baseline 11-09 (07c6c09): 846 passed. Gate logs: 891 (11-09 13:51), 965 (H1), then 1708 (tiling v2,
  11-11 22:04 UTC) and 1751 (mc v2, 23:00 UTC). Counts are the sum of the `test result` lines
  in the gate's test.log, passed 1708 and 1751, failed 0.
- The merge gate (`tests/merge-gate.sh`) has 17 steps: gate-lib, build, test, clippy, 2d-guard, xp,
  matrix, virtual, xp-faces, hud, layout, chat-keys, last-words, cube, pull, winit, wm-pane.
  Both v2 merges passed 17 of 17. A run at 22:53 UTC failed in chora-kit (`shipped_huds` colours test
  after mc dropped a panel); the build lane had run only two crates (friction 38).
- Gate faults of the PM in the week: pushed before the gate ended (14), stale binaries (15), a FAIL
  line that did not stop the chain (18), cleanup before reading the gate (25), two gates at once (26).
  The gate script came out of this. Flakes remain: core tests under load (a stack overflow test,
  a SIGSEGV handler test) and backlog F1 to F5; "three greens in a row" has not happened.
- Red first (test fails before the fix) was skipped in at least five reports on 11-10 (friction 22)
  and in the two v2 merges 0 of 49 `fix` and `feat` commits carry a red commit (judge 11-12).
- Handover matrix: 69 of 85 cells on 11-09 (12 expected gaps, 4 skips); complete after f6c09e5.
- Not run: GitHub CI (needs a pull request or push to dev or main), packaging smoke, hardware.

Review findings (per file in reports/):

| Review | Verdict | Items |
|---|---|---|
| review-tiling-v2 (round 1) | not rated in one verdict | 24 rated tiling items |
| review-tiling-v2-r2 | FIX | 21 of 24 fixed, 2 partly, 0 open; new: 2 MAJOR, 9 MINOR, 5 COSMETIC |
| review-tiling-v2-r3 | MERGE | both MAJOR fixed (power menu, screenshot); 3 of 3 MINOR spot checks fixed; left: 3 MINOR, 3 COSMETIC; rejected as not a bug: fuzzel 1.9.2 ignores the pointer in the rig. White "Chat" tab "not rated (other lane)" |
| review-mc-v2 (round 1) | FIX | 27 items (M1 to M9, N1 to N12, C1 to C6) |
| review-mc-v2-r2 | FIX | of the 27: 17 fixed, 5 partly or nearly, 4 not, 1 decided in the repo; new: 1 MAJOR (scene 2 stacks six empty texts), 12 MINOR, 6 COSMETIC |
| review-mc-v2-r3 | MERGE | the MAJOR, the card height, hover marks and the Esc path fixed; Esc right after a click on Yes: 4 of 5 closed, an injected Esc under about 70 ms is lost |

- Code reviews: mc v2 FIX with 3 MEDIUM (lost click in the card guard, gate sleeps instead of
  signals, a check that passes when ctl fails), fixed in one round; tiling v2 fixed in dd5d064 to
  b4c6ab8; construct v2 code review has 3 MEDIUM and 9 LOW, the worker is fixing them.
- Testers: persona round 3 (11-11): 5 blockers, 9 major; UX loop 1: 13 items with 3 blockers;
  swarm 2: 107 rows, 31 sections, 20 verified, 9 rejected (env or by design); swarm 3: 6 verified,
  8 rejected; swarm 4 (mouse only): 0 new, 13 duplicates; swarm 5 (new looks): 5 verified, 13
  environment. New findings per swarm fall, but the swarm stops by the user's rule, not because
  the product is clean.
- Reflections (reflections/): 11-09: 30 numbered points; Fable judge 11-10: 26; switch check 14;
  11-11 judge and judge 2: 6 sections each; 11-12 judge: 16 (2 CRITICAL, 7 HIGH, 6 MEDIUM, 1 LOW). Each
  was answered and changed rules (pm-card, decisions-now.md, rules.md). Several points repeat across days
  (steering, status page, unbuilt rules); I did not count how many are fixed. The Fable judge of 11-10
  found HIGH items in reviews of 7 of 13 merges. A third round is queued.

## 6. Demos and media

Paths are relative to the session log branch. Captures are headless (software renderer), 1280
wide, made with `tests/persona/act.py look` or `choractl record`; mocks are hand-built HTML.

- Tour of every look, 8 chapters plus all in one: `media/2026-11-12/tour-v2/tour-all.mp4`, chapters
  `01-xp.mp4` to `08-voice.mp4`, timeline `tour-v2/timeline.md`, stills `tour-v2/stills/` (163).
  Recorded with `choractl record` at 30 fps on 06044cf, so before tiling v2 and mc v2. Camera
  glides, manager panes and voice events are method calls that stand in for walking and speech.
- Design mocks: `media/2026-11-12/design/tiling/`, `design/mc/`, `design/xp/`, `design/construct/`.
- Built looks: `media/2026-11-12/tiling-v2/`, `mc-v2/`, `construct-v2/` (steps 1 to 13, a drag
  clip `cv2-step6-drag.mp4`), `xp-v2/`, `marble/`, `construct-hud-built/`.
- Review captures with flaws boxed: `media/2026-11-12/review-tiling-v2{,-r2,-r3}/`,
  `review-mc-v2{,-r2,-r3}/`.
- Status page images: `status/img/` (17 files, for example `tiling-v2.png`, `mc-v2.png`,
  `cube-float.png`, `construct-swing.gif`, `xp-wobbly.png`). Page: https://claude.ai/artifact/GXmcSEwuzRMXF1QiyUXHFe.

Honest note on the media: the stills of tiling v2 and mc v2 are the builders' captures and the
reviewers' captures, not a showcase made for the user; the user's rule is that he approves each look
from showcase captures (rc 1b). That approval has not happened.

## 7. Honest assessment

What failed:

- Cloud workers were blocked for about a day. A new worker could not run `setup.sh` (its
  permission classifier refused "code from external", frictions 30 and 37); two workers (construct-v2
  and xp-v2) were archived with no commits. Build lanes then ran only in this container, two at a
  time, limited by disk (down to 2.8 GB free on 11-10, 7.9 GB now) and four CPUs. The user fixed the
  environment 11-12; a check worker found it ready; it has not yet carried a heavy lane under load.
- The PM lost context and rules after compaction. Friction 33: `git add -A` about eight
  times, chat replies over the length rule, swarm 2 ignored section 8 (800 px, 90 steps), state.md
  and answers.md not updated after merge trains. Friction 34: the PM listed next steps and stopped
  until the user asked why no tester ran. Friction 36: the user's "85 %" for the haiku swarms was
  reported as a week cap for many turns. Friction 35: a haiku tester called `create_session`.
  Fixes: pm-card, decisions-now.md with verbatim quotes, hooks after compaction, rules.md,
  tools/check_log.py and lanes.py. The fixes are two days old; they are untested over a week.
- The user steered about 30 times in 24 h (judge 11-12: 59 messages, 4 of them readings); nine more
  after his "no more intervention" at 11-12 00:50, six steering. Repeated causes: stale sub-sessions,
  screenshots on the status page (six requests), unreviewed pixels, a missing visible plan.
  He also had to say "decide such things on your own" twice.
- Reviews passed visible flaws. The xp paperclip stretched and a second chat balloon passed UX loop 1
  and the HUD review (friction 28); one shared HUD for four looks passed review (29). The tiling v2
  merge carried the white "Chat ..." tab behind the Keys and power icons under a MERGE verdict ("not
  rated, other lane"), mc v2 merged with scene 2 dropping the Reading panel at 1280 wide, rated MINOR
  (judge 11-12 point 4). The perfection rule of 11-11 19:20 says that blocks.
- Gates and process errors: wrong times written without reading the clock (19, about 20 to 70 minutes
  off), a lane that passed its own tests and broke the workspace (38), a charter flag that no tool
  had (32), five adopted rules never built (judge 11-12 point 5), `--no-verify` used once by a worker (27).

What is weak:

- 3D and the cube are verified headless on software GL only. No frame has run on a GPU or a real display.
- The product is a set of four looks of unequal state: xp v2 8 of 15 steps, construct v2 14 of 15
  with xp on all four faces, the construct chat and helper surface not per look yet, tiling and mc
  merged with MINOR and COSMETIC items open (key list caps, mako style not shipped, `notify-send` not
  in optdepends, fuzzel minimum version, a Paper switch that leaves open windows dark).
- Gates are "green as expected": the handover and HUD gates counted known gaps as pass; the 2D guard
  has seven simple scenes; 17 gate steps passed twice, but not three times in a row, and the
  flakes are named, not closed.
- The status page and reports overstated at times ("7 of 7 complete", "pixel-identical throughout" per
  the Fable judge 11-10; the stretched paperclip still on the page until 11-12, now removed).
- The branch is 100 k lines of change away from rc.3. How it lands as the new base and restarts the
  30-day daily-use gate was a user decision (11-10 12:45, item 4); it is not yet exercised.
- Voice is tested with fakes (fake ASR, `tests/fake-voice.py`). German wording had 9 findings in persona
  round 3 (M2: "Agent", du and Sie, no time left).

What simulated users cannot tell:

- Whether anything feels right: latency, smoothness of a drag, the wobble, glass blur, real
  motion. They read screenshots at 640 or 1280 px; an 800 px look exaggerates small text (finding 17).
- Real hardware: GPU, multi-monitor, a real PipeWire, a notification daemon, real fonts. The rig has
  no Super-alone key, no sound, no LibreOffice, fuzzel 1.9.2 ignores the pointer, foot 1.16.2 is too old
  for mc.
- Real voice and accents, real people who are not technical, hours of use (leaks and stalls), taste
  and desire. The user said "Tiling is still pretty ugly" after a pass that the reviews had rated good.

## 8. Open questions and decisions for the user

From questions.md and the state files:

1. Q1 depth buffer and Q2 privileged methods: defaults taken (kept rejected, kept privileged); answer
   "allow a depth buffer" or "unprivileged" to reverse.
2. Q4 publish `chora-scene` and `chora-kit` on crates.io at 1.0 (default yes); Q5 only the user can
   create the GitHub organization `chora-desktop` (not done).
3. Q8 one cube (default taken; "keep both" reverses); Q9 to Q12 and Q14, Q15 launch defaults taken; Q13
   (the account and karma for r/unixporn) is the user's.
4. Approve each look from showcase captures (rc 1b). Nothing has been shown to you in a form made for approval.
5. Real hardware: the rc list leaves a tty and GPU run to you before 1.0.
6. Fable use next week: you got another reset; the plan below assumes Fable until it is spent.
7. Whether the rc date should hold when the budget runs out this week (estimate 11-18, range 11-17 to
   11-20; it holds only with one helper surface per look, otherwise 11-20).

## 9. Plan for week 2

Resume at the reset (Sat 11-14 10:59, trigger 04:05 UTC), budget 14 % of the week a day. Before the reset: the
watchdog (hourly at :35) keeps lanes running; construct v2 merges with claude/pkg in train 3; the tooling
lane (check_log, lanes.py, `merge-gate --worker`, red-first check, `tests/ci-local.sh`) is reviewed and
merged; swarm5-fixes (mc mouse way back, AGENDA heading, tiling light and dark switch, key list mouse
section, fuzzel version) and xp-v2 (steps 2, 4, 10, 13, tray icon, 14 px sweep, Help as a window) go
through review.

After the reset, in this order (state.md Queue):

1. Fable: per-area code review of a03be22..HEAD, read-only, core and security paths first
   (core, kit, xp, tiling, mc, construct, assistant, voice), then fix HIGH findings; judges
   simplicity and readability first, then correctness and security (rc.md).
2. Fable: the construct Host refactor, so each cube face runs its own manager (D4); until then
   the faces stay xp.
3. Sonnet fix lanes by area: helper chat entry in every look, key help in xp, tiling and mc, Super
   chords leaking into terminals, cube error state, mc first screen, the chat tab, per-look helper surface.
4. Swarm 6 on the four v2 looks with a launcher that starts `tests/fake-voice.py`, a notify-send
   stub and a fake loginctl; only while new findings arrive.
5. Flakes (core tests under load), then three green merge gates in a row and the opt-in gates once.
6. CI draft pull request, release workflow dry run, packaging smoke (RPM, Arch, distrobox).
7. Persona round 4 on the candidate (Erika, Kemal, Lena, Max): every basic task by mouse, keyboard
   and voice; tour-v3 and showcase captures for your approval.
8. rc about 11-18, range 11-17 to 11-20; then the user test round (persona round 5 at full breadth, a
   long session, battle script), then the reflection loop and a charter sharpening.

Risks: the budget stops work before the reset; the per-area review may find HIGH items in code
reviewed only per merge; CI and packaging have not run once; the 11-12 process fixes are untested.
