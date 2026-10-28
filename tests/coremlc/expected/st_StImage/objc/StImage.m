//
// StImage.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "StImage.h"

@implementation StImageInput

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
                MLFeatureValue * __img = [MLFeatureValue featureValueWithCGImage:img pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
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
                MLFeatureValue * __img = [MLFeatureValue featureValueWithImageAtURL:imgURL pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
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
        MLFeatureValue * __img = [MLFeatureValue featureValueWithCGImage:img pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
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
        MLFeatureValue * __img = [MLFeatureValue featureValueWithImageAtURL:imgURL pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
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

@implementation StImageOutput

- (instancetype)initWithAdd_0:(MLMultiArray *)add_0 {
    self = [super init];
    if (self) {
        _add_0 = add_0;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"add_0"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"add_0"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.add_0];
    }
    return nil;
}

@end

@implementation StImageState

- (instancetype)initWithState:(MLState *)state {
    self = [super init];
    if (self) {
        _state = state;
    }
    return self;
}

- (void)getMultiArrayForState:(StImageStateName)stateName
                      handler:(void (NS_NOESCAPE ^)(MLMultiArray *buffer))handler {
    NSString *stateNameString = nil;
    switch (stateName) {
        case StImageStateNameS:
            stateNameString = @"s";
            break;
    }

    [self.state getMultiArrayForStateNamed:stateNameString handler:handler];
}

@end

@implementation StImage


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"StImage" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load StImage.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize StImage instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of StImage.
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
    Initialize StImage instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize StImage instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize StImage instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for StImage.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize StImage instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for StImage.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct StImage instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid StImage instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(StImage * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct StImage instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid StImage instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(StImage * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            StImage *typedModel = [[StImage alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}


/**
    Make a new state.
*/
- (StImageState *)newState {
    MLState *state = [self.model newState];
    return [[StImageState alloc] initWithState:state];
}

- (nullable StImageOutput *)predictionFromFeatures:(StImageInput *)input usingState:(StImageState *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input usingState:state options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable StImageOutput *)predictionFromFeatures:(StImageInput *)input usingState:(StImageState *)state options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input usingState:state.state options:options error:error];
    if (!outFeatures) { return nil; }
    return [[StImageOutput alloc] initWithAdd_0:(MLMultiArray *)[outFeatures featureValueForName:@"add_0"].multiArrayValue];
}

- (void)predictionFromFeatures:(StImageInput *)input usingState:(StImageState *)state completionHandler:(void (^)(StImageOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input usingState:state.state options:[[MLPredictionOptions alloc] init] completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            StImageOutput *output = [[StImageOutput alloc] initWithAdd_0:(MLMultiArray *)[prediction featureValueForName:@"add_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(StImageInput *)input usingState:(StImageState *)state options:(MLPredictionOptions *)options completionHandler:(void (^)(StImageOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input usingState:state.state options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            StImageOutput *output = [[StImageOutput alloc] initWithAdd_0:(MLMultiArray *)[prediction featureValueForName:@"add_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable StImageOutput *)predictionFromImg:(CVPixelBufferRef)img usingState:(StImageState *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    StImageInput *input_ = [[StImageInput alloc] initWithImg:img];
    return [self predictionFromFeatures:input_ usingState:state error:error];
}

@end
