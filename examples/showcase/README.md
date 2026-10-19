# Showcase

Six small applications for a running chora, like the loader prototype:
not tested, not packaged, the shortest path from a live core to
something you can play with.

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

The user grants capabilities, not the programs (D40): no demo touches
the admin socket. Before the first run of a demo that captures, run its
grant script from this directory:

```sh
./grant-snake.sh   # also grant-pong.sh, grant-slides.sh, grant-cards.sh
./grant-news.sh    # the key injection of the reader, for the name `agent`
```

`deck.py` and `news.py` need no grant of their own. Without a grant,
`screen.capture` answers with the `choractl grant` line to run.

## Notes

- An `input` node only answers key events while its property `accept`
  is true; `snake.py` sets it with a `focus_node` command.
- A `choractl` of another version than the core prints a warning line
  before every answer, so JSON consumers break: run the binaries built
  from this tree.
- Every client owns its nodes: they leave with the connection unless
  `hello` sets `keep_nodes`, and cameras only change hands through a
  `camera` command. `cards.py` shows the release.
