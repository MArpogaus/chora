# ci r3 (rc 4): GitHub CI on the candidate (PR #106)

Lane: opus, cloud, new branch claude/ci-r3 from the session branch
claude/autonomous-dev-session-3w-i33pg7 (4e3c019e). One round. Stop at 250 k context and hand over.
Read dev-rules.md and conventions.md (with the skills it names) first. Setup as briefs/ci-r1.md.
Failure: the coverage job of PR #106 (run 43643290788, job 132595355147; Fedora 44 container,
cargo nextest --profile ci) fails 9 text-layout tests that pass in our Debian container:
chora-assistant card::tests::a_card_in_a_page_has_no_widow_and_holds_the_typing_line,
chat::face::tests::a_space_has_a_width, chat::view::tests::a_balloon_is_as_high_as_its_content_up_to_its_most,
island::tests::a_long_sentence_shows_its_last_two_lines,
island::tests::the_island_is_a_capsule_40_high_that_hugs_its_content;
chora-chat draw::tests::a_field_in_the_column_is_lines_of_the_grid_without_a_box_and_sharp_at_two,
draw::tests::the_lines_of_a_field_in_the_column_start_at_one_inset;
chora-mc audit::tests::a_bar_stands_in_the_middle_between_the_ink_of_its_neighbours,
audit::tests::the_label_columns_of_the_margin_column_are_one_width.
Read the full job log with the GitHub tools (get_job_logs, job 132595355147; also the check job of
the same run if it fails) for each assertion. Find the root cause (likely which fonts the tests
resolve: system fonts of the CI image versus the fonts the project ships in dist/fonts, or a
fontconfig fallback). Fix the cause so the tests use the same font on every host (the project's
own fonts, a test font loaded by path, or the CI image installs the fonts the docs require), not by
loosening the asserts. Never skip, disable or quarantine a test.
Done when: the 9 tests pass locally and with the CI fonts reproduced (e.g. with the Fedora font
set or by hiding the Debian fonts through FONTCONFIG_FILE); tests/merge-gate.sh passes. Report to
the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 12 lines): root cause with evidence, commits.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; stage by name; no force push; push only claude/ci-r3; never check out
another branch in an existing worktree.
