# Standing owner decisions: time UTC, sentence verbatim -> reading. Full log: answers.md
screened-through: 2026-11-14T17:02:28Z

- 11-14 16:54Z (fixes 16:47) "Fable darf gerne aufgebraucht werden. Ich löse den Reset aus." "...sollten wir weniger parallele Fable-Lanes fahren als bisher." -> use Fable up this week: max 2 Fable lanes, then none until next week
- 11-14 17:02Z "der Dev-Branch gilt mit unseren lokalen Überschreibungen" -> agent_skills dev plus overrides = conventions.md (language and chat length there)
- 11-14 16:43Z "...robuster gestalten, damit sowas nicht nochmal passiert." -> fail-open hooks, no local lanes, heartbeat; 16:47Z "Ja, das passt so." -> plan ok
- 11-14 05:33Z "try to really parallelize what's possible" -> no rc cuts; parallel lanes
- 11-12 05:47Z "So stability and the aesthetics, looks, UX and UI, shall always win over new functionality. Keep our POLS and KISS principle at the highest priority for our guidelines." -> priority; "Your job should be to collect those, put them in the backlog if they are feasible, push back if they conflict with any of our decisions or targets, and always keep the project goals into focus." -> ideas never redirect a lane; "You shall be focusing on the project goals and keep them reachable, or discuss with me if they need adaptation." -> name unreachable goals
- 11-12 02:42Z "Next time I say plan, I really need mean planning. Put it to the backlog and address it when we have the resources for that." -> plan = backlog; "Really push back when you see that certain task is useless or not necessary right now." -> push back, also on his words
- 11-12 01:45Z "Don't rely on me." -> decide the reversible
- 11-11 10:30Z "After ui ux clean. Finalize foundational work and review code until ready for rc release before feat sprints." -> order: UX/UI, code review, rc, reflection, sprints
- 11-11 10:31Z "You may released when all clean." -> PM opens CI PR, tags, publishes when rc.md holds
- 11-11 10:53Z "After the release candidate, we need an extensive user testing round to really ensure that everything is going smoothly." -> persona round 5
- 11-09 04:24Z "This shall never touch the main branches or dev branches." -> session files only on claude/session-log
- 11-09 08:20Z "You shall use my github email and name as in all previous commits." -> his identity; no AI lines
- 11-09 04:20Z "Fix them instead of working around." -> hooks: never --no-verify
- 11-09 05:05Z "The user test shall include this "Push to limit" view without sacrificing the low tech views." -> both persona kinds
- 11-09 05:11Z "All user agents shall report issues and draft feature request. You shall review rank and address them by iprtance." -> by severity
- 11-14 04:51Z "agree with decitions." -> Needs owner defaults (a)-(d); 05:25Z: no construct preview
- 11-12 04:20Z "Xp goal is pixel perfect replication and ux similar yo original. Mc shall be a modern voice first computing experience where content shapes the view, following traditional typesetting rules."
- 11-12 04:26Z "Everything else we are building shall focus on perfect behavior replication and a pixel-perfect experience when those assets are copied into place." -> MS assets: optional user import, never shipped; RC on fallback art (11-14)
- 11-14 05:30Z "I would rate the first three as rather important features that we want Might to ship with a 1.0 release" -> after rc: B2-B4, C0/A15/V1-3, L1b for 1.0; 1-2 demo lanes
- 11-11 17:48Z "Make tiling more exiting and "hyperland like" it shall stay hackable and easy yo extend but we want stunning visuals in each view all with their own design philosophy and esthetics consequently followed." -> each look its own aesthetic
- 11-11 12:45Z "Make it minimal yet complete. Avoid buttons." -> construct HUD: glass, bindings overlay
- 11-12 04:16Z "These are the kind of visual defects we want to automatically capture." -> flaw class seen by eye: an audit cell; one pixel pass per look
