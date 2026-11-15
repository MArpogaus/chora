Verdict: FIX

# review harness r4 (claude/harness 7ef5fe1f)

Independent and read-only. Probes are in /tmp/claude-0/rev-harness4: trees/<sha>
(git archive copies used to run the selftests at the red commits), nfk/ (files
for tests/no-foreign-kill.sh, p* should pass, f*/g* as noted), rt/ (directories
for tests/gate-0.8-plugin-runtime.sh, run with CHORA_GATE_LOCK_HELD=1), m2/
(a real bare origin plus clone with tests/red-first.sh as the commit-msg hook),
span.awk (counts lines the runtime grep skips in plugins/).

## BLOCK
- none.

## MAJOR
- M1 tests/gate-0.8-plugin-runtime.sh:21 (`lint { if ($0 ~ /\)\][[:space:]]*$/) lint = 0; next }`),
  added in bb818ceb (the m2 fix). `lint` opens on any `#[allow(`/`#[expect(` line, of any
  lint, and closes only on a line that ends in `)]`. An attribute with a trailing comment,
  `#[allow(clippy::too_many_lines)] // one arm per event`, never closes on its own line, so
  every following line is skipped with `next` (no hit check) until some later line ends in
  `)]`. This is the normal style in this repo: span.awk finds 20 such spans, 1217 lines of
  plugin code the grep no longer reads, e.g. chora-xp/src/window.rs:84-398 (315 lines),
  chora-mc/src/main.rs:3182-3376, chora-xp/src/wm.rs:748-898. No violation hides there
  today, but the grep is the only check for the string calls `"events.subscribe"`,
  `"output.list"`, `"methods.export"` (clippy cannot see strings), so this is a regression
  of the backstop that m2 set out to tighten, and wider than m2 itself.
  Reproducer 1 (rt/a/x.rs): `fn a() {` / `    #[allow(clippy::cast_precision_loss)] // a few items`
  / `    let x = 1;` / `}` / `fn b() {` / `    core.call("events.subscribe");` /
  `    conn::connect(p);` / `}`. HEAD: rc 0 "ok". At 4321f0e8: rc 1, both lines reported.
  Reproducer 2 (rt/e/wm.rs): the real chora-xp wm.rs with `core.call("events.subscribe");`
  put in at line 760: HEAD passes.
  Fix: close `lint` on `)]` followed by optional spaces and an optional `// ...` comment
  (or: on a one-line attribute, never set `lint` across lines). Add a selftest case
  "an allow with a trailing comment hides nothing after it". Earliest gate: the plugin
  runtime cases of tests/merge-gate-selftest.sh (none has a trailing comment). Never
  blocks a valid commit; it only misses.

## MINOR
- m1 tests/no-foreign-kill.sh:74 (`if (word ~ /^(pkill|killall|pidof|pgrep)$/) continue`)
  excuses a tool name only when the next word is another tool name. `need pgrep pkill jq`
  (nfk/g03) and `command -v pgrep pkill jq` (g04) are refused (rc 1): pkill is followed by
  `jq`. Same for `need pgrep pkill timeout` (p06). The guards case is exactly
  `need pgrep pkill` (tool last), so it passes. A valid commit is blocked; workaround: put
  the tools in another order or use a for loop. Fix idea: a word that is a tool name
  followed by a plain word without `-` and not the program name pattern is hard to tell
  apart; simplest is to warn instead of fail when the tool is an argument of a command
  that is not kill/xargs/timeout/sudo/env/the tool itself (i.e. the tool word is not in
  command position). Earliest gate: tests/guards.sh (add `need pgrep pkill jq`, pass).
  Also `[[ $x =~ ^pkill ]]` is refused (p31), contrived.
- m2 tests/no-foreign-kill.sh:31-35, quote stack: inside `$'...'` an escaped quote
  (`x=$'it\'s'`, nfk/g01) is read as the end of the string, the next `'` opens a string
  that never closes, and the rest of the file is blanked: a later `pkill chora` passes.
  An unquoted here-document body is skipped whole, so `$(pkill chora)` inside it passes
  (f21). `eval 'pkill chora'` and `bash -c "pkill chora"` pass (f25, f26), by design of
  literal blanking. All false negatives only. Earliest gate: tests/guards.sh.
- m3 tests/red-first.sh:35-40 `on_base_branch` matches only by branch name. A local
  branch of another name tracking the base (`git checkout -b work --track origin/<base>`)
  or a detached HEAD at the base tip refuses a fix whose test was pushed before (m2/ probe:
  rc 1 both). This is the brief's rule as written (HEAD is the base branch) and the lane
  explains why @{upstream} is not used (a lane cut from the base tracks it), so it is not
  open against the brief; `No test:` gets past it. Noted for the owner's habits only.
- m4 (lane's open note, rt/b) tests/gate-0.8-plugin-runtime.sh:22: inside an item with
  `#[allow(clippy::disallowed_methods)]` the string calls are also hidden (rt/b: an allowed
  fn with `c.call("events.subscribe")` passes). Severity MINOR, low: the allow is narrow
  (one item, with a reason, reviewed), the three sites (chora-mc testing.rs:133,
  chora-tiling main.rs:2291, chora-xp wm.rs:4287) hold no string call today, and a
  fake-core test may need those names. Fixing it later means a separate marker for strings;
  not worth it now. Accept as open.
- m5 `#[allow(clippy::disallowed_methods)] let c = conn::connect(p);` on one line
  (rt/d) leaves `lint` and `allowed` open and hides the next statement. Not rustfmt layout,
  and clippy catches conn::connect; same root as M1, fixed by the same change.

## r3 findings
- M1 (quoted command substitution) FIXED in 7ef5fe1f. `running "$(pgrep -x chora-xp)"`,
  `kill "$(pidof chora)"`, `pids="$(pgrep chora)"`, a quoted backtick, `kill -s KILL
  $(pgrep -x chora)`, `kill -s KILL "$(pgrep -x chora)"`, `echo "$(pkill -f foo)"`,
  `pids="a $(pgrep chora) b"`, nested `"$(echo "$(pgrep chora)")"`, a multi-line `$(...)`
  are all refused. Red first: test and fix share 7ef5fe1f, same reason as b822a958 (the
  guards hook refuses a red guards.sh); checked: guards.sh at 7ef5fe1f with the 4321f0e8
  hook gives 14 FAIL (13 kill cases + nested run), with the new hook "all guards behave".
- M2 (fix on the base branch) FIXED in 7fdac11c; red in df4118b6 (merge-gate-selftest at
  df4118b6: 2 FAIL, "base branch: ..."). Real-git probe m2/: on the base branch, test
  pushed, then fix: rc 0 with the note. The lane-branch case still refuses.
- m1 (false positives/negatives) FIXED for every r3 probe: for-loops, `need pgrep pkill`,
  `pgrep $scope` (warns, rc 0), multi-line awk, `-s` of kill/timeout, `echo $(pkill ...)`,
  `$((n << shift))`, a comment naming environ. New gaps in m1/m2 above.
- m2 (allow on any comment) FIXED in bb818ceb, red in 0209e5ba (2 FAIL); per-file reset of
  the indent done. But see M1: the new `lint` window is the regression.
- m3 (report) FIXED: reports/harness-r3.md corrected in session-log 293df021; the 4321f0e8
  allow is kept.

## Checked and fine (never-block probes, all rc 0)
Comments naming kill/pkill; `echo "use \$(pkill chora)"`; `msg="run $(date) not pkill
chora"`; `echo "$(date) pkill chora"`; quoted and `<<-` here-documents holding pkill;
`pgrep "$scope" -x foot` (warning only); -P/-s/-g/-F scopes, also after the name; awk/sed/
grep/jq programs naming pkill; `$((a << 2))`, `$(( (1<<3) ))`; `x="${y:-pkill chora}"`;
`trap "kill \$pid" EXIT`; `case $x in pkill) ;; esac`; multi-line strings; `kill -s TERM
"$pid"`; `kill %1`. The hook on all tracked tests/*.sh: rc 0, 0.45 s.
Selftests at HEAD: merge-gate-selftest rc 0, guards.sh rc 0, gate-lib-selftest rc 0.
No cargo build was needed.

## Gates missing
- merge-gate-selftest: an allow with a trailing `//` comment before a string hit (M1).
- guards.sh: `need pgrep pkill jq` must pass (m1); `x=$'it\'s'` then `pkill chora` must
  fail (m2).

## Verdict
FIX (M1: the runtime grep skips every line after a one-line allow with a trailing comment).
