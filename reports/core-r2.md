# core r2 (rc 6): open security MEDIUMs of the core review

Brief briefs/core-r2.md. Branch claude/core-r2 from eff84702 (claude/review-core), head
78f28b2d, 15 commits, pushed. The session branch did not move past 498d9de, so nothing was
merged in. Each fix has a test commit before it, run red first; the red output is quoted.

## Items

| item | state | commits (test, fix) | red evidence |
|---|---|---|---|
| 1 C05 | fixed | 3ef31958, 81fbea47 | confined `output.configure` answered code 1 "no backend" instead of 4; `configure_output` had no `_meta`; no confirm rule for `reserve_space` |
| 2 C14 | fixed | 81ae883a, 03badffd | after an injected key the idle notification got `resumed`: `left: [false] right: []` |
| 3 V02 | fixed | c01af42b, 9fc53920 (+ d8c1eab8) | info lines held `text="Mein Kennwort ist Zwiebel"`, `to coder:`, `say: Zwanzig Zeichen hier.`, `coder: task ... Mach es schön.`, the LLM text |
| 4 C13 | fixed | 6cbac164, 78f28b2d | capture at the mark showed the red of a later overlay rect, `[255, 0, 0, 255]` instead of yellow |
| 5 CR-1 | fixed | 893b3f7e, 140857c4 | `error decoding response body for url (http://...?key=sk-query/models)`, `error sending request for url (...?key=sk-query/chat/completions)` |
| 6 CR-6 | fixed | 356cdfce, 5c3dd0c7 | the harness saw `HOME` as `unset` with `llm_api_key_env: HOME` |
| 7 CR-4, CR-5 | fixed | 1a3f4b57, d660908d | docs only, no test |

## Behaviour changes

- C05 (PM 11-14 18:04): a confined connection needs `methods.privileged` for
  `output.configure` and `output.reserve`. The voice agent can use `configure_output` and
  `reserve_space` only after `choractl grant voice-mcp methods.privileged`. No grant is
  automatic (D40). Both tools carry `"privileged": true` in `_meta`. The voice agent now
  asks before `reserve_space` ("Shall I keep space free on the screen: {args}?", German too).
  tests/agent.sh and tests/gate-0.8-3.sh grant `methods.privileged` to `agent`.
- C14: injected input does not reset idle-notify (docs/api.md, input.inject row).
- V02: utterances, reply sentences, LLM texts and coding tasks plus their results log at
  debug. The stage lines with times stay at info: `turn=N asr=Xms` is a separate line from
  `turn=N text=...`. live-gate.sh runs the agent with `RUST_LOG=chora_voice=debug,info`.
  The tests capture the info log through one global subscriber: d8c1eab8 replaced a
  subscriber per test that raced in tracing (in 3 of 4 parallel runs it captured nothing).
- C13: the renderer draws the mark first, in front of the layer surfaces and the overlay.
- CR-6: the sub-agents and the coding harness keep the variables of `*_api_key_env`. Only the
  MCP child starts without them.

## Gate and scripts

- `tests/merge-gate.sh` on 78f28b2d: every step passed in one run, cube included.
- tests/gate-0.8-3.sh passes, with configure_output under the grant.
- tests/voice-acp.sh fails ("mc shows coder as question, not done": "a yes without the key
  counts as nothing"). tests/voice-events.sh fails ("after voice sleep, mc shows voice as
  none"). tests/agent.sh fails at step 2 ("timeout: xs_are 0 640"), before the changed lines.
  All three fail the same way at eff84702 and at 498d9de, the session branch. They are not
  caused by this branch and are not fixed here; they go to the backlog.
- Not run: services/voice/tests/live-gate.sh (it needs ASR, TTS and LLM servers).

## Open

- The three script failures above, all on the session branch.
- From r1, still outside this brief: V01 audio.rs children inherit the key variables; K01
  `save --file PATH` (CR-7).

## Round r3 (briefs/core-r3.md, review r2 FIX)

Head 308d22dc, 6 commits on 78f28b2d, pushed.
- V02 fixed, 50cd9108 test, 64133545 fix. Red: the info log held "Agenda" (queued), "größer"
  (over the reply), the dropped ASR prompt and the `task` argument of a tool call. Now a tool
  call logs `tool NAME Xms` at info; its arguments and result log at debug. Words over the
  reply, queued words and the dropped echo text log at debug. docs/voice.md section 11 is updated.
- C13 fixed: bde5f8b0 refactor (the cursor composition moves into render::screen_elements, no
  change), 01aafece test, 4d3816f0 fix. Red: a cursor element covering the whole output came at
  index 0, in front of the mark. Now the order is mark, cursor, then the rest. docs/architecture.md
  is corrected.
- C05: 308d22dc docs(voice) with a `BREAKING CHANGE:` footer that names output.configure,
  output.reserve, configure_output and reserve_space. `cz changelog --dry-run` lists it.
- Gate on 308d22dc: every step passed up to cube. The first cube run failed with the known F14
  flake ("the construct does not answer while the window manager is deaf"; claude/flakes). Cube
  then passed with `--only cube,pull,winit,wm-pane,walk,keys,pixels`, and every step of that run
  passed. So there is no single full run where every step passed.
