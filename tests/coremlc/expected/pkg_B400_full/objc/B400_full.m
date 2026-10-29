//
// B400_full.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "B400_full.h"

@implementation B400_fullInput

- (instancetype)initWithB_24:(int64_t)b_24 {
    self = [super init];
    if (self) {
        _b_24 = b_24;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"b_24"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"b_24"]) {
        return [MLFeatureValue featureValueWithInt64:self.b_24];
    }
    return nil;
}

@end

@implementation B400_fullOutput

- (instancetype)initWithProbs:(int64_t)probs z0:(NSDictionary<NSNumber *, NSNumber *> *)z0 {
    self = [super init];
    if (self) {
        _probs = probs;
        _z0 = z0;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"probs", @"z0"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"probs"]) {
        return [MLFeatureValue featureValueWithInt64:self.probs];
    }
    if ([featureName isEqualToString:@"z0"]) {
        return [MLFeatureValue featureValueWithDictionary:self.z0 error:nil];
    }
    return nil;
}

@end

@implementation B400_full


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"B400_full" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load B400_full.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize B400_full instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of B400_full.
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
    Initialize B400_full instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize B400_full instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize B400_full instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for B400_full.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize B400_full instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for B400_full.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct B400_full instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid B400_full instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(B400_full * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct B400_full instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid B400_full instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(B400_full * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            B400_full *typedModel = [[B400_full alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable B400_fullOutput *)predictionFromFeatures:(B400_fullInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable B400_fullOutput *)predictionFromFeatures:(B400_fullInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[B400_fullOutput alloc] initWithProbs:[outFeatures featureValueForName:@"probs"].int64Value z0:(NSDictionary<NSNumber *, NSNumber *> *)[outFeatures featureValueForName:@"z0"].dictionaryValue];
}

- (void)predictionFromFeatures:(B400_fullInput *)input completionHandler:(void (^)(B400_fullOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            B400_fullOutput *output = [[B400_fullOutput alloc] initWithProbs:[prediction featureValueForName:@"probs"].int64Value z0:(NSDictionary<NSNumber *, NSNumber *> *)[prediction featureValueForName:@"z0"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(B400_fullInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(B400_fullOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            B400_fullOutput *output = [[B400_fullOutput alloc] initWithProbs:[prediction featureValueForName:@"probs"].int64Value z0:(NSDictionary<NSNumber *, NSNumber *> *)[prediction featureValueForName:@"z0"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable B400_fullOutput *)predictionFromB_24:(int64_t)b_24 error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    B400_fullInput *input_ = [[B400_fullInput alloc] initWithB_24:b_24];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<B400_fullOutput *> *)predictionsFromInputs:(NSArray<B400_fullInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<B400_fullOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        B400_fullOutput * result = [[B400_fullOutput alloc] initWithProbs:[resultProvider featureValueForName:@"probs"].int64Value z0:(NSDictionary<NSNumber *, NSNumber *> *)[resultProvider featureValueForName:@"z0"].dictionaryValue];
        [results addObject:result];
    }
    return results;
}

@end
