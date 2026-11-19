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
