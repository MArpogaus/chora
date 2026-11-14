# ci r4 (rc 4): the two mc audit tests still red in CI

Lane: same worker as ci r3, new branch claude/ci-r4 from the session branch (1366c716, has ci r3).
One round; setup and rules as briefs/ci-r3.md.
After the font install, the coverage job of PR #106 (run 43649567833, job 132614788674) still fails
2 of the 9: chora-mc audit::tests::a_bar_stands_in_the_middle_between_the_ink_of_its_neighbours and
audit::tests::the_label_columns_of_the_margin_column_are_one_width (7 now pass). Your local
reproduction with only the Fedora 44 font files passed all 9, so something differs from CI: read
the full job log with the GitHub tools for the two assertions, and compare what mc resolves (its
serif/sans/mono families in mc.yaml and chora-mc's audit, fontconfig matching, the fc cache, a
font the project ships under dist/fonts or a path relative to the crate). Fix the cause, not the
asserts; never skip a test. Done when: the two pass with the CI font set reproduced exactly as CI
installs it (dnf packages, fc-cache state); tests/merge-gate.sh passes. Report to the PM
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 10 lines): cause with evidence, commits.
