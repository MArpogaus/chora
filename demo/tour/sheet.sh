#!/bin/bash
# sheet.sh OUT.png STILL... : 2x2 or 3x3 contact sheet of stills, with the names
out=$1; shift
args=(); filt=""; i=0
for f in "$@"; do args+=(-i "$f"); filt+="[$i:v]scale=640:360,drawtext=text='$(basename "$f" .png)':x=6:y=6:fontsize=14:fontcolor=yellow:box=1:boxcolor=black@0.6[v$i];"; i=$((i+1)); done
n=$i; cols=2; layout=""; for ((k=0;k<n;k++)); do layout+="$(( (k%cols)*640 ))_$(( (k/cols)*360 ))|"; done
ins=""; for ((k=0;k<n;k++)); do ins+="[v$k]"; done
ffmpeg -loglevel error -y "${args[@]}" -filter_complex "${filt}${ins}xstack=inputs=$n:layout=${layout%|}" "$out"
