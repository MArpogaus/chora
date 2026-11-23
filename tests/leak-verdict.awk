# Verdict of tests/gate-leak.sh over its per-round table (tests/leak-verdict-selftest.sh).
# Usage: awk -F'\t' -v warm=W -v bound=KB -v bw=BASE_WINDOWS -v bc=BASE_CLIENTS -f leak-verdict.awk TABLE
# RSS: the median of the round-to-round differences after warm-up (one bounded
# step of heap moves one difference; a leak moves all of them), bound set in
# gate-leak.sh. Columns: round look rss_kb fds windows clients ...; round 0 is the base.
NR == 1 || $1 == 0 { next }
$5 > bw { print "FAIL: round " $1 ": " $5 " windows, base " bw }
$6 > bc { print "FAIL: round " $1 ": " $6 " clients, base " bc }
$1 <= warm { next }
{
    x[++n] = $1; y[n] = $3
    if (!r0) { f0 = $4; r0 = $1 } f1 = $4; r1 = $1
}
END {
    for (i = 2; i <= n; i++) d[i - 1] = y[i] - y[i - 1]
    for (i = 2; i < n; i++) { v = d[i]; for (j = i - 1; j > 0 && d[j] > v; j--) d[j + 1] = d[j]; d[j + 1] = v }
    slope = ((n - 1) % 2) ? d[n / 2] : (d[(n - 1) / 2] + d[(n - 1) / 2 + 1]) / 2
    fd_bound = int((r1 - r0) / 10) + (((r1 - r0) % 10) > 0 ? 1 : 0)
    printf "rss slope %.0f kB/round (bound %d), fds %d -> %d over rounds %d..%d (bound +%d)\n", slope, bound, f0, f1, r0, r1, fd_bound
    if (slope > bound) printf "FAIL: the RSS of the core grows %.0f kB per round (bound %d)\n", slope, bound
    if (f1 - f0 > fd_bound) printf "FAIL: the fds of the core grow from %d to %d over %d rounds (bound +%d)\n", f0, f1, r1 - r0, fd_bound
}
