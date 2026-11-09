#!/bin/bash
# Compile every probe for each reference target and compare with Xcode's
# output in ref/<target>/ (made by build.sh on a Mac; not committed).
cd "$(dirname "$0")"; mkdir -p ours
for t in ${TARGETS:-air64-apple-ios27.2}; do
  for f in probes/*.metal ${CORPUS:+corpus/*.metal}; do
    b=$(basename $f .metal); [ -f ref/$t/$b.ll ] || continue
    mkdir -p ours/$t
    if ! python3 msl2air.py -target $t -I MySharedTypes/include -o ours/$t/$b.metallib $f > ours/$t/$b.log 2>&1; then echo "FAIL  $t $b: $(grep -m1 -E 'error|Error' ours/$t/$b.log | cut -c1-150)"; continue; fi
    md=OK; python3 compare_md.py ours/$t/$b.metallib.ll ref/$t/$b.ll > ours/$t/$b.cmp 2>&1 || md=DIFF
    lib=OK; python3 compare_lib.py ours/$t/$b.metallib ref/$t/$b.metallib > ours/$t/$b.libcmp 2>&1 || lib=DIFF
    echo "md=$md lib=$lib  $t $b"
  done
done
