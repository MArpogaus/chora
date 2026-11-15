# Review pk1 r1 (claude/pk1 c4f814a7..93521c25), PM, read-only, 11-15 00:1x UTC

Verdict: MERGE-OK. RPM requires fuzzel >= 1.11 and fontconfig, recommends the three DejaVu packages; Arch depends fontconfig and fuzzel>=1.11, optdepends ttf-dejavu. fuzzel moves from optional to required: correct, since the shared launcher door of every look (chora_kit::shared::launcher) runs fuzzel, not only tiling. tests/gate-package-deps.sh (red 8, green 8) is a merge-gate step and runs on the built RPM in packages.yaml. RPM mode untested locally (no rpm); the PR run proves it.
