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

A turn is a part of both modes: while the cube turns, no face takes
input, and the mode after the turn is the mode before it. The session
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
`label`, at the top right of the screen, in the overlay of the screen,
so that no window covers them. In use mode one chip shows, `STEP BACK`.
It is small and calm: it must not compete with the work on the face. In
navigate mode three chips show, and each face of the cube is a control
of its own: the pointer over a face lights its edge, and a click steps
into it. A chip never takes the keyboard (principle 11): a click on
`STEP BACK` moves the keyboard to the construct because the person
clicked, and a click on a face moves it into the face.

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

## What the construct does today

The construct has the two modes, the four chips, the keys of the table
and the methods `view.step_back` and `view.step_in`. A step in also
brings the camera back to where the front face shows at 1:1, and gives
the keyboard back to the window that had it. A click on a face in
navigate mode turns the cube to that face and steps in.

Not there yet: a drag on the cube that turns it, the lit edge of the
face under the pointer, `Escape` alone in use mode with the grant
`input.grab-any`, and the phrases of the voice agent.
