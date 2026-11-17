# Demo playbook "A morning with chora" (draft 1, owner 11-17 11:47; to refine with the owner)

One take, no cuts, real apps, real voice. Each beat has a caption overlay (2-3 words plus one line on
the specific: "one live API", "windows keep state across looks"). The playbook is also an e2e test:
`demo/playbook.yaml` drives choractl and voice; it runs in the gate with captures, and the recording
is only made from a green run. Target 3-4 min, 1080p60, real hardware (llvmpipe only for the gate).

| # | Beat | Look | Shows | Needs (backlog) | Exists today |
|---|---|---|---|---|---|
| 1 | Morning: agenda, mails, morning brief as cards; top task "record a chora screencast" | mc | content shapes the view, voice first | calendar and mail sources for the agent (C0), brief card | mc cards, voice chat; no mail/calendar source |
| 2 | Start the screencast from the task card | mc | agent acts, asks before risk | screencast door with confirm | screenshot door; recorder is harness-only |
| 3 | Fly out to construct | construct | handover, no lost window | DB12 one cube | handover works; seams co-21 open |
| 4 | "Open a tiling desktop" by voice; show it | tiling | voice arranges the screen; Hyprland motion | voice tool for look switch | look switch by key; voice spawn not wired |
| 5 | Fly out, turn the cube to xp, play with it; show off xp (Start, tray, Control Panel, Notepad) | xp | pixel-perfect replica, windows move across | DB12, X10-X12, N2 | xp cube in xp; tray/menu partial |
| 6 | Back in construct; switch to dark mode | construct | one setting, all looks | N2 settings service | construct theme in config only |
| 7 | Spawn a cylinder geometry, blurred background | construct | programmable scene | A14 geometries | cube only |
| 8 | Slide panes with facts about chora on the cylinder | construct | panes as content | A14 panes per source | none |
| 9 | Burn everything down to ashes | all | effects as plugins | S3 effects pack (cut 11-10 to after 1.0; owner reopens?) | none |

Rules: no beat may need a hand edit or a cut; every beat's state is asserted by a gate cell; captions
come from the playbook, not the recorder. Order of work: foundation (this week) -> UX polish to the
playbook beats -> features the beats need -> record. No demo-only code: every beat uses a shipped path.
