//
// NPipePartsNoMeta.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "NPipePartsNoMeta.h"

@implementation NPipePartsNoMetaInput

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

@implementation NPipePartsNoMetaOutput

- (instancetype)initWithY:(MLMultiArray *)y {
    self = [super init];
    if (self) {
        _y = y;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"y"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"y"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.y];
    }
    return nil;
}

@end

@implementation NPipePartsNoMeta


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"NPipePartsNoMeta" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load NPipePartsNoMeta.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize NPipePartsNoMeta instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of NPipePartsNoMeta.
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
    Initialize NPipePartsNoMeta instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize NPipePartsNoMeta instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize NPipePartsNoMeta instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for NPipePartsNoMeta.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize NPipePartsNoMeta instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for NPipePartsNoMeta.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct NPipePartsNoMeta instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid NPipePartsNoMeta instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(NPipePartsNoMeta * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct NPipePartsNoMeta instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid NPipePartsNoMeta instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(NPipePartsNoMeta * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            NPipePartsNoMeta *typedModel = [[NPipePartsNoMeta alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable NPipePartsNoMetaOutput *)predictionFromFeatures:(NPipePartsNoMetaInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable NPipePartsNoMetaOutput *)predictionFromFeatures:(NPipePartsNoMetaInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[NPipePartsNoMetaOutput alloc] initWithY:(MLMultiArray *)[outFeatures featureValueForName:@"y"].multiArrayValue];
}

- (void)predictionFromFeatures:(NPipePartsNoMetaInput *)input completionHandler:(void (^)(NPipePartsNoMetaOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            NPipePartsNoMetaOutput *output = [[NPipePartsNoMetaOutput alloc] initWithY:(MLMultiArray *)[prediction featureValueForName:@"y"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(NPipePartsNoMetaInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(NPipePartsNoMetaOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            NPipePartsNoMetaOutput *output = [[NPipePartsNoMetaOutput alloc] initWithY:(MLMultiArray *)[prediction featureValueForName:@"y"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable NPipePartsNoMetaOutput *)predictionFromX:(MLMultiArray *)x error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NPipePartsNoMetaInput *input_ = [[NPipePartsNoMetaInput alloc] initWithX:x];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<NPipePartsNoMetaOutput *> *)predictionsFromInputs:(NSArray<NPipePartsNoMetaInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<NPipePartsNoMetaOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        NPipePartsNoMetaOutput * result = [[NPipePartsNoMetaOutput alloc] initWithY:(MLMultiArray *)[resultProvider featureValueForName:@"y"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
