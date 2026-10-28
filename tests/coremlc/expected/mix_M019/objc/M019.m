//
// M019.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "M019.h"

@implementation M019Input

- (instancetype)initWithIn1:(CVPixelBufferRef)in1 {
    return [self initWithIn1:in1 z9:nil];
}
- (nullable instancetype)initWithIn1FromCGImage:(CGImageRef)in1 error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __in1 = [MLFeatureValue featureValueWithCGImage:in1 pixelsWide:10 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
                if (__in1 == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithIn1:(CVPixelBufferRef)__in1.imageBufferValue];
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

- (nullable instancetype)initWithIn1AtURL:(NSURL *)in1URL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __in1 = [MLFeatureValue featureValueWithImageAtURL:in1URL pixelsWide:10 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
                if (__in1 == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithIn1:(CVPixelBufferRef)__in1.imageBufferValue];
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

- (instancetype)initWithIn1:(CVPixelBufferRef)in1 z9:(nullable MLMultiArray *)z9 {
    self = [super init];
    if (self) {
        _in1 = in1;
        CVPixelBufferRetain(_in1);
        _z9 = z9;
    }
    return self;
}

- (void)dealloc {
    CVPixelBufferRelease(_in1);
}

- (nullable instancetype)initWithIn1FromCGImage:(CGImageRef)in1 z9:(nullable MLMultiArray *)z9 error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __in1 = [MLFeatureValue featureValueWithCGImage:in1 pixelsWide:10 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
                if (__in1 == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithIn1:(CVPixelBufferRef)__in1.imageBufferValue z9:z9];
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

- (nullable instancetype)initWithIn1AtURL:(NSURL *)in1URL z9:(nullable MLMultiArray *)z9 error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __in1 = [MLFeatureValue featureValueWithImageAtURL:in1URL pixelsWide:10 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
                if (__in1 == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithIn1:(CVPixelBufferRef)__in1.imageBufferValue z9:z9];
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

-(BOOL)setIn1WithCGImage:(CGImageRef)in1 error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __in1 = [MLFeatureValue featureValueWithCGImage:in1 pixelsWide:10 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
        if (__in1 != nil) {
            CVPixelBufferRelease(self.in1);
            self.in1 =  (CVPixelBufferRef)__in1.imageBufferValue;
            CVPixelBufferRetain(self.in1);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setIn1WithURL:(NSURL *)in1URL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __in1 = [MLFeatureValue featureValueWithImageAtURL:in1URL pixelsWide:10 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
        if (__in1 != nil) {
            CVPixelBufferRelease(self.in1);
            self.in1 =  (CVPixelBufferRef)__in1.imageBufferValue;
            CVPixelBufferRetain(self.in1);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"in1", @"z9"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"in1"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.in1];
    }
    if ([featureName isEqualToString:@"z9"]) {
        return self.z9 == nil ? nil : [MLFeatureValue featureValueWithMultiArray:(MLMultiArray *)self.z9];
    }
    return nil;
}

@end

@implementation M019Output

- (instancetype)initWithMean_in1:(MLMultiArray *)mean_in1 o_z9:(MLMultiArray *)o_z9 {
    self = [super init];
    if (self) {
        _mean_in1 = mean_in1;
        _o_z9 = o_z9;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"mean_in1", @"o_z9"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"mean_in1"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.mean_in1];
    }
    if ([featureName isEqualToString:@"o_z9"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.o_z9];
    }
    return nil;
}

@end

@implementation M019


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"M019" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load M019.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize M019 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of M019.
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
    Initialize M019 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize M019 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize M019 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for M019.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize M019 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for M019.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct M019 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid M019 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(M019 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct M019 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid M019 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(M019 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            M019 *typedModel = [[M019 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable M019Output *)predictionFromFeatures:(M019Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable M019Output *)predictionFromFeatures:(M019Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[M019Output alloc] initWithMean_in1:(MLMultiArray *)[outFeatures featureValueForName:@"mean_in1"].multiArrayValue o_z9:(MLMultiArray *)[outFeatures featureValueForName:@"o_z9"].multiArrayValue];
}

- (void)predictionFromFeatures:(M019Input *)input completionHandler:(void (^)(M019Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            M019Output *output = [[M019Output alloc] initWithMean_in1:(MLMultiArray *)[prediction featureValueForName:@"mean_in1"].multiArrayValue o_z9:(MLMultiArray *)[prediction featureValueForName:@"o_z9"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(M019Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(M019Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            M019Output *output = [[M019Output alloc] initWithMean_in1:(MLMultiArray *)[prediction featureValueForName:@"mean_in1"].multiArrayValue o_z9:(MLMultiArray *)[prediction featureValueForName:@"o_z9"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable M019Output *)predictionFromIn1:(CVPixelBufferRef)in1 z9:(nullable MLMultiArray *)z9 error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    M019Input *input_ = [[M019Input alloc] initWithIn1:in1 z9:z9];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<M019Output *> *)predictionsFromInputs:(NSArray<M019Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<M019Output*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        M019Output * result = [[M019Output alloc] initWithMean_in1:(MLMultiArray *)[resultProvider featureValueForName:@"mean_in1"].multiArrayValue o_z9:(MLMultiArray *)[resultProvider featureValueForName:@"o_z9"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
