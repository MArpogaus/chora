# Standing back from a cube face

This is the UX design note for step K4 of the construct on a cube: a
person stands back from a face to see the cube, turns it, and steps into
a face again. It follows the [design principles](design-principles.md)
and the [construct style guide](construct-style.md). It describes
behavior; the code comes after it.

## Persons and goal

A person works in a window on the front face of a cube. Each face is a
virtual output with its own window manager and taskbar. The front face
fills the screen at 1:1 and holds the keyboard. The person wants to see
the whole cube, turn it to another face, or pull a window out into the
space, and then go on working. A first-time person has no key to
remember. A person without a mouse and a person who only speaks reach
the same states.

## States

The construct with a cube is in one of two modes. The mode belongs to
the view, not to a window.

| Mode | The person sees | Input goes to |
|------|-----------------|---------------|
| use | the front face at 1:1, and the control `STEP BACK` | the face: pointer and keys reach the window manager and its windows |
| navigate | the cube from a step back, with its edges, and the controls `TURN LEFT`, `TURN RIGHT` and `STEP IN` | the construct: the pointer and the keys move the view and turn the cube |

| turning, in either mode | the cube on its way to the next face | nothing: no face takes input; one more turn key or turn chip queues one turn |

A turn is a part of both modes: while the cube turns, no face takes
input, and the mode after the turn is the mode before it. A click on a
face during a turn does nothing. A cube that the person opens, by
`Super+C`, the row `A cube of desktops` of the start card or the hint
`Cube`, starts in navigate mode: the person sees the cube of desktops
and steps into a face. A cube that `cube.open` opens starts in use
mode, as a fly-in ends today. A lock
ends neither mode; an unlocked session shows the mode of before.

## Flows

Each act has every path (principle 2). The same act has the same key as
in a construct without a cube and in xp (principle 1).

| Act | Mouse | Keyboard | Voice and API |
|-----|-------|----------|---------------|
| step back, from use mode or from a pane while a cube is open | a click on `STEP BACK` | `fly_out`: `Alt+Escape`; `Escape` alone when the grant `input.grab-any` lets the construct take it | "step back"; `view.step_back` |
| step in, from navigate mode | a click on a face, or on `STEP IN` for the front face | `fly_in`: `Return` | "step in"; `view.step_in` |
| fly into a pane beside the cube, in navigate mode | a click on the pane, or on the hint `Click Fly in` | `Tab`, then `Return` on its row | `window.show` |
| pull a window out of a face into a pane | a title drag off the face; a click on the hint `Super Alt 4 Pull out`, which shows only while a window of a face is there to pull | `Super+Alt+4`, the give key of the construct | `window.give` |
| put the window of a pane into the face at the front, in navigate mode | a drag of the pane onto a face; a click on the hint `Super Alt 2 Into face`, which shows only while the window of a pane has the keyboard | the give key of the face manager, `Super+Alt+2` for xp | `window.give` |
| turn the cube by one face | a click on `TURN LEFT` or `TURN RIGHT`; in navigate mode also a drag on the cube | `Ctrl+Alt+Left` and `Ctrl+Alt+Right`, in both modes | "turn left", "turn right"; `cube.turn` |
| walk and look, in navigate mode | a drag on the empty space, the wheel | the walk keys of the construct | the methods of the walk |
| show the keys, in navigate mode | none | `F1`, and `Super+F1` | none |
| close the cube, in navigate mode | none | `Escape`; the key list names it | `cube.close` |

In navigate mode a drag on the empty space looks around, as in the
space without a cube (F5 of `plans/construct-requirements.md`). The
drag grabs the world: the scene follows the hand, so a drag to the right
turns the view to the left and a drag down tilts it up. A press that
lands on the window of a pane gives that window the keyboard and starts
no look.

`Ctrl+Alt+Left` and `Ctrl+Alt+Right` are the turn keys of the cube of
xp on a screen. On virtual outputs xp does not grab them, so the cube of
the construct is the one cube that they turn.

`Escape` backs out of the current step (principle 1). In use mode the
window on the face has the keyboard, and a program needs `Escape` for
itself, so the construct takes `Escape` there only with the grant that
the person gave; `Alt+Escape` always works. In navigate mode the
construct has the keyboard, and `Escape` closes the cube, so that the
person is never stuck in it; `Return` steps in. `F1` reaches the program
of a pane, but stood back no program is there, so it shows the keys.
An open key list takes `Escape` first and closes itself; a click on its
`Close` or outside it closes it too, and a click never reaches the space
or a pane behind it while it is open (co-71).

In use mode the program of the face keeps `Super`: the press of `Super`
reaches it, and so do its own `Super` chords, such as `Super+E` and
`Super+D` of xp, and its tap of `Super`, which opens the Start menu of
xp. A grabbed `Super` chord of the construct, such as `Super+F1`, is
no tap: the core ends the tap that the program holds without its
release, so the Start menu does not open unasked.

## The visible way in

The controls are glass chips of the construct style, in the role
`text`, one line and at least 24 px high, in the overlay of the screen,
so that no window covers them. They sit at the bottom right, above the
taskbar of the face: there they cover no title band and no button of a
window, so a click on the close button of a maximized window never
steps back. In use mode they stay left of the desktop icons of the face
in their row, so that they cover no label. In use mode one chip shows,
`STEP BACK  Alt+Esc`, with its key. It is calm: it must not compete
with the work on the face. The hint bar at the bottom centre names the
way out as well: stepped into a face it shows `Alt Esc Step back` beside
`Super F1 All keys`, also after the first 3 s, so the person who stepped
in always sees how to step out. Between them it shows
`Super Alt 4 Pull out` while the keyboard is in a window of the face,
by the same rule as stood back: the hint is told wherever the key acts
(`pull_acts`, co-79). A pane's window is the construct's own, so with
the keyboard in a pane in front of the face the key is not told (co-68);
the step out of the pane leads on, below. The same holds in a pane that the
camera flew into from the cube stood back, as a program started from
`Programs` opens: the hint bar shows `Alt Esc Step back` and `All keys`,
never the hints of the cube, whose keys do not hold there. The
turn chips carry an arrow, `← TURN LEFT` and `TURN RIGHT →`. In
navigate mode three chips show, and each face of the cube is a control
of its own: the pointer over a face lights its edge, and a click steps
into it. A chip never takes the keyboard (principle 11): a click on
`STEP BACK` moves the keyboard to the construct because the person
clicked, and a click on a face moves it into the face. The methods
`view.step_back` and `view.step_in` and the voice leave the keyboard
where it is. A step in of the person, by `Return`, a click or a chip,
gives the keyboard back to the window of before only when its face is
at the front again; after a turn to another face, the person chooses a
window there. While the camera is in a pane no chip shows: the keys
of the cube do not hold there. A `cube.open` from inside a pane first
leaves the pane, so the cube opens ahead of where the person stood.

While no cube is open, `Super+C`, the key `cube`, opens a cube of four
faces, as `cube.open` without params does, and stands back from it; the
walk hints name it, `Super C  Cube`, while the window of a pane has the
keyboard, and a click on that hint does the same. Stood back, the
hints at the bottom centre are the chips of the cube: `Ctrl Alt ← Turn
left`, `Return Step in` and `Turn right Ctrl Alt →`; while a pane shows
beside the cube, also `Click Fly in`, the click on the pane, and, only
where it acts, with a pane beside the cube or without one,
`Super Alt 4 Pull out`, the key that pulls the window of
a face that has the keyboard (or had it at the step back) out into a
pane. The key gives a window to the construct, and the window of a pane
is its own already, so with the keyboard in a pane, or in no window,
the hint is not told (co-68); the same rule decides the hint and the key
(co-70). `Alt+Escape` out of any pane stands back: from the cube when
one is open, also from a pane flown into from a face, and there the bar
names `Super Alt 2 Into face`, the give key of the face manager, which
puts the window of the pane on the face at the front, steps in and gives
it the keyboard; with no cube open into the space, where the bar names
`Super C Cube`, which opens the cube stood back with that window told
`Into face` the same way, so the keys that the bar names lead from every
pane into a face (co-79); and the face label
under the cube names the window manager and the face, with a dot per
face. A refusal, for example while another client `xp` runs, shows in
the HUD.

## Words

| Key | English | German |
|-----|---------|--------|
| step back | Step back | Zurücktreten |
| step in | Step in | Hineingehen |
| turn left | Turn left | Nach links drehen |
| turn right | Turn right | Nach rechts drehen |
| open a cube (hint) | Cube | Würfel |
| put a pane into the face (hint) | Into face | In die Seite |

The voice agent says what happened in the same words: "I stepped back",
"I turned the cube to the left".

## Checks

| Check | Expected |
|-------|----------|
| a fresh session with a cube, no instructions: stand back | the person finds `STEP BACK` and clicks it; the cube shows whole |
| keyboard only: stand back, turn right, step in | `Alt+Escape`, `Ctrl+Alt+Right`, `Return`; the next face is at 1:1 and takes typed text |
| voice only | "step back", "turn right", "step in" reach the same states |
| type in a terminal on a face, press `Escape` in an editor | the editor gets `Escape`; the view stays |
| a still cube in navigate mode | the method `render.stats` counts no pass for any output between two looks |
| a turn | no face takes a click while it runs |
| a slow drag across a face in navigate mode | the yaw changes in every frame; a release at 30 degrees returns and at 50 degrees goes on, both within 320 ms |

## Hover, drag, voice and plain Escape

The four acts that complete the flows above. Each has a second path
(principle 2), and none takes the keyboard (principle 11).

| Act | The person does | The person sees | Rules |
|-----|-----------------|-----------------|-------|
| find the face to step into | in navigate mode, moves the pointer over a face or over a window that floats in front of it | the edge of that face lit in the accent colour, 2 px on the screen at any depth of the cube, with a glow that reaches 16 px out at 0.35, and the hint `Return Step in` with its hover fill; all light at once and go out over 120 ms when the pointer leaves | not during a turn or a drag, where they are out at once, and not in use mode; the keyboard needs no edge, as `Return` always steps into the front face |
| turn the cube by hand | in navigate mode, presses on a face and drags sideways | the cube follows the hand: a drag over the width of the view is a quarter turn, from the press, without a threshold and without lag; at the release it settles on the nearest face over 320 ms, eased out, or on the next face when the drag went past 45 degrees or the hand flicked faster than 1.5 widths of the view a second | a press that moves less than 8 px in any direction is a click and steps in on the release; a press while the cube turns does nothing; the same turn as `TURN LEFT`, `TURN RIGHT` and the turn keys; an injected press, move or release does nothing outside a test session, and an injected move during a press of the person ends that press, which settles the cube |
| speak | says "step back", "step in", "turn left" or "turn right", or the German words of the table of words | the same states as with the chips; the agent answers "I stepped back", "I stepped in", "I turned the cube to the left" | the agent calls `view.step_back`, `view.step_in` and `cube.turn` as its other plugin tools; the keyboard stays where it is |
| step back with `Escape` alone | sets `fly_out: Escape` and grants `input.grab-any` to the construct | `Escape` steps back in use mode and closes the cube in navigate mode; the chip reads `STEP BACK  Esc` | the program on the face then never gets `Escape`, as in a pane; without the grant the chip names no key, and the click and the voice still work |

## Turning

A turn by `Ctrl+Alt+Left`, `Ctrl+Alt+Right` or a chip takes 360 ms,
eased out; a second press during a turn queues one more turn, and a
third does nothing. A turn by hand shows every pixel of the drag: no
turn waits for the release. A drag that starts on a floating window
turns the cube as well: stood back, a plain drag pulls no window out.

While the cube turns in navigate mode the hints stay. The face label
names the face nearest to the front, so it changes at 45 degrees; the
dot of the face before the angle loses the accent that the dot of the
next face gains; and the hint `Step in` dims to an opacity of 0.6. In
use mode the hints stay hidden while the cube turns.

## The cube at rest

Stood back, the cube rests 14 degrees off square, turned toward the next
face, so that its depth and a sliver of the next face show at the
right. The faces stand to the right in the order of their numbers: a
drag to the left, like the key `Ctrl+Alt+Right`, brings the next face. The camera
stays at the mid height of the cube, and the cube stands back far enough
that the next face shows at that angle. The turn by key or by hand
starts from this pose and keeps it. A step in glides the 14 degrees away
with the step, over 380 ms, so that in use mode the face is square and
at 1:1: a 1 px line of the face is 1 px on the screen. The cube has the
floor shadow and the mirror image in the floor that every pane has; a
face that looks away has no mirror image, and none shows in use mode.

## Starting and failed faces

A face whose window manager is starting shows its plain fill, never a
black frame, and a flat glass card over it: the working ring and
`Starting Media center`, with the display name of the manager (`XP`,
`Tiling`, `Media center`, `Construct`; any other program is its name
with a capital). The card shows within 300 ms of `cube.open`, centred on
the face ahead, in use mode and stood back. It hides while the cube
turns or steps and while a command surface shows.

The faces fail when the manager exits before it connected, or has not
connected within 10 s. The card then says what failed and the next step:
`Media center did not start.`, the reason in a data well, such as
`chora-mc stopped after 2 s: it needs foot 1.26 or later.` (the program
and the last line it wrote to its error output) or `chora-mc did not
connect within 10 s.`, and `Click or press Return to try again.`. The
face label says `did not start`, the dot of the face is a hollow ring, and the
middle hint reads `Return Try again`. No red shows anywhere: the word
and the hollow dot carry the failure. The cube stays, with its virtual
outputs, and the error shows on the face only.

A click on the card or on the face, `Return` and a click on the middle
hint try again: the manager starts anew, the card reads `Starting`
again, and the faces work when it connects. A face
that does not work forwards no pointer input. A manager that connected
and then ends still closes the cube with its outputs, as before.

## What the construct does today

In navigate mode a drag on a face or on a window that floats in front of
it turns the cube. A press on a plane that moves less than 8 px is a
click on its face. A pane of a pulled window moves by a plain drag
and, dropped over a face, gives its window back to that face: while the
cube stands back, the windows of the panes take no pointer input.
Dropped stood back onto the face at the front, the window ends as with
the give key: the person steps in and it has the keyboard there. Dropped
onto another face, or where the face manager could not place it (a
notice says so, as for the give key), the cube stays stood back and
keeps the keyboard, so no key reaches a window the person cannot see. A
click on a pane stood back, a press that moves less than 8 px, flies
into it, as `Tab` and `Return` do, and its window takes the keyboard.

A window that a title drag pulls out of a face becomes a pane under
the pointer, a little in front of the face, so that the face never
covers it; a pane behind the resting front face in use mode is in the
window list as `covered`. On a single screen the drag cannot leave the
face, so a press on a title bar shows the hint "Ask the helper to pull
this window out" until the button goes up (`cube.pull` does it). A pane
that was pulled out or moved stays in
view when the cube stands back, and a drag drops it on a face again. A
pane in front of the cube, placed or not, such as a terminal that stood
where the cube opened, moves beside the cube while the cube stands back,
so that the cube shows whole and no window is lost behind it, and goes
home when the cube steps in or closes. A window that opens while the cube stands back, started from
Programs or by its key, opens beside the cube in the same way, clear of
it and where the person sees it; it has no home inside the cube, so it
stays there after `Alt+Escape` and after a step in, and `Tab` lists it. A pane that nobody placed and that
still covers the cube that stands back gives way to it. A saved construct file keeps `placed: true` of a pane, so it stays in view after a restart.

While the cube stands back or turns, the windows on its faces float in
front of them, the higher ones of the stack farther, as on the desktop
cube of xp; in use mode at rest they lie in the face. A window that
hangs over an edge of its face shows in full: the overhang reaches out
into the space in the plane of the face, never clipped and never
wrapped onto the next face. Where the windows of two faces would meet
at their shared edge, a window floats farther until its plane clears
the planes of the other face, at most twice the lift of the top window;
beyond that the planes cross. A face floats its windows by how
squarely it looks at the eye: at a grazing angle during a turn they lie
in the face, so no plane shows as a sliver beside the cube.

The construct has the two modes, the four chips, the keys of the table
and the methods `view.step_back` and `view.step_in`. A step in also
brings the camera back to where the front face shows at 1:1. A click on
a face in navigate mode turns the cube to that face and steps in. The
help panel lists the keys of the cube while a cube is open. While the window switcher of xp
shows, `Alt+Escape` belongs to the switcher.

In navigate mode the edge of the face under the pointer is lit, and a
drag on a face turns the cube. With `fly_out: Escape` and the grant
`input.grab-any`, `Escape` alone steps back. The voice agent knows the
words of the cube and calls the methods.

Open: the agent answers in its own words, which a live session with a
model must check against the words of this note.
