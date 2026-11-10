# Plan data

The data the PM uses to sharpen the plan (user request 11-10: note the
decisions and sharpen the plan as data and experience grow). Updated at
the end of every run; the weekly retro turns it into plan changes.

## Velocity, 11-10 (06:28 to 12:30 Bangkok, about 10 % of the week)

| Kind | Count | Notes |
|---|---|---|
| Core steps merged (3D 2 to 6, H4c) | 6 | planned for Tue to Fri; done by 11:44 |
| Kit, plugin and service steps merged (H1 to H6, H4a, H4b) | 8 | |
| Fixes of real bugs found by flaky tests | 2 | tiling focus race, mc clock |
| Merges through merge-gate.sh | 9 | 2 stopped correctly (flakes), 1 refused a dirty tree |
| Review rounds | 13 | each found something; high findings in 7 |
| Container restarts | 2 | no work lost after the WIP-push rule |
| Disk stops | 2 | rule: at most 3 building lanes |

## What the data says

- One step with its review and fixes takes about 1 to 1.5 hours of wall
  time and about 0.5 to 1 % of the week. The plan of one 3D step a day
  was four times too slow; plan in half days.
- Reviews pay: 7 of 13 found a high problem before the merge (security
  or freeze). Keep a review on every merge; opus for core and security.
- The developers skip red-first when not forced (friction 22); the red
  commit rule is new.
- Flaky tests were real bugs twice; never rerun to green.

## Plan changes from this data (11-10)

- 3D base done three days early; week 0 gets the construct on 3D (K5),
  the shared effects driver (E1) and 3D wobble (E2), the HUD sprint to
  its end, and the stability backlog.
- Week 1 headline: the construct demo with wobbly windows out of the XP
  cube (design/wobbly-3d.md), then the windowed playground and distrobox
  image, the website and USPs, then the effects pack.
