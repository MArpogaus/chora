# lint-reason

## r1

Head: 1f99a98fbbcad0ba49201df8378824452a1a2fee on claude/lint-reason (base 6aa0bd37).

Counts (per attribute, `#[allow]`/`#[expect]` in crates, plugins, services, *.rs):
- before: 374 attributes, 117 without `reason`
- after: 376 attributes, 0 without `reason` (2 added: `too_many_lines` expects)

Note: the brief's `rg -U ... | grep -vc reason` counts continuation lines of multi-line
attributes (374 before, 538 after), so it does not measure this; the per-attribute scan does.

Changes:
- every attribute outside the excluded files gets a reason; `#[allow]` became `#[expect]`
- lints that no longer fire are removed from their list (text.rs rasterize and blit,
  chora-xp cube.rs), found by clippy's unfulfilled-expectation errors
- chora-draw theme.rs draw_file and chora-mc layout.rs panel: `#[expect(clippy::too_many_lines)]`
  with a reason, because rustfmt wraps the reasoned attributes and the functions reach 64 lines
- no code change beyond attributes and rustfmt wrapping; excluded files untouched

Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy` on 1f99a98f:
pre-commit PASS, build PASS, test PASS, clippy PASS, ci-check PASS.
