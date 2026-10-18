# Showcase

Five small applications that were built live against the API in one
session, the way an agent uses it. They are not tested and not packaged,
like the loader prototype; they are the shortest path from a running
chora to something you can play with.

All of them need the environment of a live session (`CHORA_SOCKET`, see
`tests/headless-session.sh`) and talk through the IPC socket. `snake.py`
and `cards.py` run on plain python3; `pong.py`, `slides.py`, `news.py`
and `deck.py` also render pictures with PIL, for example from the venv
of the loader: `examples/loader/.venv/bin/python`.

| program | what it shows |
|---|---|
| `snake.py` | a keyboard snake of rect nodes on a 32 px grid; `node.key` events drive it and `choractl inject` steers it from outside. With `SNAKE_FOOD=60` it grows into a stress test of the per-frame element budget. |
| `pong.py` | two paddles and a ball with an effect shader (`ball.frag`), moved by injected key presses; captures each round. |
| `slides.py` | text slides as nodes: font, color and size changes of one node, plus camera pans. |
| `news.py` | a newspaper reader: index and article pages rendered by PIL, swapped on `Page_Down`, one image node updated per page. |
| `cards.py` | four shader cards that glow and lift, the camera panning to each: a `vec4` uniform and `time` in a fragment shader read from stdin. |
| `deck.py` | a text-only presentation: PIL renders the slides, an image node under `screen:NAME` shows them. |

The captures and rendered slides go to
`$XDG_RUNTIME_DIR/chora-showcase-shots`, not into the repository.

## Run

```sh
tests/headless-session.sh &       # source its printed environment
export CHORACTL=target/debug/choractl

python3 examples/showcase/snake.py           # or cards.py, no dependencies
examples/loader/.venv/bin/python examples/showcase/pong.py   # the PIL ones
```

Keys reach the games through injected events, so they can be steered
from a second terminal:

```sh
echo '[{"key":{"key":"Left","state":"pressed"}}]' | $CHORACTL inject
```

`cards.py` and `deck.py` ask for their grants through the admin socket
of `choractl`; run them once from the repository root, where
`target/debug/choractl` lives, or point `CHORACTL` at the binary.

## What they learned

The apps are the record of the session that produced them: the input
property of a node answers `accept`, paints come after the kind in the
JSON, and one version of the core and its clients should be built
together, because every `choractl` call of a mismatched pair prints a
warning line.
