//
// BoostC_d3_i5_m0.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "BoostC_d3_i5_m0.h"

@implementation BoostC_d3_i5_m0Input

- (instancetype)initWithA:(double)a b:(double)b {
    self = [super init];
    if (self) {
        _a = a;
        _b = b;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"a", @"b"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"a"]) {
        return [MLFeatureValue featureValueWithDouble:self.a];
    }
    if ([featureName isEqualToString:@"b"]) {
        return [MLFeatureValue featureValueWithDouble:self.b];
    }
    return nil;
}

@end

@implementation BoostC_d3_i5_m0Output

- (instancetype)initWithY:(NSString *)y yProbability:(NSDictionary<NSString *, NSNumber *> *)yProbability {
    self = [super init];
    if (self) {
        _y = y;
        _yProbability = yProbability;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"y", @"yProbability"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"y"]) {
        return [MLFeatureValue featureValueWithString:self.y];
    }
    if ([featureName isEqualToString:@"yProbability"]) {
        return [MLFeatureValue featureValueWithDictionary:self.yProbability error:nil];
    }
    return nil;
}

@end

@implementation BoostC_d3_i5_m0


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"BoostC_d3_i5_m0" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load BoostC_d3_i5_m0.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize BoostC_d3_i5_m0 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of BoostC_d3_i5_m0.
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
    Initialize BoostC_d3_i5_m0 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize BoostC_d3_i5_m0 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize BoostC_d3_i5_m0 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for BoostC_d3_i5_m0.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize BoostC_d3_i5_m0 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for BoostC_d3_i5_m0.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct BoostC_d3_i5_m0 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid BoostC_d3_i5_m0 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(BoostC_d3_i5_m0 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct BoostC_d3_i5_m0 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid BoostC_d3_i5_m0 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(BoostC_d3_i5_m0 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            BoostC_d3_i5_m0 *typedModel = [[BoostC_d3_i5_m0 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable BoostC_d3_i5_m0Output *)predictionFromFeatures:(BoostC_d3_i5_m0Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable BoostC_d3_i5_m0Output *)predictionFromFeatures:(BoostC_d3_i5_m0Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[BoostC_d3_i5_m0Output alloc] initWithY:(NSString *)[outFeatures featureValueForName:@"y"].stringValue yProbability:(NSDictionary<NSString *, NSNumber *> *)[outFeatures featureValueForName:@"yProbability"].dictionaryValue];
}

- (void)predictionFromFeatures:(BoostC_d3_i5_m0Input *)input completionHandler:(void (^)(BoostC_d3_i5_m0Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            BoostC_d3_i5_m0Output *output = [[BoostC_d3_i5_m0Output alloc] initWithY:(NSString *)[prediction featureValueForName:@"y"].stringValue yProbability:(NSDictionary<NSString *, NSNumber *> *)[prediction featureValueForName:@"yProbability"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(BoostC_d3_i5_m0Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(BoostC_d3_i5_m0Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            BoostC_d3_i5_m0Output *output = [[BoostC_d3_i5_m0Output alloc] initWithY:(NSString *)[prediction featureValueForName:@"y"].stringValue yProbability:(NSDictionary<NSString *, NSNumber *> *)[prediction featureValueForName:@"yProbability"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable BoostC_d3_i5_m0Output *)predictionFromA:(double)a b:(double)b error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    BoostC_d3_i5_m0Input *input_ = [[BoostC_d3_i5_m0Input alloc] initWithA:a b:b];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<BoostC_d3_i5_m0Output *> *)predictionsFromInputs:(NSArray<BoostC_d3_i5_m0Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<BoostC_d3_i5_m0Output*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        BoostC_d3_i5_m0Output * result = [[BoostC_d3_i5_m0Output alloc] initWithY:(NSString *)[resultProvider featureValueForName:@"y"].stringValue yProbability:(NSDictionary<NSString *, NSNumber *> *)[resultProvider featureValueForName:@"yProbability"].dictionaryValue];
        [results addObject:result];
    }
    return results;
}

@end
