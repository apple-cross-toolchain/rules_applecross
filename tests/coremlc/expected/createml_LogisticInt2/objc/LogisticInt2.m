//
// LogisticInt2.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "LogisticInt2.h"

@implementation LogisticInt2Input

- (instancetype)initWithA:(double)a c:(int64_t)c {
    self = [super init];
    if (self) {
        _a = a;
        _c = c;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"a", @"c"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"a"]) {
        return [MLFeatureValue featureValueWithDouble:self.a];
    }
    if ([featureName isEqualToString:@"c"]) {
        return [MLFeatureValue featureValueWithInt64:self.c];
    }
    return nil;
}

@end

@implementation LogisticInt2Output

- (instancetype)initWithYi:(int64_t)yi yiProbability:(NSDictionary<NSNumber *, NSNumber *> *)yiProbability {
    self = [super init];
    if (self) {
        _yi = yi;
        _yiProbability = yiProbability;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"yi", @"yiProbability"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"yi"]) {
        return [MLFeatureValue featureValueWithInt64:self.yi];
    }
    if ([featureName isEqualToString:@"yiProbability"]) {
        return [MLFeatureValue featureValueWithDictionary:self.yiProbability error:nil];
    }
    return nil;
}

@end

@implementation LogisticInt2


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"LogisticInt2" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load LogisticInt2.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize LogisticInt2 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of LogisticInt2.
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
    Initialize LogisticInt2 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize LogisticInt2 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize LogisticInt2 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for LogisticInt2.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize LogisticInt2 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for LogisticInt2.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct LogisticInt2 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid LogisticInt2 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(LogisticInt2 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct LogisticInt2 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid LogisticInt2 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(LogisticInt2 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            LogisticInt2 *typedModel = [[LogisticInt2 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable LogisticInt2Output *)predictionFromFeatures:(LogisticInt2Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable LogisticInt2Output *)predictionFromFeatures:(LogisticInt2Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[LogisticInt2Output alloc] initWithYi:[outFeatures featureValueForName:@"yi"].int64Value yiProbability:(NSDictionary<NSNumber *, NSNumber *> *)[outFeatures featureValueForName:@"yiProbability"].dictionaryValue];
}

- (void)predictionFromFeatures:(LogisticInt2Input *)input completionHandler:(void (^)(LogisticInt2Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            LogisticInt2Output *output = [[LogisticInt2Output alloc] initWithYi:[prediction featureValueForName:@"yi"].int64Value yiProbability:(NSDictionary<NSNumber *, NSNumber *> *)[prediction featureValueForName:@"yiProbability"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(LogisticInt2Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(LogisticInt2Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            LogisticInt2Output *output = [[LogisticInt2Output alloc] initWithYi:[prediction featureValueForName:@"yi"].int64Value yiProbability:(NSDictionary<NSNumber *, NSNumber *> *)[prediction featureValueForName:@"yiProbability"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable LogisticInt2Output *)predictionFromA:(double)a c:(int64_t)c error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    LogisticInt2Input *input_ = [[LogisticInt2Input alloc] initWithA:a c:c];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<LogisticInt2Output *> *)predictionsFromInputs:(NSArray<LogisticInt2Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<LogisticInt2Output*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        LogisticInt2Output * result = [[LogisticInt2Output alloc] initWithYi:[resultProvider featureValueForName:@"yi"].int64Value yiProbability:(NSDictionary<NSNumber *, NSNumber *> *)[resultProvider featureValueForName:@"yiProbability"].dictionaryValue];
        [results addObject:result];
    }
    return results;
}

@end
