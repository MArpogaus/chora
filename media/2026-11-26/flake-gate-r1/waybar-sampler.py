import os, sys, time, collections
rt = sys.argv[1]; dur = float(sys.argv[2])
end = time.time() + dur
seen = collections.Counter(); maxn = 0; forks = 0; samples = 0
def env_has(pid):
    try:
        return f"XDG_RUNTIME_DIR={rt}".encode() in open(f"/proc/{pid}/environ","rb").read().split(b"\0")
    except OSError: return False
while time.time() < end:
    procs = {}
    for p in os.listdir("/proc"):
        if not p.isdigit(): continue
        try:
            st = open(f"/proc/{p}/stat").read()
        except OSError: continue
        comm = st[st.index("(")+1:st.rindex(")")]
        if comm != "waybar": continue
        ppid = int(st[st.rindex(")")+2:].split()[1])
        procs[int(p)] = ppid
    procs = {p: pp for p, pp in procs.items() if env_has(p)}
    samples += 1
    n = len(procs)
    if n > maxn: maxn = n
    for p, pp in procs.items():
        if pp in procs: forks += 1
    time.sleep(0.002)
print("samples", samples, "max waybar count", maxn, "child-of-waybar sightings", forks)
