# shellcheck shell=bash
# shellcheck disable=SC2154
# Diagnostics of tests/nested.sh, sourced by it and by its selftest.
# Needs the functions pixel and top_color, and the variable out.

not_black_diag() {
    echo "not_black: pixel 100,100 = $(pixel 100 100 2>&1 || true)"
    echo "not_black: top color = $(top_color 2>&1 || true)"
    echo "not_black: tail of $out/probe.log:"
    tail -n 20 "$out/probe.log" 2>&1 | sed 's/^/  /' || true
    echo "not_black: lock lines of $out/chora.log:"
    { grep -i 'lock' "$out/chora.log" 2>&1 | tail -n 20 | sed 's/^/  /'; } || true
}
