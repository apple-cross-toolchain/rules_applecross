//
// M001.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "M001.h"

@implementation M001Input

- (instancetype)initWithFeatures:(CVPixelBufferRef)features z9:(CVPixelBufferRef)z9 {
    self = [super init];
    if (self) {
        _features = features;
        CVPixelBufferRetain(_features);
        _z9 = z9;
        CVPixelBufferRetain(_z9);
    }
    return self;
}

- (void)dealloc {
    CVPixelBufferRelease(_features);
    CVPixelBufferRelease(_z9);
}

- (nullable instancetype)initWithFeaturesFromCGImage:(CGImageRef)features z9CGImage:(CGImageRef)z9 error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __features = [MLFeatureValue featureValueWithCGImage:features pixelsWide:10 pixelsHigh:6 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
                if (__features == nil) {
                    result = NO;
                    break;
                }
                MLFeatureValue * __z9 = [MLFeatureValue featureValueWithCGImage:z9 pixelsWide:6 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_OneComponent8 options:nil error:&localError];
                if (__z9 == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithFeatures:(CVPixelBufferRef)__features.imageBufferValue z9:(CVPixelBufferRef)__z9.imageBufferValue];
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

- (nullable instancetype)initWithFeaturesAtURL:(NSURL *)featuresURL z9AtURL:(NSURL *)z9URL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __features = [MLFeatureValue featureValueWithImageAtURL:featuresURL pixelsWide:10 pixelsHigh:6 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
                if (__features == nil) {
                    result = NO;
                    break;
                }
                MLFeatureValue * __z9 = [MLFeatureValue featureValueWithImageAtURL:z9URL pixelsWide:6 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_OneComponent8 options:nil error:&localError];
                if (__z9 == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithFeatures:(CVPixelBufferRef)__features.imageBufferValue z9:(CVPixelBufferRef)__z9.imageBufferValue];
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

-(BOOL)setFeaturesWithCGImage:(CGImageRef)features error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __features = [MLFeatureValue featureValueWithCGImage:features pixelsWide:10 pixelsHigh:6 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
        if (__features != nil) {
            CVPixelBufferRelease(self.features);
            self.features =  (CVPixelBufferRef)__features.imageBufferValue;
            CVPixelBufferRetain(self.features);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setZ9WithCGImage:(CGImageRef)z9 error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __z9 = [MLFeatureValue featureValueWithCGImage:z9 pixelsWide:6 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_OneComponent8 options:nil error:&localError];
        if (__z9 != nil) {
            CVPixelBufferRelease(self.z9);
            self.z9 =  (CVPixelBufferRef)__z9.imageBufferValue;
            CVPixelBufferRetain(self.z9);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setFeaturesWithURL:(NSURL *)featuresURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __features = [MLFeatureValue featureValueWithImageAtURL:featuresURL pixelsWide:10 pixelsHigh:6 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
        if (__features != nil) {
            CVPixelBufferRelease(self.features);
            self.features =  (CVPixelBufferRef)__features.imageBufferValue;
            CVPixelBufferRetain(self.features);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setZ9WithURL:(NSURL *)z9URL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __z9 = [MLFeatureValue featureValueWithImageAtURL:z9URL pixelsWide:6 pixelsHigh:8 pixelFormatType:kCVPixelFormatType_OneComponent8 options:nil error:&localError];
        if (__z9 != nil) {
            CVPixelBufferRelease(self.z9);
            self.z9 =  (CVPixelBufferRef)__z9.imageBufferValue;
            CVPixelBufferRetain(self.z9);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"features", @"z9"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"features"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.features];
    }
    if ([featureName isEqualToString:@"z9"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.z9];
    }
    return nil;
}

@end

@implementation M001Output

- (instancetype)initWithMean_features:(MLMultiArray *)mean_features mean_z9:(MLMultiArray *)mean_z9 s_image:(MLMultiArray *)s_image {
    self = [super init];
    if (self) {
        _mean_features = mean_features;
        _mean_z9 = mean_z9;
        _s_image = s_image;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"mean_features", @"mean_z9", @"s_image"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"mean_features"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.mean_features];
    }
    if ([featureName isEqualToString:@"mean_z9"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.mean_z9];
    }
    if ([featureName isEqualToString:@"s_image"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.s_image];
    }
    return nil;
}

@end

@implementation M001State

- (instancetype)initWithState:(MLState *)state {
    self = [super init];
    if (self) {
        _state = state;
    }
    return self;
}

- (void)getMultiArrayForState:(M001StateName)stateName
                      handler:(void (NS_NOESCAPE ^)(MLMultiArray *buffer))handler {
    NSString *stateNameString = nil;
    switch (stateName) {
        case M001StateNameImage:
            stateNameString = @"image";
            break;
    }

    [self.state getMultiArrayForStateNamed:stateNameString handler:handler];
}

@end

@implementation M001


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"M001" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load M001.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize M001 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of M001.
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
    Initialize M001 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize M001 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize M001 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for M001.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize M001 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for M001.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct M001 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid M001 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(M001 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct M001 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid M001 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(M001 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            M001 *typedModel = [[M001 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}


/**
    Make a new state.
*/
- (M001State *)newState {
    MLState *state = [self.model newState];
    return [[M001State alloc] initWithState:state];
}

- (nullable M001Output *)predictionFromFeatures:(M001Input *)input usingState:(M001State *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input usingState:state options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable M001Output *)predictionFromFeatures:(M001Input *)input usingState:(M001State *)state options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input usingState:state.state options:options error:error];
    if (!outFeatures) { return nil; }
    return [[M001Output alloc] initWithMean_features:(MLMultiArray *)[outFeatures featureValueForName:@"mean_features"].multiArrayValue mean_z9:(MLMultiArray *)[outFeatures featureValueForName:@"mean_z9"].multiArrayValue s_image:(MLMultiArray *)[outFeatures featureValueForName:@"s_image"].multiArrayValue];
}

- (void)predictionFromFeatures:(M001Input *)input usingState:(M001State *)state completionHandler:(void (^)(M001Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input usingState:state.state options:[[MLPredictionOptions alloc] init] completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            M001Output *output = [[M001Output alloc] initWithMean_features:(MLMultiArray *)[prediction featureValueForName:@"mean_features"].multiArrayValue mean_z9:(MLMultiArray *)[prediction featureValueForName:@"mean_z9"].multiArrayValue s_image:(MLMultiArray *)[prediction featureValueForName:@"s_image"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(M001Input *)input usingState:(M001State *)state options:(MLPredictionOptions *)options completionHandler:(void (^)(M001Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input usingState:state.state options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            M001Output *output = [[M001Output alloc] initWithMean_features:(MLMultiArray *)[prediction featureValueForName:@"mean_features"].multiArrayValue mean_z9:(MLMultiArray *)[prediction featureValueForName:@"mean_z9"].multiArrayValue s_image:(MLMultiArray *)[prediction featureValueForName:@"s_image"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable M001Output *)predictionFromFeatures_:(CVPixelBufferRef)features z9:(CVPixelBufferRef)z9 usingState:(M001State *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    M001Input *input_ = [[M001Input alloc] initWithFeatures:features z9:z9];
    return [self predictionFromFeatures:input_ usingState:state error:error];
}

@end
