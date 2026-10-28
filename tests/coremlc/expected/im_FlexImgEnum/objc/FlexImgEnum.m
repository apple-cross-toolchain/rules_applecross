//
// FlexImgEnum.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "FlexImgEnum.h"

@implementation FlexImgEnumInput

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

@implementation FlexImgEnumOutput

- (instancetype)initWithReduce_mean_1:(MLMultiArray *)reduce_mean_1 {
    self = [super init];
    if (self) {
        _reduce_mean_1 = reduce_mean_1;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"reduce_mean_1"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"reduce_mean_1"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.reduce_mean_1];
    }
    return nil;
}

@end

@implementation FlexImgEnum


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"FlexImgEnum" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load FlexImgEnum.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize FlexImgEnum instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of FlexImgEnum.
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
    Initialize FlexImgEnum instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize FlexImgEnum instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize FlexImgEnum instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for FlexImgEnum.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize FlexImgEnum instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for FlexImgEnum.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct FlexImgEnum instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid FlexImgEnum instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(FlexImgEnum * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct FlexImgEnum instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid FlexImgEnum instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(FlexImgEnum * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            FlexImgEnum *typedModel = [[FlexImgEnum alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable FlexImgEnumOutput *)predictionFromFeatures:(FlexImgEnumInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable FlexImgEnumOutput *)predictionFromFeatures:(FlexImgEnumInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[FlexImgEnumOutput alloc] initWithReduce_mean_1:(MLMultiArray *)[outFeatures featureValueForName:@"reduce_mean_1"].multiArrayValue];
}

- (void)predictionFromFeatures:(FlexImgEnumInput *)input completionHandler:(void (^)(FlexImgEnumOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            FlexImgEnumOutput *output = [[FlexImgEnumOutput alloc] initWithReduce_mean_1:(MLMultiArray *)[prediction featureValueForName:@"reduce_mean_1"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(FlexImgEnumInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(FlexImgEnumOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            FlexImgEnumOutput *output = [[FlexImgEnumOutput alloc] initWithReduce_mean_1:(MLMultiArray *)[prediction featureValueForName:@"reduce_mean_1"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable FlexImgEnumOutput *)predictionFromImg:(CVPixelBufferRef)img error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    FlexImgEnumInput *input_ = [[FlexImgEnumInput alloc] initWithImg:img];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<FlexImgEnumOutput *> *)predictionsFromInputs:(NSArray<FlexImgEnumInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<FlexImgEnumOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        FlexImgEnumOutput * result = [[FlexImgEnumOutput alloc] initWithReduce_mean_1:(MLMultiArray *)[resultProvider featureValueForName:@"reduce_mean_1"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
