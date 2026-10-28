//
// Cimg1.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "Cimg1.h"

@implementation Cimg1Input

- (instancetype)initWithImg:(CVPixelBufferRef)img s:(NSString *)s {
    self = [super init];
    if (self) {
        _img = img;
        CVPixelBufferRetain(_img);
        _s = s;
    }
    return self;
}

- (void)dealloc {
    CVPixelBufferRelease(_img);
}

- (nullable instancetype)initWithImgFromCGImage:(CGImageRef)img s:(NSString *)s error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __img = [MLFeatureValue featureValueWithCGImage:img pixelsWide:4 pixelsHigh:3 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
                if (__img == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithImg:(CVPixelBufferRef)__img.imageBufferValue s:s];
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

- (nullable instancetype)initWithImgAtURL:(NSURL *)imgURL s:(NSString *)s error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __img = [MLFeatureValue featureValueWithImageAtURL:imgURL pixelsWide:4 pixelsHigh:3 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
                if (__img == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithImg:(CVPixelBufferRef)__img.imageBufferValue s:s];
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
        MLFeatureValue * __img = [MLFeatureValue featureValueWithCGImage:img pixelsWide:4 pixelsHigh:3 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
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
        MLFeatureValue * __img = [MLFeatureValue featureValueWithImageAtURL:imgURL pixelsWide:4 pixelsHigh:3 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
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
    return [NSSet setWithArray:@[@"img", @"s"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"img"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.img];
    }
    if ([featureName isEqualToString:@"s"]) {
        return [MLFeatureValue featureValueWithString:self.s];
    }
    return nil;
}

@end

@implementation Cimg1Output

- (instancetype)initWithY:(double)y {
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
        return [MLFeatureValue featureValueWithDouble:self.y];
    }
    return nil;
}

@end

@implementation Cimg1


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"Cimg1" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load Cimg1.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize Cimg1 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of Cimg1.
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
    Initialize Cimg1 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize Cimg1 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize Cimg1 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Cimg1.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize Cimg1 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Cimg1.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct Cimg1 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Cimg1 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Cimg1 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct Cimg1 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Cimg1 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Cimg1 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            Cimg1 *typedModel = [[Cimg1 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable Cimg1Output *)predictionFromFeatures:(Cimg1Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable Cimg1Output *)predictionFromFeatures:(Cimg1Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[Cimg1Output alloc] initWithY:[outFeatures featureValueForName:@"y"].doubleValue];
}

- (void)predictionFromFeatures:(Cimg1Input *)input completionHandler:(void (^)(Cimg1Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            Cimg1Output *output = [[Cimg1Output alloc] initWithY:[prediction featureValueForName:@"y"].doubleValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(Cimg1Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(Cimg1Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            Cimg1Output *output = [[Cimg1Output alloc] initWithY:[prediction featureValueForName:@"y"].doubleValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable Cimg1Output *)predictionFromImg:(CVPixelBufferRef)img s:(NSString *)s error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    Cimg1Input *input_ = [[Cimg1Input alloc] initWithImg:img s:s];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<Cimg1Output *> *)predictionsFromInputs:(NSArray<Cimg1Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<Cimg1Output*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        Cimg1Output * result = [[Cimg1Output alloc] initWithY:[resultProvider featureValueForName:@"y"].doubleValue];
        [results addObject:result];
    }
    return results;
}

@end
