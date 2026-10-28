//
// PKnn.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "PKnn.h"

@implementation PKnnInput

- (instancetype)initWithZ:(MLMultiArray *)z {
    self = [super init];
    if (self) {
        _z = z;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"z"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"z"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.z];
    }
    return nil;
}

@end

@implementation PKnnOutput

- (instancetype)initWithLabel:(NSString *)label {
    self = [super init];
    if (self) {
        _label = label;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"label"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"label"]) {
        return [MLFeatureValue featureValueWithString:self.label];
    }
    return nil;
}

@end

@implementation PKnn


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"PKnn" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load PKnn.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize PKnn instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of PKnn.
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
    Initialize PKnn instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize PKnn instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize PKnn instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for PKnn.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize PKnn instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for PKnn.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct PKnn instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid PKnn instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(PKnn * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct PKnn instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid PKnn instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(PKnn * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            PKnn *typedModel = [[PKnn alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable PKnnOutput *)predictionFromFeatures:(PKnnInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable PKnnOutput *)predictionFromFeatures:(PKnnInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[PKnnOutput alloc] initWithLabel:(NSString *)[outFeatures featureValueForName:@"label"].stringValue];
}

- (void)predictionFromFeatures:(PKnnInput *)input completionHandler:(void (^)(PKnnOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            PKnnOutput *output = [[PKnnOutput alloc] initWithLabel:(NSString *)[prediction featureValueForName:@"label"].stringValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(PKnnInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(PKnnOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            PKnnOutput *output = [[PKnnOutput alloc] initWithLabel:(NSString *)[prediction featureValueForName:@"label"].stringValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable PKnnOutput *)predictionFromZ:(MLMultiArray *)z error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    PKnnInput *input_ = [[PKnnInput alloc] initWithZ:z];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<PKnnOutput *> *)predictionsFromInputs:(NSArray<PKnnInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<PKnnOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        PKnnOutput * result = [[PKnnOutput alloc] initWithLabel:(NSString *)[resultProvider featureValueForName:@"label"].stringValue];
        [results addObject:result];
    }
    return results;
}

@end
