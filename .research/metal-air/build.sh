#!/bin/bash
# Compile every probe and corpus shader as rules_apple does, for several targets.
set -u
cd ~/m0
TARGETS="air64-apple-ios12.0 air64-apple-ios16.0 air64-apple-ios17.0 air64-apple-ios18.0 air64-apple-ios26.0 air64-apple-ios27.2 air64-apple-ios17.0-simulator air64-apple-macosx11.0 air64-apple-macosx14.0 air64-apple-macosx27.0 air64-apple-tvos17.0 air64-apple-xros2.0"
sdk_for() { case $1 in *ios*-simulator) echo iphonesimulator;; *ios*) echo iphoneos;; *macosx*) echo macosx;; *tvos*) echo appletvos;; *xros*) echo xros;; esac; }
rm -rf out
for t in $TARGETS; do
  mkdir -p out/$t; S=$(sdk_for $t)
  # corpus/ holds public shaders for measurement only: local, never committed.
  for f in probes/*.metal corpus/*.metal; do
    b=$(basename $f .metal)
    xcrun -sdk $S metal -c -target $t -ffast-math -o out/$t/$b.air $f 2> out/$t/$b.err || { echo "FAIL $t $b"; continue; }
    xcrun -sdk $S metal -S -emit-llvm -target $t -ffast-math -o out/$t/$b.ll $f 2>/dev/null
    xcrun -sdk $S metallib -o out/$t/$b.metallib out/$t/$b.air 2>>out/$t/$b.err
  done
  xcrun -sdk $S metallib -o out/$t/ALL_probes.metallib out/$t/p*.air 2>/dev/null
done
echo done
