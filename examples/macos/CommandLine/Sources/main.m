#import <Foundation/Foundation.h>

int main(int argc, char **argv) {
  NSBundle *bundle = [NSBundle mainBundle];
  NSLog(@"Hello World from %@", bundle.bundleIdentifier);
  return 0;
}
