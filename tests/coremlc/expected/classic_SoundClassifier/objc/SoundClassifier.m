//
// SoundClassifier.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "SoundClassifier.h"

@implementation SoundClassifierInput

- (instancetype)initWithAudioSamples:(MLMultiArray *)audioSamples {
    self = [super init];
    if (self) {
        _audioSamples = audioSamples;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"audioSamples"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"audioSamples"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.audioSamples];
    }
    return nil;
}

@end

@implementation SoundClassifierOutput

- (instancetype)initWithTarget:(NSString *)target targetProbability:(NSDictionary<NSString *, NSNumber *> *)targetProbability {
    self = [super init];
    if (self) {
        _target = target;
        _targetProbability = targetProbability;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"target", @"targetProbability"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"target"]) {
        return [MLFeatureValue featureValueWithString:self.target];
    }
    if ([featureName isEqualToString:@"targetProbability"]) {
        return [MLFeatureValue featureValueWithDictionary:self.targetProbability error:nil];
    }
    return nil;
}

@end

@implementation SoundClassifier


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"SoundClassifier" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load SoundClassifier.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize SoundClassifier instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of SoundClassifier.
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
    Initialize SoundClassifier instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize SoundClassifier instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize SoundClassifier instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for SoundClassifier.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize SoundClassifier instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for SoundClassifier.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct SoundClassifier instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid SoundClassifier instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(SoundClassifier * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct SoundClassifier instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid SoundClassifier instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(SoundClassifier * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            SoundClassifier *typedModel = [[SoundClassifier alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable SoundClassifierOutput *)predictionFromFeatures:(SoundClassifierInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable SoundClassifierOutput *)predictionFromFeatures:(SoundClassifierInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[SoundClassifierOutput alloc] initWithTarget:(NSString *)[outFeatures featureValueForName:@"target"].stringValue targetProbability:(NSDictionary<NSString *, NSNumber *> *)[outFeatures featureValueForName:@"targetProbability"].dictionaryValue];
}

- (void)predictionFromFeatures:(SoundClassifierInput *)input completionHandler:(void (^)(SoundClassifierOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            SoundClassifierOutput *output = [[SoundClassifierOutput alloc] initWithTarget:(NSString *)[prediction featureValueForName:@"target"].stringValue targetProbability:(NSDictionary<NSString *, NSNumber *> *)[prediction featureValueForName:@"targetProbability"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(SoundClassifierInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(SoundClassifierOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            SoundClassifierOutput *output = [[SoundClassifierOutput alloc] initWithTarget:(NSString *)[prediction featureValueForName:@"target"].stringValue targetProbability:(NSDictionary<NSString *, NSNumber *> *)[prediction featureValueForName:@"targetProbability"].dictionaryValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable SoundClassifierOutput *)predictionFromAudioSamples:(MLMultiArray *)audioSamples error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    SoundClassifierInput *input_ = [[SoundClassifierInput alloc] initWithAudioSamples:audioSamples];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<SoundClassifierOutput *> *)predictionsFromInputs:(NSArray<SoundClassifierInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<SoundClassifierOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        SoundClassifierOutput * result = [[SoundClassifierOutput alloc] initWithTarget:(NSString *)[resultProvider featureValueForName:@"target"].stringValue targetProbability:(NSDictionary<NSString *, NSNumber *> *)[resultProvider featureValueForName:@"targetProbability"].dictionaryValue];
        [results addObject:result];
    }
    return results;
}

@end
