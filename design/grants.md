# Grants that the user raises

Design for configurable grants of the voice agent. Read on 2026-11-09 at `45d85f2`.
References are `path:line` in the chora repository. Values: POLS, KISS, reuse.

## 1. Today

* Seven capabilities: `chora-scene/src/ipc.rs:627-671` (`Capability`, `ALL`). D33 also names
  `node.portal` (D70), which is planned and not in the enum.
* `Grants` is a map from client name to a set of capabilities, in core memory only
  (`grants.rs:28`, field `state.rs:254`). It has no end time.
* `grants.set` adds, `grants.revoke` removes some or all (`grants.rs:46-71`, `grants.rs:86-110`).
  Only the admin socket changes grants: `ipc.rs:989-995` answers code 8 elsewhere.
  `grants.list` works on every connection (`ipc.rs:990`).
* A name with a client socket refuses `grants.set` with code 9 (`grants.rs:91-96`). Its
  capabilities are exact: `clients.create` calls `set_exact` (`ipc/clients.rs:118`).
* `choractl grant`, `revoke` and `grants` (`choractl/src/main.rs:105-123`) connect to the admin
  socket (`main.rs:422-433`). `clients up` reads `clients.yaml` (`main.rs:952-1008`) and opens
  one socket per name. A connected name only gets its grants synced (`main.rs:1013`).
  Nothing runs `clients up` at session start (`dist/session/chora-flavor`).
* Checks: `check_grant` answers code 4 with "X needs a grant: choractl grant NAME X"
  (`grants.rs:123-142`). `check_socket_grant` checks only confined connections
  (`grants.rs:114-120`). `confined()` is a client socket or `hello {confined: true}`
  (`ipc.rs:470-474`, set at `ipc.rs:1084`).
* The scene rules of a confined connection run before every `scene.apply`
  (`ipc.rs:843-844`). `check_paths` refuses image, shader and effect paths that are not own
  assets (`ipc/clients.rs:271-294`). `confine_one` refuses `camera` (`ipc/clients.rs:318`),
  `attach`, `focus`, `close`, `configure` on foreign surfaces (`:319-330`) and a focus without
  a press (`:331-335`). All answer code 9 (`chora-scene/src/ipc.rs:154`).
* `choractl mcp` confines both connections (`mcp.rs:111`, `events.rs:73`). `load_view`
  confines `view-NAME` (`view.rs:169-177`). No MCP tool has "grant" in its name
  (`mcp.rs:893-899`). A program of `run_program` can grant itself (`mcp.rs:701`).
* The revoke binding Ctrl+Alt+i (`safety.rs:262`) and the reset (`safety.rs:292`) end all grants.
* Voice: `confirmed()` asks a question (`services/voice/src/chat.rs:1071`). A security key
  makes it physical (`chat.rs:1081-1087`). Only `confirm_keys` or a HUD button answer it
  (`core.rs:1057-1086`). An injected key answers nothing (`client.rs:477-482`). The button is
  `confirm.answer`, refused for callers without `indicator.claim` (`client.rs:108-133`).
* Gap: `indicator.claim` is only privileged (`client.rs:410-415`). A confined `voice-mcp` with
  `methods.privileged` can claim it and press the HUD button itself (`docs/voice.md:390-395`).

## 2. Capabilities

One capability lifts exactly one confinement rule. Names follow the style of `input.grab-any`.

| Capability | Lifts | Where |
| --- | --- | --- |
| `window.any` | `attach`, `focus`, `close`, `configure` of any window; `focus` without a press | `ipc/clients.rs:319-335` |
| `scene.camera` | `camera` commands | `ipc/clients.rs:318` |
| `assets.any` | `image`, `shader` and effect paths of any file | `ipc/clients.rs:271-294` |

* `focus_node` keeps its press rule, and no capability lifts the `screen:*` rule. Nobody
  asked for them.
* `load_view` with asset paths needs `assets.any` for `view-NAME`, not for `voice-mcp`. The
  grant names one view. A shader path reads any file in a compile error, so the user
  decides. The capability says so in its doc.
* A missing capability answers code 4, not 9, with `data: {capability, client, command}`. Code 9
  stays for rules that no grant lifts. The message reuses the text of `check_grant`.
* The grant decides whether an action may happen at all. The voice `confirm` rules still ask
  each time, for example before every `close`.
* On a client socket the query filter stays (`ipc.rs:1121`). `clients.yaml` may list the three.

## 3. Temporary grants

* `grants.set` gets `duration` in seconds, as `animate` names it. 1 to 86400. Missing means
  until the session ends, as today. 0 or more than 86400 answers -32602.
* Three ends: the duration, the session end, a revoke. Also the revoke binding and the reset.
* `Grants` keeps an optional end per capability: `BTreeMap<String, BTreeMap<Capability,
  Option<Instant>>>`. Core memory only, by the fake clock of `state.time`.
* Two grants of one capability: the longer wins. No end beats an end. A grant never shortens.
* `grants.set` with a duration calls `expire_at(end)` (`time.rs:64`). `expire()` (`time.rs:42`)
  calls `expire_grants(now)`. It removes due grants and runs the side effects of a revoke:
  `disconnect_wayland` and `revoke_free_grabs`, moved out of `grants.rs:100-107` into one
  function.
* A name with a client socket takes no duration: code 9, as today. Its file decides.
* Seeing: `grants.list {detail: true}` answers `{name: {capability: seconds_left | null}}`.
  Without `detail` the answer stays as it is, for `sync_grants` and scripts.
  `choractl grants` passes `detail` and prints `voice-mcp window.any 9 min`.
* The LLM sees its grants in the text of `describe_screen`: one line per grant of its name.
  No new tool, so the test at `mcp.rs:893` stays.
* Event `grants.changed {client, grants, ended}` on every set, revoke, end, revoke binding and
  `clients up`. `grants` is the new map as in `detail`. `ended` lists what ran out. Every
  connection may subscribe, as `grants.list` is open. The voice agent says
  `phrases.grant_ended` for its own names. A HUD can show live grants.

## 4. Permanent grants

* They live in `clients.yaml`. A name of the main socket gets `socket: false`:

```yaml
voice-mcp: {socket: false, grants: [window.any]}
view-desk: {socket: false, grants: [assets.any]}
```

* `clients up` opens no socket for it. It calls `sync_grants` (`main.rs:1013`): the grants of
  the name become the list, without an end. A temporary grant of that name not in the list
  ends. The file decides, as for sockets.
* Applied on start: `chora-flavor` runs `choractl clients up` when the file exists, before it
  starts the target. The voice unit starts after the target.
* `choractl grant NAME CAP --always` adds CAP to the entry of NAME and grants it. It rewrites
  the file by rename, mode 0600, through a symlink, as D62 saves. Comments go. It refuses a
  name whose entry has a socket.

## 5. Who raises a grant

* Only the user. Paths: `choractl grant` on the admin socket; the confirm keys; a HUD button.
* A confined client never: no admin socket (`ipc.rs:990`), no MCP tool (`mcp.rs:893`), no
  method of `voice` for the LLM (`docs/voice.md:343`). New: the core adds `confined: true` to
  `method.called`, and `voice` refuses `indicator.claim` and `confirm.answer` from a confined
  caller. That closes the gap of section 1.
* Flow in voice. `choractl mcp` puts `_meta: {missing: {capability, client}}` on a tool error of
  code 4. `tools.rs:379` keeps it. When the capability is in `grant_offers` and the client is
  `NAME-mcp` or `view-*`, the agent asks a physical question:
  "Shall I allow closing and moving windows for 10 minutes? Alt+y for 10 minutes, Alt+a for
  always, Alt+n for no." The detail is the exact command, such as
  `choractl grant voice-mcp window.any --minutes 10`.
* Yes runs `CMD grant NAME CAP --minutes N`, always runs `--always`. CMD is the command of
  `chora_mcp`, run on the host, never in `sandbox_cmd`. The LLM gets "The user allowed
  window.any for 10 minutes; call again". No or a timeout gives the usual refusal.
* `confirm_keys` gets `always` (default `Alt+a`). `voice.confirm` gets `always: true` for a grant
  question, and `confirm.answer` takes `always`. Only grant questions take it.
* New security keys: `grant_offers` (default `[window.any, scene.camera, assets.any]`) and
  `grant_minutes` (default 10). `activation.token` and `methods.privileged` are never good
  offers: each opens every other grant.
* Why physical: a voice can come from a speaker, and a text that the LLM reads can steer it.
  A spoken yes counts as nothing. Injected keys carry `injected: true` and are ignored
  (`client.rs:481`). The agent refuses `inject_input` while a question waits, so no injected
  click hits the HUD button (`docs/voice.md:396-400`).
* Rejected: a spoken "always" with a key yes. A played voice could turn 10 minutes into always.

## 6. API, docs, tests

API: `grants.set(client, capabilities, duration)`; `grants.list(detail)`; event `grants.changed`;
`method.called` gets `confined`; code 4 carries `data.capability` and `data.client` for every
capability; three capabilities; `choractl grant --minutes N | --always`; `clients.yaml`
`socket: false`; voice `confirm.answer {id, yes, always}`, `confirm_keys.always`,
`grant_offers`, `grant_minutes`, `phrases.grant`, `phrases.grant_ended`.

Docs: `docs/api.md` (methods table :472, Error codes :634, Grants :667-698, Client sockets
rules :718-767, Events, MCP tools :1145); `docs/configuration.md` Clients file (:148-179);
`docs/voice.md` section 5 and the method table (:647-655); `docs/decisions.md` D33, D40, D78;
`docs/architecture.md` revoke binding (:337); module doc of `grants.rs:1-17`; CHANGELOG.

Tests (core in `grants.rs` and `ipc/clients/tests.rs`, fake clock `testing.rs:105`):

* `window_any_lifts_the_foreign_window_rule`: close, configure, attach, focus answer code 4 with
  `data.capability`, then pass after the grant. Focus needs no press then.
* `scene_camera_lifts_the_camera_rule`, `assets_any_lifts_the_path_rule`: own assets still pass.
* `rules_without_a_capability_keep_code_9`: a `screen:*` parent, `focus_node` without a press.
* `a_free_main_socket_connection_needs_none_of_the_three`.
* `a_timed_grant_ends`: duration 60; at 59 s allowed, at 60 s code 4, `grants.changed` with
  `ended`, `grants.list` without the entry.
* `the_longer_grant_wins`, `a_duration_out_of_range_fails`, `a_socket_name_takes_no_duration`.
* `a_timed_grab_any_ends_its_free_grabs`; `the_revoke_binding_and_the_reset_end_timed_grants`.
* `a_confined_client_cannot_raise_its_own_grant`: `voice-mcp` with `confined: true` sends
  `grants.set` with and without duration: code 8, list unchanged.
* `method_called_names_a_confined_caller`.
* choractl: `grant --minutes` sends `duration`; `--always` writes a temp file and keeps other
  entries; refuses a socket entry; `clients up` with `socket: false` opens no socket and syncs.
* mcp: a code 4 error carries `_meta.missing`; `describe_screen` lists the grants; the
  no grant tool test stays.
* voice: an offered capability asks a physical question with the command as detail; another
  asks nothing; a spoken yes or always grants nothing; the yes key runs `--minutes`, the always
  key `--always`; an injected always key does nothing; a HUD button with `always`;
  `confirm.answer` and `indicator.claim` refuse a confined caller; both new keys are security
  keys.

## 7. Steps

1. Gap fix: `confined` in `method.called`; voice refuses confined callers. Tests of both.
2. Capabilities: enum, `confine_one` and `check_paths` lift, code 4 with data. Core tests.
   Docs of the rules and D33.
3. Timed grants: `duration`, `expire_grants`, `detail`, `grants.changed`. Fake clock tests.
4. choractl: `--minutes`, `--always`, `grants` print, `socket: false`, `chora-flavor` line.
   Unit tests and a temp file.
5. mcp: `_meta.missing`, grants in `describe_screen`, the hint in `INSTRUCTIONS` (`mcp.rs:58-62`).
6. Voice: the grant question, `confirm_keys.always`, the settings, the event phrase. Tests.
7. HUDs: the Always button in chora-construct (`hud.rs:53`), chora-mc and chora-xp. Tests.
8. Docs pass, D40 and D78, CHANGELOG. A live check on Gufo: grant 1 minute, see it end.

## 8. Draft decision

| D78 | A confined connection lifts a rule of client sockets with one capability per rule: `window.any` acts on any window and focuses without a press, `scene.camera` sends `camera` commands, `assets.any` names any file as image, shader or effect. A missing one answers code 4 with the capability in `data`. `grants.set` takes a `duration` in seconds; such a grant ends in core memory, with `grants.changed`; the longer grant wins. A name of the main socket keeps permanent grants in `clients.yaml` with `socket: false`, and `chora-flavor` runs `clients up` at start. Only the user raises a grant: `choractl grant`, or a key or HUD button that answers the physical question of the voice agent after a call that lacks an offered capability. A confined caller can neither claim the indicator nor answer it. D78 refines D33 and D40 | The owner wants the confinement of `choractl mcp` configurable. One capability per rule keeps the set small and the refusal exact. A grant for minutes gives what a task needs and ends alone. A voice or an injected key is no proof that the user agrees, a key or a click is. |
