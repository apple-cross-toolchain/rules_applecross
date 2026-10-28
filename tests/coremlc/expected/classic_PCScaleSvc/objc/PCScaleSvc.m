//
// PCScaleSvc.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "PCScaleSvc.h"

@implementation PCScaleSvcInput

- (instancetype)initWithX:(MLMultiArray *)x {
    self = [super init];
    if (self) {
        _x = x;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"x"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"x"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.x];
    }
    return nil;
}

@end

@implementation PCScaleSvcOutput

- (instancetype)initWithLabel:(NSString *)label probs:(NSDictionary<NSString *, NSNumber *> *)probs {
    self = [super init];
    if (self) {
        _label = label;
        _probs = probs;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"label", @"probs"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"label"]) {
        return [MLFeatureValue featureValueWithString:self.label];
    }
    if ([featureName isEqualToString:@"probs"]) {
        return [MLFeatureValue featureValueWithDictionary:self.probs error:nil];
    }
    return nil;
}

@end

@implementation PCScaleSvc


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"PCScaleSvc" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load PCScaleSvc.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize PCScaleSvc instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of PCScaleSvc.
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
    Initialize PCScaleSvc instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize PCScaleSvc instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize PCScaleSvc instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for PCScaleSvc.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize PCScaleSvc instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for PCScaleSvc.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct PCScaleSvc instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid PCScaleSvc instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(PCScaleSvc * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct PCScaleSvc instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid PCScaleSvc instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(PCScaleSvc * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            PCScaleSvc *typedModel = [[PCScaleSvc alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable PCScaleSvcOutput *)predictionFromFeatures:(PCScaleSvcInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable PCScaleSvcOutput *)predictionFromFeatures:(PCScaleSvcInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[PCScaleSvcOutput alloc] initWithLabel:(NSString *)[outFeatures featureValueForName:@"label"].stringValue probs:(NSDictionary<NSString *, NSNumber *> *)[outFeatures featureValueForName:@"probs"].dictionaryValue];
}

- (void)predictionFromFeatures:(PCScaleSvcInput *)input completionHandler:(void (^)(PCScaleSvcOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            PCScaleSvcOutput *output = [[PCScaleSvcOutput alloc] initWithLabel:(NSString *)[prediction featureValueForName:@"label"].stringValue probs:(NSDictionary<NSString *, NSNumber *> *)[prediction featureValueForName:@"probs"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(PCScaleSvcInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(PCScaleSvcOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            PCScaleSvcOutput *output = [[PCScaleSvcOutput alloc] initWithLabel:(NSString *)[prediction featureValueForName:@"label"].stringValue probs:(NSDictionary<NSString *, NSNumber *> *)[prediction featureValueForName:@"probs"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable PCScaleSvcOutput *)predictionFromX:(MLMultiArray *)x error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    PCScaleSvcInput *input_ = [[PCScaleSvcInput alloc] initWithX:x];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<PCScaleSvcOutput *> *)predictionsFromInputs:(NSArray<PCScaleSvcInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<PCScaleSvcOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        PCScaleSvcOutput * result = [[PCScaleSvcOutput alloc] initWithLabel:(NSString *)[resultProvider featureValueForName:@"label"].stringValue probs:(NSDictionary<NSString *, NSNumber *> *)[resultProvider featureValueForName:@"probs"].dictionaryValue];
        [results addObject:result];
    }
    return results;
}

@end
