#import "examples/ios/ExplicitModules/Sources/Greeting.h"

@implementation Greeting
+ (NSString *)greetingFor:(NSString *)name {
  return [NSString stringWithFormat:@"Hello, %@", name];
}
@end
