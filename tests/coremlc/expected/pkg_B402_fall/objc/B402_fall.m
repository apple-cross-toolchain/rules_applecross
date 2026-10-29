//
// B402_fall.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "B402_fall.h"

@implementation B402_fallInput

- (instancetype)initWithX0:(double)x0 {
    self = [super init];
    if (self) {
        _x0 = x0;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"x0"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"x0"]) {
        return [MLFeatureValue featureValueWithDouble:self.x0];
    }
    return nil;
}

@end

@implementation B402_fallOutput

- (instancetype)initWithLabel:(NSString *)label classProbability:(NSDictionary<NSString *, NSNumber *> *)classProbability {
    self = [super init];
    if (self) {
        _label = label;
        _classProbability = classProbability;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"label", @"classProbability"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"label"]) {
        return [MLFeatureValue featureValueWithString:self.label];
    }
    if ([featureName isEqualToString:@"classProbability"]) {
        return [MLFeatureValue featureValueWithDictionary:self.classProbability error:nil];
    }
    return nil;
}

@end

@implementation B402_fall


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"B402_fall" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load B402_fall.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize B402_fall instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of B402_fall.
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
    Initialize B402_fall instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize B402_fall instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize B402_fall instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for B402_fall.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize B402_fall instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for B402_fall.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct B402_fall instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid B402_fall instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(B402_fall * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct B402_fall instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid B402_fall instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(B402_fall * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            B402_fall *typedModel = [[B402_fall alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable B402_fallOutput *)predictionFromFeatures:(B402_fallInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable B402_fallOutput *)predictionFromFeatures:(B402_fallInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[B402_fallOutput alloc] initWithLabel:(NSString *)[outFeatures featureValueForName:@"label"].stringValue classProbability:(NSDictionary<NSString *, NSNumber *> *)[outFeatures featureValueForName:@"classProbability"].dictionaryValue];
}

- (void)predictionFromFeatures:(B402_fallInput *)input completionHandler:(void (^)(B402_fallOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            B402_fallOutput *output = [[B402_fallOutput alloc] initWithLabel:(NSString *)[prediction featureValueForName:@"label"].stringValue classProbability:(NSDictionary<NSString *, NSNumber *> *)[prediction featureValueForName:@"classProbability"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(B402_fallInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(B402_fallOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            B402_fallOutput *output = [[B402_fallOutput alloc] initWithLabel:(NSString *)[prediction featureValueForName:@"label"].stringValue classProbability:(NSDictionary<NSString *, NSNumber *> *)[prediction featureValueForName:@"classProbability"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable B402_fallOutput *)predictionFromX0:(double)x0 error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    B402_fallInput *input_ = [[B402_fallInput alloc] initWithX0:x0];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<B402_fallOutput *> *)predictionsFromInputs:(NSArray<B402_fallInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<B402_fallOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        B402_fallOutput * result = [[B402_fallOutput alloc] initWithLabel:(NSString *)[resultProvider featureValueForName:@"label"].stringValue classProbability:(NSDictionary<NSString *, NSNumber *> *)[resultProvider featureValueForName:@"classProbability"].dictionaryValue];
        [results addObject:result];
    }
    return results;
}

@end
