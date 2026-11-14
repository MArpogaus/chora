# Baseline

Date: 2026-11-09. HEAD: dbc46d4 (working tree has other edits, tests/persona/ untracked).

Worker: Ubuntu 24.04, 4 cores, no GPU, Mesa llvmpipe.

## Times

Build (cargo build --workspace): 136s. Tests (cargo test --workspace): 140s, 846 passed, 0 failed.

## Results

| Command or script | Result | Note |
|---|---|---|
| cargo build --workspace | PASS | 136s |
| cargo test --workspace | PASS | 846 passed, 0 failed, 140s |
| cargo clippy (-D warnings) | PASS | 82s |
| cargo deny check | FAIL (defect) | advisories: yanked crate async-executor v1.14.0 via zbus 5.19.0 and chora-xp; 3s |
| cargo machete | PASS | 0s |
| cargo doc --no-deps | PASS | 67s |
| tests/settings-docs.sh | PASS | 1s |
| pre-commit (SKIP=lychee) | PASS | 27s |
| tests/nested.sh | PASS | 45s |
| tests/agent.sh | PASS | 32s |
| tests/xp.sh | PASS | 46s |
| gate-0.7-3 | SKIP | needs root and the vkms module |
| gate-0.7-4 | SKIP | podman is not installed |
| gate-0.7-5 | SKIP | podman is not installed |
| gate-0.7-6 | FAIL (environment) | the keymap has no key F13: xkb-data here has no option fkeys:basic_13-24 |
| gate-0.7-7 | PASS | 4s |
| gate-0.7-8 | PASS | 4s |
| gate-0.7-9 | PASS | 3s |
| gate-0.7-10 | PASS | 35s |
| gate-0.8-1 | PASS | 5s |
| gate-0.8-2 | PASS | 3s |
| gate-0.8-3 | PASS | 8s |
| gate-0.8-4 | PASS | 4s |
| gate-0.8-5 | PASS | 5s |
| gate-0.8-6 | PASS | 3s |
| gate-0.8-7 | PASS | 2s |
| gate-0.8-8 | PASS | 1s |
| gate-0.8-9 | SKIP | no font for CJK and emoji in ~/.local/share/fonts/chora-test |
| gate-0.8-10 | SKIP | no font for CJK and emoji in ~/.local/share/fonts/chora-test |
| gate-0.8-11 | PASS | 116s |
| gate-0.8-plugin-runtime | PASS | 0s |
| gate-0.8-tiling-look | PASS | 3s |
| gate-0.8-tiling-style | PASS | 3s |
| gate-0.8-xp-settings | PASS | 5s |
| gate-0.8-xp-token | SKIP | thunar is not installed |
| gate-0.9-daily | SKIP | interactive daily helper, needs the chora session and a user manager |
| gate-0.9-hardware | FAIL (environment) | choractl is not in PATH; run this inside the chora session (real hardware checklist) |
| gate-0.9-input | PASS | 2s |
| gate-0.9-screencast | SKIP | pipewire is not installed |
| gate-0.9-session-units | SKIP | desktop-file-validate is not installed |
| gate-0.9-shader-hang | PASS | 15s |
| gate-0.9-tiling-tags | PASS | 3s |
| gate-0.9-vkms-hotplug | SKIP | needs root and the vkms module |
| gate-0.9-x11 | SKIP | xwayland-satellite is not installed |
| gate-construct-hud | PASS | 225s |
| gate-construct-k1 | PASS | 12s |
| gate-construct-k2a | PASS | 23s |
| gate-construct-keys | FAIL (defect) | the launcher became a pane: launcher term; same on rerun |
| gate-construct-mcpane | PASS | 7s |
| gate-construct-nested | PASS | 13s |
| gate-construct-windows | FAIL (defect) | exit 143 (SIGTERM) after the last ok line, no FAIL message; same on rerun, 84s |
| gate-handover | PASS | 19s |

gate-0.9-daily wrote a BAD line to ~/gate-0.9-daily.log; that file was removed again.

## Daily e2e set

- tests/nested.sh
- tests/agent.sh
- tests/xp.sh
- tests/gate-0.7-7.sh
- tests/gate-0.7-8.sh
- tests/gate-0.7-9.sh
- tests/gate-0.7-10.sh
- tests/gate-0.8-1.sh
- tests/gate-0.8-2.sh
- tests/gate-0.8-3.sh
- tests/gate-0.8-4.sh
- tests/gate-0.8-5.sh
- tests/gate-0.8-6.sh
- tests/gate-0.8-7.sh
- tests/gate-0.8-8.sh
- tests/gate-0.8-11.sh
- tests/gate-0.8-plugin-runtime.sh
- tests/gate-0.8-tiling-look.sh
- tests/gate-0.8-tiling-style.sh
- tests/gate-0.8-xp-settings.sh
- tests/gate-0.9-input.sh
- tests/gate-0.9-shader-hang.sh
- tests/gate-0.9-tiling-tags.sh
- tests/gate-construct-hud.sh
- tests/gate-construct-k1.sh
- tests/gate-construct-k2a.sh
- tests/gate-construct-mcpane.sh
- tests/gate-construct-nested.sh
- tests/gate-handover.sh

## Installed tools

Installed: sway, foot, grim, jq, waybar, fuzzel, wlrctl, swaylock, wlr-randr, python3, cargo-deny, cargo-machete, pre-commit.

Missing: xwayland-satellite, python dbusmock (no _dbus_bindings), podman, thunar, pipewire, desktop-file-validate, vkms module.

## Everyday tasks

Matrix of 2026-11-09 (read-only analysis of the repo; ways marked
"untested" were not run).


Read only analysis. Paths are under /home/user/chora. File:line points to the evidence.

### How to read the tables

- "MK" is mouse and keyboard. "Voice" is services/voice.
- Voice has no system tools of its own. Its LLM tools come from `choractl mcp` (crates/choractl/src/mcp.rs:441-600): `run_program`, `configure_output`, `capture`, `describe_screen`, `inject_input`, `call_method` and one tool per plugin method. So "voice" can start any CLI the user installed, for example `wpctl` or `nmcli`, through `run_program`. Facts:
  - `run_program` needs the grant `activation.token` (docs/api.md:1143).
  - It is on the confirm list by default: "Shall I start the program {program}?" (docs/voice.md:588).
  - Its output never comes back (docs/api.md:1124), so voice cannot read a volume, a battery level or a Wi-Fi list.
  - Voice has no read of state, so "what is my battery" is a gap everywhere.
- Voice is a separate service, the same for every flavor. It needs three GPU model containers (services/voice/README.md:21-27). It is not a default part of a session. Cells below say "voice: run_program X" when a CLI would work in principle, and mark it "untested path" because no doc or test names these programs.
- Core protocols that matter (crates/chora/src/state.rs:34-75, layers.rs, docs/architecture.md:102): layer shell, session lock, idle notify, idle inhibit, foreign toplevel, wlr output management, wlr screencopy, ext image capture, primary selection, data device, text input and input method. grep finds no wlr gamma control, no output power management and no wlr data control in crates/chora/src. Docs say it too: "The core has no protocol to turn outputs off yet" (docs/configuration.md:1025).
- Nothing in the repo talks to NetworkManager, BlueZ, UPower or logind, except `systemctl poweroff/reboot/suspend` and `loginctl lock-session` as shell commands. The only zbus use is the notification server in plugins/chora-xp (plugins/chora-xp/Cargo.toml:10, src/notify.rs:18). The autonomy charter already asks for a shared plugin over these services (plans/autonomy/charter.md:466-483).

### Tiling (plugins/chora-tiling, flavors/tiling)

Session units: waybar, mako (from its package), swaybg, swayidle with swaylock, kanshi if `~/.config/kanshi/config` exists, xwayland-satellite (docs/configuration.md:986-1053). Panel modules: tags, layout, clock, pulseaudio, network, cpu, memory, battery, tray (flavors/tiling/waybar/config.jsonc:9). The tray module shows StatusNotifier icons, so nm-applet and blueman-applet would show there if installed.

| Task | Mouse and keyboard | Voice |
|---|---|---|
| Wi-Fi | Waybar `network` shows SSID and signal (config.jsonc:30-35, read only, no click action). Connecting: run `nmtui` in foot (MOD+Return) or `nm-applet` in the tray. Not installed or started by the flavor. Uses NetworkManager. Partial. | Gap. Untested path: run_program `nmcli` with a password spoken aloud, no result read back. |
| Bluetooth | Gap. No module, no unit. External: `blueman-applet` in the tray, or `bluetoothctl` or `bluetui` in foot. Uses BlueZ. | Gap. |
| Volume and output | Waybar `pulseaudio` shows volume and mute (config.jsonc:26-29), read only. No volume keys bound (tiling.yaml bindings have none). External: `wpctl set-volume`, `pavucontrol`, `pwvucontrol`. Clicks on the module do nothing as configured. Partial. | Gap. Untested path: run_program `wpctl`. |
| Brightness, night light | Gap for both. No brightness binding or module. `brightnessctl` would work from a bound `run`. Night light needs wlr gamma control, which the core lacks, so `wlsunset` and `gammastep` cannot work. | Gap. |
| Displays and scale | kanshi profiles in `~/.config/kanshi/config` (docs/configuration.md:1030-1043), applied through wlr-output-management. `wlr-randr` for one shot. Keys: none. Editing a text file is needed. | `configure_output` tool: scale and transform only (mcp.rs:100, docs/api.md:1126). No mode, position or on/off. Partial. |
| Keyboard layout | Gap at runtime. Layout comes from `XKB_DEFAULT_LAYOUT` or `localectl` through env-chora (docs/configuration.md:938-940, dist/session/env-chora). No switcher binding, no panel module. Repeat rate and touchpad settings exist via view `input` (docs/configuration.md:90). | Gap. |
| Notifications | mako starts with the target, no config needed (docs/configuration.md:1007). `notify-send` works. Dismiss with `makoctl`, no binding. | Gap (voice cannot read or dismiss). |
| Lock | MOD+Shift+Escape runs `loginctl lock-session`, swayidle runs `swaylock -f` (tiling.yaml, chora-tiling-swayidle.service). Auto lock after 300 s and before sleep. Screens stay on while locked. Uses logind. | Untested path: run_program `loginctl lock-session` (confirm needed). |
| Suspend, power off | Gap in the UI. Only `systemctl suspend/poweroff` typed in a terminal. No power menu, no `wlogout` or fuzzel menu in the flavor. Idle does not suspend. | Untested path: run_program `systemctl poweroff` with confirm. |
| Battery | Waybar `battery`, `bat N%`, warning 20, critical 10 (config.jsonc:43-46). Reads sysfs (waybar reads UPower or sysfs itself). No low battery notice. | Gap (cannot read). |
| Screenshot, recording | Gap in the flavor: no binding. Core serves wlr screencopy and ext image capture, so `grim`, `slurp`, `wf-recorder`, `wl-screenrec` and OBS would work (docs/architecture.md:102). Portal for screen sharing is wired (docs/configuration.md:949-975). | `capture` and `describe_screen` give the picture to the LLM only, not to a file. Not a user screenshot. Gap. |
| Files, USB stick | Gap. No file manager in the flavor, no automount. External: `thunar` or `pcmanfm` plus `udiskie` (UDisks2). | Gap. |
| Clipboard | Wayland data device and primary selection work in the core. No clipboard manager (no wlr data control found), so `wl-clipboard` copy works but `wl-paste --watch` tools and `cliphist` do not. Copy and paste inside programs work. | Gap. |
| Printing | Gap. No CUPS tool, no applet. External: `system-config-printer` or the CUPS web page. Print dialogs of GTK programs work through the portal gtk backend (installed per docs/configuration.md:963). | Gap. |
| Start and find programs | MOD+r opens fuzzel (tiling.yaml). MOD+Return opens foot. Tags 1 to 9 with MOD+digit, MOD+Tab focus back, MOD+u urgent. No window search by name. | Voice can start a program with `run_program` and ask `call_method tiling` to change layout, tags and focus (every method is a tool, docs/api.md). Works today. |
| Updates | Gap. Package manager in a terminal. No notice of updates, no fwupd, no Flatpak UI. | Gap. |

### XP (plugins/chora-xp, flavors/xp)

Session units: chora-xp and xwayland-satellite only (flavors/xp/chora-xp.target). No waybar, no mako, no swayidle, no kanshi. chora-xp draws the taskbar, start menu, balloons and its own dialogs. It is called a demo, not a daily driver (docs/configuration.md:1057). Settings: plugins/chora-xp/xp.yaml. There is no tray, no StatusNotifier host (grep tray in plugins/chora-xp/src finds only the layout of the notification area), so applets cannot show. Control Panel has five entries: Date and Time, Desktop, Display, Effects, Taskbar and Start Menu (plugins/chora-xp/src/control.rs:120-155).

| Task | Mouse and keyboard | Voice |
|---|---|---|
| Wi-Fi | Gap. No network icon, no applet host. Only a terminal: `nmtui` through Super+e or Start, Terminal (xp.yaml:6). "My Network Places is left out" (docs/configuration.md:1180). | Gap. |
| Bluetooth | Gap. | Gap. |
| Volume and output | Gap. No volume icon in the tray, no Sounds and Audio Devices panel. The `effects.sounds` toggle only turns XP's event sounds on or off (xp.yaml:42-47). External: `pavucontrol` from All Programs. | Gap. |
| Brightness, night light | Gap. | Gap. |
| Displays and scale | Display Properties, Settings tab: a slider picks scale 100, 125, 150, 200 percent through `output.configure` (docs/configuration.md:1316-1322). No resolution, refresh, position or second monitor layout. kanshi is not started (no unit in flavors/xp). | `configure_output` tool for scale. Partial. |
| Keyboard layout | Gap. Same env-only layout as tiling. XP has no Regional and Language Options entry. | Gap. |
| Notifications | Yes. chora-xp is the notification server on the session bus, with XP balloons, timeouts, actions and click to invoke (docs/configuration.md:1370-1384, src/notify.rs). Does not own the name if GNOME owns it. Customize Notifications is gray. No history. | Gap (voice cannot read or dismiss). Voice sentences show in the paperclip balloon (docs/configuration.md:1394). |
| Lock | Gap. No idle lock unit, no Lock button. Start menu has Log Off and Turn Off only. Switch User is always disabled (docs/configuration.md:1350). Screen Saver tab is all gray (docs/configuration.md:1320). `swaylock` could be run by hand. | Untested path: run_program `loginctl lock-session` does nothing without a locker. |
| Suspend, power off | Turn Off dialog with Stand By, Turn Off, Restart. Commands in `session:` (xp.yaml:49-54): `systemctl suspend`, `poweroff`, `reboot`. Log Off is `choractl quit`. Start menu keys: u for Turn Off, l for Log Off (startmenu.rs:603-604) but the dialogs "take no keys" (docs/configuration.md:1337). Uses logind through systemctl. Works with the mouse. | Gap. Untested path: run_program `systemctl`. |
| Battery | Gap. No battery icon, no power settings ("no power settings", docs/configuration.md:1320). | Gap. |
| Screenshot, recording | Gap. Core allows grim and slurp. No Print Screen binding. | Gap (`capture` goes to the LLM). |
| Files, USB stick | File manager: `file_manager: thunar` (xp.yaml:14). My Documents, My Pictures, My Music, My Computer (root `/`), Recycle Bin opens the Trash folder, desktop icons for the Desktop folder (docs/configuration.md:1160-1180). Moving files by drag on the desktop does not change files. No automount: a USB stick needs `udiskie` or `gvfs` in thunar (not installed by the flavor). My Computer shows `/`, not drives. Partial. | Gap. |
| Clipboard | Same as tiling. XP has no clipboard UI. Desktop menu Paste is gray. | Gap. |
| Printing | Gap. No Printers and Faxes entry. | Gap. |
| Start and find programs | Start menu with All Programs from desktop files, grouped (startmenu.rs, docs/configuration.md:1112), Run opens fuzzel (xp.yaml:9), Ctrl+Esc, Alt+Tab switcher, taskbar buttons, four desktops on a cube (docs/configuration.md:1429-1441). No search. Strong. | `start_menu` method, `turn_to`, `activate` (docs/configuration.md:1445-1452) and `run_program`. The paperclip shows voice state. Works today. |
| Updates | Gap. | Gap. |

### mc (plugins/chora-mc, experimental)

mc is a voice first scene viewer, not a window manager by default (docs/configuration.md:277-296). It has no flavor directory and no session file in dist/session. Normally it runs in a pane of construct or beside tiling. Its bindings (plugins/chora-mc/mc.yaml:151-160): Super+Left and Right for scenes, Super+a, Super+o, Super+space runs fuzzel, Super+Return toggles a docked foot, Super+n, Up, Down toggle regions. The charter says mc needs its own way that does not depend on voice (plans/autonomy/charter.md:476-478). Today that way does not exist for system tasks. Collector `system.py` shows load, memory, net bytes, thermal, journal errors and failed units, but no battery, Wi-Fi or volume (docs/configuration.md:612-613).

| Task | Mouse and keyboard | Voice |
|---|---|---|
| Wi-Fi | Gap. Only the docked foot (Super+Return) with `nmtui`. | Gap. |
| Bluetooth | Gap. | Gap. |
| Volume and output | Gap. | Gap. |
| Brightness, night light | Gap. | Gap. |
| Displays and scale | Gap in mc. Host flavor decides. | `configure_output` tool, scale only. Partial. |
| Keyboard layout | Gap. | Gap. |
| Notifications | Method `notify.push {title, body, urgency}` fills the right side and a critical one shows red in the footer (docs/configuration.md:389-390, 560-563, 628). No server of `org.freedesktop.Notifications` in mc (grep finds it only in chora-xp), so `notify-send` does not reach mc. Scripts must call choractl. Partial. | The agent can call `notify.push`, and its words show in the header and footer. Notices from programs are not read aloud. Partial. |
| Lock | Gap. No lock unit with mc. | Gap. Untested path: `loginctl lock-session`, no locker. |
| Suspend, power off | Gap. Only a terminal. | Untested path: run_program `systemctl` with confirm. |
| Battery | Gap. | Gap. |
| Screenshot, recording | Gap. | Gap. |
| Files, USB stick | Gap. The agent can read project folders (`read` tool, docs/voice.md:404-412) but that is for coding. No file manager. | Gap. |
| Clipboard | Gap (core only). | Gap. |
| Printing | Gap. | Gap. |
| Start and find programs | Super+space runs fuzzel. The window gets a scene of its own. `Super+o` overview of scenes, Super+Left and Right to move (mc.yaml:151-160). Scene `terminal`. Method `app.open` and `scene.move`. | Core use case: agent runs `run_program`, `scene` methods, `agent_*` tools (docs/voice.md section 18). Works today but needs the voice stack. |
| Updates | Gap. | Gap. |

### construct (plugins/chora-construct, flavors/construct)

Session units: chora-construct only (flavors/construct/chora-construct.target and .service). No xwayland unit in flavors/construct (xp and tiling have one), no panel, no notification server, no idle lock. It is a 3D space. Keys: w a s d walk, `t` terminal, `space` launcher (fuzzel), `x` close, Tab window list, F1 help, Alt+Escape fly out (docs/configuration.md:667-680, keys table at 705). It has a voice HUD.

| Task | Mouse and keyboard | Voice |
|---|---|---|
| Wi-Fi | Gap. `t` for a terminal pane and `nmtui`. | Gap. |
| Bluetooth | Gap. | Gap. |
| Volume and output | Gap. | Gap. |
| Brightness, night light | Gap. | Gap. |
| Displays and scale | Gap. No settings UI. kanshi unit is not in the flavor. | `configure_output`, scale only. Partial. |
| Keyboard layout | Gap. Keys are by keysym name (`keys` setting), so a non US layout moves the walk keys. | Gap. |
| Notifications | Gap. No server. A user can start `mako` by hand (layer shell works, fuzzel shows over the space, docs/configuration.md:688-690). No default. | Gap. |
| Lock | Gap. | Gap. |
| Suspend, power off | Gap. | Untested path: run_program with confirm. |
| Battery | Gap. | Gap. |
| Screenshot, recording | Gap in the flavor. Core captures work for `grim`. | Gap. |
| Files, USB stick | Gap. Panes can run any program including a file manager, started by `t` then typed. | Gap. |
| Clipboard | Gap (core only). | Gap. |
| Printing | Gap. | Gap. |
| Start and find programs | `space` runs fuzzel, the first window after it becomes a pane. `t` runs foot. Tab lists windows without a pane. F1 shows help. Panes can show a nested tiling or xp (docs/configuration.md:796-830). | Voice HUD, `window.show`, `help.toggle` (docs/configuration.md:832-838, 872-906). `run_program`. Works with the voice stack. |
| Updates | Gap. | Gap. |

### Ranked gaps for a non technical user

1. Wi-Fi and network: only tiling shows state, and nobody can connect without `nmtui`. A user with no network cannot do anything else. XP, mc, construct show nothing.
2. Audio volume and output: tiling shows volume but cannot change it. No volume keys in any flavor. XP, mc, construct have nothing.
3. Power: no suspend or power off in tiling, mc, construct (XP has the Turn Off dialog). Lock exists only in tiling and has no screen off.
4. Battery and low battery warning: tiling shows a number only. XP, mc, construct have nothing.
5. Brightness: gap everywhere. A laptop with a dark or bright screen has no fix. Keys `XF86MonBrightness*` are bound nowhere.
6. Files and a USB stick: only XP has a file manager and desktop folders. No automount in any flavor. A stick is invisible.
7. Screenshots and recording: no binding, no tool in any flavor, though the core supports them.
8. Keyboard layout: no runtime switch in any flavor. A user with a second language is stuck.
9. Bluetooth: gap everywhere. Headphones and mice cannot pair without a terminal.
10. Displays beyond scale: kanshi by file in tiling only. Plugging in a projector is a text file task. Screens stay on while locked (no output power).
11. Notifications: only tiling (mako) and XP (balloons) show them. mc takes only `notify.push`. construct shows none.
12. Night light: needs a core protocol (wlr gamma control) first.
13. Printing and updates: gap everywhere, lower urgency.
14. Clipboard: the core has no wlr data control, so history tools do not work. Basic copy and paste works.
15. Voice has no way to read system state (no result of `run_program`), so no flavor can answer by voice "am I online" or "how much battery".

### Proposal

### One shared plugin over the D-Bus services

Name idea: `chora-system` (client `system`), one process for all four flavors. It exports methods (so choractl, voice and mc get one tool per method for free, docs/api.md plugin methods) and emits events. It must not reimplement services, only call them. zbus 5.19 is already a dependency of chora-xp (plugins/chora-xp/Cargo.toml:10). Add the `zbus` crate to the workspace and share it. For PipeWire, the `pipewire` crate is avoided by the voice service because of bindgen and C headers (docs/voice.md:60-62), so use `wpctl` through `std::process` or the WirePlumber D-Bus free path described below.

| Task | Service and interface | Crate or method | Gaps closed |
|---|---|---|---|
| Wi-Fi | NetworkManager `org.freedesktop.NetworkManager`: Devices, AccessPoints, `AddAndActivateConnection`, `ActivateConnection`; password through a secret agent or the connection settings | `zbus` plus `zvariant` (optional crate `nmrs` for a ready client) | network state and connect for all flavors |
| Bluetooth | BlueZ `org.bluez`: Adapter1, Device1 Pair, Connect, Trust, and an Agent1 for the PIN | `zbus` (optional `bluer`) | pair and connect |
| Battery | UPower `org.freedesktop.UPower` and `DisplayDevice`: Percentage, State, TimeToEmpty, signals | `zbus` (optional `upower_dbus`) | battery state and low warning notices |
| Lock, suspend, power off, idle | logind `org.freedesktop.login1.Manager`: `Suspend`, `PowerOff`, `Reboot`, `LockSession`, `Lock` and `Unlock` signals, `PrepareForSleep`, `PowerKey` handling | `zbus` | one power menu for all flavors, replaces `systemctl` shell strings in xp.yaml:49-54 |
| Brightness | logind `Session.SetBrightness` for `backlight` devices (no root needed), sysfs read | `zbus` plus `/sys/class/backlight` | brightness keys and state |
| Volume and output | PipeWire has no stable D-Bus API. Use WirePlumber through `wpctl` (inspect, set-volume, set-default), or the `libspa` and `pipewire` crates | process call to `wpctl` (already a dependency in voice: `pw-record`, `pw-play`) | volume, mute, sink choice |
| Removable drives | UDisks2 `org.freedesktop.UDisks2`: Filesystem.Mount, Unmount, Drive.Eject, `InterfacesAdded` signal | `zbus` (optional `udisks2`) | automount, eject, notice "USB stick connected" |
| Keyboard layout | systemd `org.freedesktop.locale1` for the list, `input.configure` of the core for the live layout. Needs a new `layout` key next to `keyboard` in the core (docs/configuration.md:90 lists only repeat settings) | `zbus` plus core change | layout switch |
| Notifications | `org.freedesktop.Notifications` server. Move the code out of plugins/chora-xp/src/notify.rs into the shared plugin, with a pure data model, and let each flavor draw: XP balloons, mc `notify.push`, construct HUD. Tiling keeps mako or uses the same history | `zbus` (already there) | notifications for mc and construct, a history for all |
| Printing | CUPS has `cups-pk-helper` D-Bus and IPP. Out of scope for the first version | `ipp` over HTTP | later |
| Updates | PackageKit `org.freedesktop.PackageKit` (GetUpdates) or Flatpak and fwupd D-Bus | `zbus` | an "updates available" notice only |

What the plugin gives each flavor on top of methods:
- Events that other plugins subscribe to (`system.battery`, `system.network`, `system.volume`). XP draws tray icons from them, construct draws a status pill, mc fills a `system` content, tiling can keep waybar modules or call `choractl call system get_status`.
- Voice tools for free, one per method, with the confirm rules already supported (docs/voice.md:588-598): "Shall I connect to X?", "Shall I power off?". Plus real read answers ("battery 42 percent"), which `run_program` cannot give.
- A secret and PIN dialog is the hard part: NetworkManager secret agents and the BlueZ agent need a UI. The plugin should ask the flavor (a `system.prompt` event answered by a method), like `voice.confirm` does (docs/configuration.md:1394-1399).
- Tests: python3-dbusmock templates for NetworkManager, BlueZ, UPower and logind as the charter says (plans/autonomy/charter.md:482). UDisks2 has a template too.

### What needs an external tool per flavor

| Task | Tiling | XP | mc | construct |
|---|---|---|---|---|
| Panel or status | waybar modules fed by the shared plugin (`custom/` exec with `choractl call system ...`), or waybar `network`, `bluetooth`, `pulseaudio`, `battery` modules with click actions (`nmtui`, `blueman-manager`, `pavucontrol`) | draw tray icons and a simple dialog in chora-xp from events, as it does for Date and Time, since no tray host exists | status line content `system` through `content.add` | HUD pill in construct from events |
| Screenshot, recording | `grim`, `slurp`, `wf-recorder`, bound to Print and MOD+Shift+s | the same tools, bound in chora-xp (needs a global key grab binding setting) | `grim` from a binding `{run: ...}` (mc.yaml binding type exists) | `grim` through the `keys` table (needs a new action type) |
| Brightness and volume keys | `brightnessctl` and `wpctl` as `{run: ...}` bindings on XF86 keys, or the plugin methods | needs a binding table in xp.yaml first (none today besides Alt+Tab, Ctrl+Esc, Super+e) | `{run: ...}` bindings | needs a binding action type |
| Night light | Needs a core protocol, wlr gamma control, then `wlsunset` or `gammastep`. Not a plugin job. | same | same | same |
| Files and USB | `thunar` or `pcmanfm` and `udiskie` (or the plugin over UDisks2) | thunar is wired; add `udiskie` or the plugin and show drives under My Computer | docked foot with `yazi`, or a scene fed by the plugin | a pane with a file manager |
| Clipboard | `wl-clipboard`, `cliphist` after core gets wlr data control | same | same | same |
| Printing | `system-config-printer` and CUPS | same, add Printers and Faxes as a Control Panel entry that starts it | same | same |
| Lock and idle | swayidle and swaylock exist. Add `swayidle` and `swaylock` units to xp, mc and construct (copy chora-tiling-swayidle.service). Add output power in the core so the screens turn off | same | same | same |
| Displays | kanshi exists. Use `wdisplays` or `nwg-displays` for a GUI, and add a kanshi unit to xp and construct | Display Properties, extend with modes through `output.configure` (core method) | agent tool `configure_output` | same |
| Keyboard layout | `XKB_DEFAULT_LAYOUT` today. Needs core `input.configure` layout key, then a binding or a waybar `custom` module | same, plus an XP Regional entry | same | same, and the `keys` of construct should use keycodes or a fixed layout for walking |
| Updates | `gnome-software` or `plasma-discover` or a terminal | same | same | same |

### Order of work (suggested)

1. Core: add wlr output power management, wlr gamma control, wlr data control, a `layout` key in `input.configure`. These are small protocol additions and unblock existing external tools.
2. Shared plugin `chora-system` with logind, UPower, NetworkManager, volume through `wpctl`, brightness. This closes ranks 1 to 5 for all four flavors and gives voice real tools.
3. Per flavor glue: waybar modules and bindings for tiling, tray icons and a Settings entry for XP, content for mc, a HUD pill for construct.
4. BlueZ, UDisks2, notification server move, then printing and updates.
