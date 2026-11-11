# Max (ricer, 3D desktops) on construct, 1280x800, FLAVOR construct

Binaries: session started with CHORA_BIN=/tmp/claude-0/-home-user/8a28ab73-2182-54dd-8448-69a0fbb58e8a/scratchpad/swarm-bin (the first two tries used BIN= and failed, and the lock was then queued behind other sessions).
Session env: /tmp/persona.YVitmH/session.env (dir removed on stop; PERSONA_KEEP was not set, so no screenshots kept). Shots 001-010 were read in-session.

Task 1 (walk, fly into a window, back out): NOT reached. No window was visible. W moved over the floor very slowly (about 35 presses for a small change). Left arrow turned the view a little. Mouse drag and scroll changed nothing obvious. The yellow dot at top-left is the only thing on screen besides the OPEN CUBE button.
Task 2 (list of windows): NOT reached. Super+Tab showed nothing.
Task 3 (open the xp cube, step back, turn, step in): PARTLY. OPEN CUBE opened a black screen with "STEP BACK Alt+Esc" at bottom right, still black after 5s. Alt+Escape brought back the floor view. The cube did not show, turn or open a face.
Tasks 4 and 5: NOT reached (no window to pull out, and the cube never showed its glass panels or floor).

Findings (ranked):
- MAJOR: no window in the 3D space from the start, so nothing to fly into or list. Breaks the "visible way in" principle (3) and the same-act principle (1). Persona shots 002-007.
- MAJOR: OPEN CUBE leads to a black screen with no feedback for 5s+ (shot 008, 009). Principle 4 (say what happens) and principle 3.
- MINOR: walking and turning are very slow and give no hint of how to move. Principle 3.
- MINOR: the yellow dot at top-left (shots 002-007) reads as a stray marker, not a cursor. Principle 6 (calm).
- MINOR: the floor marble and pale sky are calm, but the OPEN CUBE button is small and sits in the corner. Principle 7 (targets 24px min: it is about 77x20 px at capture scale).

Actions: about 57 (looks included), 0 failed by the tool's reply. The log was lost with the session dir.
