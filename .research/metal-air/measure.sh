#!/bin/bash
# Compile every .metal under a corpus directory; summarise the first errors.
cd "$(dirname "$0")"; out=/tmp/measure; rm -rf $out; mkdir -p $out; res=${RES:-/tmp/meas.txt}; : > $res
t=${TARGET:-air64-apple-ios27.2}
find "$1" -name '*.metal' | sort > $out/list
for f in $(cat $out/list); do
  n=$(echo "$f" | tr '/' '_')
  if python3 msl2air.py -target $t -I "$(dirname "$f")" $EXTRA_INCS -o $out/$n.metallib "$f" < /dev/null > $out/$n.log 2>&1; then echo "OK   $f" >> $res; else echo "FAIL $f :: $(grep -m1 -E 'error|Error' $out/$n.log | sed 's/.*error: //' | cut -c1-110)" >> $res; fi
done
