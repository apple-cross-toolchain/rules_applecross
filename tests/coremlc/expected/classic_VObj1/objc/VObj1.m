//
// VObj1.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "VObj1.h"

@implementation VObj1Input

- (instancetype)initWithImage:(CVPixelBufferRef)image {
    self = [super init];
    if (self) {
        _image = image;
        CVPixelBufferRetain(_image);
    }
    return self;
}

- (void)dealloc {
    CVPixelBufferRelease(_image);
}

- (nullable instancetype)initWithImageFromCGImage:(CGImageRef)image error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __image = [MLFeatureValue featureValueWithCGImage:image pixelsWide:299 pixelsHigh:299 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
                if (__image == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithImage:(CVPixelBufferRef)__image.imageBufferValue];
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

- (nullable instancetype)initWithImageAtURL:(NSURL *)imageURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __image = [MLFeatureValue featureValueWithImageAtURL:imageURL pixelsWide:299 pixelsHigh:299 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
                if (__image == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithImage:(CVPixelBufferRef)__image.imageBufferValue];
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

-(BOOL)setImageWithCGImage:(CGImageRef)image error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __image = [MLFeatureValue featureValueWithCGImage:image pixelsWide:299 pixelsHigh:299 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
        if (__image != nil) {
            CVPixelBufferRelease(self.image);
            self.image =  (CVPixelBufferRef)__image.imageBufferValue;
            CVPixelBufferRetain(self.image);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setImageWithURL:(NSURL *)imageURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __image = [MLFeatureValue featureValueWithImageAtURL:imageURL pixelsWide:299 pixelsHigh:299 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
        if (__image != nil) {
            CVPixelBufferRelease(self.image);
            self.image =  (CVPixelBufferRef)__image.imageBufferValue;
            CVPixelBufferRetain(self.image);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"image"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"image"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.image];
    }
    return nil;
}

@end

@implementation VObj1Output

- (instancetype)initWithHi:(MLMultiArray *)hi lo:(MLMultiArray *)lo {
    self = [super init];
    if (self) {
        _hi = hi;
        _lo = lo;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"hi", @"lo"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"hi"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.hi];
    }
    if ([featureName isEqualToString:@"lo"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.lo];
    }
    return nil;
}

@end

@implementation VObj1


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"VObj1" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load VObj1.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize VObj1 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of VObj1.
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
    Initialize VObj1 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize VObj1 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize VObj1 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for VObj1.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize VObj1 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for VObj1.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct VObj1 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid VObj1 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(VObj1 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct VObj1 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid VObj1 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(VObj1 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            VObj1 *typedModel = [[VObj1 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable VObj1Output *)predictionFromFeatures:(VObj1Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable VObj1Output *)predictionFromFeatures:(VObj1Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[VObj1Output alloc] initWithHi:(MLMultiArray *)[outFeatures featureValueForName:@"hi"].multiArrayValue lo:(MLMultiArray *)[outFeatures featureValueForName:@"lo"].multiArrayValue];
}

- (void)predictionFromFeatures:(VObj1Input *)input completionHandler:(void (^)(VObj1Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            VObj1Output *output = [[VObj1Output alloc] initWithHi:(MLMultiArray *)[prediction featureValueForName:@"hi"].multiArrayValue lo:(MLMultiArray *)[prediction featureValueForName:@"lo"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(VObj1Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(VObj1Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            VObj1Output *output = [[VObj1Output alloc] initWithHi:(MLMultiArray *)[prediction featureValueForName:@"hi"].multiArrayValue lo:(MLMultiArray *)[prediction featureValueForName:@"lo"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable VObj1Output *)predictionFromImage:(CVPixelBufferRef)image error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    VObj1Input *input_ = [[VObj1Input alloc] initWithImage:image];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<VObj1Output *> *)predictionsFromInputs:(NSArray<VObj1Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<VObj1Output*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        VObj1Output * result = [[VObj1Output alloc] initWithHi:(MLMultiArray *)[resultProvider featureValueForName:@"hi"].multiArrayValue lo:(MLMultiArray *)[resultProvider featureValueForName:@"lo"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
