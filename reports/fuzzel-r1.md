# fuzzel r1: report

Result: fuzzel 1.11.1 builds under `$HOME/.local` (done); the ti-1 click still fails with 1.11.1 (0/3), so ti-1 stays open with the findings below. No fix in this round, as the brief says.

## setup.sh (committed on claude/session-log)
- Builds fuzzel 1.11.1 from the codeberg tag tarball, pinned by sha256 `8f7d00c9...dfb54`, with meson from pip (`--user`) into `$HOME/.local`; `$HOME/.local/bin` goes first on PATH. Skips when `fuzzel --version` is >= 1.11. The apt line is unchanged (fallback); a failed build only logs.
- Time added in a fresh fuzzel state (binary removed, apt deps present, meson present): 10 s. Idempotent re-run: 3 s total.
- Deviation: tllist 1.1.0 and fcft 3.1.8 of Ubuntu 24.04 already meet fuzzel's pins (tllist >= 1.0.1, fcft >= 3.0.0 and < 4), so they are not built under `$HOME`. Three apt packages are installed instead: `wayland-protocols` (1.45; fuzzel needs >= 1.32 and the container had none), `libfcft-dev`, `libtllist-dev`. A fresh container adds the apt install time on top (not measured).

## ti-1 click check (tiling look, Apps menu, pointer to the Zebra row, click)
No existing cell covers a pointer click on a row (`tests/doors/tiling.sh` clicks the bar; `gate-tiling-apps.sh` uses keys only). I ran a scratch cell built from the header of `tests/gate-tiling-apps.sh` on 1014dddf (not committed): `chora-tiling --launch fuzzel`, `pointer_move` to (600, 496) (Zebra row), `click` button 272, wait for `app_id:zebra`.
- 3/3 failed. Captures: the hover capture shows the selection still on the first row (alpha); the Apps list is open at 385..895 x 194..606.
- Variant with a second `pointer_move` (602, 498) after the first: 3/3 failed the same way.
- WAYLAND_DEBUG of fuzzel 1.11.1 in that run: `wl_pointer.enter` (215, 302 = the Zebra row), `wl_pointer.motion` (217, 304), `button` press and release, then fuzzel exits. The program that started is `alpha`, the first row.
- Reading: the pointer events reach fuzzel under chora, so it is no longer "no mouse selection" (1.9.2). A click runs the pre-selected first row, not the row under the pointer. Not yet known whether fuzzel 1.11.1 needs something the compositor does not send (for example more than one motion, or `wl_pointer.frame`; the debug log shows no `frame` event) or whether the row hit test of fuzzel differs from the capture. The control with sway (wlrctl) was inconclusive: wlrctl gave no pointer effect in headless sway, so I cannot say fuzzel selects by pointer there.
- Next step (not done): in the same WAYLAND_DEBUG run look for `wl_pointer.frame` after enter, motion and button; run fuzzel 1.11.1 against a compositor with a known-good pointer (sway with a virtual pointer through `wlr-virtual-pointer` that sends frames) as the control.

Skips: no gate was run (no gate script was touched); the scratch cells are in the container only.
