# t-everyday (tiling), simulated user
goal: volume lowered: no, tries 3
goal: screenshot taken: no, tries 1
goal: word copied terminal to terminal: partly, tries 2
goal: lock or power off found: no, tries 1 (step cap reached)

## Sound indicator does nothing when clicked
kind: issue
persona: Jonas, 38, office worker, tech affinity 2, mouse and keyboard, used Windows and a bit of Ubuntu
flavor: tiling
severity: high
steps: Clicked the "Sound unavailable" text at the top right of the top bar.
expected: A volume control or sound menu opens.
actual: Nothing changed on screen. The volume-down media key also gave no feedback.
evidence: shots/002.png
status: rejected: env, the container has no audio device and the label already says Sound unavailable; honest, not a silent control

## Pasted word does not appear in the second terminal
kind: issue
persona: Jonas, 38, office worker, tech affinity 2, mouse and keyboard, used Windows and a bit of Ubuntu
flavor: tiling
severity: high
steps: Typed echo bananas in the left terminal. Double-clicked bananas, pressed ctrl+shift+c, clicked the right terminal, pressed ctrl+shift+v.
expected: bananas appears at the prompt of the right terminal.
actual: The right terminal stayed empty. The word showed no selection highlight, so I could not confirm the copy.
evidence: shots/012.png
status: rejected: not shown, shot 012 has no selection highlight on bananas so the double-click never selected anything (harness click timing); tiling.yaml binds no clipboard key and foot owns copy and paste

## Lock and power controls are not findable
kind: ux
persona: Jonas, 38, office worker, tech affinity 2, mouse and keyboard, used Windows and a bit of Ubuntu
flavor: tiling
severity: high
steps: Looked at the top bar, clicked the top-left icon (it showed an unlabelled yellow square) and the blue "Connected" button at the top right. Right-click on the desktop showed nothing.
expected: A visible power or session menu with lock and power off options.
actual: No lock or power control was found. The step cap was reached.
evidence: shots/013.png
status: verified; FIXED by tiling v2 (swarm 5 t5-mouse/009: power icon opens Lock, Log out, Restart, Shut down)

## Tiling has no screenshot key and Print gives no feedback
kind: feature
persona: Jonas, 38, office worker, tech affinity 2, mouse and keyboard, used Windows and a bit of Ubuntu
flavor: tiling
severity: medium
steps: Pressed Print on the empty desktop.
expected: A screenshot, or a message saying how to take one.
actual: Nothing changed (shot 005, only the harness pointer square). tiling.yaml binds no Print key, so no screenshot exists in tiling.
evidence: media/2026-11-12/swarm-3/t-everyday/shots/005.png
status: verified; Print FIXED by tiling v2 (Super+Print saves a PNG, swarm 5 t5-keys); the missing notice is env, no notification daemon in the persona session
