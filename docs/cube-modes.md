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
face during a turn does nothing. The session
starts in use mode when a cube opens, as a fly-in ends today. A lock
ends neither mode; an unlocked session shows the mode of before.

## Flows

Each act has every path (principle 2). The same act has the same key as
in a construct without a cube and in xp (principle 1).

| Act | Mouse | Keyboard | Voice and API |
|-----|-------|----------|---------------|
| step back, from use mode | a click on `STEP BACK` | `fly_out`: `Alt+Escape`; `Escape` alone when the grant `input.grab-any` lets the construct take it | "step back"; `view.step_back` |
| step in, from navigate mode | a click on a face, or on `STEP IN` for the front face | `fly_in`: `Return` | "step in"; `view.step_in` |
| turn the cube by one face | a click on `TURN LEFT` or `TURN RIGHT`; in navigate mode also a drag on the cube | `Ctrl+Alt+Left` and `Ctrl+Alt+Right`, in both modes | "turn left", "turn right"; `cube.turn` |
| walk and look, in navigate mode | a drag on the empty space, the wheel | the walk keys of the construct | the methods of the walk |
| back out of navigate mode | a click on a face | `Escape` steps in again, to the face of before | "step in" |

`Ctrl+Alt+Left` and `Ctrl+Alt+Right` are the turn keys of the cube of
xp on a screen. On virtual outputs xp does not grab them, so the cube of
the construct is the one cube that they turn.

`Escape` backs out of the current step (principle 1). In use mode the
window on the face has the keyboard, and a program needs `Escape` for
itself, so the construct takes `Escape` there only with the grant that
the person gave; `Alt+Escape` always works. In navigate mode the
construct has the keyboard, and `Escape` goes one step back to use
mode.

## The visible way in

The controls are glass chips of the construct style, in the role
`text`, one line and at least 24 px high, in the overlay of the screen,
so that no window covers them. They sit at the bottom right, above the
taskbar of the face: there they cover no title band and no button of a
window, so a click on the close button of a maximized window never
steps back. In use mode one chip shows, `STEP BACK  Alt+Esc`, with its
key. It is calm: it must not compete with the work on the face. The
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
window there.

## Words

| Key | English | German |
|-----|---------|--------|
| step back | STEP BACK | ZURÜCKTRETEN |
| step in | STEP IN | HINEINGEHEN |
| turn left | TURN LEFT | NACH LINKS DREHEN |
| turn right | TURN RIGHT | NACH RECHTS DREHEN |

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

## Hover, drag, voice and plain Escape

The four acts that complete the flows above. Each has a second path
(principle 2), and none takes the keyboard (principle 11).

| Act | The person does | The person sees | Rules |
|-----|-----------------|-----------------|-------|
| find the face to step into | in navigate mode, moves the pointer over a face or over a window that floats in front of it | the edge of that face lit in the accent colour, 2 px; it goes out when the pointer leaves | not during a turn and not in use mode; the keyboard needs no edge, as `Return` always steps into the front face |
| turn the cube by hand | in navigate mode, presses on a face and drags sideways | the cube turns by one face in the direction of the drag when the pointer moved a third of the width of the view, and again for each further third from where the cube took the turn; during a turn one more turn is queued, as with the keys | a press that moves less than 8 px in any direction is a click and steps in on the release, and a shorter drag does nothing; the same turn as `TURN LEFT`, `TURN RIGHT` and the turn keys; an injected press, move or release does nothing outside a test session, and an injected move during a press of the person ends that press |
| speak | says "step back", "step in", "turn left" or "turn right", or the German words of the table of words | the same states as with the chips; the agent answers "I stepped back", "I stepped in", "I turned the cube to the left" | the agent calls `view.step_back`, `view.step_in` and `cube.turn` as its other plugin tools; the keyboard stays where it is |
| step back with `Escape` alone | sets `fly_out: Escape` and grants `input.grab-any` to the construct | `Escape` steps back in use mode and steps in again in navigate mode; the chip reads `STEP BACK  Esc` | the program on the face then never gets `Escape`, as in a pane; without the grant the chip names no key, and the click and the voice still work |

## What the construct does today

While the cube stands back or turns, the windows on its faces float in
front of them, the higher ones of the stack farther, as on the desktop
cube of xp; in use mode at rest they lie in the face.

The construct has the two modes, the four chips, the keys of the table
and the methods `view.step_back` and `view.step_in`. A step in also
brings the camera back to where the front face shows at 1:1. A click on
a face in navigate mode turns the cube to that face and steps in. The
help panel lists the keys of the cube. While the window switcher of xp
shows, `Alt+Escape` belongs to the switcher.

In navigate mode the edge of the face under the pointer is lit, and a
drag on a face turns the cube. With `fly_out: Escape` and the grant
`input.grab-any`, `Escape` alone steps back. The voice agent knows the
words of the cube and calls the methods.

Open: the agent answers in its own words, which a live session with a
model must check against the words of this note.
