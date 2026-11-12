# Standing user decisions (verbatim quotes; injected after every compaction)

Every user decision in chat goes here the same turn, quoted word for word,
with date and the PM's reading. Remove an entry only when the user replaces it.
answers.md stays the full log; this file is the current set. Keep it under 9,000 characters (hook cap 10,000).
Times Bangkok (UTC+7). Charter section 1 still holds where not replaced below.
screened-through: 2026-11-12T02:11:00Z (bump to the newest user message recorded; tools/pm_context.py lists newer ones)

## Budget and models
- 11-10 15:20 "Don't be too paranoid about the limit. We shall not waste it but make sure work gets finished and weekly limit gets fully spend"
  Reading: pacing guide, not a cap; "don't mindlessly burn tokens".
- 11-10 16:20 "Respect 5h limit in your planning?" 16:46 "Why cap yo only two hevy workers? ... We have stable sub sessions now"
  Reading: the 5-hour limit is the real pace; at most four heavy lanes (charter 10).
- 11-11 13:13 "Fable used up. Ive reset. Finalize, then no more fable for this week"
  Reading: no new Fable lane until the weekly reset Saturday 10:59.
- 11-11 17:56 "Got another fable reset for next week. So we can haven other full fable round next week."
- 11-11 23:53 "Haiku should use considerable less tokens. Only keep testing as new data arrives. I think limiting to 85% shall be sufficient. Reserving the rest for major fixes and the review loop."
- 11-12 00:29 "No, its not this weeks cap. Its the cap for the user test swarm if they keep discovering new issues etc"
  Reading: haiku swarms may run until weekly use reaches 85 %, only while they find new issues. NOT a week cap: the rest goes to major fixes and the review loop.
- 11-11 23:53 "Reset is sat 10:59. Well then finalize this week with heavy tasks layer off yo fable. Resetting once its fully used."
  Reading (garbled dictation): after the Saturday reset, heavy work (major fixes, review per area) runs on Fable until used up; then the user resets it. No Fable before (13:13).
- Usage readings the user posts go to answers.md the same turn (charter 1.2).

## Communication
- 11-12 ~08:00 "87 %. Update status page with screenshots and screen cast when new looks are ready"
  Reading: each look that lands gets screenshots and a screencast on the status page.
- 11-09 00:48 "Antworte ab sofort immer english"
- 11-10 07:12 "Keep summaries here to a minimum to save tokens. I have the status page, or asking if I need info"
  Reading: chat replies one or two lines; each run ends with done, running, next, blocked, budget (charter 11).
- 11-11 18:21 "Rework status page to cluttered ... folding or tabbing section. Post detailed screenshots and screen cast showcasing all features"
- 11-09 11:16 run and week reports: "Don't overstate anything here, don't oversell it. Be critical" (charter 11b)

## Git and files
- 11-09 11:24 "All your separate files shall go on a separate unrelated branch ... This shall never touch the main branches or dev branches." Reading: claude/session-log.
- 11-09 11:22 "avoid inline comments ... We don't want to have any anecdotes or storytelling inside the docs."
- 11-09 11:27 "every rule we define shall be guarded when possible. Manual checking rules shall be the exception" (guards.md)
- 11-09 11:20 "Fix them instead of working around." (pre-commit hooks) Reading: never --no-verify.
- 11-09 15:20 "You shall use my github email and name as in all previous commits." Reading: no AI trailers; stage by name, never git add -A (charter 3).
- 11-09 11:33 "You may delete stale branches."

## Testing
- 11-08 23:58 "viele verschiedene persänlichkeiten (insbesondere auch wenig technisch affine)"; 11-09 00:28 everyday tasks ("Wi-Fi-Netzwerk verbinden")
- 11-09 12:05 "The user test shall include this "Push to limit" view without sacrificing the low tech views."
- 11-09 12:11 "All user agents shall report issues and draft feature request. You shall review rank and address them by iprtance."
- 11-09 15:07 "User tests shall allwats report ux / uiui flaws from their view" (feedback/README.md; haiku 640 px, 30 steps)
- 11-11 17:53 "After the release candidate, we need an extensive user testing round to really ensure that everything is going smoothly."

## Release and order of work
- 11-11 14:04 "When all merged and user reports are in have a ux/ui loop over the current featured set." 14:06 "Emphasis on kiss/minimalist design with pols. When clean feature sprints can start"
- 11-11 17:30 "After ui ux clean. Finalize foundational work and review code until ready for rc release before feat sprints."
  Reading: UX/UI loop clean, then foundation and code review, then rc, then feature sprints (refines charter 1.20).
- 11-11 17:31 "You may released when all clean. Review code for simplicity and readability."
  Reading: replaces charter 1.4 for the rc: the PM opens the CI PR, tags and publishes the rc when design/rc.md holds.
- 11-11 18:03 "After rc gates passed: have intense reflection loop and sharpen our charter etc for the next days"
- 11-11 18:11 "Ensure good an detailed logging in all components"; anonymised report script, later paste bin + GH issue text (backlog L1)

## Design
- Older design and process decisions already built (cube floating windows, ACP chat themed, grants, split reviews, status page form): see answers.md.
- 11-09 15:28 "Minimal/flawless/flexible" (docs/design-principles.md); POLS and KISS core values (charter 1.16).
- 11-11 18:30 "different ... design documents for all our different components and window managers" (charter "Design per look")
- 11-11 18:21 "Ui reviews shall hunt for perfectionism"; 18:26 "Such things shall be caught by ui agent"
- 11-11 18:26 xp HUD: "click on balloon expands it to chat. Confirmed or normal replys show in ballow. ... No other balloons next to it."
- 11-11 19:45 "expensive blurred glass ... minimal yet complete. Avoid buttons. Keyboard / mouse focus. Add overlay to show all bindings. Shared the able plugin all other wms might also use."
- 11-11 20:17 "Go with your defaults for the HUD mockups"; reflections "like polished marbal", 20:19 "small scratches and bumps"; backlog: osx-like theme on xp code.
- 11-11 12:22 "Lets make it impressive demos AS features Land. Polished ... Decide such things on your own from now"
- 11-11 18:01 voice: "local models, but all of them shall be interchangeable ... Still api endpins shall be configurable."

## PM process
- 11-12 ~02:10 UTC "Wake up more often? All 30 min eg?  What about cloud workers? No visual polishing and test swarms planned anymore?"
  Reading: wake often; cloud workers; polish and swarms on the new looks.
- 11-12 ~01:40 UTC "Is construct worker still run ng? Remove stale sub sessions. Arm a watchdog timer to Lee you going. What ere your next steps? What can be started right away in parallel?"
  Reading: archive finished worker sessions; a recurring wake-up keeps the PM working; start parallel lanes now.
- 11-11 23:59 "Reread the whole charter after compactation. ... The pm session hast to remain robust"
- 11-12 00:27 "you did not pick up work after compactation" Reading: resume the first open item at once (pm-card step 4).
- 11-12 00:30 "Keep note of all decition and read them after compacting. You are the pm. You must keep an overview"
- 11-09 11:13 daily opus reflection (charter 11a); 11-11 18:34 "Make sure you note these frictions and problems."
- 11-11 11:43 "Check you sub session. All running? Close stale ones" Reading: check at each run start.
- 11-10 14:45 "Agreed, go with the hybrid" Reading: workers for heavy lanes and testers, merges only here; one persistent PM.
- 11-11 ~18:40 UTC "Polish the ui ux towards perfection is the goal for this week. ... Mc shall rather include in the right margin where notifactions are shown anyway instead of stealing half the screan. Make tiling more exiting and "hyperland like" it shall stay hackable ... stunning visuals in each view all with their own design philosophy" (full text in answers.md)
  Reading: polish toward perfection; a design agent per look; each look its own aesthetic.
- 11-11 ~18:50 UTC "Not deciding anything now. You work autonomously until sat. ... take wise decitens. Self adapt to frictions and keep logging your progress."
  Reading: the PM decides taste until the user returns (overrides reflection 2 rule 4), each decision logged and reversible.
- 11-11 ~19:00 UTC "Get you head around what each plugin shall deliver. Define clear requirements USPS and target groups for each. Be a wise leader and give clear instructions to your workforce."
  Reading: the PM writes a product brief per plugin (what it delivers, requirements, USPs, target groups) and briefs every worker from it.
- 11-11 ~19:20 UTC "Stay focused on our main targets and try to deliver the best results. You as aiming to words perfection"
  Reading: focus on the main targets (polish of the four looks, RC requirements in design/product-briefs.md); quality over quantity; aim for perfection.
