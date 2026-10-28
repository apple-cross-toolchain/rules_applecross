//
// ImKeyword.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "ImKeyword.h"

@implementation ImKeywordInput

- (instancetype)initWithSelf:(CVPixelBufferRef)self_ var_:(MLMultiArray *)var_ {
    self = [super init];
    if (self) {
        _self_ = self_;
        CVPixelBufferRetain(_self_);
        _var_ = var_;
    }
    return self;
}

- (void)dealloc {
    CVPixelBufferRelease(_self_);
}

- (nullable instancetype)initWithSelfFromCGImage:(CGImageRef)self_ var_:(MLMultiArray *)var_ error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __self_ = [MLFeatureValue featureValueWithCGImage:self_ pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
                if (__self_ == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithSelf:(CVPixelBufferRef)__self_.imageBufferValue var_:var_];
            }
            while(0);
        }
        if (error != NULL) {
            *error = localError;
        }
        return result ? retVal : nil;
    }
    return self;
}

- (nullable instancetype)initWithSelfAtURL:(NSURL *)self_URL var_:(MLMultiArray *)var_ error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __self_ = [MLFeatureValue featureValueWithImageAtURL:self_URL pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
                if (__self_ == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithSelf:(CVPixelBufferRef)__self_.imageBufferValue var_:var_];
            }
            while(0);
        }
        if (error != NULL) {
            *error = localError;
        }
        return result ? retVal : nil;
    }
    return self;
}

-(BOOL)setSelf_WithCGImage:(CGImageRef)self_ error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __self_ = [MLFeatureValue featureValueWithCGImage:self_ pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
        if (__self_ != nil) {
            CVPixelBufferRelease(self.self_);
            self.self_ =  (CVPixelBufferRef)__self_.imageBufferValue;
            CVPixelBufferRetain(self.self_);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setSelf_WithURL:(NSURL *)self_URL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __self_ = [MLFeatureValue featureValueWithImageAtURL:self_URL pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
        if (__self_ != nil) {
            CVPixelBufferRelease(self.self_);
            self.self_ =  (CVPixelBufferRef)__self_.imageBufferValue;
            CVPixelBufferRetain(self.self_);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"self", @"var"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"self"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.self_];
    }
    if ([featureName isEqualToString:@"var"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.var_];
    }
    return nil;
}

@end

@implementation ImKeywordOutput

- (instancetype)initWithReshape_0:(MLMultiArray *)reshape_0 {
    self = [super init];
    if (self) {
        _reshape_0 = reshape_0;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"reshape_0"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"reshape_0"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.reshape_0];
    }
    return nil;
}

@end

@implementation ImKeyword


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"ImKeyword" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load ImKeyword.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize ImKeyword instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of ImKeyword.
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
    Initialize ImKeyword instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize ImKeyword instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize ImKeyword instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImKeyword.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize ImKeyword instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImKeyword.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct ImKeyword instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImKeyword instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImKeyword * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct ImKeyword instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImKeyword instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImKeyword * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            ImKeyword *typedModel = [[ImKeyword alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable ImKeywordOutput *)predictionFromFeatures:(ImKeywordInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable ImKeywordOutput *)predictionFromFeatures:(ImKeywordInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[ImKeywordOutput alloc] initWithReshape_0:(MLMultiArray *)[outFeatures featureValueForName:@"reshape_0"].multiArrayValue];
}

- (void)predictionFromFeatures:(ImKeywordInput *)input completionHandler:(void (^)(ImKeywordOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ImKeywordOutput *output = [[ImKeywordOutput alloc] initWithReshape_0:(MLMultiArray *)[prediction featureValueForName:@"reshape_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(ImKeywordInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(ImKeywordOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ImKeywordOutput *output = [[ImKeywordOutput alloc] initWithReshape_0:(MLMultiArray *)[prediction featureValueForName:@"reshape_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable ImKeywordOutput *)predictionFromSelf:(CVPixelBufferRef)self_ var_:(MLMultiArray *)var_ error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    ImKeywordInput *input_ = [[ImKeywordInput alloc] initWithSelf:self_ var_:var_];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<ImKeywordOutput *> *)predictionsFromInputs:(NSArray<ImKeywordInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<ImKeywordOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        ImKeywordOutput * result = [[ImKeywordOutput alloc] initWithReshape_0:(MLMultiArray *)[resultProvider featureValueForName:@"reshape_0"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
