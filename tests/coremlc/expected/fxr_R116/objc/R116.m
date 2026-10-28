//
// R116.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "R116.h"

@implementation R116Input

- (instancetype)initWithIn0:(MLMultiArray *)in0 in1:(MLMultiArray *)in1 in2:(MLMultiArray *)in2 {
    self = [super init];
    if (self) {
        _in0 = in0;
        _in1 = in1;
        _in2 = in2;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"in0", @"in1", @"in2"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"in0"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.in0];
    }
    if ([featureName isEqualToString:@"in1"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.in1];
    }
    if ([featureName isEqualToString:@"in2"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.in2];
    }
    return nil;
}

@end

@implementation R116Output

- (instancetype)initWithRelu_0:(MLMultiArray *)relu_0 relu_1:(MLMultiArray *)relu_1 relu_2:(MLMultiArray *)relu_2 {
    self = [super init];
    if (self) {
        _relu_0 = relu_0;
        _relu_1 = relu_1;
        _relu_2 = relu_2;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"relu_0", @"relu_1", @"relu_2"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"relu_0"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.relu_0];
    }
    if ([featureName isEqualToString:@"relu_1"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.relu_1];
    }
    if ([featureName isEqualToString:@"relu_2"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.relu_2];
    }
    return nil;
}

@end

@implementation R116


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"R116" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load R116.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize R116 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of R116.
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
    Initialize R116 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize R116 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize R116 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for R116.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize R116 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for R116.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct R116 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid R116 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(R116 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct R116 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid R116 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(R116 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            R116 *typedModel = [[R116 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable R116Output *)predictionFromFeatures:(R116Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable R116Output *)predictionFromFeatures:(R116Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[R116Output alloc] initWithRelu_0:(MLMultiArray *)[outFeatures featureValueForName:@"relu_0"].multiArrayValue relu_1:(MLMultiArray *)[outFeatures featureValueForName:@"relu_1"].multiArrayValue relu_2:(MLMultiArray *)[outFeatures featureValueForName:@"relu_2"].multiArrayValue];
}

- (void)predictionFromFeatures:(R116Input *)input completionHandler:(void (^)(R116Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            R116Output *output = [[R116Output alloc] initWithRelu_0:(MLMultiArray *)[prediction featureValueForName:@"relu_0"].multiArrayValue relu_1:(MLMultiArray *)[prediction featureValueForName:@"relu_1"].multiArrayValue relu_2:(MLMultiArray *)[prediction featureValueForName:@"relu_2"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(R116Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(R116Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            R116Output *output = [[R116Output alloc] initWithRelu_0:(MLMultiArray *)[prediction featureValueForName:@"relu_0"].multiArrayValue relu_1:(MLMultiArray *)[prediction featureValueForName:@"relu_1"].multiArrayValue relu_2:(MLMultiArray *)[prediction featureValueForName:@"relu_2"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable R116Output *)predictionFromIn0:(MLMultiArray *)in0 in1:(MLMultiArray *)in1 in2:(MLMultiArray *)in2 error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    R116Input *input_ = [[R116Input alloc] initWithIn0:in0 in1:in1 in2:in2];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<R116Output *> *)predictionsFromInputs:(NSArray<R116Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<R116Output*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        R116Output * result = [[R116Output alloc] initWithRelu_0:(MLMultiArray *)[resultProvider featureValueForName:@"relu_0"].multiArrayValue relu_1:(MLMultiArray *)[resultProvider featureValueForName:@"relu_1"].multiArrayValue relu_2:(MLMultiArray *)[resultProvider featureValueForName:@"relu_2"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
