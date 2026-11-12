# Vision card: what chora is, for whom, and why (injected after every compaction)

Read before deciding anything. If a task cannot be traced to a line here,
question it before spending budget. Sources: docs/vision.md (chora repo),
design/product-briefs.md, docs/xp-split.md, design/xp-reference.md,
decisions-now.md. Update this card the same turn a goal changes.

## The project in one breath
chora is a Wayland compositor that is a programmable scene, not a window
manager: the core has no policy; plugins, scripts and agents bring the forms
through one live API. Motivation: an agent (ideally by voice) arranges the
screen for the task. Proof of range: four complete looks in one session,
switched live, windows moving between them without loss.

## Order of work (user 11-11)
UX/UI loop clean (this week: polish toward perfection) -> foundation and code
review -> rc (design/rc.md; PM may tag) -> intense reflection -> feature
sprints. Week 2 starts Sat 11-14 with a critical Fable reflection (W2-1).

## Components: purpose and target group
- **core**: lean, safe (escape hatch no plugin overrides), one API, live.
  Everything else is a plugin.
- **chora-xp**: SHOWCASE and GROUNDWORK. Goal: pixel-perfect XP SP2/SP3 Luna
  replica and XP-like UX, benchmarked WITH the imported Microsoft assets
  (optional user-run import, model: kde-plasma-winxp-launcher
  get-microsoft-files.sh); our fallback art must stay acceptable. Never ship
  Microsoft files. Under it: generic plugins (chora-stack stacking WM,
  chora-desk, chora-panel as a WIDGET HOST, chora-draw) so a macOS-like dock
  plus top bar, or anything stacking/floating, is a config, not a rewrite
  (roadmap X1-X7, backlog N1). Users: people who know XP (Erika 74, Murat 52,
  mouse only); nostalgic pixel-watchers. Accessibility: XP's Large size.
- **chora-tiling**: the hacker's desktop. Hyprland-class motion and looks,
  plain-text hackable config. Users: keyboard-first power users, ricers;
  beginners must not be stranded.
- **chora-mc**: voice-first computing. Content shapes the view (page form
  per content kind), traditional typesetting rules (baseline grid, measure,
  bars cap-to-baseline). Remote and mouse are a full second path. Users:
  hands-busy people (kitchen, sofa, nurse), low-tech users at distance.
- **chora-construct**: the 3D space; live desktops on cube faces, step out
  and in, pull windows across. Glass, calm, minimal HUD. Users: explorers,
  demo watchers, ricers. Not: VR, physics.
- **chora-assistant + chat**: one helper, many costumes per look; asks before
  risky acts; honest states (working, no helper). Everyone.
- **chora-notify**: quiet, never covers work; placed per look.
- **services/voice**: local, interchangeable models, configurable endpoints.

## Shared RC bar (every look)
Visible way to start a program, the key list and the helper (mouse and
keys); no stray keys into clients; lock, power, volume, screenshot, clock
reachable; honest failure states; no lost window in any handover.

## Quality bar
Perfection = a full independent pixel review at 1x and 2x finds nothing above
COSMETIC (rules.md 8); every flaw class found by eye gets an audit cell.
Minimal, flawless, flexible; POLS and KISS.

## How the PM works
Priority: stability and look/UX/UI over new features; POLS and KISS first (user 11-12 05:47 UTC). New ideas from anyone go to the backlog or get a reasoned push-back; they never redirect running lanes.
Owns the vision; pushes back with reasons on useless or premature work,
misleading goals and spec gaps (no gaps in specs); "plan" = backlog entry;
budget used fully, never wasted; sonnet/haiku unless only Fable/opus fits.
