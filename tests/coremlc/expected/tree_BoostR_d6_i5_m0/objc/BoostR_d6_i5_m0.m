//
// BoostR_d6_i5_m0.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "BoostR_d6_i5_m0.h"

@implementation BoostR_d6_i5_m0Input

- (instancetype)initWithA:(double)a c:(int64_t)c color:(NSString *)color {
    self = [super init];
    if (self) {
        _a = a;
        _c = c;
        _color = color;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"a", @"c", @"color"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"a"]) {
        return [MLFeatureValue featureValueWithDouble:self.a];
    }
    if ([featureName isEqualToString:@"c"]) {
        return [MLFeatureValue featureValueWithInt64:self.c];
    }
    if ([featureName isEqualToString:@"color"]) {
        return [MLFeatureValue featureValueWithString:self.color];
    }
    return nil;
}

@end

@implementation BoostR_d6_i5_m0Output

- (instancetype)initWithR:(double)r {
    self = [super init];
    if (self) {
        _r = r;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"r"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"r"]) {
        return [MLFeatureValue featureValueWithDouble:self.r];
    }
    return nil;
}

@end

@implementation BoostR_d6_i5_m0


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"BoostR_d6_i5_m0" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load BoostR_d6_i5_m0.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize BoostR_d6_i5_m0 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of BoostR_d6_i5_m0.
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
    Initialize BoostR_d6_i5_m0 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize BoostR_d6_i5_m0 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize BoostR_d6_i5_m0 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for BoostR_d6_i5_m0.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize BoostR_d6_i5_m0 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for BoostR_d6_i5_m0.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct BoostR_d6_i5_m0 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid BoostR_d6_i5_m0 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(BoostR_d6_i5_m0 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct BoostR_d6_i5_m0 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid BoostR_d6_i5_m0 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(BoostR_d6_i5_m0 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            BoostR_d6_i5_m0 *typedModel = [[BoostR_d6_i5_m0 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable BoostR_d6_i5_m0Output *)predictionFromFeatures:(BoostR_d6_i5_m0Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable BoostR_d6_i5_m0Output *)predictionFromFeatures:(BoostR_d6_i5_m0Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[BoostR_d6_i5_m0Output alloc] initWithR:[outFeatures featureValueForName:@"r"].doubleValue];
}

- (void)predictionFromFeatures:(BoostR_d6_i5_m0Input *)input completionHandler:(void (^)(BoostR_d6_i5_m0Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            BoostR_d6_i5_m0Output *output = [[BoostR_d6_i5_m0Output alloc] initWithR:[prediction featureValueForName:@"r"].doubleValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(BoostR_d6_i5_m0Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(BoostR_d6_i5_m0Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            BoostR_d6_i5_m0Output *output = [[BoostR_d6_i5_m0Output alloc] initWithR:[prediction featureValueForName:@"r"].doubleValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable BoostR_d6_i5_m0Output *)predictionFromA:(double)a c:(int64_t)c color:(NSString *)color error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    BoostR_d6_i5_m0Input *input_ = [[BoostR_d6_i5_m0Input alloc] initWithA:a c:c color:color];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<BoostR_d6_i5_m0Output *> *)predictionsFromInputs:(NSArray<BoostR_d6_i5_m0Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<BoostR_d6_i5_m0Output*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        BoostR_d6_i5_m0Output * result = [[BoostR_d6_i5_m0Output alloc] initWithR:[resultProvider featureValueForName:@"r"].doubleValue];
        [results addObject:result];
    }
    return results;
}

@end
