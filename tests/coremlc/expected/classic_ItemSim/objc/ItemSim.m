//
// ItemSim.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "ItemSim.h"

@implementation ItemSimInput

- (instancetype)initWithItems:(NSDictionary<NSNumber *, NSNumber *> *)items {
    return [self initWithItems:items k:nil];
}
- (instancetype)initWithItems:(NSDictionary<NSNumber *, NSNumber *> *)items k:(nullable NSNumber *)k {
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

@implementation ItemSimOutput

- (instancetype)initWithRecommendations:(NSArray<NSNumber *> *)recommendations {
    self = [super init];
    if (self) {
        _recommendations = recommendations;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"recommendations"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"recommendations"]) {
        return [MLFeatureValue featureValueWithSequence:[MLSequence sequenceWithInt64Array:self.recommendations]];
    }
    return nil;
}

@end

@implementation ItemSim


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"ItemSim" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load ItemSim.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize ItemSim instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of ItemSim.
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
    Initialize ItemSim instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize ItemSim instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize ItemSim instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ItemSim.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize ItemSim instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ItemSim.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct ItemSim instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ItemSim instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ItemSim * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct ItemSim instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ItemSim instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ItemSim * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            ItemSim *typedModel = [[ItemSim alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable ItemSimOutput *)predictionFromFeatures:(ItemSimInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable ItemSimOutput *)predictionFromFeatures:(ItemSimInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[ItemSimOutput alloc] initWithRecommendations:(NSArray<NSNumber *> *)[outFeatures featureValueForName:@"recommendations"].sequenceValue.int64Values];
}

- (void)predictionFromFeatures:(ItemSimInput *)input completionHandler:(void (^)(ItemSimOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ItemSimOutput *output = [[ItemSimOutput alloc] initWithRecommendations:(NSArray<NSNumber *> *)[prediction featureValueForName:@"recommendations"].sequenceValue.int64Values];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(ItemSimInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(ItemSimOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ItemSimOutput *output = [[ItemSimOutput alloc] initWithRecommendations:(NSArray<NSNumber *> *)[prediction featureValueForName:@"recommendations"].sequenceValue.int64Values];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable ItemSimOutput *)predictionFromItems:(NSDictionary<NSNumber *, NSNumber *> *)items k:(nullable NSNumber *)k error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    ItemSimInput *input_ = [[ItemSimInput alloc] initWithItems:items k:k];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<ItemSimOutput *> *)predictionsFromInputs:(NSArray<ItemSimInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<ItemSimOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        ItemSimOutput * result = [[ItemSimOutput alloc] initWithRecommendations:(NSArray<NSNumber *> *)[resultProvider featureValueForName:@"recommendations"].sequenceValue.int64Values];
        [results addObject:result];
    }
    return results;
}

@end
