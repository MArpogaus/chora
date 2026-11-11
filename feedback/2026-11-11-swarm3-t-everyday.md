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
status: new

## Pasted word does not appear in the second terminal
kind: issue
persona: Jonas, 38, office worker, tech affinity 2, mouse and keyboard, used Windows and a bit of Ubuntu
flavor: tiling
severity: high
steps: Typed echo bananas in the left terminal. Double-clicked bananas, pressed ctrl+shift+c, clicked the right terminal, pressed ctrl+shift+v.
expected: bananas appears at the prompt of the right terminal.
actual: The right terminal stayed empty. The word showed no selection highlight, so I could not confirm the copy.
evidence: shots/012.png
status: new

## Lock and power controls are not findable
kind: ux
persona: Jonas, 38, office worker, tech affinity 2, mouse and keyboard, used Windows and a bit of Ubuntu
flavor: tiling
severity: high
steps: Looked at the top bar, clicked the top-left icon (it showed an unlabelled yellow square) and the blue "Connected" button at the top right. Right-click on the desktop showed nothing.
expected: A visible power or session menu with lock and power off options.
actual: No lock or power control was found. The step cap was reached.
evidence: shots/013.png
status: new
