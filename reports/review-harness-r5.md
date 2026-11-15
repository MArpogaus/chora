Verdict: MERGE-OK

# Review harness r5 (claude/harness 5f0977f1 over 7ef5fe1f)

Commits: c6d39f8c (red selftest), 761ea365 (gate fix), 5f0977f1 (no-foreign-kill test+fix).
Scope: tests/gate-0.8-plugin-runtime.sh, tests/merge-gate-selftest.sh, tests/no-foreign-kill.sh,
tests/guards.sh. Probes ran in /tmp/claude-0/rev-harness5 (deleted afterwards).

## 1. r4 MAJOR (allow window + trailing comment): CLOSED
tests/gate-0.8-plugin-runtime.sh:22 now closes the window on `)]` + optional spaces + optional `//...`.
Probe (one file, attribute variant, then `fn b() { core.call("events.subscribe"); }`), rc new/old:
- `#[allow(clippy::x)] // reason`                 1 / 0  refused now
- `#[allow(a, b)]  // c`                          1 / 0
- `#[expect(clippy::x, reason = "...")]`          1 / 1
- `#[expect(clippy::x, reason = "y")] // c`       1 / 0
- multi-line `#[allow(\n..\n)] // c`              1 / 0
- `#[allow(clippy::x)]//c` (no space)             1 / 0
- `#![allow(clippy::x)] // c`                     1 / 0
- `#[allow(x)] fn f() {` (one line)               0 / 0  still hidden (see m1)
- `#[allow(x)] let y = 1;` (one line)             0 / 0  still hidden (see m1)
- `#[allow(x)] /* c */ let z = 2;`                0 / 0  still hidden, rustfmt-clean (see m1)
Today's plugins/: gate rc 0 on HEAD. Windows left open past their line: 20 spans / 1143 lines
with the old close regex (my count; lane says 27/4085 with another counting method; r4 said
20/1217), 0/0 with the new one. Raw pattern grep on plugins/ hits only the 3 allowed sites
(chora-mc testing.rs:140-141, chora-tiling main.rs:2323,2328, chora-xp wm.rs:4318-4319), all
excused. Negative control: renaming the lint at chora-xp wm.rs:4287 in a scratch copy makes the
gate fail on wm.rs:4318-4319, so the excuse is real and not a blanket skip.

## 2. r4 MINOR (`need pgrep pkill jq` refused): CLOSED at command start
tests/no-foreign-kill.sh:72 returns no verdict for `need|command -v` followed only by plain words
(`[[:alnum:]_./>-]`). Probes, rc new/old/want:
- `need pgrep pkill jq`, `need pkill jq pgrep`              0/1/0
- `need pkill jq >/dev/null 2>&1`                          0/1/0
- `command -v pkill pgrep jq >/dev/null 2>&1 || exit 1`    0/1/0
- `command -v pgrep pkill jq >/dev/null || { echo; exit 1; }`, `&>/dev/null`, `2>/dev/null >&2`,
  leading spaces, `need pidof killall jq || exit 1`         0/1/0
No hole (all refused, rc 1): `need pgrep; pkill chora`, `need x && pkill chora`,
`command -v pkill && pkill -f chora`, `need pgrep pkill jq\npkill chora`, `need x & pkill chora`,
`need $(pkill chora)`, need with backticks, `need x >$(pkill chora)`, `need x\n(pkill chora)`,
`command -v x pkill chora; pkill chora`. The split on `;|&` runs before the excuse, and `$`, `(`,
backtick and quotes are outside the word class, so substitutions never reach the excuse.

## 3. Red-first
- c6d39f8c alone (full tree archive): merge-gate-selftest rc 1 with exactly the 2 FAILs quoted in
  the lane report; 761ea365 changes only the gate (selftest byte-identical) and turns it rc 0. OK.
- 5f0977f1 (accepted exception, R14): guards.sh of 5f0977f1 against the 7ef5fe1f hook: rc 1 with
  the 2 new no-foreign-kill FAILs plus the nested "guards.sh leaves the repository of a hook
  environment alone" FAIL; with the new hook "all guards behave", rc 0. The red is real.

## Suites (scratch archive of 5f0977f1)
gate-lib-selftest rc 0; merge-gate-selftest rc 0 (all plugin runtime cases ok); guards.sh rc 0.

## Findings

m1 MINOR tests/gate-0.8-plugin-runtime.sh:22 - an attribute line that does not END in `)]`
(+comment) still keeps the window open to the next line that does, hiding string calls after it.
The one rustfmt-clean form: rustfmt keeps `#[allow(x)] /* c */ let z = 2;` on one line.
Reproducer: plugins/x/lib.rs =
  `fn h() {\n    #[allow(x)] /* c */ let z = 2;\n}\nfn b() {\n    core.call("events.subscribe");\n}\n`
  -> `rustfmt --check` clean, gate rc 0 (want 1). Same for one-line `#[allow(x)] fn f() {` and
  `#[allow(x)] stmt;` (lane's open m5), which rustfmt (pre-commit cargo-fmt, plugins/* are
  workspace members) splits, so they cannot land via a normal commit. No instance in plugins/
  today. Not a MAJOR: needs a block comment after an attribute, and clippy disallowed_methods
  still catches the connect calls (not the string calls). Suggested fix (fail closed): close the
  window on any line containing `)]` (an early close in a reason string only makes the gate
  stricter), or fail when a window spans more than N lines without a close.
  Earliest gate: merge-gate step plugin runtime / merge-gate-selftest. Backlog is fine.

m2 MINOR tests/no-foreign-kill.sh:72 - the excuse is anchored at command start, so the repo's
own idiom `if ! command -v X >/dev/null; then` (tests/guards.sh:6, gate-construct-keys.sh:192)
with several tools, kill tool not last, is still refused:
  `if ! command -v pgrep pkill jq >/dev/null; then exit 1; fi` -> rc 1 (want 0);
  `if command -v pgrep pkill jq` likewise. Fails closed (blocks a valid commit, opens nothing).
  Fix: allow an optional `(if|while|until|!)[[:space:]]+` prefix (repeated) before need/command -v.
  Earliest gate: pre-commit no-foreign-kill; guards.sh case.

n1 NIT tests/no-foreign-kill.sh:72 - a quoted tool name (`need "pgrep" pkill jq`) is refused,
since the code() pass keeps the quote chars and `"` is outside the word class. Fails closed.

n2 NIT reports/harness-r5.md - span counts (27/4085) differ from mine (20/1143) and r4's
(20/1217); the counting method is not stated. Only the "after = 0" matters, and that holds.

No BLOCK, no MAJOR. Both r4 findings are closed as briefed; m1/m2 are residual, fail-closed or
outside rustfmt layout, fit for backlog R13.
