# Loader prototype

A prototype of a "loading program" in the style of The Matrix, built
through the API alone. The code is a prototype: it is not tested, not
packaged and not clean.

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

Limitations found with this prototype:

- The layer budget of 64 Mi pixels is easy to exceed at scale 3; the
  `layers.dropped` event names the nodes that stay out of a frame.
  Captures do not show it, because a capture renders outside the frame
  loop.
- Every connection of a client gets each event (#20).
- A camera belongs to one client; only `camera.release` on the admin
  socket takes it (D40).
- Only the owner of a window may focus it. The walk cannot take the
  keyboard focus from the terminal and give it back.
- Shaders: no `fwidth` without the extension, and `mediump` breaks
  hashes (#16).
- Nested sessions use the keymap of the host (#12).
- Captures render outside the frame loop, so a layer that stays out of
  the budget on screen still shows in them.
- GNOME Shell 51 runs nested in chora with `gnome-shell --devkit` in an
  Arch distrobox. It needs `mutter-devkit`, a full `pacman -Syu` (partial
  upgrades break the schemas and libraries), and
  `DBUS_SYSTEM_BUS_ADDRESS=unix:path=/run/host/run/dbus/system_bus_socket`.
  With the system bus of the host, it tries to mount the volumes of the
  host.

Open:

- Keyboard focus in the walk (#27).
- The GNOME window on the `VOID` panel instead of in the tiling.
- Mipmaps or smaller panel textures against shimmer at a distance.
- Move the code into a plugin with a manifest, and use views.
