#import <Foundation/Foundation.h>
#import <ColorSync/ColorSync.h>
static void dump(id v, int depth) {
  NSString *pad = [@"" stringByPaddingToLength:depth*2 withString:@" " startingAtIndex:0];
  if ([v isKindOfClass:[NSDictionary class]]) {
    for (id k in [[v allKeys] sortedArrayUsingSelector:@selector(compare:)]) {
      if ([k isEqual:@"com.apple.cmm.ProfileSequnce"] || [k hasSuffix:@"LUT"]) continue;
      printf("%s%s:\n", pad.UTF8String, [[k description] UTF8String]); dump(v[k], depth+1);
    }
  } else if ([v isKindOfClass:[NSArray class]]) {
    for (id x in v) { printf("%s-\n", pad.UTF8String); dump(x, depth+1); }
  } else if ([v isKindOfClass:[NSNumber class]]) {
    printf("%s%s %.17g (%s)\n", pad.UTF8String, [[v description] UTF8String], [v doubleValue], [v objCType]);
  } else printf("%s%s\n", pad.UTF8String, [[v description] UTF8String]);
}
int main(int argc, char **argv) {
  @autoreleasepool {
    NSDictionary *names = @{@"GenericRGB": (__bridge id)kColorSyncGenericRGBProfile, @"DisplayP3": (__bridge id)kColorSyncDisplayP3Profile,
      @"sRGB": (__bridge id)kColorSyncSRGBProfile, @"GenericGray": (__bridge id)kColorSyncGenericGrayProfile, @"GenericGray22": (__bridge id)kColorSyncGenericGrayGamma22Profile, @"ExtSRGB": @"kCGColorSpaceExtendedSRGB"};
    ColorSyncProfileRef s = ColorSyncProfileCreateWithName((__bridge CFStringRef)names[[NSString stringWithUTF8String:argv[1]]]);
    ColorSyncProfileRef d = ColorSyncProfileCreateWithName((__bridge CFStringRef)names[[NSString stringWithUTF8String:argv[2]]]);
    id intent = argc > 3 ? @(argv[3]) : (__bridge id)kColorSyncRenderingIntentPerceptual;
    NSArray *profiles = @[
      @{(__bridge id)kColorSyncProfile: (__bridge id)s, (__bridge id)kColorSyncRenderingIntent: intent, (__bridge id)kColorSyncTransformTag: (__bridge id)kColorSyncTransformDeviceToPCS},
      @{(__bridge id)kColorSyncProfile: (__bridge id)d, (__bridge id)kColorSyncRenderingIntent: intent, (__bridge id)kColorSyncTransformTag: (__bridge id)kColorSyncTransformPCSToDevice},
    ];
    ColorSyncTransformRef t = ColorSyncTransformCreate((__bridge CFArrayRef)profiles, NULL);
    dump(CFBridgingRelease(ColorSyncTransformCopyProperty(t, kColorSyncTransformParametricConversionData, NULL)), 0);
  }
}
