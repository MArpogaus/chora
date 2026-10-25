#!/bin/bash
# Gate check 9 of 0.7: each error code of the API has a test that
# provokes it. The codes are the rows of the table "Error codes" in
# docs/api.md. The script runs the test of every row and refuses a row
# that has no test, so table and tests stay together.
#
# Usage: tests/gate-0.7-9.sh
set -u
. "$(dirname "$0")/gate-lib.sh"
command -v micromamba >/dev/null 2>&1 || skip "micromamba is not installed here"

# The test that provokes each code, by the number of the table row.
test_for() {
    case $1 in
    -32700) echo a_line_that_is_no_request_gets_a_parse_error ;;
    -32601 | -32602) echo an_unknown_method_and_wrong_params_fail_with_their_codes ;;
    1) echo a_rejected_transaction_names_the_command_in_data ;;
    2) echo a_gone_surface_has_its_own_code ;;
    3) echo hello_with_a_higher_protocol_fails_with_the_protocol_code ;;
    4) echo a_method_without_its_grant_answers_missing_capability ;;
    5) echo a_command_for_a_missing_node_answers_unknown_node ;;
    6) echo an_answer_beyond_the_output_limit_is_an_error ;;
    7) echo a_second_hello_of_a_live_name_answers_name_taken ;;
    8) echo an_admin_method_off_the_admin_socket_answers_its_code ;;
    esac
}

cd "$repo" || fail "no repo"
codes=$(sed -n '/^## Error codes/,/^## /p' docs/api.md |
    sed -nE 's/^\| *(-?[0-9]+) *\|.*/\1/p')
test -n "$codes" || fail "no table Error codes in docs/api.md"
for code in $codes; do
    name=$(test_for "$code")
    test -n "$name" || fail "code $code of docs/api.md has no test in this script"
    out=$(micromamba run -n chora cargo test -p chora --lib --locked -- "$name" 2>&1) ||
        fail "code $code: $(tail -n 5 <<<"$out")"
    grep -q "test result: ok. 1 passed" <<<"$out" ||
        fail "code $code: the test $name did not run green: $(tail -n 5 <<<"$out")"
done
ok "every one of the $(echo "$codes" | wc -l) error codes has a green test"
