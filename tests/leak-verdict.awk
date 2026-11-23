# Verdict of tests/gate-leak.sh over its per-round table (tests/leak-verdict-selftest.sh).
# Usage: awk -F'\t' -v warm=W -v bound=KB -v bw=BASE_WINDOWS -v bc=BASE_CLIENTS -f leak-verdict.awk TABLE
# Columns: round look rss_kb fds windows clients ...; round 0 is the base.
NR == 1 || $1 == 0 { next }
$5 > bw { print "FAIL: round " $1 ": " $5 " windows, base " bw }
$6 > bc { print "FAIL: round " $1 ": " $6 " clients, base " bc }
$1 <= warm { next }
{
    x[++n] = $1; y[n] = $3
    if (!r0) { f0 = $4; r0 = $1 } f1 = $4; r1 = $1
}
END {
    for (i = 1; i < n; i++) for (j = i + 1; j <= n; j++) s[++m] = (y[j] - y[i]) / (x[j] - x[i])
    for (i = 2; i <= m; i++) { v = s[i]; for (j = i - 1; j > 0 && s[j] > v; j--) s[j + 1] = s[j]; s[j + 1] = v }
    slope = (m % 2) ? s[(m + 1) / 2] : (s[m / 2] + s[m / 2 + 1]) / 2
    fd_bound = int((r1 - r0) / 10) + (((r1 - r0) % 10) > 0 ? 1 : 0)
    printf "rss slope %.0f kB/round (bound %d), fds %d -> %d over rounds %d..%d (bound +%d)\n", slope, bound, f0, f1, r0, r1, fd_bound
    if (slope > bound) printf "FAIL: the RSS of the core grows %.0f kB per round (bound %d)\n", slope, bound
    if (f1 - f0 > fd_bound) printf "FAIL: the fds of the core grow from %d to %d over %d rounds (bound +%d)\n", f0, f1, r1 - r0, fd_bound
}
