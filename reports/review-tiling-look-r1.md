# Review tiling-look r1 (sonnet, read only)

Base 66588880, head 47fef21b (9 files, +211/-25). Verdict: MERGE-OK.

| # | Sev | Where | Finding |
|---|-----|-------|---------|
| 1 | MINOR | docs/xp-split.md:282 | Still says chora-tiling takes `wobbly` "off by default"; now stale (default is `[wobbly]`). |
| 2 | LOW | plugins/chora-tiling/tiling.yaml:81, config.rs | Wobbly on by default: no reduced-motion setting exists in tiling to honour; opt-out is `effects: []` (documented in docs/configuration.md:255, comment in yaml). Only floating windows bend, 60 Hz frame loop runs only while `effects.animating()` (main.rs:377), so no idle cost. |
| 3 | LOW | tests/gate-tiling-bar.sh | Not in merge gate; edit only fixes the stale separator colour. |
| 4 | COSMETIC | flavors/tiling/waybar/style.css:26-38 | Inner glow 4 px is faint; outer glow impossible (32 px surface), builder disclosed. |

No regression in xp, mc, construct: the diff touches only tiling's yaml/default config, flavors/tiling/waybar CSS, docs and tests. Wobbly default does not affect input or gates (gate-tiling-wobbly.sh checks tiled windows never bend).

Reran:
- `cargo test -p chora-tiling`: 140 passed.
- Red check: with `effects: []` in tiling.yaml, `the_default_config_wobbles_floating_windows` and `mistakes_fail` fail (138 pass); restored, green.
- tests/gate-tiling-wobbly.sh, gate-tiling-bar-palette.sh: UNVERIFIED, need the full workspace binaries (chora, choractl, chora-probe) and headless sessions; too costly here. Builder reports the merge gate green.
- gate-tiling-bar.sh layout-width step: PRE-EXISTING. `#custom-layout` (style.css:120-125) has padding only, no min-width, and the branch does not touch that rule or the waybar config; docs claim 80 px but the CSS never set it (ti-15). Not caused by this branch.
- Captures (after-bar-ink-2x, after-bar-paper-2x vs before-bar-ink-2x): islands with gradient accent edge, accent current-tag pill, mono clock; legible, balanced, consistent with window borders in both themes. Looks right.
