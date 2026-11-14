# pk1 r1 (rc 5, rc 1 ti-1): package dependencies

Lane: sonnet, cloud, new branch claude/pk1 from the session branch
claude/autonomous-dev-session-3w-i33pg7 (1366c716). One round. Stop at 250 k context and hand over.
Read dev-rules.md and conventions.md (with the skills it names) first. Setup as briefs/ci-r1.md.
Items (backlog PK1, PM decisions 11-14 22:16 and 23:1x):
1. RPM (dist/build-rpm.sh and its spec or cargo-generate-rpm metadata) and Arch (dist/arch PKGBUILD):
   require fuzzel >= 1.11 (mouse selection, ti-1) and fontconfig (chora-mc face lookup); recommend
   (RPM Recommends, Arch optdepends) dejavu-sans, dejavu-serif and dejavu-sans-mono fonts (the
   fallback faces of chora-assistant) next to the Inter recommendation that exists.
2. docs/install.md names these dependencies and why (one line each).
3. A check that fails without them: a script or test that reads the generated package metadata
   (rpm -qp --requires on the built RPM if the container can build it, else the metadata source)
   and asserts the four entries.
Done when: tests/merge-gate.sh passes; the package workflow (.github/workflows/packages.yaml)
still builds (the PR shows it). Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 8 lines).
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; stage by name; no force push; push only claude/pk1; never check out
another branch in an existing worktree; never end a turn to wait for a long background job.
