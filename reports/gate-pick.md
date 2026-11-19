
## r1

Head: 31cfe52d on claude/gate-pick (selftest commit 98cbc875 red, tool 31cfe52d green).
Gate: `--only pre-commit,ci-check,red-first,identity,gate-lib,build` green.
Selftest cases (temp repo, stub merge-gate): notify picks xp; gate-hud.sh picks hud; hud-lib.sh picks hud; services/voice picks chat; chora-xp picks only the fixed five; docs-only picks the fixed five; crates/chora picks all.
`tests/merge-gate.sh --commands` is new; the selftest runs from the gate-lib step as a last `&&` term.
`tests/cells-for.sh --base 219d5afe --head 88f441e4` (kit-minors r1; it changes chora-kit, so every cell, xp among them; chora-notify alone also picks xp):
pre-commit,ci-check,build,test,clippy,red-first,identity,gate-lib,2d-guard,package-deps,package-lists,xp,matrix,virtual,xp-faces,xp-paperclip,xp-fields,xp-question,xp-notepad,hud,layout,tiling-audit,tiling-look,chat-keys,doors,acts,last-words,cube,pull,placed,construct-hud,out,winit,wm-pane,pane-keys,walk,keys,pixels,mc-paths,mc-sofa,mc-handover,e2e
Own diff: pre-commit,ci-check,build,test,clippy,red-first,identity,gate-lib,matrix,mc-handover
