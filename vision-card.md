# Vision card: what chora is, for whom, and why

chora is a Wayland compositor that is a programmable scene, not a window manager.
The core has no policy: plugins, scripts and agents bring the forms through one live API, so an agent (ideally by voice) can arrange the screen for the task.
Proof of range: four complete looks in one session, switched live, windows moving between them without loss.

## The four looks
- chora-xp: the showcase. A pixel-perfect XP SP2/SP3 Luna replica with XP-like UX. Microsoft assets come only through an optional user-run import and never ship; our fallback art must stay acceptable. Later its stacking and widgets become a general API. Users: people who know XP (Erika 74, Murat 52, mouse only), nostalgic pixel-watchers.
- chora-tiling: the hacker's desktop. Hyprland-class motion and looks, plain-text hackable config. Users: keyboard-first power users and ricers; beginners must not be stranded.
- chora-mc: voice-first computing. Content shapes the view; traditional typesetting. Remote and mouse are a full second path. Users: hands-busy people (kitchen, sofa, nurse), low-tech users at a distance.
- chora-construct: the 3D space. Desktops on cube faces; step out and in, pull windows across. Glass, calm, minimal HUD. Users: explorers, demo watchers, ricers. Not VR, not physics.

## Shared parts
- core: lean and safe (an escape hatch no plugin overrides); everything else is a plugin.
- chora-assistant and chat: one helper in each look's costume; asks before risky acts; honest states (working, no helper).
- chora-notify: quiet, never covers work, placed per look.
- services/voice: local, interchangeable models, configurable endpoints.

## Shared RC bar (every look)
A visible way to start a program, the key list and the helper (mouse and keys); no stray keys into clients; lock, power, volume, screenshot and clock reachable; honest failure states; no lost window in any handover.

## Quality bar
Minimal, flawless, flexible; KISS and POLS. A look is done for the RC when it meets the shared bar and one independent review finds no open BLOCK or MAJOR (rules.md 6, 7). Stop there: MINOR and COSMETIC findings go to the known issues.
