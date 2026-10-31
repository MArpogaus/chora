#!/bin/bash
# Prints the line coverage per crate as a Markdown table, from the JSON
# summary of cargo-llvm-cov. CI appends it to the job summary.
#
# Usage: cargo llvm-cov report --json --summary-only --output-path cov.json
#        tests/coverage-summary.sh cov.json
set -eu
jq -r --arg root "$PWD/" '
    def pct: if .n > 0 then (.c * 1000 / .n | round) / 10 else 0 end;
    def row: "| \(.crate) | \(.n) | \(.c) | \(pct) % |";
    [.data[0].files[]
        | select(.filename | startswith($root))
        | {crate: (.filename | ltrimstr($root) | split("/")[:2] | join("/")),
           n: .summary.lines.count, c: .summary.lines.covered}]
    | (group_by(.crate)
        | map({crate: .[0].crate, n: (map(.n) | add), c: (map(.c) | add)})) as $crates
    | "| Crate | Lines | Covered | Coverage |",
      "|---|--:|--:|--:|",
      ($crates[] | row),
      ({crate: "**total**", n: ($crates | map(.n) | add),
        c: ($crates | map(.c) | add)} | row)
' "$1"
