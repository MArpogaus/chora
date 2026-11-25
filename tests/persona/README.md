# Simulated users

A persona is an LLM subagent that uses a chora session like a person: it
looks at the screen and acts through input, and has no other way in. The
driver here gives it exactly that.

- `session.sh FLAVOR[+FLAVOR] [WxH]` starts a private headless session (`tiling`,
  `xp`, `mc`, `construct`) and grants the client `persona` only
  `screen.capture` and `input.inject`, and the client `persona-check`
  `screen.capture` and `methods.call`. It also starts the companions of
  the flavor with `choractl run`, as its units do: `tiling` gets swaybg
  (the wallpaper) and waybar (the panel with the tags), each when its
  program is installed. `xp` gets `chora-notify` on a session bus of its
  own, so that the balloons of a program that cannot start show; `mc` and
  `construct` have none.
- Every session also gets what a real desktop has: `chora-assistant`
  (pill, chat entry, figure) with the shipped huds linked into
  `$PERSONA_DIR/data/chora/huds` (`XDG_DATA_HOME` points there),
  `tests/fake-voice.py` as the voice agent, a `notify-send` that logs to
  `$PERSONA_DIR/fake/calls.log` (and, on `xp`, also reaches `chora-notify`
  on the session bus) and a `loginctl` that logs `lock-session`.
- An xp session also has what a real one has: the shared scripts as a
  package installs them (`CHORA_SHARED_DIR` is a copy of `flavors/shared`
  under `$PERSONA_DIR/share`), a `swaylock` that logs and draws a
  window `Screen locked` for 8 seconds (a real lock would shut the checker
  out), the editor `mousepad` (the `editor` of the
  generated `xp.yaml`; a `foot` window titled `Untitled - Notepad` that
  shows the text that is typed and appends it to `$PERSONA_DIR/notepad.txt`,
  a probe window when `foot` is not installed), the command `persona-probe`
  (a window `Probe window` with a frame that the look draws, so that a drag
  moves it, and the report of `chora-probe window --report` appended to
  `PERSONA_PROBE`, the file that `check.py end --probe` reads for T5) and a
  helper `atlas` that answers `agent_say` with `Hello` after 3 seconds of
  `working`. Start the window of T5 with `persona-probe`, not with a bare
  `chora-probe window`, which has no title bar to drag.
- The programs that a click can run on the host are stand-ins in the PATH
  of the session: `systemctl` (xp Stand By runs `systemctl suspend`),
  `loginctl`, `swaylock`, `wpctl` (it keeps the last level and mute: `get-volume` starts at
  `Volume: 0.50` and answers what `set-volume` and `set-mute` set), `grim`
  (writes its file) and `notify-send`. Each writes `NAME ARGS` to
  `$PERSONA_DIR/fake/calls.log` (`PERSONA_CALLS` in `session.env`) and none
  reaches the host. `tests/persona-selftest.sh` shows it.
- `session.sh tiling+xp` starts a second look beside the first, as the
  looks of a handover are (`PERSONA_FLAVORS` in `session.env`). The first
  look is where the person is: the session clicks once on the empty claim of
  the first look, so that its keys and the windows they start (Super+Return
  of tiling) stay with it, and the desktop of the look that joined last does
  not hold the keyboard. The click chooses a look only with
  `CHORA_TEST_FLAGS=--test-injected-press-is-real` (`round.sh` sets it).
- `act.py` is the only command of the persona: `look`, `click`, `move`,
  `drag`, `type`, `key`, `scroll`. It logs every action.
- `smoke.sh` checks the driver on a tiling session and that each of the four
  looks has the assistant, the voice, `notify-send` and `lock-session`.

## Start a session (PM)

A tiling session needs `waybar`, `swaybg`, `jq`, `setsid` and
`dbus-run-session` besides the binaries of the repo; `session.sh` stops
with a message when `jq`, `setsid` or `dbus-run-session` is missing, and
skips a companion that is not installed. `smoke.sh` also needs `foot`.

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

## Round and check (driver)

A driver runs a round with `round.sh`, and the checker tells what the
persona cannot: whether the screen state is right and whether an invariant
of the acts (`tests/acts/invariants.py`) broke on the way.

```console
$ tests/persona/round.sh start erika xp   # a free slot (2 at once), prints PERSONA_ENV=FILE
$ export PERSONA_ENV=FILE
$ tests/persona/round.sh probe erika       # T5 only: a chora-probe window; prints PROBE_REPORT=FILE
$ tests/persona/round.sh begin erika           # before EACH goal (check.py begin): the windows open now, the goal is held against them
  ... the persona works with act.py; PERSONA_CHECK=1 is set by round.sh ...
$ python3 -I tests/persona/check.py end T1 T4:volume   # one line per task, then the breaches; `end` uses the begin up, the next goal needs its own `begin`
$ tests/persona/round.sh stop erika
```

- `round.sh snapshot` copies the binaries of `target/debug` once (one sha
  per round); `start` does it when there is no copy. The folder is
  `ROUND_DIR` (default `/tmp/persona-round`), the kept captures `ROUND_KEEP`.
  `round.sh smoke` starts each of the four looks, runs one `act.py` action
  and `check.py end T7`.
- With `PERSONA_CHECK=1` in the session (round.sh sets it) `act.py` runs
  `check.py after ACTION` after each action. It takes a snapshot once the
  looks are quiet and runs the invariants on the change since the last one,
  but not I4 (a persona declares no open) and with every disconnect declared
  (I14 still reads panics and failed effects). A breach is one line of
  `$PERSONA_DIR/breaches.jsonl` (time, action, look, invariant, breach) and
  a capture in `$PERSONA_DIR/evidence/`. The persona never sees them;
  `PERSONA_KEEP` keeps them. A checker that fails writes
  `$PERSONA_DIR/check/errors.log` and changes nothing for the persona.
- `check.py end TASK...` holds the end state to the contracts of
  `reports/p2-plan.md` section 1: T1 to T7 (`T4:volume|clock|screenshot|lock|power`),
  X1 and X2, L1 to L3, M1 and M2, C1 to C3. It reads `state.get`,
  `scene.boxes`, the stand-in calls and the history of the run, never pixels.
  `--look LOOK` names the look of the task when it is the second one,
  `--probe FILE` the `--report` output of the `chora-probe window` that T5
  needs (`round.sh probe NAME` starts it on the socket name, `WAYLAND_DISPLAY`,
  not the socket path, and fails when it does not stay up; T5 is red when
  the report is missing or no `chora-probe` window is in the scene).
  `start` is the first snapshot of the run (round.sh runs it); `begin`
  takes the one a task is held against (one `begin` per goal, never once
  per persona), and `end` holds the goal to that `begin` and uses it up: a
  goal without its own `begin` gets no verdict (never the `start`, the begin or
  the end of the goal before), so every goal is compared with its own begin.
  L1 holds the goal to its own begin and wants exactly one window closed (open
  at the begin or opened during the goal); L2 prints `not measured (no config)`
  when the session has no non-empty config, neither red nor green, and is red when the
  value of gap_inner on disk did not change against the begin (a changed file with the same
  value does not count), the running config (`get_layout`) does not report the new value after the
  reload, or the look does not hold the focus after the reload; T7, C2 and L3 print `not reached (no event since begin)`, neither pass nor
  FAIL, when the persona's actions left no observation since the begin; a known-red row
  hides only the breaches its capture and match name, not the whole look and invariant; L3 does not
  count a holder of a window that a later goal gave to another look; T7 fails
  unless a window changed look (a tag move inside one look is none) and passes
  a given window that is shown but drawn under another one and prints a
  separate `minor T7 covered: SID under SID` line (root order, known MINOR).
  C2 fails only for a window that was open at the begin, or came while the
  task ran, and is gone at the end. A key list left open by the persona is no
  red of a contract. Exit status 1 when a contract is red or a breach is outside
  `tests/acts/known-red.toml`.
- A task passes when `check.py end` is green and the persona says reached.
  Contracts are selftested in `tests/test_acts.py`, a planted breach in
  `tests/acts/planted.py`.

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

## Tiling goals

The goals of the tiling persona round, in the words of a person (no goal names a
row, a key or a desktop of the product). T7 names its target as the bar names it:
the word of the bar is `Looks: xp` while xp runs.

- T7: "Send the terminal over to the xp look (the other desktop that is running)"

Ricer order: T1, T7, L3, T2, L2.

T7 runs before L3 (ti-55, tiling-doors r4). The persona session starts xp on the
one output of tiling, so an empty tag shows the xp desktop; with L3 just before T7
the persona took tag 2 for the xp look in r8, r9 and r10. A real user starts xp on
an output or pane of its own and is not sent to a second workspace just before.
