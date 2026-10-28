//
// RND63.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "RND63.h"

@implementation RND63Input

- (instancetype)initWithX0:(double)x0 x1:(double)x1 x2:(double)x2 x3:(double)x3 x4:(double)x4 {
    self = [super init];
    if (self) {
        _x0 = x0;
        _x1 = x1;
        _x2 = x2;
        _x3 = x3;
        _x4 = x4;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"x0", @"x1", @"x2", @"x3", @"x4"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"x0"]) {
        return [MLFeatureValue featureValueWithDouble:self.x0];
    }
    if ([featureName isEqualToString:@"x1"]) {
        return [MLFeatureValue featureValueWithDouble:self.x1];
    }
    if ([featureName isEqualToString:@"x2"]) {
        return [MLFeatureValue featureValueWithDouble:self.x2];
    }
    if ([featureName isEqualToString:@"x3"]) {
        return [MLFeatureValue featureValueWithDouble:self.x3];
    }
    if ([featureName isEqualToString:@"x4"]) {
        return [MLFeatureValue featureValueWithDouble:self.x4];
    }
    return nil;
}

@end

@implementation RND63Output

- (instancetype)initWithLabel:(NSString *)label classProbability:(NSDictionary<NSString *, NSNumber *> *)classProbability {
    self = [super init];
    if (self) {
        _label = label;
        _classProbability = classProbability;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"label", @"classProbability"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"label"]) {
        return [MLFeatureValue featureValueWithString:self.label];
    }
    if ([featureName isEqualToString:@"classProbability"]) {
        return [MLFeatureValue featureValueWithDictionary:self.classProbability error:nil];
    }
    return nil;
}

@end

@implementation RND63


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"RND63" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load RND63.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize RND63 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of RND63.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model {
    if (model == nil) {
        return nil;
    }
    self = [super init];
    if (self != nil) {
        _model = model;
    }
    return self;
}


/**
    Initialize RND63 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize RND63 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize RND63 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for RND63.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize RND63 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for RND63.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct RND63 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid RND63 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(RND63 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct RND63 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid RND63 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(RND63 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            RND63 *typedModel = [[RND63 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable RND63Output *)predictionFromFeatures:(RND63Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable RND63Output *)predictionFromFeatures:(RND63Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[RND63Output alloc] initWithLabel:(NSString *)[outFeatures featureValueForName:@"label"].stringValue classProbability:(NSDictionary<NSString *, NSNumber *> *)[outFeatures featureValueForName:@"classProbability"].dictionaryValue];
}

- (void)predictionFromFeatures:(RND63Input *)input completionHandler:(void (^)(RND63Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            RND63Output *output = [[RND63Output alloc] initWithLabel:(NSString *)[prediction featureValueForName:@"label"].stringValue classProbability:(NSDictionary<NSString *, NSNumber *> *)[prediction featureValueForName:@"classProbability"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(RND63Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(RND63Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            RND63Output *output = [[RND63Output alloc] initWithLabel:(NSString *)[prediction featureValueForName:@"label"].stringValue classProbability:(NSDictionary<NSString *, NSNumber *> *)[prediction featureValueForName:@"classProbability"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable RND63Output *)predictionFromX0:(double)x0 x1:(double)x1 x2:(double)x2 x3:(double)x3 x4:(double)x4 error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    RND63Input *input_ = [[RND63Input alloc] initWithX0:x0 x1:x1 x2:x2 x3:x3 x4:x4];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<RND63Output *> *)predictionsFromInputs:(NSArray<RND63Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<RND63Output*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        RND63Output * result = [[RND63Output alloc] initWithLabel:(NSString *)[resultProvider featureValueForName:@"label"].stringValue classProbability:(NSDictionary<NSString *, NSNumber *> *)[resultProvider featureValueForName:@"classProbability"].dictionaryValue];
        [results addObject:result];
    }
    return results;
}

@end
