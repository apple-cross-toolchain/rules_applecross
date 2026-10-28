//
// ImNonSqPlus.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "ImNonSqPlus.h"

@implementation ImNonSqPlusInput

- (instancetype)initWithPhoto:(CVPixelBufferRef)photo k:(MLMultiArray *)k {
    self = [super init];
    if (self) {
        _photo = photo;
        CVPixelBufferRetain(_photo);
        _k = k;
    }
    return self;
}

- (void)dealloc {
    CVPixelBufferRelease(_photo);
}

- (nullable instancetype)initWithPhotoFromCGImage:(CGImageRef)photo k:(MLMultiArray *)k error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __photo = [MLFeatureValue featureValueWithCGImage:photo pixelsWide:10 pixelsHigh:6 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
                if (__photo == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithPhoto:(CVPixelBufferRef)__photo.imageBufferValue k:k];
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

- (nullable instancetype)initWithPhotoAtURL:(NSURL *)photoURL k:(MLMultiArray *)k error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __photo = [MLFeatureValue featureValueWithImageAtURL:photoURL pixelsWide:10 pixelsHigh:6 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
                if (__photo == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithPhoto:(CVPixelBufferRef)__photo.imageBufferValue k:k];
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

-(BOOL)setPhotoWithCGImage:(CGImageRef)photo error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __photo = [MLFeatureValue featureValueWithCGImage:photo pixelsWide:10 pixelsHigh:6 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
        if (__photo != nil) {
            CVPixelBufferRelease(self.photo);
            self.photo =  (CVPixelBufferRef)__photo.imageBufferValue;
            CVPixelBufferRetain(self.photo);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setPhotoWithURL:(NSURL *)photoURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __photo = [MLFeatureValue featureValueWithImageAtURL:photoURL pixelsWide:10 pixelsHigh:6 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
        if (__photo != nil) {
            CVPixelBufferRelease(self.photo);
            self.photo =  (CVPixelBufferRef)__photo.imageBufferValue;
            CVPixelBufferRetain(self.photo);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"photo", @"k"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"photo"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.photo];
    }
    if ([featureName isEqualToString:@"k"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.k];
    }
    return nil;
}

@end

@implementation ImNonSqPlusOutput

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

@implementation ImNonSqPlus


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"ImNonSqPlus" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load ImNonSqPlus.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize ImNonSqPlus instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of ImNonSqPlus.
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
    Initialize ImNonSqPlus instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize ImNonSqPlus instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize ImNonSqPlus instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImNonSqPlus.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize ImNonSqPlus instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImNonSqPlus.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct ImNonSqPlus instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImNonSqPlus instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImNonSqPlus * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct ImNonSqPlus instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImNonSqPlus instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImNonSqPlus * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            ImNonSqPlus *typedModel = [[ImNonSqPlus alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable ImNonSqPlusOutput *)predictionFromFeatures:(ImNonSqPlusInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable ImNonSqPlusOutput *)predictionFromFeatures:(ImNonSqPlusInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[ImNonSqPlusOutput alloc] initWithReshape_0:(MLMultiArray *)[outFeatures featureValueForName:@"reshape_0"].multiArrayValue];
}

- (void)predictionFromFeatures:(ImNonSqPlusInput *)input completionHandler:(void (^)(ImNonSqPlusOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ImNonSqPlusOutput *output = [[ImNonSqPlusOutput alloc] initWithReshape_0:(MLMultiArray *)[prediction featureValueForName:@"reshape_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(ImNonSqPlusInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(ImNonSqPlusOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ImNonSqPlusOutput *output = [[ImNonSqPlusOutput alloc] initWithReshape_0:(MLMultiArray *)[prediction featureValueForName:@"reshape_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable ImNonSqPlusOutput *)predictionFromPhoto:(CVPixelBufferRef)photo k:(MLMultiArray *)k error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    ImNonSqPlusInput *input_ = [[ImNonSqPlusInput alloc] initWithPhoto:photo k:k];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<ImNonSqPlusOutput *> *)predictionsFromInputs:(NSArray<ImNonSqPlusInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<ImNonSqPlusOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        ImNonSqPlusOutput * result = [[ImNonSqPlusOutput alloc] initWithReshape_0:(MLMultiArray *)[resultProvider featureValueForName:@"reshape_0"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
