//
// ImTwo.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "ImTwo.h"

@implementation ImTwoInput

- (instancetype)initWithLeft:(CVPixelBufferRef)left_ right_:(CVPixelBufferRef)right_ {
    self = [super init];
    if (self) {
        _left_ = left_;
        CVPixelBufferRetain(_left_);
        _right_ = right_;
        CVPixelBufferRetain(_right_);
    }
    return self;
}

- (void)dealloc {
    CVPixelBufferRelease(_left_);
    CVPixelBufferRelease(_right_);
}

- (nullable instancetype)initWithLeftFromCGImage:(CGImageRef)left_ right_CGImage:(CGImageRef)right_ error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __left_ = [MLFeatureValue featureValueWithCGImage:left_ pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
                if (__left_ == nil) {
                    result = NO;
                    break;
                }
                MLFeatureValue * __right_ = [MLFeatureValue featureValueWithCGImage:right_ pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
                if (__right_ == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithLeft:(CVPixelBufferRef)__left_.imageBufferValue right_:(CVPixelBufferRef)__right_.imageBufferValue];
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

- (nullable instancetype)initWithLeftAtURL:(NSURL *)left_URL right_AtURL:(NSURL *)right_URL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __left_ = [MLFeatureValue featureValueWithImageAtURL:left_URL pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
                if (__left_ == nil) {
                    result = NO;
                    break;
                }
                MLFeatureValue * __right_ = [MLFeatureValue featureValueWithImageAtURL:right_URL pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
                if (__right_ == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithLeft:(CVPixelBufferRef)__left_.imageBufferValue right_:(CVPixelBufferRef)__right_.imageBufferValue];
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

-(BOOL)setLeft_WithCGImage:(CGImageRef)left_ error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __left_ = [MLFeatureValue featureValueWithCGImage:left_ pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
        if (__left_ != nil) {
            CVPixelBufferRelease(self.left_);
            self.left_ =  (CVPixelBufferRef)__left_.imageBufferValue;
            CVPixelBufferRetain(self.left_);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setRight_WithCGImage:(CGImageRef)right_ error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __right_ = [MLFeatureValue featureValueWithCGImage:right_ pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
        if (__right_ != nil) {
            CVPixelBufferRelease(self.right_);
            self.right_ =  (CVPixelBufferRef)__right_.imageBufferValue;
            CVPixelBufferRetain(self.right_);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setLeft_WithURL:(NSURL *)left_URL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __left_ = [MLFeatureValue featureValueWithImageAtURL:left_URL pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
        if (__left_ != nil) {
            CVPixelBufferRelease(self.left_);
            self.left_ =  (CVPixelBufferRef)__left_.imageBufferValue;
            CVPixelBufferRetain(self.left_);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setRight_WithURL:(NSURL *)right_URL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __right_ = [MLFeatureValue featureValueWithImageAtURL:right_URL pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
        if (__right_ != nil) {
            CVPixelBufferRelease(self.right_);
            self.right_ =  (CVPixelBufferRef)__right_.imageBufferValue;
            CVPixelBufferRetain(self.right_);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"left", @"right"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"left"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.left_];
    }
    if ([featureName isEqualToString:@"right"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.right_];
    }
    return nil;
}

@end

@implementation ImTwoOutput

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

@implementation ImTwo


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"ImTwo" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load ImTwo.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize ImTwo instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of ImTwo.
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
    Initialize ImTwo instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize ImTwo instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize ImTwo instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImTwo.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize ImTwo instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImTwo.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct ImTwo instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImTwo instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImTwo * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct ImTwo instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImTwo instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImTwo * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            ImTwo *typedModel = [[ImTwo alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable ImTwoOutput *)predictionFromFeatures:(ImTwoInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable ImTwoOutput *)predictionFromFeatures:(ImTwoInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[ImTwoOutput alloc] initWithReshape_0:(MLMultiArray *)[outFeatures featureValueForName:@"reshape_0"].multiArrayValue];
}

- (void)predictionFromFeatures:(ImTwoInput *)input completionHandler:(void (^)(ImTwoOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ImTwoOutput *output = [[ImTwoOutput alloc] initWithReshape_0:(MLMultiArray *)[prediction featureValueForName:@"reshape_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(ImTwoInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(ImTwoOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ImTwoOutput *output = [[ImTwoOutput alloc] initWithReshape_0:(MLMultiArray *)[prediction featureValueForName:@"reshape_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable ImTwoOutput *)predictionFromLeft:(CVPixelBufferRef)left_ right_:(CVPixelBufferRef)right_ error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    ImTwoInput *input_ = [[ImTwoInput alloc] initWithLeft:left_ right_:right_];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<ImTwoOutput *> *)predictionsFromInputs:(NSArray<ImTwoInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<ImTwoOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        ImTwoOutput * result = [[ImTwoOutput alloc] initWithReshape_0:(MLMultiArray *)[resultProvider featureValueForName:@"reshape_0"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
