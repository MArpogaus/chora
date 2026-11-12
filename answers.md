# Answers of the user

Newest first. The PM reads this file at the start of each run.


## 11-10 17:05: demo captures
"Have some impressive screenshots and some gifs for construct cube and 3d
wobble demos. We might be able to reuse for reddit etc"
Reading: E5 demo media are wanted as soon as each piece exists, not only
at the end. Plan: a capture lane (demo/record.sh: headless frames,
ffmpeg to GIF and MP4) after K5 merges; construct walk and 3D wobble
first, the cube and the pull-out when D69, cube geometry and E3 land.

## 2026-11-10

- 11-11 ~22:30: model use at 68 %; use the rest wisely. Haiku test swarm for user tests; record an extensive slow screencast of every look, every transition through the construct, several window drags; analyse the user's path for consistency and least surprise. Plan in design/rc.md "Screencast and swarm".
- 11-11 ~21:45: post screenshots when the HUD rework lands; floor reflections with marble texture (small scratches and bumps), realistic. Added to the marble lane spec.
- 11-11 ~21:30: HUD mockup defaults approved; UX/UI refinement loop; construct reflections with marble-like blur; push to visual perfection. Backlog N1: a modern macOS-inspired look on the xp stacking code, taskbar as a shared widget.
- 11-11 ~20:00: PM answer to reflection 2: all five changes adopted (charter "Rules of reflection 2", rc.md 1b, ux-hud-sprint superseded).
- 11-11 ~19:30: "We need different guidelines and philosophy or design documents for all our components and window managers ... we can not generalize the HUD ... polish the rough edges when combined." Confirmed; charter rule "Design per look"; designer drafts started.
- 11-11 ~19:20: rework the status page (too cluttered; folding or tabs); post detailed screenshots and screencasts of all features for verification; UI reviews hunt for perfection (charter rule added).
- 11-11 ~19:00: "Later have this frictionless with automatic paste bin upload + GH issue text after confirmation and some additional questions in the script." Added as L1 item 4.
- 11-11 ~18:50: "Ensure good and detailed logging in all components ... users submit anonymised reports using a small shell script." Noted as backlog L1 and rc criterion 9a.
- 11-11 ~18:40: PM answer to reflections/2026-11-11-opus-judge.md: all five top changes adopted (charter section "Rules of the 11-11 reflection", dev-rules, rc.md); the 11-10 judge is answered by the same rules (lanes, red first, review cap).
- 11-11 ~17:20 Bangkok: "You may release when all clean. Review code for simplicity and readability." Recorded in design/rc.md: the PM opens the CI pull request and tags and publishes the rc once all criteria hold.
- 11-11 ~17:10 Bangkok: Fable 7 %, all 54 %. "After ui ux clean. Finalize foundational work and review code until ready for rc release before feat sprints." Agreed; plan in design/rc.md.
- 11-11 ~13:40 Bangkok: Fable 5 %, all 43 % (after the Fable reset). The running Fable lanes finish; no new Fable lanes this week. Remaining week on opus/sonnet: there is room (57 %) for steady opus work until Saturday, paced by the 5-hour limit.
- 11-11 ~13:15 Bangkok: "Fable used up. I've reset. Finalize, then no more fable for this week." Plan: the four running Fable lanes finish their current task (stale-frame bug, K7 manager panes, HUD look fixes, cube/float/pull demos) with their reviews; no new Fable lane until Monday. After that the week runs on opus/sonnet for light work (persona tests, docs, website), within the remaining weekly budget.
- 11-11 ~12:40 Bangkok: Fable 86 %, all 83 %. Four Fable lanes run on; at the Fable limit the user resets it. Non-Fable use stays minimal (PM only).
- 11-11 ~12:30 Bangkok: "Yes do the demos. Lets make it impressive demos as features land. Polished so we can reuse them later for advertising. Decide such things on your own from now." Standing rule in the charter.
- 11-11 ~11:10 Bangkok: Fable 70 %, all 75 %. Pace: Fable lanes go on (user resets Fable when spent); non-Fable use kept low: no new opus lanes, reviews on Fable, PM replies short.
- 11-11 ~08:00 Bangkok: "New cube has no floating Windows. Ensure same look as before." The old xp desktop cube lifted each window as its own plane in front of its face, staggered by stacking (status/img/xp-cube-3d.png). The construct cube must look the same: windows float while standing back or turning, flat at 1:1 in use mode. Sent to the D69 worker as the next lane, design note first.
- 11-11 05:59 Bangkok (message during the 5-hour stop): Fable 40 %, all 58 %. "Floating windows in cube still working?" The 5-hour limit stopped all lanes at about 18:25 Bangkok (reset 21:20); the container restarted; resumed at 06:00. Five heavy lanes hit the 5-hour limit in about 3 h: run at most four, with demo polish and virtual-2 held until a slot frees.
- 18:00 Bangkok: "Reset Fable when it hits the limit, keep going." No throttling: the user resets Fable at its limit. Heavy lanes lean on Fable workers; demo polish D2 started as a Fable worker (session_016iS3kMz3yvq4o7Xxb8ASmu, claude/demo).
- 17:52 Bangkok: Fable 30 %, all 52 %. Today used about 37 points of the week (15 % at the end of 11-09); the week resets Saturday. Pace from now: about 14 points a day for the rest of the week; heavy work leans on Fable (its pool can be reset), no new lanes tonight, reviews stay compact.
- 16:20 Bangkok: the 5-hour limit was hit (reset 16:20 Bangkok). Fable 16 %, all 41 %. Respect the 5-hour limit in planning.

- Don't be too paranoid about the limit: no waste, but finish the work and spend the weekly limit fully.

- The 38 % includes Fable, which gets reset, so there is still room.

- 15:20 Bangkok: Fable 12 %, all models 38 % of the week. Post screenshots on the status page when things merge.

- Hybrid workers agreed: separate worker sessions (own container, started
  with the repo as source) for long heavy dev lanes and for testers
  (personas, battle, load); reviewers stay local subagents; the PM stays in
  this session; at most 2 to 3 workers; merges only here. Shared compile
  cache (sccache) from now on.

- 14:34 Bangkok: Fable at 9 % used (judge, core review, switch check, D69 design, core-death lane so far).

- Drop the session switch, no more time on it. Fable at 6 % used. "Go, let's get serious work done."

- Decisions after the Fable judge audit (12:45 Bangkok): (1) cut the
  effects pack, the Nix flake and the windowed playground from this
  session (backlog, after 1.0); (2) window manager bindings ignore
  injected keys (fail closed), automation uses the privileged methods;
  (3) each scheduled run starts a fresh PM session from state.md;
  (4) this branch becomes the new base after the session, and the 30-day
  daily-use gate restarts from it.

- "Don't mindlessly burn tokens. Use it (Fable) for the heavy lifting."

- The user reports Fable usage. Pull as much heavy work as possible to
  Fable this week while it performs well.

- Fable 5.2 is out with a one-time reset (for Fable only). Use it as the
  strong senior developer for big and hard tasks and as an honest,
  strong judge of the whole project, the plan and the decisions.

- Note this down and keep sharpening the plan as data and experience
  grow (charter 11c, plan-data.md).

- Decided (question in chat): packaging before 1.0 stays Fedora/COPR and
  Arch, plus a distrobox/OCI image with windowed mode and a Nix flake;
  native packages for other distributions after 1.0. Website framing:
  one headline ("a programmable 3D desktop you shape by hand or with an
  agent") with two paths, "Rice it" and "Build with it".

- Yes, start step 1 (effects). If things keep moving this fast, there
  is room to push further. Wanted: deliberately over-the-top effects for
  attention: a closing window burns down with realistic flames and
  falling ashes; exploding windows with a visible shock wave; other
  useless but impressive effects.
- Marketing: design and marketing agents work hand in hand on a good
  website and the USPs. Target the ricing nerds first, but frame chora
  also as a universal design study for agentic computing, for many use
  cases and target groups.
- Packages for all major Linux distributions with Wayland, plus a
  distrobox image for quick exploration. New use case: a windowed chora
  with an agent as a universal 2D/3D canvas and playground. Discuss,
  push back, judge critically.

- XP split: window effects shall become general and usable by other window
  managers. Wobbly windows in 3D with the same code? Scenario: walk
  through the construct, stand in front of the cube, grab a floating
  window and move it into the 3D construct, wobbling. Possible as a
  r/unixporn showoff demo? (design/wobbly-3d.md)

- 11:12 Bangkok: weekly limit at 24 %. tokens.py for 11-10: estimate: 8.3 % of the weekly limit (15 % at the end of 11-09, so about 9 % today; the estimate holds).

- Keep chat summaries to a minimum to save tokens; the user reads the
  status page or asks.

- 06:27 Bangkok: "Start at 6. Begin now." The scheduled runs moved to
  05:58, 11:58 and 16:58 Bangkok; run 1 started at once.
- 11-09 evening: finish 3D this week, so that week 1 has more time for a
  playful 3D demo and feature sprints (plan in state.md).

## 2026-11-09

- 18:04 Bangkok: weekly limit at 15 %. tokens.py estimated 15.2 % for the
  day at the same time (seen 644,447,392 cache read, 2,597,917 output).
  With 13 % against 13.8 % at 17:19, the day's estimate matches the
  weekly reading, so the week stood near 0 % at the start of the day,
  and the 2 % of 10:26 is not a separate offset.

- 17:19 Bangkok: weekly limit at 13 %. tokens.py estimated 13.8 % at the
  same time (seen 580,229,950 cache read, 2,346,207 output).

- Define the design principles, core values and philosophy that the UX/UI
  follows: `docs/design-principles.md` (e22ce6b), linked from charter 5a.

- The ACP chat shall be themed and integrated into the matching UI, for
  example the agents view of mc and the assistant figure of xp.

- Force push allowed for the author rewrite. Then: use the user's GitHub
  name and email as in all previous commits; rewrite and force push once
  more (done: 130 commits on the code branch, 48 on session-log).

- Sharpen the plan: a UX/UI design agent sharpens every user-facing
  design before programming starts and takes part in the review rounds.
  User tests always report UX/UI flaws from the persona's view. Done in
  charter sections 5, 5a, 3 (Review) and 8, `feedback/README.md` and
  `triage.py`.

- 14:15 Bangkok: 11 % of the weekly limit; tokens.py estimates 11.8 %, within the rounding; the calibration of 13:11 holds.

- The real meter is not reachable from the session (no token in the
  container; the Admin API covers API organizations, not the
  subscription). Cross-check from the session's event log, cumulative
  cost at list price: 3 % at $24.27, 6 % at $65.60, 8 % at $90.26, 9 % at
  $112.09; about $12 to 15 per percent lately, not linear (the meter may
  count other use of the account and lag).

- 13:11 Bangkok: 9 % of the weekly limit. Calibration now weights the
  tokens by model.

- Question 6 answered: make it configurable; the user shall be able to
  raise grants temporarily or permanently.

- 12:30 Bangkok: 8 % of the weekly limit. Week totals 273,598,608 cache
  read, 1,279,909 output. Calibration: 32 million cache read per percent.

- Focus on 3D, plugin separation, stability and bug fixes before
  starting new feature sprints (charter section 1 item 20).

- All user agents report issues and draft feature requests; the PM
  reviews, ranks and addresses them by importance; cheap agents run
  user tests continuously, big rounds for demos and features.

- For r/unixporn: a heavy-user scenario that pushes the concept to its
  limits and shows something unseen; the user tests get this "push to
  the limit" view without dropping the low-tech views.

- Brainstorm on panes (recorded as D76, 8b567b5): a pane is a view of a
  source (window, window manager on a virtual output, plugin view,
  prompt); several panes may show one source and every pane takes
  input; geometries (cube, cylinder, wall, slider, more) are small
  plugins with one interface; the slider is a geometry of the construct
  (D67 revised); "globe in XP" meant the XP cube: leaving XP flies out
  to the whole cube with its windows. Launching the construct from XP
  and a pane with tiling are K6 and K7.

- Hard requirement: windows move flawlessly between all window
  managers (construct, mc, xp, tiling) by key bindings, automation,
  voice or whatever the user wants; easily extensible (charter section
  1 item 19).

- 11:48 Bangkok: 6 % of the weekly limit used. Week totals: 180,125,337
  cache read and 897,769 output tokens. Calibration updated.

- Every plugin is considered for a split once it reaches a certain
  size or feature set (charter section 1 item 18).

- Question 3 answered: the split of `design/split.md` (four
  repositories) is accepted for now; finer splits only if they stay
  simple. Prepare it, do not split.
- POLS and KISS are core values and a view of every review round.
- Check the plugins regularly for code that can be separated and
  generalized into its own plugin.
- Correction: the plugin to split is chora-xp, not chora-mc (a
  transcription error). The XP split is planned in D73 and the roadmap
  track X1 to X7; it goes into the backlog. The mc replan was stopped.

- Review rules too tight (100 lines burns tokens): rounds at merges,
  tags, the end of the week and when critical components changed; no
  rounds for routine work, sprint prototypes, test rounds and demos; an
  automatic suggestion after some time (`review_due.py`).
- Delete stale branches (done for `claude/report-media` locally; the
  proxy refuses remote deletes).

- Every rule shall be guarded where possible; manual checks are the
  exception (charter section 1 item 15, `guards.md`).

- No inline comments in code: names make it readable, docs carry the
  rest; no anecdotes or storytelling in docs; the PM's decisions and
  friction go into the reports, not the docs.

- Fix the pre-commit hooks instead of working around them.
- Prepare the split of the repository into a few meaningful repos (not
  one per plugin); decide the split, do not split yet; unrelated
  branches are fine for clean parts such as the website.

- Reports: a small report per session (run), a longer one per week;
  every decision clearly stated, every resource detailed, screenshots,
  screencasts, GIFs; transparent, critical, no overselling, always the
  next steps (charter section 11b).

- Daily reflection or read-through rounds: separate opus agents
  criticize the PM's work, to keep it aligned with the goals and its
  decisions sane (charter section 1 item 14, section 11a).
- Share the build cache and drop debug info to save disk.

- 10:33 Bangkok: weekly model limit at 3 %. Tokens up to then (all
  dates): 54,636,124 cache read and 395,192 output. This replaces the
  calibration of 11-08, which used the 5-hour limit.

- 10:59 Bangkok: correction: the weekly limit still resets on Saturday
  at 10:59 Bangkok time; the 2 % of 10:26 may be a wrong number. Raise
  the daily limit for this week accordingly (19 % a day until the reset).
- 10:26 Bangkok: the weekly limit seems to have reset; 2 % used now.
- Align the working times to Thailand (Asia/Bangkok). Start the work now.

- Feature request (evening): ACP chats shall be possible. HUD drafts for
  mc, xp and construct. The voice agent HUD becomes more flexible and
  adapts to the context. Users can easily ship their own HUDs. This is
  the first feature sprint, with user tests. The user opens it before
  the foundation gate of charter section 1 item 20; the foundation work
  keeps running in parallel and keeps priority on merges.

## 2026-11-08

- Reply to the user in English from now on.
- Look at the Dependabot PRs too (charter section 15, item 7).

- 17:44 UTC: the planning session has used 4 % of the weekly limit so
  far. Token totals at that time: opus-5-5 151 input, 329,453 cache
  write, 14,276,774 cache read, 93,298 output; sonnet-5-5 96 input,
  535,364 cache write, 3,045,660 cache read, 70,524 output.

- Calibration: the user reports the used share of the weekly limit in
  the session chat during the first days; each report is added here.

## 11-11 (chat, copied late; friction 33)
- 68% at 15:08 UTC; 71% at 16:42 UTC (weekly limit, all models).
- Use the remaining budget for mass haiku user tests, data only, no fixes yet; haiku swarms may run up to 85% weekly use while they find new issues (not a week cap; user correction 11-11), the rest for major fixes and the review loop. Haiku must use far fewer tokens; keep testing only while new data arrives.
- The reset is Saturday 10:59; after the reset heavy work runs on Fable until it is used up ("no more fable for this week", 13:13 Bangkok), and the user resets again.
- After the RC (user, earlier 11-11): an extensive user testing round. Release authorized when all is clean.
- Context management leaks after compaction: note as a major issue; the PM session must stay robust (friction 33).
- 71% at 17:02 UTC (user); usage.py estimate since 16:42: 1.88 M units, i.e. under 1%, consistent.
- Correction (user, 11-11 ~17:55 UTC): 85 % is not a cap for the week; it is the limit up to which the haiku test swarms may run while they keep finding new issues. The rest of the week goes to major fixes and the review loop.
- 11-11 ~18:10 UTC (user): a subagent screens the transcripts and the session log files for every user decision, and the PM hook is made more robust.
- Copied late by the transcript screen (11-11, see decisions-now.md for the verbatim quotes): no more Fable this week, second Fable reset next week; respect the 5-hour limit with up to four heavy lanes; rc reflection loop and opus for reflections this week; voice uses local, interchangeable models with configurable endpoints; construct HUD rework details (glass, no buttons, bindings overlay, shareable plugin); xp balloon behaviour; "Close stale ones".
- 11-11 ~18:30 UTC: swarm 4 brought no new findings; the swarms stop per the user's rule.
- 11-11 ~18:40 UTC (user, verbatim in decisions-now.md): goal of this week is UI/UX polish toward perfection; design agents per look; tiling Hyprland-like and hackable; mc helper into the right notification margin.
- 11-11 ~18:50 UTC (user, verbatim in decisions-now.md): xp and construct design passes in parallel; the PM decides taste and everything else autonomously until Saturday; no user intervention; self-adapt to frictions, keep logging.
- 11-11 ~19:00 UTC (user, verbatim in decisions-now.md): product brief per plugin (deliverables, requirements, USPs, target groups); clear worker instructions.
- 11-11 ~19:20 UTC (user, verbatim in decisions-now.md): stay focused on the main targets, aim for perfection.
- 11-11 ~20:00 UTC estimate (usage.py): 38.3 M units since the 71 % reading = about 14 %, so about 85 % plus the tour worker; pace about 4.5 %/h with three heavy lanes. PM pacing: finish tiling and mc (round 3, review, merge train) and the construct worker; xp waits; when the weekly limit stops the session, resume at the Saturday reset (send_later).
- 11-12 ~01:00 UTC (user): 87 % weekly use (usage.py estimate of ~95 % was high: the 2.8 M units per percent calibration overstates; recalibrate from this reading). Status page gets screenshots and a screencast per look as it lands (verbatim in decisions-now.md).
- 11-12 ~01:40 UTC (user): archive stale sub-sessions; arm a watchdog to keep the PM going; start parallel lanes now (verbatim in decisions-now.md).
