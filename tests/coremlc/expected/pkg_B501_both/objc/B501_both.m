//
// B501_both.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "B501_both.h"

@implementation B501_bothInput

- (instancetype)initWithItems:(NSDictionary<NSString *, NSNumber *> *)items {
    return [self initWithItems:items k:nil];
}
- (instancetype)initWithItems:(NSDictionary<NSString *, NSNumber *> *)items k:(nullable NSNumber *)k {
    self = [super init];
    if (self) {
        _items = items;
        _k = k;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"items", @"k"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"items"]) {
        return [MLFeatureValue featureValueWithDictionary:self.items error:nil];
    }
    if ([featureName isEqualToString:@"k"]) {
        return self.k == nil ? nil : [MLFeatureValue featureValueWithInt64:self.k.longLongValue];
    }
    return nil;
}

@end

@implementation B501_bothOutput

- (instancetype)initWithRecs:(NSArray<NSString *> *)recs {
    self = [super init];
    if (self) {
        _recs = recs;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"recs"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"recs"]) {
        return [MLFeatureValue featureValueWithSequence:[MLSequence sequenceWithStringArray:self.recs]];
    }
    return nil;
}

@end

@implementation B501_both


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"B501_both" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load B501_both.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize B501_both instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of B501_both.
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
    Initialize B501_both instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize B501_both instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize B501_both instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for B501_both.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize B501_both instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for B501_both.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct B501_both instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid B501_both instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(B501_both * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct B501_both instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid B501_both instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(B501_both * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            B501_both *typedModel = [[B501_both alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable B501_bothOutput *)predictionFromFeatures:(B501_bothInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable B501_bothOutput *)predictionFromFeatures:(B501_bothInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[B501_bothOutput alloc] initWithRecs:(NSArray<NSString *> *)[outFeatures featureValueForName:@"recs"].sequenceValue.stringValues];
}

- (void)predictionFromFeatures:(B501_bothInput *)input completionHandler:(void (^)(B501_bothOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            B501_bothOutput *output = [[B501_bothOutput alloc] initWithRecs:(NSArray<NSString *> *)[prediction featureValueForName:@"recs"].sequenceValue.stringValues];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(B501_bothInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(B501_bothOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            B501_bothOutput *output = [[B501_bothOutput alloc] initWithRecs:(NSArray<NSString *> *)[prediction featureValueForName:@"recs"].sequenceValue.stringValues];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable B501_bothOutput *)predictionFromItems:(NSDictionary<NSString *, NSNumber *> *)items k:(nullable NSNumber *)k error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    B501_bothInput *input_ = [[B501_bothInput alloc] initWithItems:items k:k];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<B501_bothOutput *> *)predictionsFromInputs:(NSArray<B501_bothInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<B501_bothOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        B501_bothOutput * result = [[B501_bothOutput alloc] initWithRecs:(NSArray<NSString *> *)[resultProvider featureValueForName:@"recs"].sequenceValue.stringValues];
        [results addObject:result];
    }
    return results;
}

@end
