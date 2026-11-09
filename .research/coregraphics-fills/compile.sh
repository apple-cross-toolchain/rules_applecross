#!/bin/zsh
# compile.sh NAME PDF: a catalog with PDF as a universal vector image set, compiled by Xcode for iOS; bitmaps to NAME/raw
n=$1; rm -rf $n; mkdir -p $n/A.xcassets/v.imageset $n/x
cp $2 $n/A.xcassets/v.imageset/v.pdf
echo '{"images":[{"idiom":"universal","filename":"v.pdf"}],"info":{"version":1,"author":"xcode"}}' > $n/A.xcassets/v.imageset/Contents.json
echo '{"info":{"version":1,"author":"xcode"}}' > $n/A.xcassets/Contents.json
xcrun actool --compile $n/x --platform iphonesimulator --target-device iphone --minimum-deployment-target 17.0 --output-partial-info-plist $n/x/p.plist $n/A.xcassets > $n/log 2>&1
python3 -I /tmp/cg/car.py $n/x/Assets.car $n/raw
