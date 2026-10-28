//
// HalfPlusF32.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "HalfPlusF32.h"

@implementation HalfPlusF32Input

- (instancetype)initWithX:(MLMultiArray *)x y:(MLMultiArray *)y {
    self = [super init];
    if (self) {
        _x = x;
        _y = y;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"x", @"y"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"x"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.x];
    }
    if ([featureName isEqualToString:@"y"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.y];
    }
    return nil;
}

@end

@implementation HalfPlusF32Output

- (instancetype)initWithRelu_0:(MLMultiArray *)relu_0 relu_1:(MLMultiArray *)relu_1 {
    self = [super init];
    if (self) {
        _relu_0 = relu_0;
        _relu_1 = relu_1;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"relu_0", @"relu_1"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"relu_0"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.relu_0];
    }
    if ([featureName isEqualToString:@"relu_1"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.relu_1];
    }
    return nil;
}

@end

@implementation HalfPlusF32


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"HalfPlusF32" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load HalfPlusF32.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize HalfPlusF32 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of HalfPlusF32.
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
    Initialize HalfPlusF32 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize HalfPlusF32 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize HalfPlusF32 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for HalfPlusF32.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize HalfPlusF32 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for HalfPlusF32.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct HalfPlusF32 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid HalfPlusF32 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(HalfPlusF32 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct HalfPlusF32 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid HalfPlusF32 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(HalfPlusF32 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            HalfPlusF32 *typedModel = [[HalfPlusF32 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable HalfPlusF32Output *)predictionFromFeatures:(HalfPlusF32Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable HalfPlusF32Output *)predictionFromFeatures:(HalfPlusF32Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[HalfPlusF32Output alloc] initWithRelu_0:(MLMultiArray *)[outFeatures featureValueForName:@"relu_0"].multiArrayValue relu_1:(MLMultiArray *)[outFeatures featureValueForName:@"relu_1"].multiArrayValue];
}

- (void)predictionFromFeatures:(HalfPlusF32Input *)input completionHandler:(void (^)(HalfPlusF32Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            HalfPlusF32Output *output = [[HalfPlusF32Output alloc] initWithRelu_0:(MLMultiArray *)[prediction featureValueForName:@"relu_0"].multiArrayValue relu_1:(MLMultiArray *)[prediction featureValueForName:@"relu_1"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(HalfPlusF32Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(HalfPlusF32Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            HalfPlusF32Output *output = [[HalfPlusF32Output alloc] initWithRelu_0:(MLMultiArray *)[prediction featureValueForName:@"relu_0"].multiArrayValue relu_1:(MLMultiArray *)[prediction featureValueForName:@"relu_1"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable HalfPlusF32Output *)predictionFromX:(MLMultiArray *)x y:(MLMultiArray *)y error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    HalfPlusF32Input *input_ = [[HalfPlusF32Input alloc] initWithX:x y:y];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<HalfPlusF32Output *> *)predictionsFromInputs:(NSArray<HalfPlusF32Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<HalfPlusF32Output*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        HalfPlusF32Output * result = [[HalfPlusF32Output alloc] initWithRelu_0:(MLMultiArray *)[resultProvider featureValueForName:@"relu_0"].multiArrayValue relu_1:(MLMultiArray *)[resultProvider featureValueForName:@"relu_1"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
