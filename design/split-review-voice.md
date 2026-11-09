# Split review: voice

At `7dccda3`, 2026-11-09. References are `path:line` under `services/voice/src/` when no
folder is named. Values: POLS, KISS. Voice has 16,933 lines of Rust in 25 files: 11,014 of
code, 5,919 of tests (with `fake.rs`). Over 5,000 lines, so a split review (charter 18).

## 1. The parts today

One binary, one flat module list (`main.rs:8-32`). Lines as total / code without tests.

| Part | Files | Lines |
| --- | --- | --- |
| Speech front end | `audio.rs` 350/298, `pcm.rs` 260/187, `vad.rs` 124/79, `gate.rs` 490/314, `words.rs` 413/216, `earcon.rs` 64/45 | 1,701 / 1,139 |
| Speech clients | `asr.rs` 251/157, `tts.rs` 198/148, `speech.rs` 742/492 | 1,191 / 797 |
| LLM and chat | `llm.rs` 444/336, `chat.rs` 2,887/1,491, `prompt.md` 121 | 3,331 / 1,827 |
| Tools, MCP bridge | `tools.rs` 1,302/713 | 1,302 / 713 |
| Harnesses | `agents.rs` 1,392/1,064 (ACP), `coder.rs` 675/500 | 2,067 / 1,564 |
| Core state machine | `core.rs` 2,444/1,289 | 2,444 / 1,289 |
| chora client, HUD | `client.rs` 814/605, `mc.rs` 412/343 | 1,226 / 948 |
| Settings | `config.rs` 1,388/1,128 (83 keys, `config.rs:176-334`) | 1,388 / 1,128 |
| Wiring, dev tools | `main.rs` 938/734, `live.rs` 173, `record.rs` 195, `replay.rs` 465, `health.rs` 203, `log.rs` 19, `fake.rs` 290 | 2,283 / 1,609 |

Inside the big files:

* `chat.rs` holds three jobs. Context: `prompt` (:522-575), `drop_old_images` (:937), `note`,
  `fit`, `shrink`, `save` (:1229-1395). Turn loop: `turn` (:579), `round` (:842), `run` (:816),
  `calls` (:970), `guard` (:1013). Gate: `session_key`, `listed`, `detail`, `too_long`
  (:125-201), `reload_is_security` (:1043), `rule` (:1059), `confirmed` (:1071), `ask` (:1144),
  `removes_own` (:1183), `journal` and `hide` (:1428-1475).
* `tools.rs`: the MCP child (:83-97), guards of the own client (:132-218), loading and naming
  (:229-315), dispatch to MCP, coder, agents and `read` (:321-385), result trimming (:388-477),
  the `read` tool (:496-630), the rule matcher (:662-712).
* `core.rs` is pure (`Core::handle` :530 does no IO); its confirmation state is at :289-341.
* `config.rs`: the struct, the phrases and their German (:111-175, :701-800), the spoken
  questions (:525-660), `security_key` (:664), the checks (:973-1127).

History: 33 voice commits since 2026-10-01 touch `tools.rs` 12 times, `chat.rs` 11,
`config.rs` 8, `core.rs` 7. 12 fix confirmations, confined callers or secrets (`0ed7d44`,
`da5f693`, `f425ff9`, `100d4b6`, `51496ce`, `c96bab8`, `7f608df`, `173581b`, and 4 more).

## 2. Security code

About 1,300 lines of code in 7 files decide what runs and who may say yes.

* May a call run, and how to ask: `chat.rs:125-201`, `:1043-1236`; `tools.rs:132-218`,
  `:329-339` (no `inject_input` while a key question waits, no call of the own client),
  `:662-712`; `config.rs:10-50` (`Confirm`, `ConfirmKeys`), `:367-390` (default rules),
  `:525-660` (questions), `:664-698` (security keys).
* Who answers: `core.rs:798-879` (`answer`, `asked`, `answers`), `:1047-1095` (`always`,
  `yes_no`, `confirm_key`, `confirm_button`); `client.rs:110-167` (confined callers, claims,
  `confirm.answer`), `:252-283` (routing), `:483-500` (confirm keys grabbed only while a
  physical question waits), `:523-530` (an injected key answers nothing); `mc.rs:187-217`.
* Confinement: `tools.rs:83-97` starts `CMD --name NAME-mcp mcp`, which `choractl` confines
  (grants.md section 1); `tools.rs:518-585` reads only inside the project roots, by the real
  path and the open handle; `config.rs:395-400` and `:1104-1127` put harnesses in
  `sandbox_cmd`; `coder.rs:340-420` ends process groups; `agents.rs:886-925` asks for an ACP
  permission; `agents.rs:1001-1030` spawns the agent.
* Secrets: three redactors, `chat.rs:1448`, `tools.rs:458-477`, `client.rs:319`.

Easy to review? Each piece, yes: pure helpers, doc comments that name the review that asked
for them, and tests (`chat.rs` 1,396 test lines, `core.rs` 1,155). The whole, no:

* One call passes `chat.rs`, `tools.rs` and `config.rs` before it runs. The rule order lives
  only in `confirmed` (`chat.rs:1071-1140`).
* An `AtomicBool` couples them: `physical` (`tools.rs:79`), set in `chat.rs:1153`, read in
  `tools.rs:329`.
* A second question path: an ACP permission (`agents.rs:886`) is answered by a spoken yes
  (`core.rs:819-822`), and an always word counts as a yes there (`core.rs:1047`). Sandboxed, so
  acceptable, but nothing shows the difference to the main gate.
* `review_due.py:24-27` marks `config`, `chat`, `tools` and `agents` as critical: 6,969 lines
  of weight for about 1,300 lines that matter.

## 3. Verdicts

One library crate, the rest stays modules. No candidate moves out of the voice repository.

| Candidate | Verdict | Why |
| --- | --- | --- |
| Audio in and out (`audio`, `pcm`, `vad`, `gate`, `words`, `earcon`, `record`, `replay`) | module | Pure, tested, and the replay harness covers it. No second user. `tract-onnx` stays here. |
| ASR | module | 157 lines, one websocket. |
| TTS and `speech` | module | 640 lines of code; tied to the player and the barge-in. |
| LLM client | module | 336 lines, one endpoint. A crate only with a second user. |
| Chat loop and context | modules | `chat.rs` splits in two: `chat.rs` (the turn loop) and `context.rs` (prompt, layers, budget, history file). The gate leaves for the crate. |
| Tools and MCP bridge | module folder | `tools/` with the registry (`mod.rs`), `mcp.rs`, `read.rs`, `trim.rs`. The guards leave for the crate. |
| Confirmations | crate `chora-voice-policy` | The security core. Without tokio, reqwest and rmcp it holds pure functions; a reviewer reads one small place. |
| ACP agents | module | Keeps `agents.rs`; its permission question uses the crate. A crate only when chora-construct K3 (prompt panes with ACP agents, roadmap) needs a client; check the upstream ACP Rust crate first. |
| Coder | module | Shares process groups with ACP (`agents.rs:25`). One harness path later is a product question. |
| Settings | module | The flat YAML stays (D62, POLS). `Confirm` and `security_key` move to the crate, the phrases and their German to `phrases.rs`. |
| chora client and HUD protocol | module | The event examples belong in `docs/voice.md` (split.md section 3). |

The crate `chora-voice-policy` (about 700 lines of code, dependencies `serde`, `serde_json`,
`schemars`) holds:

* the rule types and matching: `Confirm`, `glob`, `rule`, `program`, `listed`, `session_key`;
* one decision: `decide(policy, call) -> Run | Ask {prompt, detail, physical, key} | Refuse`.
  The IO results (own nodes of `removes_own`, a reload that differs, the privileged set) come
  in as inputs;
* who answers: `may_answer(question, by)` for a spoken word, a key (injected or not) and a
  button (claimed, confined); `may_call(method, confined)` for `client.rs`;
* the spoken questions and the detail with its cap; `security_key`; one `hide`.

It lives in `services/voice/policy/` (member in `Cargo.toml:62`) and moves with voice (D77).

## 4. Tool plugins and layered context

Tool plugins. Today four sources are wired by name: the chora MCP, the coder
(`coder.rs:27`), the agents (`agents.rs:31`) and `read` (`tools.rs:496`), merged at
`tools.rs:229-249` and dispatched at `tools.rs:337-347`.

* The in-process set is closed: the registry takes an enum `Source`, not a trait. The name
  lists go. A tool plugin is one MCP server in `sandbox_cmd`: MCP is the plugin interface.
* One grant per tool plugin: a plugin is off until the user grants it in the settings key
  `tool_plugins`, a security key, so only a key press changes it (`chat.rs:1083-1089`).
  The chora source keeps the core grants of `voice-mcp` (D33, D40, draft D78 in grants.md).
* The LLM sees `PLUGIN__tool`; a `confirm` rule matches `PLUGIN.*` as for plugin methods.
  `decide` gets the plugin name, so every call still passes the one gate.

Layered context. Today `prompt` (`chat.rs:522-575`) builds one fixed system text and
`history` is the conversation. Scene and task reach the LLM only through tool calls, though
`core.rs:428-445` already builds the task records for the HUD.

* `context.rs` renders three layers, each with its own budget: scene (a short window list and
  the grants of `voice-mcp`, one query per turn), task (coder, sub-agents, the open question),
  conversation (today's history, `fit` and `shrink`).
* Order: the fixed system text, then the conversation, then scene and task at the head of the
  last user message. A change then keeps the prompt cache of the server, and one system
  message stays (`chat.rs:913-916`).
* Scene and tool text is data, not instructions, as the earlier requests say
  (`chat.rs:565-569`). It is the main injection path; the gate does not trust it.

## 5. Result

split review: voice 16933 2026-11-09 (one policy crate, the rest modules)

## 6. Steps

Each step is one commit with its tests; voice tests and clippy pass after each.
1. Crate with moves only: `Confirm`, `glob`, `rule`, `program`, `security_key`, `listed`,
   `session_key`, `detail`, `too_long`, and one `hide` for the three redactors. Their tests
   move along. Test: `cargo tree -p chora-voice-policy` has no tokio, reqwest or rmcp.
   `review_due.py` lists the crate as critical.
2. `decide` replaces the branches of `chat.rs:1071-1140` and the guards of
   `tools.rs:132-218, 329-339`; `physical` stays one value of the decision. Table test per
   branch, with finding 13 of `state.md` (an unknown privileged method).
3. `may_answer` and `may_call` for `core.rs:1047-1095` and `client.rs:110-167, 523-530`. The
   core tests stay; a table test: injected key, confined button, spoken yes to a key question.
4. The spoken questions (`config.rs:525-660`) into the crate, with findings 1 and 12.
   Tests on the spoken text.
5. The ACP permission through `decide` and `may_answer`. Test: its question follows the same
   table, and an always option is never allowed unheard.
6. `tools/` with the `Source` registry. Existing tools tests pass; a test for the name map.
7. Tool plugins: `tool_plugins`, one MCP server each in the sandbox. Tests with a fake MCP
   child: off without a grant, a rule `PLUGIN.*` asks, the key changes the grant.
8. `context.rs` out of `chat.rs`, no behavior change; the budget tests pass. Then the scene
   and task layers. Tests: the layer order, a byte-equal fixed prefix in two turns.
9. `phrases.rs` out of `config.rs`. No YAML change; `tests/settings-docs.sh` passes.
