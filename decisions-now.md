# Standing user decisions (verbatim quotes; injected after every compaction)

Every user decision in chat goes here the same turn, quoted word for word,
with date and the PM's reading. Remove an entry only when the user replaces it.
answers.md stays the full log; this file is the current set. Two hooks print it (split at the PM process heading; Design is last); keep each part under 9,000 characters (hook cap 10,000).
Times: UTC unless marked Bkk; undated-zone entries before 2026-11-11 12:00 are Bangkok (UTC+7). Charter section 1 still holds where not replaced below.
screened-through: 2026-11-12T04:29:58Z (bump to the newest user message recorded; tools/pm_context.py lists newer ones)

## Budget and models
- 11-12 02:24 UTC "Plan a detailed analysis where tokens went this week to be more efficient in the next week."
  Reading: superseded by 02:43 (plan = backlog); done anyway: reports/token-analysis-week1.md.
- 11-12 02:21 UTC "Burning tokens quickly. Prefer sonnet and haiku  now where possible to last through the day."
  Reading: opus only where nothing else fits (none planned today); reviews and builds on sonnet, swarms on haiku; PM turns short.
- 11-12 02:17 UTC "Okay, good. We're at 90% now, when all the running tasks are finished and everything is merged and clean. Finish this week off with intensive UX/UI reviews and testing to collect as much data and find as much frictions, bugs, and inconsistencies that we can address next week. One thing I just noted on the piling screenshots, there seems to be still a floating chat button. Maybe this is still an old state, and have the UX/UI agents intensively visually validating our individual views and ensure that we are following consistent design languages and giving the best user experience."
  Reading: finish and merge running lanes; then UX/UI reviews and testing to the week's end (data for next week, few fixes).
- 11-10 15:20 "Don't be too paranoid about the limit. We shall not waste it but make sure work gets finished and weekly limit gets fully spend"
  Reading: pacing guide, not a cap; "don't mindlessly burn tokens".
- 11-10 16:20 "Respect 5h limit in your planning?" 16:46 "Why cap yo only two hevy workers? ... We have stable sub sessions now"
  Reading: the 5-hour limit is the real pace; at most four heavy lanes (charter 10).
- 11-11 13:13 "Fable used up. Ive reset. Finalize, then no more fable for this week"
  Reading: no new Fable lane until the weekly reset Saturday 10:59.
- 11-11 17:56 "Got another fable reset for next week. So we can haven other full fable round next week."
- 11-11 23:53 "Haiku should use considerable less tokens. Only keep testing as new data arrives. I think limiting to 85% shall be sufficient. Reserving the rest for major fixes and the review loop."
- 11-12 00:29 Bkk "No, its not this weeks cap. Its the cap for the user test swarm if they keep discovering new issues etc"
  Reading: haiku swarms may run until weekly use reaches 85 %, only while they find new issues. NOT a week cap: the rest goes to major fixes and the review loop.
- 11-11 23:53 "Reset is sat 10:59. Well then finalize this week with heavy tasks layer off yo fable. Resetting once its fully used."
  Reading: after the Saturday reset heavy work runs on Fable until used up.
- Usage readings the user posts go to answers.md the same turn (charter 1.2).

## Communication
- 11-12 01:26 UTC "87 %. Update status page with screenshots and screen cast when new looks are ready"; 01:42 UTC "Still at 87. Don't be to paranoid about token limit"
  Reading: report the user's reading; do not hold work back for budget.
- 11-12 01:45 UTC "Don't rely on me. I was just assisting this week, you shall work autonomously from now. Improve your estimate or respect ist uncertainty bit don't hold back work tokens shall be used up by end of the week. Still don't waste them"
  Reading: full autonomy; estimate per rules.md 10; use the week fully, no waste.
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

## PM process
- Done and moved to answers.md: 11-12 01:29, 01:35, 01:41, 02:12 UTC; 11-11 11:43, 11-09 11:13 (sub-session checks, wake-ups, the reflection loop).
- 11-12 02:43 UTC "Alright, I changed the auto compactation to 300K. Next time I say plan, I really need mean planning. Put it to the backlog and address it when we have the resources for that. Honestly, you need to get some kind of a stronger personality. Really push back when you see that certain task is useless or not necessary right now. We really need to prioritize and push this project forward, design in the interests of the project, and get better structured. This is something we want to address next week. Write down the report and the findings of the token usage profile. And we will start Saturday with a big reflection using Fable to really improve our setup and be very, very critical about the overall progress we had so far."
  Reading: "plan" means a backlog entry, not running the work; the PM pushes back with reasons on tasks that are useless or not needed now, and prioritizes what moves the project forward; structure and priorities are next week's work; the token report is written (reports/token-analysis-week1.md); Saturday starts with a big, very critical Fable reflection on setup and progress. Auto-compaction is at 300k.
- 11-12 02:13 UTC "Don't take my word for granted."
  Reading: the user's suggestions are input, not orders to follow blindly; the PM weighs them against evidence and pushes back with reasons when they conflict (decisions stay verbatim; the PM's reading may disagree and says why).
- 11-12 02:09 UTC "Establish regular routines triggered by time or certain conditions. Also, use a time trigger to regularly reflect on your work and ask you several questions like, am I still on track? Do I need new data to have new decisions? Do I need to run a cleanup? Do I need to run tests? Am I still aligned? Am I using time and resources efficiently? Do I need another reflection round? Be self-critical and try to be the best project manager and learn from your own faults."
  Reading: routines.md lists time and condition routines; a 3-hourly self-check trigger asks these questions and logs answers and actions.
- 11-11 23:59 Bkk "Reread the whole charter after compactation. ... The pm session hast to remain robust"
- 11-12 00:27 Bkk "you did not pick up work after compactation" Reading: resume the first open item at once (pm-card step 4).
- 11-12 00:30 Bkk "Keep note of all decition and read them after compacting. You are the pm. You must keep an overview"
- 11-10 14:45 "Agreed, go with the hybrid" Reading: workers for heavy lanes and testers, merges only here; one persistent PM.
- 11-11 17:48 UTC "Polish the ui ux towards perfection is the goal for this week. ... Mc shall rather include in the right margin where notifactions are shown anyway instead of stealing half the screan. Make tiling more exiting and "hyperland like" it shall stay hackable ... stunning visuals in each view all with their own design philosophy" (full text in answers.md)
  Reading: polish toward perfection; a design agent per look; each look its own aesthetic.
- 11-11 17:50 UTC "Not deciding anything now. You work autonomously until sat. ... take wise decitens. Self adapt to frictions and keep logging your progress."
  Reading: the PM decides taste until the user returns (overrides reflection 2 rule 4), each decision logged and reversible.
- 11-11 17:53 UTC "Get you head around what each plugin shall deliver. Define clear requirements USPS and target groups for each. Be a wise leader and give clear instructions to your workforce."
  Reading: the PM writes a product brief per plugin (what it delivers, requirements, USPs, target groups) and briefs every worker from it.
- 11-11 17:54 UTC "Stay focused on our main targets and try to deliver the best results. You as aiming to words perfection"
  Reading: focus on the main targets (polish of the four looks, RC requirements in design/product-briefs.md); quality over quantity; aim for perfection.

## Design
- 11-12 04:29 UTC "from now on, install those assets, use them in all your tests and screenshots to really benchmark on the pixel perfect version, while regularly checking all fallbacks are also releasing acceptable results. Regarding the XP thing, still understanding as a opinionated theming or gimmick type of flavor, where the underlying mechanics, the stacking window manager, and the drawn widgets shall be available as a generalized API. So at a later point, we want to use this as the groundwork to, for example, mimic a Mac OS-like desktop experience or whatever the user wants. So XP is a showcase that is trying to impress and show as possible while still being a production-ready and helpful set of plugins and API that we can use to build on other kind of desktop experiences. So in this understanding, the taskbar and start menu shall become some kind of widgets, and it must be possible to replace those with a, for example, OS X like dock and a top taskbar or something completely different, as long as we stay in a stacking or floating window manager philosophy. Do you confirm and agree to this? And is this already noted down on the roadmap?"
  Reading: design/xp-reference.md; backlog N1 (X4 panel = widget host).
- 11-12 04:27 UTC "Okay, then look through my repositories. There's an KDE repository that is doing a pixel perfect XP replication on KDE Plasma 6. We there use and have a migration script that is building upon the XFCE/TC repository. You shall find all the necessary information there. Make sure you note it so this information is not lost."
  Reading: design/xp-reference.md (reference and import sources).
- 11-12 04:25 UTC "So you may run until you run out of tokens, but don't waste them. Try to invest the remaining budget as good as possible to make the best out of the remaining week. Reset is on Saturday morning. Regarding the Microsoft-owned material, we had migration script to copy over files as an optional step for users from the XFCE total conversion repository, including the iConcept, XP sounds, and the background images. Everything else we are building shall focus on perfect behavior replication and a pixel-perfect experience when those assets are copied into place. Confirm"
  Reading: use the budget fully, no waste; no Microsoft art shipped; xp pixel perfect with the imported assets, whole with our fallbacks (design/xp-reference.md).
- 11-12 04:20 UTC "Xp goal is pixel perfect replication and ux similar yo original. Mc shall be a modern voice first computing experience where content shapes the view, following traditional typesetting rules. Sharpen plugin goals accordingly. Critically reflect on thus and any other instructions. Push back when out of budget miss leading and roast me in unclrities their shall be no gaps in our specifications."
  Reading: design/product-briefs.md xp and mc rewritten, with a "Spec gaps" list and PM defaults; the xp legal line goes to Needs you.
- 11-12 04:16 UTC "Did you already have the visual reviews over the new versions of our reviews? I already realized there is a glitch or a error in the XP start menu on the user icon, and also the tray icons look a bit off. For MC, the vertical bar should have the heights of the text. These are the kind of visual defects we want to automatically capture. We are aiming for perfect aesthetics. Have that on your rule list and ensure that we are having visual reviews for every remake until it's polished to perfection."
  Reading: rules.md 8 (full pixel review of every remake until only COSMETIC; audit cells per flaw class).
- Older design and process decisions already built (cube floating windows, ACP chat themed, grants, split reviews, status page form): see answers.md.
- 11-09 15:28 Bkk "Minimal/flawless/flexible" (docs/design-principles.md); POLS and KISS core values (charter 1.16).
- 11-11 18:30 UTC "different ... design documents for all our different components and window managers" (charter "Design per look")
- 11-11 18:21 UTC "Ui reviews shall hunt for perfectionism"; 18:26 UTC "Such things shall be caught by ui agent"
- 11-11 18:26 UTC xp HUD: "click on balloon expands it to chat. Confirmed or normal replys show in ballow. ... No other balloons next to it."
- 11-11 19:45 UTC "expensive blurred glass ... minimal yet complete. Avoid buttons. Keyboard / mouse focus. Add overlay to show all bindings. Shared the able plugin all other wms might also use."
- 11-11 20:17 UTC "Go with your defaults for the HUD mockups"; reflections "like polished marbal", 20:19 UTC "small scratches and bumps"; backlog: osx-like theme on xp code.
- 11-11 12:22 UTC "Lets make it impressive demos AS features Land. Polished ... Decide such things on your own from now"
- 11-11 18:01 UTC voice: "local models, but all of them shall be interchangeable ... Still api endpins shall be configurable."
