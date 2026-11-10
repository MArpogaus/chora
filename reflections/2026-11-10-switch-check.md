# Switch check: fresh PM session per run (2026-11-10, 12:35 Bangkok)

Verdict: **NO-GO as the files stand at 12:35. GO at 16:58 if findings 1
to 5 are fixed and the dry run of finding 4 passes.** All five are small
edits or one test; none needs the user.

Checked read-only in the old container: `bash -n setup.sh` (rc 0),
`/var/log/dpkg.log` and `/var/log/apt/history.log`, `~/.local/bin`,
`~/.cargo/bin`, `uv tool list`, the trigger (`get_trigger`), repo access,
`gh`, git state of the clone and of every worktree. Not checked: what
`add_repo` does in a session that starts without a repository; nothing
here can show that (finding 4).

What a fresh container has (base image of 10-24 and 10-27): rustup with
stable 1.97, uv, ruff, jq, ffmpeg, pkg-config, python3-gi, python3-pil,
dbus-daemon, libgl1-mesa-dri, gh. What this session installed later, and a
fresh container lacks: the 11 build packages (11-08), rust 1.99 (rustup
fetches it by itself), sway, swaybg, foot, grim, waybar, fuzzel, wlrctl,
swaylock, wlr-randr, python3-dbusmock (11-09), pre-commit, vale,
commitizen, git-filter-repo, lychee, cargo-deny 0.20.2, cargo-machete
0.9.2, `~/.cargo/config.toml`, the local git identity, the hooks,
`/home/user/guard-cache`, `/home/user/wt/*`, `/home/user/agent_skills`,
the transcripts under `~/.claude/projects`, the scratchpad.

## Blockers

### 1. setup.sh installs no build package; `cargo build` fails

`libdisplay-info-dev libdrm-dev libinput-dev libseat-dev libsystemd-dev
libudev-dev libwayland-dev libxkbcommon-dev libegl1-mesa-dev libgbm-dev
libegl-mesa0` were installed by hand on 11-08 (apt history). setup.sh
installs only the e2e packages. The charter names the build packages as
a fallback (section 3 step 1), so a careful PM recovers, but the routine
says setup.sh does the tools.

Fix, in setup.sh, replace the `for p in ...` line:

```bash
build_pkgs="libdisplay-info-dev libdrm-dev libinput-dev libseat-dev libsystemd-dev libudev-dev libwayland-dev libxkbcommon-dev libegl1-mesa-dev libgbm-dev libegl-mesa0 libgl1-mesa-dri"
e2e_pkgs="sway swaybg foot grim waybar fuzzel wlrctl swaylock wlr-randr python3-dbusmock jq"
for p in $build_pkgs $e2e_pkgs; do
```

and make the install survive one bad package:

```bash
    apt-get update -qq
    DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "${need_apt[@]}" >/dev/null ||
        for p in "${need_apt[@]}"; do
            DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "$p" >/dev/null || log "FAILED apt: $p"
        done
```

(`swaybg` is a dependency of sway today; `tests/2d-guard.sh` dies
without it, so name it.)

### 2. The fresh clone is probably shallow and single-branch: the 2D guard, the merge gate and the log worktree fail

- `tests/merge-gate.sh` runs `tests/2d-guard.sh 8742dee`. 8742dee is 221
  commits behind HEAD (commit date 11-01). Cloud clones are shallow (this
  one is: `rev-parse --is-shallow-repository` = true; it was deepened to
  658 commits at some time). With the usual depth of 50, `git rev-parse
  8742dee^{commit}` fails, the guard exits 2, and no merge can pass the
  gate. `review_due.py` (`last_reviewed..HEAD`) and `git merge` of older
  branches fail the same way.
- If the clone has a single-branch fetch refspec, `git fetch origin A B`
  writes only FETCH_HEAD, `origin/claude/session-log` does not exist, and
  `git worktree add /home/user/wt/log claude/session-log` ends with
  "invalid reference". Then there is no setup.sh, no charter, no run.

Fix, routine.md step 1, replace the text after "so that the clone is at
/home/user/chora." with:

```
Then run:
git -C /home/user/chora config remote.origin.fetch '+refs/heads/*:refs/remotes/origin/*'
git -C /home/user/chora fetch --unshallow origin || git -C /home/user/chora fetch origin
git -C /home/user/chora cat-file -e 8742dee^{commit}   (must print nothing; else stop and report)
git -C /home/user/chora checkout -B claude/autonomous-dev-session-3w-i33pg7 origin/claude/autonomous-dev-session-3w-i33pg7
git -C /home/user/chora worktree add -B claude/session-log /home/user/wt/log origin/claude/session-log
If add_repo or the clone fails twice, try `git clone https://github.com/MArpogaus/chora /home/user/chora`; if that fails too, write the error as the report and stop.
```

Update the trigger prompt with the same text (`update_trigger`,
trig_01WbCwdAwHzBa5fPkqjSMvar); the stored prompt equals routine.md today.

### 3. state.md does not describe what is in flight; a fresh PM would continue the wrong work

At 12:30 Bangkok:

| state.md says | Real state (git) |
|---|---|
| Running: H4b (`claude/hud-h4b`), 3D step 6 (`claude/3d-step6`) | both merged in a03be22 (e1b35dc, a03be22) and pushed |
| nothing about effects | `/home/user/wt/effects` on the LOCAL branch `merge-effects`, merge commit 30d1216, 6 ahead of origin; `tests/merge-gate.sh --push` is running now. The merge commit exists only in this container |
| nothing about give-key | `claude/give-key` pushed at 2339c0a (a red test commit); 7 files modified and uncommitted in `/home/user/wt/give-key` (an agent is fixing) |
| nothing about H7 | `claude/hud-h7` pushed at 4446395 (review fixes); not merged; no note whether its review is clean |
| `block: three work branches in parallel, started 11:50` | which three, and which model runs them, is not said |
| Foundation gate table, "status at the end of 11-09": 3D 2 of 5, #102 open | 3D steps 0 to 6 merged, H4c fixed #102 |
| "Next step" items 0 to 5 | the list of 11-09 ("Then 3D step 0", "Review round on 02cdeed..HEAD (running)") |
| "Plan of the day" | done before noon; no plan for the rest of 11-10 |
| `base_2d: ... no scene or renderer change since` | 3D steps 2 to 6 changed the renderer; step 6 (#76) changed 2D sampling; the guard still passes against 8742dee, say so, or move the base |

Also missing: the `ledger.md` row of 11-10, `reports/2026-11-10.md`, and
the answers.md entry "Decisions after the Fable judge audit (12:45
Bangkok)" carries a time that had not come yet (friction 19 again).

Fix, before 16:45, the old PM:

1. Lets the effects gate end; pushes or records the result. Stops or
   finishes every running agent; every worktree is clean and pushed
   (`git -C /home/user/wt/X status -sb` shows no `M`, no `ahead`). Work
   that cannot finish is pushed as a WIP commit through the hooks.
2. Rewrites state.md: `block: none`; the foundation table from today's
   facts; one "Plan" for 11-10 evening and 11-11; "Next step" as an
   ordered list of today; and "Running" in this form, one entry per
   branch that is not merged:

```
## Running

- claude/give-key (give keys ignore injected input; design/window-handover.md
  section 6; answers.md 11-10 decision 2). Pushed: 2339c0a, the red test.
  Open: the fix (kit runtime, construct, mc), then an opus security
  review, then merge. Developer: sonnet. Finish: recreate the worktree,
  brief a developer with dev-rules.md, review, merge with
  `tests/merge-gate.sh --push claude/autonomous-dev-session-3w-i33pg7`.
- claude/hud-h7 (...). Pushed: 4446395. Review: <clean | findings: ...>.
  Open: <merge | fixes>.
- claude/effects (...). Pushed: 9f03f14. Review: done, fixes in 9f03f14.
  Open: merge (the gate ran green on <date> | the gate failed at <step>).
```

3. Writes the ledger row of 11-10 with `seen` from `tokens.py` (finding
   8) and the run report, pushes the log branch, and does no PM work
   after 16:45.

### 4. The setup path was never run; the first real run is the test, with nobody watching

Unknown and not checkable from the old container: whether `add_repo`
with push needs a confirmation of the user in an unattended session
(its description says a confirmation is possible on some surfaces, and
there is a repository-trust question that only the owner can answer);
where its clone lands; whether `GH_TOKEN` and push rights exist then;
whether a session without a repository gets the permission mode that
lets `apt-get` and `cargo` run without prompts.

Fix: a dry run while the old PM still watches, as early as possible
(13:00). After fixes 1 and 2 are pushed and the trigger prompt is
updated: `fire_trigger` trig_01WbCwdAwHzBa5fPkqjSMvar with the text

```
DRY RUN. Do only setup steps 1 and 2, then in /home/user/chora:
`cargo build --workspace` and `tests/merge-gate.sh --only 2d-guard,xp`.
Then push an empty test branch: `git push origin HEAD:refs/heads/claude/switch-dryrun`.
Commit nothing, change no log file, do not touch the status page.
Report each step with its exit code and seconds, and the output of
`df -h /home/user`, `git -C /home/user/chora rev-parse --is-shallow-repository`,
`gh api repos/MArpogaus/chora --jq .permissions`.
```

Read the result with `list_events` (kinds assistant, result). GO only
if clone, setup, build, the 2D guard and the push pass. If `add_repo`
waits for the user: NO-GO; re-enable the old trigger
(trig_012VGJoVzsJ81hFa6AufBc9S), disable the new one, and ask the user
in questions.md to attach the repository to the scheduled task.

### 5. Two PM sessions can work at once

Before, a firing queued in the one session. Now a new session starts at
05:58, 11:58 and 16:58 whether or not the last one still works (runs of
today lasted more than 5 hours), and the old persistent session wakes
whenever one of its background agents ends. Two PMs then merge and push
the same branches and overwrite state.md and the status page.

Fix, routine.md, add after step 3:

```
4. One PM at a time. state.md has a line `pm: <session id>, started <time>`.
   If it names another session and `get_session` of that id shows it
   working, wait 10 minutes once; if it still works, write one line to
   the report and stop. Else write your own id (get_session without an
   id) and the time (`TZ=Asia/Bangkok date`), and push.
   End your run at most 4 h 30 min after its start: stop the agents, push
   every branch as it is, update state.md, set `pm: none`.
```

and charter section 11: "The old persistent session
(session_016iS3kQmq6Q6Ce3hxtenYSR) does no PM work after 11-10 16:45."

## High

### 6. GitHub issues and Dependabot PRs: no GitHub MCP, and `gh issue`/`gh pr` fail

Measured here: `gh issue list` and `gh pr list` answer "HTTP 403: GitHub
GraphQL is not available"; `gh api repos/MArpogaus/chora` (REST) works
with push and triage rights. Charter section 1 item 4 and section 8 say
"the GitHub tools of the PM session".

Fix, charter section 1 item 4, replace "Issues through the GitHub tools
of the PM session" with:

```
Issues through the REST API with `gh api` (GraphQL, and so `gh issue`
and `gh pr`, is blocked): list `gh api 'repos/MArpogaus/chora/issues?state=open&per_page=100'`,
read `gh api repos/MArpogaus/chora/issues/N`, create `gh api repos/MArpogaus/chora/issues -f title=... -f body=... -f 'labels[]=bug'`,
comment `gh api repos/MArpogaus/chora/issues/N/comments -f body=...`,
close `gh api -X PATCH repos/MArpogaus/chora/issues/N -f state=closed -f state_reason=completed`.
Dependabot PRs: `gh api 'repos/MArpogaus/chora/pulls?state=open'`; never merge them.
```

Include `gh api` in the dry run (finding 4).

### 7. setup.sh reports "done" and exit 0 whatever failed

Every install can fail without a trace (`-q`, `>/dev/null`, `set -u`
only); the last line asks the PM to check by hand.

Fix, replace the last line of setup.sh:

```bash
missing=()
for t in cargo pre-commit vale cz lychee sway swaybg foot grim jq gh; do
    command -v "$t" >/dev/null 2>&1 || missing+=("$t")
done
for c in deny machete; do cargo "$c" --version >/dev/null 2>&1 || missing+=("cargo-$c"); done
pkg-config --exists libdisplay-info libdrm libinput libseat libudev wayland-server xkbcommon egl gbm || missing+=("build-packages")
[ "$(git -C /home/user/chora config --local user.email 2>/dev/null)" = "38564291+MArpogaus@users.noreply.github.com" ] || missing+=("git-identity")
[ -x /home/user/chora/.git/hooks/commit-msg ] || missing+=("hooks")
if [ "${#missing[@]}" -gt 0 ]; then log "MISSING: ${missing[*]}"; exit 1; fi
log "done, all present"
```

and in routine.md step 2: "If it prints MISSING, run it once more; what
is still missing goes to friction.md, and a missing build package,
cargo, git identity or hooks stops the run with a report."

### 8. tokens.py counts only the current session; the day's budget is undercounted

`~/.claude/projects` starts empty each run. The carry rule of section 10
works only if every run writes `seen` before it ends; a run that the
5-hour limit stops loses its tokens, and the 11-10 row does not exist
yet, so the about 10 % of the old container vanish at 16:58 and the
17:00 run sees a nearly unspent day (19 % allowed).

Fix: (a) the old PM writes the 11-10 row now (finding 3). (b) Charter
section 10, replace the carry rule with:

```
Each run is a new container. At run start: add `seen` of today's row to
`carry` (if the row is of today), set `seen` to 0,0, and write the
session id of this run into the note column. After each block and at the
end: `tokens.py DAY --carry CR,OUT`, write `seen` and the percent, push.
Cross-check at run start: for the session id of the last run in the
ledger, `list_events` (kinds result, limit 5) gives `total_cost_usd`;
if the cost divided by $13 per percent is more than 2 points above that
run's ledger share, take the larger value.
```

Run start step 4 keeps its text otherwise.

### 9. The user and the PM

The user can still reach a PM: each firing makes a new session in the
session list ("Chora PM run (fresh session)"), and the routine tells it
to answer. Gaps: (a) a message to an older run session, or to the old
persistent session, wakes a PM with stale files that may start work
(finding 5); (b) weekly-limit readings and answers given there never
reach answers.md unless that session pushes them; (c) between runs
nobody answers.

Fix, routine.md, add to the rules paragraph:

```
If the user writes after your run has ended, or you are not the session
named in state.md `pm:`: pull the log branch, write the message into
answers.md, push, answer in one line, and do no other work.
```

and one line on the status page under "needs you": "Write to the newest
session named Chora PM run, or wait for the next run (05:58, 11:58,
16:58)."

### 10. cargo-deny and cargo-machete build from source with whatever toolchain the working directory gives

Outside the repo `cargo` is stable 1.97 (base image); the newest
cargo-deny may need a newer compiler, and each build takes minutes of
every run although both tools run once a day.

Fix, setup.sh: pin and use the repo toolchain, and do not block the run:

```bash
for c in cargo-deny@0.20.2 cargo-machete@0.9.2; do
    n=${c%@*}
    cargo "${n#cargo-}" --version >/dev/null 2>&1 ||
        { log "cargo install $c (background, log /tmp/$n.log)"; (cd /home/user/chora 2>/dev/null; nohup cargo install -q --locked "$c" >"/tmp/$n.log" 2>&1 &); }
done
```

(then drop both from the MISSING check of finding 7, or check them only
before the daily `cargo deny check`).

## Medium

### 11. Charter text that assumes one long session

| Where | Text | Change to |
|---|---|---|
| Intro, line 3 | "fire into the Claude Code session that wrote this plan (the PM session)" | "each start a fresh PM session in a fresh container (section 11)" |
| Section 1 item 13 | "07:23, 12:23 and 17:23" | "05:58, 11:58 and 16:58" (also section 11a "the 17:23 run") |
| Section 1 item 2 | "in the chat of the PM session" | "in the chat of the newest run session" |
| Section 3, first paragraph | "(the PM session keeps them in context)" and `agent_skills` | "`/home/user/agent_skills` does not exist in a fresh container; this section and dev-rules.md are the rules" |
| Section 3 step 1 | the install list | "`setup.sh` did this; check its last line" |
| Section 3 step 2 | "(the PM session has it; clone it only when the container is fresh)" | "(the routine cloned it)" |
| Section 3 step 3 | leftovers in worktrees | "A fresh container has none; what an earlier run left is on the pushed branches of state.md Running" |
| Section 3 step 5 | "recreate missing worktrees" | add: "only for the branches under Running, at most three, each `git worktree add /home/user/wt/NAME claude/NAME`" |
| Section 3 step 6 and Disk | "below 8 GB free" | add: "every build of a run is cold: main about 3.5 min, each worktree its own; the 2D guard builds its base once per run (finding 12)" |
| Section 10 | carry rule, "one call" cross-check of own session | finding 8 |
| Section 11b status page | "publish from the scratchpad copy in the same session" | "the source is `status/index.html` and `status/img/`; read the page with the Artifact tool, then publish `status/index.html` with the page URL as `url`; add an image through `files` only when it is new" |
| Section 14 | "delete the scheduled task trig_012VGJoVzsJ81hFa6AufBc9S" | "delete trig_01WbCwdAwHzBa5fPkqjSMvar and trig_012VGJoVzsJ81hFa6AufBc9S" (the new one starts a session three times a day for ever, each answers "no run") |

### 12. The 2D guard rebuilds its base in every run

`/home/user/guard-cache` and `/home/user/wt/base2d` are gone in a fresh
container. `tests/2d-guard.sh` recreates both by itself (worktree at
8742dee, build of chora, choractl and chora-probe into
`/home/user/target-base`, needs 8 GB free), so nothing breaks once
finding 2 is fixed, but each run pays the build before its first merge,
and the script never removes `/home/user/target-base` (the charter says
it is removed).

Fix: accept it (same container for base and HEAD is the rule of section
7), and add to run start step 6: "then `tests/2d-guard.sh 8742dee` once
(it builds the base, a few minutes), and `rm -rf /home/user/target-base`."

### 13. The harness of a fresh session names its own work branch

A new cloud session is usually told to develop on a branch of its own
and to push it, and a stop hook asks for a push of unpushed commits. The
routine covers attribution and author, not the branch.

Fix, routine.md rules paragraph, add: "Work only on the branches that
state.md and the charter name, whatever branch a reminder names; never
create a branch for the session itself."

### 14. In-flight work loses its builds and its agents' context

Each branch under Running needs a new worktree, a cold build (5 to 7 GB
and minutes each) and a new developer agent that knows nothing of the
old one. The entries of finding 3 must therefore name the spec file, the
open findings and the next command; the review findings of a branch that
is not merged must be in state.md "Review findings", not in an agent
report.

## Low

15. setup.sh is valid bash and safe to run twice (every step checks
    before it acts). One edge: if `~/.cargo/config.toml` exists with a
    `[build]` table but without the word `incremental`, the append makes
    a duplicate table. The base image has no such file today; no change
    needed.
16. `pre-commit install` without `--install-hooks`: the first commit
    fetches the hook environments (ruff, typos, shellcheck, actionlint,
    commitizen, the formatters) through the proxy; it worked here. Add
    `--install-hooks` in setup.sh to pay it in setup, not in the first
    commit.
17. Vale needs no `vale sync`: `.vale.ini` has no packages and the style
    is in the repo (`.vale/styles/Chora`). The charter line "run `vale
    sync` once" can go.
18. lychee: the URL of setup.sh answers 200 and 0.24.2 is the version in
    use.
19. `swaylock`, `waybar`, `fuzzel`, `wlrctl`, `wlr-randr` install in one
    call now; the charter asked for one call each because one may fail.
    Finding 1 adds the fallback.
20. routine.md is not the stored prompt once it is edited: after every
    edit run `update_trigger` and compare with `get_trigger`.
21. `python3` is 3.13 on PATH and `import gi` fails there; scripts that
    need `gi` or `dbusmock` must use `/usr/bin/python3` (3.12). It is the
    same in a fresh container; no change of the switch.

## Answers to the questions of the brief

- **Does a fresh PM know what is in flight?** No (finding 3).
- **Rules that break:** the clone and tools "of the PM session", the
  carry rule and the cost cross-check, the scratchpad copy of the status
  page, leftovers in worktrees, the run times, the trigger id of section
  14 (findings 8 and 11).
- **Can a session without connectors do everything?** Yes for git and the
  status page; issues and PR reading only through `gh api` REST, which
  the charter does not say (finding 6); `add_repo` unattended is not
  proven (finding 4).
- **Can the user reach the PM?** Yes, in the newest run session; old
  sessions need the rule of finding 9.
