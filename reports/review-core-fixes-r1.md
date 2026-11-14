# Review: core fixes r1 (rc 6), claude/review-core at eff84702

Reviewer lane: opus, read-only. Diff: `git diff claude/autonomous-dev-session-3w-i33pg7...eff84702`
(37 commits, 28 files). Brief: briefs/review-core-r1.md; worker report reports/review-core-r1.md.

Gates (worktree /home/user/wt/core):
- `tests/merge-gate.sh --red-first-check`: passes (each fix has its test commit before it; C15 has `No test:`).
- `tests/merge-gate.sh --worker ''`: build, test, clippy PASS (81 s, 75 s, 48 s), exit 0.

## Fixes against their findings

Each fix matches its finding: C01 (`ipc::keysym` refuses a NUL before xkbcommon, grab, ungrab and
inject), C02 (injected keys on `KeyboardSource::Auxiliary(0)`; smithay tracks holders per source, so
an injected release no longer lifts the Alt of the person), C03 (`MIN_LINE`, NaN too), C04
(`attach` in the `screen:*` rule), C06, C08, C09, C10, C11, C12, C15, K01 to K03, V01, V03, V05 to
V08, V10. V09 is partial (CR-1).

## Behaviour changes (brief list)

- Safety chords off while locked (input.rs:438): right. Quit at the lock is data loss by anyone at
  the keyboard; reset and revoke do not unlock, and VT switch plus SIGUSR1 stay. Documented in
  architecture.md:473; D23 not amended (CR-4).
- Confirm key needs 1 s (core.rs:1138): right against the replaced-question race; documented in
  voice.md section on keys; the ignored press shows nothing to the person (CR-3).
- 256 grabs, 64 KiB exports, code 6 for a full exporter: right; documented in the limits table of
  architecture.md:348; the api.md rows of the three methods do not name them (CR-5).
- Confined headless `output.add`/`remove`: right; documented in api.md:811 and tested.

## Findings

| id | sev | file:line | text | evidence |
|---|---|---|---|---|
| CR-1 | MINOR | services/voice/src/llm.rs:205, 253, 124 | V09 is partial: the stream chunk error, the send error of the round without stream and the body error of `/models` still use `e.to_string()`, which prints ` for url (URL)` with userinfo and query | reqwest-0.13.5 src/error.rs:301 `write!(f, " for url ({url})")`; only :118 and :177 use `without_url()` |
| CR-2 | MINOR | services/voice/src/core.rs:1094 | The echo filter now runs before yes/no: a real "Ja" or "Nein" within 3 s of a question that contains that word counts as nothing, with only a log line (dev-rules 11-11: every refusal shows an honest state) | `words::is_echo` is a word-subset match over the last `ECHO_MEMORY` (3 s); the new test core.rs:2551 drops "Ja." after "..., ja oder nein?" |
| CR-3 | MINOR | services/voice/src/core.rs:1138 | A confirm key in the first second counts as nothing with only a log line; the person sees no sign that the press was ignored (dev-rules 11-11) | `confirm_key` returns `Vec::new()` without an earcon or HUD effect |
| CR-4 | MINOR | docs/decisions.md:43 | D23 still says the chords quit and reset without condition; the lock exception lives only in architecture.md:473 | grep of D23 |
| CR-5 | MINOR | docs/api.md:798, 815 | The rows of `input.grab`, `methods.export` and `methods.call` do not name the new limits (256 grabs, 64 KiB, code 6 when the exporter's output is full), while other rows (events.emit) name theirs | api.md rows; limits only in architecture.md:348-350 |
| CR-6 | MINOR | services/voice/src/config.rs:545 | The sub-agents and the coding harness lose the variables of `*_api_key_env`; a harness that needs the same variable (one key for the voice LLM and the harness) now fails at its own auth with no sign in chora. Documented in the voice.md table; a decision for the owner, not a bug | `without_keys` in agents.rs:1324, coder.rs:285 |
| CR-7 | MINOR | reports/review-core-r1.md:13, 26 | V01 and K01 are marked FIXED while parts stay open (audio.rs children inherit the keys; `save --file PATH` of an unloaded view); neither part is in backlog.md | grep backlog.md: no entry |
| CR-8 | COSMETIC | crates/chora-kit/src/settings.rs:272 | `without_value` cuts the value out of the serde text by string search; it depends on the wording of serde_saphyr errors. The test covers the four forms; keep, but a change of the dependency can leak again silently | settings.rs:269-285 |

## Verdict

No BLOCK or MAJOR open: **MERGE-OK**. CR-1 to CR-7 go to the fix lane or the backlog.
