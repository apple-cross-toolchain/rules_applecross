#!/bin/zsh
# bitmap.sh NAME PNG1X PNG2X PNG3X: those PNGs as an image set, compiled by Xcode for iOS
n=$1; rm -rf $n; mkdir -p $n/A.xcassets/v.imageset $n/x
cp $2 $n/A.xcassets/v.imageset/v.png; cp $3 $n/A.xcassets/v.imageset/v@2x.png; cp $4 $n/A.xcassets/v.imageset/v@3x.png
echo '{"images":[{"idiom":"universal","filename":"v.png","scale":"1x"},{"idiom":"universal","filename":"v@2x.png","scale":"2x"},{"idiom":"universal","filename":"v@3x.png","scale":"3x"}],"info":{"version":1,"author":"xcode"}}' > $n/A.xcassets/v.imageset/Contents.json
echo '{"info":{"version":1,"author":"xcode"}}' > $n/A.xcassets/Contents.json
xcrun actool --compile $n/x --platform iphonesimulator --target-device iphone --minimum-deployment-target 17.0 --output-partial-info-plist $n/x/p.plist $n/A.xcassets > $n/log 2>&1
