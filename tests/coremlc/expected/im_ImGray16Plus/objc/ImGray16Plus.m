//
// ImGray16Plus.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "ImGray16Plus.h"

@implementation ImGray16PlusInput

- (instancetype)initWithG:(CVPixelBufferRef)g t:(MLMultiArray *)t {
    self = [super init];
    if (self) {
        _g = g;
        CVPixelBufferRetain(_g);
        _t = t;
    }
    return self;
}

- (void)dealloc {
    CVPixelBufferRelease(_g);
}

- (nullable instancetype)initWithGFromCGImage:(CGImageRef)g t:(MLMultiArray *)t error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __g = [MLFeatureValue featureValueWithCGImage:g pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_OneComponent16Half options:nil error:&localError];
                if (__g == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithG:(CVPixelBufferRef)__g.imageBufferValue t:t];
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

- (nullable instancetype)initWithGAtURL:(NSURL *)gURL t:(MLMultiArray *)t error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __g = [MLFeatureValue featureValueWithImageAtURL:gURL pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_OneComponent16Half options:nil error:&localError];
                if (__g == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithG:(CVPixelBufferRef)__g.imageBufferValue t:t];
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

-(BOOL)setGWithCGImage:(CGImageRef)g error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __g = [MLFeatureValue featureValueWithCGImage:g pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_OneComponent16Half options:nil error:&localError];
        if (__g != nil) {
            CVPixelBufferRelease(self.g);
            self.g =  (CVPixelBufferRef)__g.imageBufferValue;
            CVPixelBufferRetain(self.g);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setGWithURL:(NSURL *)gURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __g = [MLFeatureValue featureValueWithImageAtURL:gURL pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_OneComponent16Half options:nil error:&localError];
        if (__g != nil) {
            CVPixelBufferRelease(self.g);
            self.g =  (CVPixelBufferRef)__g.imageBufferValue;
            CVPixelBufferRetain(self.g);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"g", @"t"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"g"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.g];
    }
    if ([featureName isEqualToString:@"t"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.t];
    }
    return nil;
}

@end

@implementation ImGray16PlusOutput

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

@implementation ImGray16Plus


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"ImGray16Plus" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load ImGray16Plus.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize ImGray16Plus instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of ImGray16Plus.
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
    Initialize ImGray16Plus instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize ImGray16Plus instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize ImGray16Plus instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImGray16Plus.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize ImGray16Plus instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImGray16Plus.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct ImGray16Plus instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImGray16Plus instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImGray16Plus * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct ImGray16Plus instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImGray16Plus instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImGray16Plus * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            ImGray16Plus *typedModel = [[ImGray16Plus alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable ImGray16PlusOutput *)predictionFromFeatures:(ImGray16PlusInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable ImGray16PlusOutput *)predictionFromFeatures:(ImGray16PlusInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[ImGray16PlusOutput alloc] initWithReshape_0:(MLMultiArray *)[outFeatures featureValueForName:@"reshape_0"].multiArrayValue];
}

- (void)predictionFromFeatures:(ImGray16PlusInput *)input completionHandler:(void (^)(ImGray16PlusOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ImGray16PlusOutput *output = [[ImGray16PlusOutput alloc] initWithReshape_0:(MLMultiArray *)[prediction featureValueForName:@"reshape_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(ImGray16PlusInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(ImGray16PlusOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ImGray16PlusOutput *output = [[ImGray16PlusOutput alloc] initWithReshape_0:(MLMultiArray *)[prediction featureValueForName:@"reshape_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable ImGray16PlusOutput *)predictionFromG:(CVPixelBufferRef)g t:(MLMultiArray *)t error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    ImGray16PlusInput *input_ = [[ImGray16PlusInput alloc] initWithG:g t:t];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<ImGray16PlusOutput *> *)predictionsFromInputs:(NSArray<ImGray16PlusInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<ImGray16PlusOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        ImGray16PlusOutput * result = [[ImGray16PlusOutput alloc] initWithReshape_0:(MLMultiArray *)[resultProvider featureValueForName:@"reshape_0"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
