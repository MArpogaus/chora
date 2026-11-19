
## r1

Head: 31cfe52d on claude/gate-pick (selftest commit 98cbc875 red, tool 31cfe52d green).
Gate: `--only pre-commit,ci-check,red-first,identity,gate-lib,build` green.
Selftest cases (temp repo, stub merge-gate): notify picks xp; gate-hud.sh picks hud; hud-lib.sh picks hud; services/voice picks chat; chora-xp picks only the fixed five; docs-only picks the fixed five; crates/chora picks all.
`tests/merge-gate.sh --commands` is new; the selftest runs from the gate-lib step as a last `&&` term.
`tests/cells-for.sh --base 219d5afe --head 88f441e4` (kit-minors r1; it changes chora-kit, so every cell, xp among them; chora-notify alone also picks xp):
pre-commit,ci-check,build,test,clippy,red-first,identity,gate-lib,2d-guard,package-deps,package-lists,xp,matrix,virtual,xp-faces,xp-paperclip,xp-fields,xp-question,xp-notepad,hud,layout,tiling-audit,tiling-look,chat-keys,doors,acts,last-words,cube,pull,placed,construct-hud,out,winit,wm-pane,pane-keys,walk,keys,pixels,mc-paths,mc-sofa,mc-handover,e2e
Own diff: pre-commit,ci-check,build,test,clippy,red-first,identity,gate-lib,matrix,mc-handover

## PM review r1 (2026-11-19 21:40 UTC; PM, not the builder)

Diff a9fd91f3..31cfe52d: tests/cells-for.sh (142), tests/cells-for-selftest.sh (75), merge-gate.sh `--commands` (+15), CONTRIBUTING (+2). Red first: 98cbc875 selftest before the tool.
Checks: done criteria met (tool, `--commands`, selftest in gate-lib, docs). Real run by the PM: `--base 219d5afe --head 8e677c1c` and `--head 582ab506` both pick every cell (both touch chora-kit), hud and xp included.
Findings:
- gp-1 MINOR: without `--head` it diffs the work tree; untracked new files are not seen (a new test script before `git add` picks nothing).
- gp-2 MINOR: a change to tests/merge-gate.sh picks only red-first and identity, not gate-lib (whose selftests prove the gate).
- gp-3 MINOR: a plugin a cell starts only through the session config (not named in the cell's text) is not picked; the tool is advisory and the train runs the full gate; CONTRIBUTING should say so.
Verdict (computed): no open BLOCK or MAJOR -> MERGE-OK. Train 74 with id-gate (harness train). MINORs to backlog HARNESS gp-1..3.
