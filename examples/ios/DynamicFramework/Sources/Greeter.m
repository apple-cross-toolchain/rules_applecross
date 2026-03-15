#import "examples/ios/DynamicFramework/Sources/Greeter.h"

@implementation Greeter

+ (NSString *)greet:(NSString *)name {
    return [NSString stringWithFormat:@"Hello, %@!", name];
}

@end
