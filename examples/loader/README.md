# Loader prototype (WIP)

A quick prototype of a "loading program" in the style of The Matrix. An
agent built it live through the API in one session. The code is a
prototype: it is not tested, not packaged and not clean.

## What it does

- **Construct.** A white world behind the views. `Alt+1`, `Alt+2` and
  `Alt+3` fly the camera to a view: `TILING` (the real windows), `RAIN`
  (a shader) and `VOID` (white). A click on a screen flies there too.
- **Walk.** `Alt+0` enters the construct in 3D. `Ctrl+W/A/S/D` walk,
  `Ctrl+Q/E` turn, a drag of the mouse looks around, and a click on a
  screen glides into its view. The real windows show live on the
  `TILING` screen, through node textures.
- **Donut.** Behind the start of the walk floats a ring of frosted
  glass with live system stats (CPU, cores, memory, GPU, temperatures,
  network, load, processes, uptime, swap, battery). The panels lie flat
  in one group; a mesh bends the visible arc onto the ring. Only the
  world behind the band is blurred.
- **HUD.** `hud.py set PERCENT TEXT` shows a progress bar that stays at
  one place on the screen, also when the camera moves.

## Run

```console
$ export CHORA_SOCKET=$XDG_RUNTIME_DIR/chora/wayland-1.sock
$ python -u loader.py &
$ python hud.py set 50 "half way"
```

It needs Python with Pillow. `LOADER_FONT` sets the path of a monospace
TTF font. The nodes stay after the loader ends (`keep_nodes`); a new
start replaces them.

## How it works

- The camera of the API is 2D. The walk computes a 4x4 projection for
  every node in Python; the renderer divides by w. The floor is a ray
  cast shader.
- One thread renders 30 frames a second from the current pose. Keys and
  drags only change the pose. Two writers made the picture jump.
- The donut is one group of 12 images and a 64x8 mesh. Its visible arc
  scrolls through the strip. A hidden rect with the same mesh is the
  mask; a shader reads the world and the mask and blurs only behind
  the band.
- The world and the mask render at reduced size where they can: the
  core counts node size times output scale against its layer budget.

## Reflection

What worked well:

- Node textures, meshes, shader nodes and 4x4 transforms are enough to
  build a 3D scene with frosted glass, without a change of the core.
- `screen.capture` and `input.inject` let the agent test every step.

Frictions and bugs found (issues #11 to #23):

- The layer budget of 64 Mi pixels is easy to exceed at scale 3, and
  then layers stay out silently (#17). It caused missing worlds, black
  and stale panes and "jumping" frames. Captures did not show it,
  because a capture renders outside of the frame loop.
- Grabs report presses only, need a modifier, and end only with their
  connection (#18). WASD, held keys and mouse look without a click are
  not possible.
- No screen-fixed nodes (#19): the HUD follows the inverse camera.
- Every connection of a client gets each event (#20).
- A camera belongs to one client (#21).
- Only the owner of a window may focus it. The walk cannot take the
  keyboard focus from the terminal and give it back.
- Shaders: no `fwidth` without the extension, and `mediump` breaks
  hashes (#16).
- Nested: stuck keys (#11, fixed), the keymap of the host (#12), the
  camera moves the windows of the tiling plugin (#13, fixed).

Later in the session:

- Captures hid the worst bugs: they render outside the frame loop, so a
  layer that stays out of the budget on screen still shows in them. The
  user saw the terminal through the walk, black panes and jumps; the
  agent saw clean pictures (#24).
- A mesh hides as a whole when one point is behind the eye (#25), so the
  donut shows only the arc that lies fully in front.
- Meshes map textures affinely: coarse grids ripple (#26). Without
  mipmaps, small text shimmers at a distance.
- GNOME Shell 51 runs nested in chora with `gnome-shell --devkit` in an
  Arch distrobox. It needs `mutter-devkit`, a full `pacman -Syu` (partial
  upgrades break the schemas and libraries), and
  `DBUS_SYSTEM_BUS_ADDRESS=unix:path=/run/host/run/dbus/system_bus_socket`.
  With the system bus of the host, it tries to mount the volumes of the
  host.

Open for the next session:

- Keyboard focus in the walk (#27).
- The GNOME window on the `VOID` panel instead of in the tiling.
- Mipmaps or smaller panel textures against shimmer at a distance.
- Move the code into a plugin with a manifest, and use views.
