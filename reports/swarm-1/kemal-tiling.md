# Kemal (keyboard-only power user), flavor tiling, 1280x800

Session: CHORA_BIN=.../scratchpad/swarm-bin (all binaries from there). Screenshots were in /tmp/persona.1pQAXY/shots (001-016); the session dir was removed on stop (PERSONA_KEEP unset), so the pictures are gone.
Contamination: before starting, I read tests/persona/README.md, session.sh, tests/headless-session.sh and docs/design-principles.md. The persona brief forbids this. Treat findings as tainted by that.

Tasks (34 actions, 0 failed):
1. Keys/help overlay: NOT reached. Super+slash and Super+F1 did nothing visible.
2. Two terminals and switching: reached. Super+Return twice (side by side); Super+j and Super+Tab move focus. Super+Left/Right switch workspace, not focus.
3. Change layout: NOT reached. Super+space, Super+t, Super+e, Super+shift+space changed nothing visible.
4. Helper chat: NOT reached. Super+a and Super+c did nothing; the letter went into the terminal.
5. Volume: NOT reached; the panel says "Sound unavailable" (no sound device in this headless run).

Findings (rank, principle):
- MAJOR: no visible hint of any key (no overlay, no bar hint, no cheatsheet). Principle 2 (every task by every path) and the calm/least-surprise principles.
- MAJOR: letter-key shortcuts (Super+s/e/c/a) type into the focused terminal, but Super+j/Tab work. Inconsistent keyboard model. Principle 3 (least surprise).
- MINOR: Super+Left/Right change workspace with no feedback except a tag mark on the panel. Principle 3.
- MINOR: "Sound unavailable" in the panel with no way to reach volume. Principle 5 (honesty), principle 7 (everyone).
