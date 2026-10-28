//
// Recommender.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "Recommender.h"

@implementation RecommenderInput

- (instancetype)initWithItems:(NSDictionary<NSString *, NSNumber *> *)items k:(int64_t)k {
    return [self initWithItems:items k:k restrict_:nil exclude:nil];
}
- (instancetype)initWithItems:(NSDictionary<NSString *, NSNumber *> *)items k:(int64_t)k restrict_:(nullable NSArray<NSString *> *)restrict_ exclude:(nullable NSArray<NSString *> *)exclude {
    self = [super init];
    if (self) {
        _items = items;
        _k = k;
        _restrict_ = restrict_;
        _exclude = exclude;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"items", @"k", @"restrict", @"exclude"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"items"]) {
        return [MLFeatureValue featureValueWithDictionary:self.items error:nil];
    }
    if ([featureName isEqualToString:@"k"]) {
        return [MLFeatureValue featureValueWithInt64:self.k];
    }
    if ([featureName isEqualToString:@"restrict"]) {
        return self.restrict_ == nil ? nil : [MLFeatureValue featureValueWithSequence:[MLSequence sequenceWithStringArray:(NSArray<NSString *> *)self.restrict_]];
    }
    if ([featureName isEqualToString:@"exclude"]) {
        return self.exclude == nil ? nil : [MLFeatureValue featureValueWithSequence:[MLSequence sequenceWithStringArray:(NSArray<NSString *> *)self.exclude]];
    }
    return nil;
}

@end

@implementation RecommenderOutput

- (instancetype)initWithRecommendations:(NSArray<NSString *> *)recommendations scores:(NSDictionary<NSString *, NSNumber *> *)scores {
    self = [super init];
    if (self) {
        _recommendations = recommendations;
        _scores = scores;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"recommendations", @"scores"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"recommendations"]) {
        return [MLFeatureValue featureValueWithSequence:[MLSequence sequenceWithStringArray:self.recommendations]];
    }
    if ([featureName isEqualToString:@"scores"]) {
        return [MLFeatureValue featureValueWithDictionary:self.scores error:nil];
    }
    return nil;
}

@end

@implementation Recommender


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"Recommender" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load Recommender.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize Recommender instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of Recommender.
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
    Initialize Recommender instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize Recommender instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize Recommender instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Recommender.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize Recommender instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Recommender.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct Recommender instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Recommender instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Recommender * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct Recommender instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Recommender instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Recommender * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            Recommender *typedModel = [[Recommender alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable RecommenderOutput *)predictionFromFeatures:(RecommenderInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable RecommenderOutput *)predictionFromFeatures:(RecommenderInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[RecommenderOutput alloc] initWithRecommendations:(NSArray<NSString *> *)[outFeatures featureValueForName:@"recommendations"].sequenceValue.stringValues scores:(NSDictionary<NSString *, NSNumber *> *)[outFeatures featureValueForName:@"scores"].dictionaryValue];
}

- (void)predictionFromFeatures:(RecommenderInput *)input completionHandler:(void (^)(RecommenderOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            RecommenderOutput *output = [[RecommenderOutput alloc] initWithRecommendations:(NSArray<NSString *> *)[prediction featureValueForName:@"recommendations"].sequenceValue.stringValues scores:(NSDictionary<NSString *, NSNumber *> *)[prediction featureValueForName:@"scores"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(RecommenderInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(RecommenderOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            RecommenderOutput *output = [[RecommenderOutput alloc] initWithRecommendations:(NSArray<NSString *> *)[prediction featureValueForName:@"recommendations"].sequenceValue.stringValues scores:(NSDictionary<NSString *, NSNumber *> *)[prediction featureValueForName:@"scores"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable RecommenderOutput *)predictionFromItems:(NSDictionary<NSString *, NSNumber *> *)items k:(int64_t)k restrict_:(nullable NSArray<NSString *> *)restrict_ exclude:(nullable NSArray<NSString *> *)exclude error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    RecommenderInput *input_ = [[RecommenderInput alloc] initWithItems:items k:k restrict_:restrict_ exclude:exclude];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<RecommenderOutput *> *)predictionsFromInputs:(NSArray<RecommenderInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<RecommenderOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        RecommenderOutput * result = [[RecommenderOutput alloc] initWithRecommendations:(NSArray<NSString *> *)[resultProvider featureValueForName:@"recommendations"].sequenceValue.stringValues scores:(NSDictionary<NSString *, NSNumber *> *)[resultProvider featureValueForName:@"scores"].dictionaryValue];
        [results addObject:result];
    }
    return results;
}

@end
