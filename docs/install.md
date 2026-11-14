# Install

chora is pre-release. The API changes without notice before 1.0.

## Packages

Each [release](https://github.com/MArpogaus/chora/releases) has an RPM for
Fedora 44 and a package for Arch Linux, with `SHA256SUMS`.

```bash
sha256sum -c --ignore-missing SHA256SUMS
sudo dnf install ./chora-*.rpm          # Fedora
sudo pacman -U ./chora-*.pkg.tar.zst    # Arch Linux
```

On an image-based system, layer the RPM and reboot:

```bash
rpm-ostree install ./chora-*.rpm
systemctl reboot
```

The package installs the core `chora`, the tool `choractl`, the plugins
and the session files. It also holds the voice agent `chora-voice` and
the experiments `chora-mc` and `chora-construct`.

## Other programs

The tiling look starts these programs: waybar, fuzzel, mako, swaybg,
swaylock, swayidle, kanshi and foot. The RPM recommends them. On Arch
Linux, install them as optional dependencies:

```bash
sudo pacman -S --asdeps uwsm foot fuzzel waybar mako swaybg swayidle swaylock kanshi
```

[uwsm](https://github.com/Vladimir-csp/uwsm) is optional. The session
"chora (uwsm)" shows on the login screen only when uwsm is installed.
uwsm is not in Fedora 44; there the session "chora" is the way in.

chora-mc uses the fonts Inter and TeX Gyre Pagella. On Arch Linux,
install `inter-font` and `tex-gyre-fonts`.

## Optional user import

The XP look can use pictures and fonts that you install yourself in the
XDG data directory. chora does not ship them. Without them, the look
falls back to its own art. [Configuration](configuration.md#flavors)
lists the folders.

## Build from source

You need the Rust toolchain of `rust-toolchain.toml`.

```bash
cargo build --workspace
PATH=$PWD/target/debug:$PATH chora -- chora-tiling
```

Put the binaries on `PATH`. The plugins start programs with
`choractl run`.

## Files that chora reads

| Path | Holds |
|------|-------|
| `~/.config/chora/chora-tiling.yaml` | options and bindings of the tiling look |
| `~/.config/chora/chora-notify.yaml` | options of the notifications |
| `~/.config/chora/chora-assistant.yaml` | options of the voice HUD and chat |
| `~/.config/chora/clients.yaml` | names and grants of client sockets |
| `~/.config/chora/views/` | views |

`chora-tiling --print-config` prints every option. Copy the output into
the file and change it there. The full list is in
[Configuration](configuration.md).

## First start

Inside a Wayland session, chora opens as a window:

```bash
chora -- chora-tiling
```

Use `XKB_DEFAULT_LAYOUT=de` to set another keyboard layout. On the login
screen, choose a session instead. The safety layer has fixed keys:
Ctrl+Alt+Escape resets the scene, Ctrl+Alt+BackSpace quits chora and
Ctrl+Alt+F1 to F12 switch the VT.
