# kit5 r1 (F2): kit::notice, chora-notify the only server

Branch claude/kit5, head af0dc379 (merge of the session branch 4548feb3). Base a17e7858.

## Done
- `chora_kit::notice` (show, replace, dismiss, `Gone`, `show_or_log`) calls chora-notify on the live API:
  `notify.show {app, replaces, icon, summary, body, expire_ms, urgency}` -> `{id}`, `notify.close {id}`.
  zbus is out of chora-kit (`cargo tree -p chora-kit` has no zbus); kit `notify.rs` is deleted.
- chora-notify serves the two methods beside the session bus (the bus is optional now: no bus, the plugins are
  still served) and shares the id count between both ways in. `state.get` is the NET3 one; the entry of `shown`
  gains `body`, and `node` is the card picture (`notify-card-body`), the node that `scene.boxes` lists.
- Ports: the volume worker and xp (kit calls), construct (`Construct::tell`, drawn notice deleted), mc
  (`notify.push` forwards, `notify.clear` closes the last 32 ids, own column and urgent line deleted, slot in
  `plugins/chora-mc/notify/`), tiling (chora-notify unit instead of mako, `--launch mako`, mako.conf and
  CHORA_NOTIFIER gone, launch error through kit::notice). Built by three sonnet agents in worktrees; their
  commits were cherry-picked (mc by merge, see deviation 2).
- Cells red first: kit `tests/notice.rs` (4 cells, red on a stub: 827ab5ab), tiling cut-check rows and palette test
  (7589e533), construct notice cells (34e08b31), mc act and unit test (4ac59ac7). The I9 act cell in every look
  (`notice_acts`, tests/acts/looks/common.py) came after the feat commit, not before: deviation 1.
- copies.toml rows at 0: bus client outside chora-notify, zbus in kit, tiling notify-send and mako, mc own stack,
  construct-notice. `python3 tools/cut-check`: no FAIL.
- Coverage `cargo llvm-cov -p chora-kit --features chora-kit/testing`: notice.rs 97.2 % lines, 87.5 % functions;
  crate 89.9 % lines.

## Draft decision record: where mc notices show
Context: mc stacked its own notifications (a column, an urgent footer line); chora-notify must be the only server.
Decision: `notify.push` of mc forwards to `kit::notice::show` (critical stays until clicked); `notify.clear` closes
what mc showed. The slot is `plugins/chora-mc/notify/` (values.yaml margin [160, 56], notify.yaml
`position: bottom_left`, own copies of the close icons), passed to chora-notify with `--assets` and `--config`;
it sits in the 56 px margin between the text block and the footer, clear of the chat column. No code change in
chora-notify. Consequences: mc needs chora-notify running (`notify.push` errors without it); no folding or
stacking; a card taller than title plus one line grows into the last page lines; dark colours only; no
`flavors/mc` unit starts chora-notify for mc yet.

## Gate state (honest)
- Head af0dc379: `tests/merge-gate.sh` full run in progress at this report: 23 steps PASS (red-first to chat-keys,
  incl. xp, matrix, virtual, xp-*, hud, layout, tiling-audit, tiling-look), then doors, acts and the rest were
  still running (log /tmp/merge-gate.pMDLqq, scratch gate5.log; the container is not durable).
- On the head before the session-branch merge (06c7ad6d, split over two runs) every step passed: run 3
  (/tmp/merge-gate.hrJJlF) to tiling-audit, run 4 (/tmp/merge-gate.f5YFkU) tiling-look to e2e (all PASS).
  So there is no single run that finished on one head; the merged head's remaining steps are not proven yet.
- Fixed on the way (gates that read notices from the bus): tests/xp.sh, gate-xp-notepad-fallback.sh,
  gate-xp-notepad-gui-only.sh, gate-tiling-power.sh now read `state.get` of chora-notify.
- Not run: shellcheck and desktop-file-validate are not installed (gate-0.9-session-units skipped by the agent);
  mc gate scripts changed by the mc agent ran only through merge-gate steps mc-paths, mc-sofa, mc-handover
  (all PASS on 06c7ad6d); demo tours v5-v8 still call `mc notify.push` and now need chora-notify with the mc slot.
- Acts on the merged head: `gate-acts.sh --no-selftest` 86 pass, 8 known red, 0 blocking.

## Deviations
1. I9 cell and kit feat order: the I9 cells came after the kit feat commit. The NET3 I9 (already on the
   session branch) replaced mine in the merge; my cells now only send notices, NET3's invariant judges them.
   NET3's `node: notify-card` pointed at a group that has no box, so I9 failed live in all four looks until
   the state named the picture node. That is a defect of the NET3 pair, found by this lane's cells.
2. History: an agent worktree based on dev (d060dc03, rc.4) was merged into claude/kit5 before I saw it, so the
   branch contains the dev rc.4 merge history. The release files (CHANGELOG, Cargo version, two tour notes) were
   restored in 6aad3457; the PM should merge claude/kit5 knowing d060dc03 is an ancestor, or cherry-pick the
   feature commits (listed by `git log --no-merges a17e7858..claude/kit5`). I did not force-push.
3. Tiling: chora-notify has no chat mode, so a tiling notice can cover the chat panel input; the `chattoast`
   audit cell was removed and `toast` checks the card is in the lower-right quarter and clear of the helper dot.
4. Plugins may not name other plugins (new cut-check rule): comments and one user text now say "the notification
   server".

## Open
- Wait for the merged-head gate (doors onward) and rerun it once if a step fails; then the PM merges.
- Unit/flavor for chora-notify with the mc slot; tiling chat avoidance in chora-notify.
- docs/decisions.md D-record for the mc slot (draft above); the PM files it.

## Refactor proposals (ranked)
1. M: mc region.rs dead notification machinery (`folds`, `empty`, `note_items`, `folded_items`, `split_time`,
   `Item.shed`, `fit`, `shed_updates`, `Item.mark`, close-mark drawing): about 300 lines and two option docs.
2. S-M: chora-notify "avoid node group" option so the card steps aside for the chat panel (card.rs:251).
3. S: `flavors/shared/screenshot.sh:8-50` drop CHORA_NOTIFIER, say_on_own_bus, daemon_runs (no flavor sets it).
4. S: gate-lib helpers: one pair for notify (`notify_start`/`start_notify`, `notices`/`shown_notices` exist twice);
   `start_bus` for tiling-audit and tiling-look-r2.
5. S: construct input.rs `give_key` builds words and calls `tell` in several places; one `refused_give`.
6. S: plugins/chora-mc/notify/ holds copies of the two close icons of chora-notify (cut rule: no links out).

## r2 (F2): kit5 on the session branch without the dev merge, gate proven

Branch claude/kit5b, head 4e003748, from claude/autonomous-dev-session-3w-i33pg7 (4548feb3). 18 commits, no merge,
no commit of dev (`git log origin/claude/autonomous-dev-session-3w-i33pg7..HEAD`).

### Picks
17 of the 18 kit5 commits of `git log --reverse --no-merges <session>..origin/claude/kit5 ^origin/dev`, in order:
827ab5ab 7123479e 855445c4 4ac59ac7 f5ba85c2 c5bbaf54 34e08b31 9f957abf 4908c52a e0a70383 7589e533 57dccba8
59399e78 7cdde49c 03a5e045 ef81d6a4 06c7ad6d. Skipped: 6aad3457 (release-file restore). No other commit was release only.

### Conflicts (resolved to both sides, the kit5 merge af0dc379 as the model)
- 7123479e, plugins/chora-notify/src/main.rs (+ card.rs): NET3's `state_value` with the kit5 `notify.show` and
  `notify.close` methods, bus optional, `body` in `shown`, `node` is `card::BODY`; the duplicate `state()` of the pick
  dropped. The file is kit5's.
- 855445c4, tests/acts/{harness,invariants,looks/construct}.py and tests/test_acts.py: NET3's harness and I9 stay;
  the pick's own I9, `notices` and `dismissed` arguments dropped; construct keeps the cube acts and adds `notice_acts`.
- 9f957abf, plugins/chora-construct/src/{input,keys}.rs: the session branch moved the construct onto kit::actions
  (`System`, `Own`); the construct tree of kit5 is taken (`tell`, `say_failed` returns nothing).

### Own commit
- 4e003748 fix: the picked kit5 cells follow the session branch. These changes were made inside the kit5 merge commit and
  no pick has them: cut-check names/assets rule (mc and tiling say chora-notify in words no rule matches, the
  mc notify slot holds copies of the two close icons, not links), and the notice acts of mc and common.py without the
  `notices`/`dismissed` arguments. Red before: `python3 tools/cut-check` FAIL on 2 assets and 2 names rows.

### Differs from claude/kit5
`git diff origin/claude/kit5 HEAD`: only demo/tour/notes/v1-ch3.json and v2-ch3.json. Kit5 took them back to the rc.4
text in 6aad3457 ("NOTES and NOTIFICATIONS"); here they keep the text of c5bbaf54 ("only the heading NOTES", since mc
has no notifications column).

### Done criteria
- `cargo test -p chora-kit --features testing -p chora-notify -p chora-xp -p chora-mc -p chora-tiling -p chora-construct`: ok, exit 0.
- kit `notice.rs` coverage (`cargo llvm-cov -p chora-kit --features chora-kit/testing`): 97.2 % lines, 87.5 % functions; crate 90.3 % lines.
- `tests/merge-gate.sh` full run on 4e003748 in one run: all steps PASS (red-first to e2e, 41 steps), log dir
  /tmp/merge-gate.eoD64e, stdout /tmp/gate-kit5b.log (the container is not durable). This closes the open item of r1.
- Not run: shellcheck, desktop-file-validate (not installed, as in r1).

## Review r1

claude/kit5b 4e003748 over 4548feb3 (18 commits, no merge, no dev commit). Verdict: **FIX** (0 BLOCK, 3 MAJOR, 5 MINOR).

Checks run: `cargo test -p chora-kit --features testing --test notice` 4/4 ok, `cargo test -p chora-notify` 19/19 ok;
red first replayed in a worktree: 6853fa1 (827ab5ab) kit 4/4 red, 6ddd808 (4ac59ac7) mc red, 8117979 (7589e533)
tiling palette test and 3 cut-check rows red, 6ae8a8f (34e08b31) construct does not compile on kit5b (stale
`shared_key(Action::Chat)`), red 3/3 once that call is patched; `python3 tools/cut-check` ok (no FAIL);
`cargo tree -p chora-kit` has no zbus; no file crosses 1,500 lines (mc main.rs, region.rs and tiling main.rs were over
before and shrink). Conflict picks: invariants.py untouched (NET3 I9 kept), notify main.rs keeps NET3 `state_value`,
construct input/keys keep the session `System`/`Own` actions with `tell`; nothing of the session side lost.
4e003748: the cut-check part is a real fix (words, copied icons); the act part drops the only expectations (rw-4).

- rw-1, MAJOR, plugins/chora-notify/src/main.rs:224, when another daemon owns org.freedesktop.Notifications (mako was a recommended package until this diff and its D-Bus activation file wins any early notify-send) chora-notify returns `Ok(None)` and exits 0, so `notify.show`/`notify.close` are unserved and every kit::notice of all four looks is lost (Restart=on-failure does not restart it); fix: on `Taken` keep serving the live API with `bus: None`, as for no bus.
- rw-2, MAJOR, flavors/construct/chora-construct.target:7, the construct flavor starts no chora-notify, yet the construct now draws no notice and `Construct::tell` only logs without the server, so every refusal, dead key and "no room" notice of a construct session is lost (mc has no unit for its slot either); fix: add chora-construct-notify.service to the target (and a unit or documented start for the mc slot), packaged in RPM and PKGBUILD.
- rw-3, MAJOR, flavors/tiling/chora-tiling-notify.service:7, the mako `[mode=chat]` margin (424) and the `assistant.chat` follow in chora-tiling are deleted with no replacement, so a tiling notice can cover the chat panel input (deviation 3 of r1), against the vision card (never covers work) and done criterion 5 (visible behaviour unchanged); fix: chora-notify steps aside of the chat panel on `assistant.chat` (the S-M proposal 2 of r1) before the merge.
- rw-4, MINOR, tests/acts/looks/common.py:78, the notice acts (and mc.py push/clear) carry no expectation after 4e003748: I9 is vacuous when nothing is accepted and never checks a dismissal or `notify.clear`, and no commit on kit5b shows I9 red (its red was seen live only in the kit5 merge); fix: in the acts wait on `s.state("notify")` for the summary in `shown` after show/push and the id in `closed` after close/clear.
- rw-5, MINOR, plugins/chora-mc/src/agent.rs:826, `notify.clear` closes only the last 32 ids, which expired ones fill too, so an older critical notice (stays until a click) survives a clear, and `mem::take` drops the remaining ids on the first `Gone` (each dismiss may wait 2 s); fix: keep only ids not yet closed (critical ones always) and keep the ids whose dismissal failed.
- rw-6, MINOR, docs/api.md:1552, api.md documents neither `notify.show {app, replaces, icon, summary, body, expire_ms, urgency} -> {id}` nor `notify.close {id}`, the id shared with the bus, nor `body` in `shown` and `node` as the card picture; fix: add the two methods and the changed `shown` entry to api.md.
- rw-7, MINOR, flavors/shared/screenshot.sh:42, with CHORA_NOTIFIER gone a session without a session bus no longer gets the own-bus daemon, and chora-notify serves no bus there, so the screenshot notice only reaches stderr; fix: fall back to `choractl` `notify.show` when notify-send fails, and drop the dead CHORA_NOTIFIER path (proposal 3 of r1).
- rw-8, MINOR, crates/chora/Cargo.toml:47, the RPM assets lack `plugins/chora-mc/notify/*` while PKGBUILD installs it to /usr/share/chora/mc/notify and docs/configuration.md tells the person to start chora-notify from there; fix: add the asset row to the generate-rpm list and to dist/build-arch.sh checks.

## r3 (F2): review r1 fixed, gate green

claude/kit5b head 7b01dd5a. Merge d7cd08b9 of the session branch (head daf83391, no rebase); conflicts, each to
both sides: chora-kit Cargo.toml (tests notice and launch), kit testing.rs (`answers` and `mints_tokens`/`selects`),
copies.toml (KIT5 and KIT6/KIT2c rows), tiling main.rs imports, xp dispatch.rs (session `window_created`; kit5b
`dismiss_notice` moved into lifecycle.rs), xp startup.rs (no `Notifier`), mc region.rs (session split into
region/items, hits, fit; kit5b removal of urgent/notify applied there).

| Finding | Red | Fix |
|---|---|---|
| rw-1 MAJOR | 4da3634e (notify unit test on a private bus with the name taken) | bb650abb; 7b01dd5a: a second chora-notify (bus name and core name `notify` both taken) still ends 0, red was step 17 of tests/xp.sh at 5157c260 |
| rw-2 MAJOR | c6031968 (tests/flavor-notify.py in ci-check and CI) | 9e388870: chora-construct-notify.service in the target (packaged by the flavors/construct/* globs, build-arch.sh row); mc: documented start in README and configuration.md |
| rw-3 MAJOR | bdb5af3a (card/chat unit tests red on stubs; gate-chat-keys cell: card box vs `hud-chat` box from `scene.query`, red: card [1102,692,162,52] covers chat [868,12,400,776]) | 03e06230: assistant puts `rect` in `assistant.chat`; chora-notify keeps the card beside it on that output, lists `rect` and `chat` in `state.get`; `notify.ready` makes the host say an open chat again |
| rw-4 MINOR | f1df4b1f is test only; red shown locally with `notify.close` stubbed (2 act cells FAIL), not committed | f1df4b1f |
| rw-5 MINOR | 7d9de9a4 | 40631574: critical ids never pushed out by the cap, a clear stops at the first Gone and keeps the rest |
| rw-6 MINOR | docs only | c440ed57 |
| rw-7 MINOR | a5f4f048 (tests/screenshot-notice.sh, fake grim/choractl, no bus) | 2cbeaa44: notify-send, else `choractl call notify notify.show`; CHORA_NOTIFIER path removed |
| rw-8 MINOR | c16e2449 (packaging-files.py reads plugins/*/notify) | 5157c260 |

Checks on 7b01dd5a: `cargo test -p chora-kit --features testing -p chora-notify -p chora-mc -p chora-tiling
-p chora-construct` all ok; `python3 tools/cut-check` ok; `tests/merge-gate.sh --only build,ci-check,acts,xp,hud,
construct-hud,mc-handover,keys,chat-keys` all PASS (run /tmp/merge-gate.4KGfXi; the first run on 5157c260 failed at
xp step 17, fixed by 7b01dd5a).
Open: mc still has no unit of its own (documented start only); the `assistant.chat` `rect` and `notify.ready` are
new event fields for docs/api.md's event list if the PM wants them there (documented in configuration.md).

## Review r2 (F2): r3 fixes of kit5b checked

claude/kit5b head 7b01dd5a (r1 head 4e003748, merge d7cd08b9 of daf83391). Verdict: **MERGE-OK** (0 BLOCK, 0 MAJOR,
2 MINOR, 1 NIT). No merge gate run, as briefed.

Checks run: step 0 ok (`cargo check -p chora`; foot, fuzzel, waybar present). On 7b01dd5a `cargo test -p chora-notify
-p chora-mc` 22/22 and 139/139 ok (dbus-daemon present, so the private-bus tests ran, not returned early), `cargo test
-p chora-kit --features testing --test notice` 4/4 ok; `cargo clippy -p chora-notify -p chora-kit -p chora-mc
--all-targets` clean; `cargo check -p chora-xp -p chora-tiling -p chora-construct --all-targets` ok;
`python3 tools/cut-check` ok; `tools/size-ratchet --base daf83391` ok (one file over 1,500, the same as before; the
others over 1,500 are docs and old files that kit5b does not grow past the ratchet).

1. Red then green, each replayed at its red sha in a worktree:
   rw-1 4da3634 `a_taken_name_leaves_the_live_api_served` FAILED, ok at 7b01dd5a. rw-3 bdb5af3 `the_card_steps_aside_of_
   the_chat_panel` and `the_chat_of_the_host_names_its_box` FAILED, ok at head. rw-5 7d9de9a `a_clear_dismisses_an_old_
   critical_notice` FAILED, ok at head (`a_failed_dismissal_keeps_the_ids` ok). rw-2 c603196 `tests/flavor-notify.py`
   rc 1 (2 gaps: construct target, mc README), head 0 gaps. rw-7 a5f4f04 `tests/screenshot-notice.sh` FAIL (no
   notify.show through choractl), head ok. rw-8 c16e244 `tests/packaging-files.py` 5 gaps (rpm: mc notify files), head
   0 gaps. rw-6 is docs: `docs/api.md` now lists `notify.show`, `notify.close`, `shown` with `body`, `node`, `rect`,
   `chat` (read, matches the code). rw-4 f1df4b1 is test only, as the report says.
   rw-1 is right as reported: bus name taken and core name `notify` free (mako) keeps serving the live API with
   `bus: None`; both taken ends 0 (see r2-1 for the one case outside that). rw-3 cell read: gate-chat-keys starts
   chora-notify after the chat is open (so it also exercises `notify.ready` and `chat_again`), shows a notice, takes
   the `rects` of `hud-chat` and `notify-card-body` from `scene.query` and fails on any overlap or when it does not get
   exactly two boxes: not vacuous; `Frame::rect()` is the box of the view (the slot box for a slot). rw-4 read:
   `shown_id` waits for the summary in `shown` of `state.get` of notify, `closed` for the id in `closed` and not in
   `shown`; `state_filter` is only set by planted.py, so `state("notify")` is unfiltered in the looks; a failed show
   makes the dismiss act fail by `shown_before`: not vacuous. rw-2: target Wants, unit, build-arch row, README and
   configuration.md for the mc slot (documented start only, as r3 says).
2. Merge d7cd08b9: `git show --cc` of the 7 conflict files read. chora-kit Cargo.toml has `launch` and `notice` tests;
   lib.rs has `launch` and `notice`; testing.rs has `answers`, `mints_tokens`, `selects`, `answer_with`,
   `mint_tokens`; copies.toml has the KIT5 rows and the KIT6 and KIT2c rows; tiling main.rs imports `actions` and
   `notice`; xp lifecycle.rs `window_created` calls `dismiss_notice` (panels.rs), startup.rs has `book` and no
   `notifier`; mc region.rs keeps the `fit`/`hits`/`items` split, the `fit` and `column_width` tests live on in
   region/fit.rs and only the urgent test and `URGENT` are gone (kit5b removal). Nothing of daf83391 lost.
3. No new work beyond the findings.

- r2-1, MINOR, plugins/chora-notify/src/main.rs:246, the exit 0 needs the bus name taken as well as the core name `notify`, so a second chora-notify in a session whose first one runs without a bus (`Err(NoBus)`, the case rw-7 names) gets NAME_TAKEN, returns `Err` and Restart=on-failure starts it every second; fix: end 0 on NAME_TAKEN of the core name alone (drop the `matches!(bus, Err(Taken))`).
- r2-2, MINOR, plugins/chora-mc/src/agent.rs:50, `Notices::remember` never drops a critical id, also not one that the person clicked away, so the list grows by one per critical notice until a clear (8 bytes each, a clear closes them all as a no-op); fix: cap the critical ids too (e.g. 64, oldest first).
- r2-3, NIT, plugins/chora-notify/src/card.rs:274, `x.max(0)` leaves the card over the chat when the chat is wider than the room beside it (the unit test `wide` asserts `[0, 704]` over a chat from x 100); fix: none needed for a chat panel of 400 px, or fall back to a vertical step.

Open from r3 stays: mc has no unit of its own for the notice slot (documented start only).
