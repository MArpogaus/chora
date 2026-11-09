# Simulated users

A persona is an LLM subagent that uses a chora session like a person: it
looks at the screen and acts through input, and has no other way in. The
driver here gives it exactly that.

- `session.sh FLAVOR [WxH]` starts a private headless session (`tiling`,
  `xp`, `mc`, `construct`) and grants the client `persona` only
  `screen.capture` and `input.inject`. It also starts the companions of
  the flavor with `choractl run`, as its units do: `tiling` gets swaybg
  (the wallpaper) and waybar (the panel with the tags), each when its
  program is installed. `xp`, `mc` and `construct` have none.
- `act.py` is the only command of the persona: `look`, `click`, `move`,
  `drag`, `type`, `key`, `scroll`. It logs every action.
- `smoke.sh` checks the driver on a tiling session.

## Start a session (PM)

```console
$ tests/persona/session.sh xp 1280x800 > /tmp/persona.out 2>&1 &
$ sleep 5; cat /tmp/persona.out
PERSONA_ENV=/tmp/persona.AbC123/session.env
```

The session stays up until it gets SIGTERM. End it with
`tests/persona/session.sh --stop /tmp/persona.AbC123/session.env` (it
kills the session by the pid in the file, never by name). It then removes
its directory, when it made the directory itself. Set `PERSONA_KEEP` to a directory, for example
`/home/user/wt/log/media/DATE/persona-NAME/`, and the session copies
`shots/` and `actions.jsonl` there first, so that a feedback file can
cite the pictures. A `PERSONA_KEEP` that is not empty is refused at the start. Without
`PERSONA_KEEP` they go with the directory.
A directory that you give in `PERSONA_DIR` is never removed; its files
stay.

The persona needs only the path in `PERSONA_ENV`.

`look` scales the capture down to `--max-side` (default 800), so the PNG
is usually smaller than the screen. It prints the path, the size of the
capture, the size of the screen and the scale:

```console
$ act.py look --max-side 640
/tmp/persona.AbC123/shots/001.png capture=640x400 screen=1280x800 scale=0.5
```

The coordinates of `click`, `move` and `drag` are always pixels of the
last capture, as the persona reads them off the picture. `act.py` keeps
the scale of the last `look` in `$PERSONA_DIR/last_look.json` and turns
the position into logical pixels of the output. Before the first `look`
they are logical pixels. `scroll` takes no position and is not scaled.

## Brief for the persona subagent

Fill in the angle brackets. Give the persona nothing else: no repo path,
no docs, no hints on the flavor.

```text
You are <who: for example "a person who has never used this desktop, and
wants to write a short note">. You sit in front of a screen. You can
only look at it and use the mouse and the keyboard.

Your goal: <one sentence, in the words of a user>.

Your only tools:
1. Bash, for exactly this command and no other:
   PERSONA_ENV=<path> python3 -I <repo>/tests/persona/act.py ACTION ...
   look [--max-side N]       takes a picture of the screen, prints a PNG path
                             and the size of the picture
   click X Y [--button left|right] [--double]
   move X Y
   drag X1 Y1 X2 Y2
   type TEXT                 the rest of the line is the text
   key COMBO                 for example Super+Return, Return, Ctrl+c
   scroll DX DY
   X and Y are pixels of the picture of your last `look`, counted from
   its top left.
2. Read, only to look at the PNG that `look` printed.
Do not read any other file, run any other command, or search the web.

How to work: look first, then act, then look again to see what happened.
Say in one line what you see and what you try before each action.
You have at most <N, for example 40> actions, `look` included. Stop at
the cap, also when you are not done.

If a step fails three times (the same goal, three attempts that did not
do what you expected), stop. Do not try a fourth time and do not work
around it. Write what you expected to happen, what happened instead, and
the picture path.

When you are done, or stopped, answer with:
- GOAL: reached, or not reached
- STEPS: what you did, in order, as a short list
- STUCK: where you stopped, what you expected, what you saw (or "none")
- SURPRISES: anything that confused you or looked wrong
- PICTURE: the path of your last `look`
```

The persona's judgment is the result. Do not coach it between steps.

## Read the log

Each action is one JSON line in `$PERSONA_DIR/actions.jsonl`:

```json
{"time": 1762700000.1, "action": "key", "args": {"combo": "Super+Return"}, "ok": true, "seconds": 0.05, "result": "ok"}
```

`ok: false` is a failed attempt: the command failed, not the goal. A
click on the wrong place is `ok: true`, so read the persona's answer too.

```console
$ f=/tmp/persona.AbC123/actions.jsonl
$ wc -l < "$f"                                         # actions
$ grep -c '"ok": false' "$f"                           # failed attempts
$ python3 -I -c 'import json,sys; r=[json.loads(l) for l in open(sys.argv[1])]; print(r[-1]["time"]-r[0]["time"])' "$f"   # seconds, first to last
$ grep -c '"action": "look"' "$f"                      # looks
```

Time to goal is the time of the action after which the persona says the
goal is reached, minus the time of the first line. Compare runs of the
same brief across flavors or builds by actions, failures and time to goal.
