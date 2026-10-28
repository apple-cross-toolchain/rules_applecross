//
// MfImage.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "MfImage.h"

@implementation MfImageInput

- (instancetype)initWithImg:(CVPixelBufferRef)img {
    self = [super init];
    if (self) {
        _img = img;
        CVPixelBufferRetain(_img);
    }
    return self;
}

- (void)dealloc {
    CVPixelBufferRelease(_img);
}

- (nullable instancetype)initWithImgFromCGImage:(CGImageRef)img error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __img = [MLFeatureValue featureValueWithCGImage:img pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
                if (__img == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithImg:(CVPixelBufferRef)__img.imageBufferValue];
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

- (nullable instancetype)initWithImgAtURL:(NSURL *)imgURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __img = [MLFeatureValue featureValueWithImageAtURL:imgURL pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
                if (__img == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithImg:(CVPixelBufferRef)__img.imageBufferValue];
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

-(BOOL)setImgWithCGImage:(CGImageRef)img error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __img = [MLFeatureValue featureValueWithCGImage:img pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
        if (__img != nil) {
            CVPixelBufferRelease(self.img);
            self.img =  (CVPixelBufferRef)__img.imageBufferValue;
            CVPixelBufferRetain(self.img);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setImgWithURL:(NSURL *)imgURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __img = [MLFeatureValue featureValueWithImageAtURL:imgURL pixelsWide:8 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
        if (__img != nil) {
            CVPixelBufferRelease(self.img);
            self.img =  (CVPixelBufferRef)__img.imageBufferValue;
            CVPixelBufferRetain(self.img);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"img"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"img"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.img];
    }
    return nil;
}

@end

@implementation MfImageOutput

- (instancetype)initWithReduce_mean_0:(MLMultiArray *)reduce_mean_0 {
    self = [super init];
    if (self) {
        _reduce_mean_0 = reduce_mean_0;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"reduce_mean_0"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"reduce_mean_0"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.reduce_mean_0];
    }
    return nil;
}

@end

@implementation MfImage


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"MfImage" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load MfImage.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize MfImage instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of MfImage.
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
    Initialize MfImage instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize MfImage instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize MfImage instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for MfImage.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize MfImage instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for MfImage.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct MfImage instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid MfImage instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(MfImage * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct MfImage instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid MfImage instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(MfImage * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            MfImage *typedModel = [[MfImage alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable MfImageOutput *)predictionFromFeatures:(MfImageInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable MfImageOutput *)predictionFromFeatures:(MfImageInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[MfImageOutput alloc] initWithReduce_mean_0:(MLMultiArray *)[outFeatures featureValueForName:@"reduce_mean_0"].multiArrayValue];
}

- (void)predictionFromFeatures:(MfImageInput *)input completionHandler:(void (^)(MfImageOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            MfImageOutput *output = [[MfImageOutput alloc] initWithReduce_mean_0:(MLMultiArray *)[prediction featureValueForName:@"reduce_mean_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(MfImageInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(MfImageOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            MfImageOutput *output = [[MfImageOutput alloc] initWithReduce_mean_0:(MLMultiArray *)[prediction featureValueForName:@"reduce_mean_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable MfImageOutput *)predictionFromImg:(CVPixelBufferRef)img error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MfImageInput *input_ = [[MfImageInput alloc] initWithImg:img];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<MfImageOutput *> *)predictionsFromInputs:(NSArray<MfImageInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<MfImageOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        MfImageOutput * result = [[MfImageOutput alloc] initWithReduce_mean_0:(MLMultiArray *)[resultProvider featureValueForName:@"reduce_mean_0"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
