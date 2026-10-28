//
// Nms.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "Nms.h"

@implementation NmsInput

- (instancetype)initWithConfidence:(MLMultiArray *)confidence coordinates:(MLMultiArray *)coordinates {
    self = [super init];
    if (self) {
        _confidence = confidence;
        _coordinates = coordinates;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"confidence", @"coordinates"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"confidence"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.confidence];
    }
    if ([featureName isEqualToString:@"coordinates"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.coordinates];
    }
    return nil;
}

@end

@implementation NmsOutput

- (instancetype)initWithC:(MLMultiArray *)c b:(MLMultiArray *)b {
    self = [super init];
    if (self) {
        _c = c;
        _b = b;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"c", @"b"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"c"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.c];
    }
    if ([featureName isEqualToString:@"b"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.b];
    }
    return nil;
}

@end

@implementation Nms


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"Nms" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load Nms.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize Nms instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of Nms.
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
    Initialize Nms instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize Nms instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize Nms instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Nms.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize Nms instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Nms.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct Nms instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Nms instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Nms * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct Nms instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Nms instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Nms * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            Nms *typedModel = [[Nms alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable NmsOutput *)predictionFromFeatures:(NmsInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable NmsOutput *)predictionFromFeatures:(NmsInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[NmsOutput alloc] initWithC:(MLMultiArray *)[outFeatures featureValueForName:@"c"].multiArrayValue b:(MLMultiArray *)[outFeatures featureValueForName:@"b"].multiArrayValue];
}

- (void)predictionFromFeatures:(NmsInput *)input completionHandler:(void (^)(NmsOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            NmsOutput *output = [[NmsOutput alloc] initWithC:(MLMultiArray *)[prediction featureValueForName:@"c"].multiArrayValue b:(MLMultiArray *)[prediction featureValueForName:@"b"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(NmsInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(NmsOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            NmsOutput *output = [[NmsOutput alloc] initWithC:(MLMultiArray *)[prediction featureValueForName:@"c"].multiArrayValue b:(MLMultiArray *)[prediction featureValueForName:@"b"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable NmsOutput *)predictionFromConfidence:(MLMultiArray *)confidence coordinates:(MLMultiArray *)coordinates error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NmsInput *input_ = [[NmsInput alloc] initWithConfidence:confidence coordinates:coordinates];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<NmsOutput *> *)predictionsFromInputs:(NSArray<NmsInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<NmsOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        NmsOutput * result = [[NmsOutput alloc] initWithC:(MLMultiArray *)[resultProvider featureValueForName:@"c"].multiArrayValue b:(MLMultiArray *)[resultProvider featureValueForName:@"b"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
