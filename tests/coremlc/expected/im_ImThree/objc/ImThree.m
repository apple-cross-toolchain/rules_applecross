//
// ImThree.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "ImThree.h"

@implementation ImThreeInput

- (instancetype)initWithA:(CVPixelBufferRef)a b:(MLMultiArray *)b c:(CVPixelBufferRef)c {
    self = [super init];
    if (self) {
        _a = a;
        CVPixelBufferRetain(_a);
        _b = b;
        _c = c;
        CVPixelBufferRetain(_c);
    }
    return self;
}

- (void)dealloc {
    CVPixelBufferRelease(_a);
    CVPixelBufferRelease(_c);
}

- (nullable instancetype)initWithAFromCGImage:(CGImageRef)a b:(MLMultiArray *)b cCGImage:(CGImageRef)c error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __a = [MLFeatureValue featureValueWithCGImage:a pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
                if (__a == nil) {
                    result = NO;
                    break;
                }
                MLFeatureValue * __c = [MLFeatureValue featureValueWithCGImage:c pixelsWide:5 pixelsHigh:5 pixelFormatType:kCVPixelFormatType_OneComponent8 options:nil error:&localError];
                if (__c == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithA:(CVPixelBufferRef)__a.imageBufferValue b:b c:(CVPixelBufferRef)__c.imageBufferValue];
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

- (nullable instancetype)initWithAAtURL:(NSURL *)aURL b:(MLMultiArray *)b cAtURL:(NSURL *)cURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __a = [MLFeatureValue featureValueWithImageAtURL:aURL pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
                if (__a == nil) {
                    result = NO;
                    break;
                }
                MLFeatureValue * __c = [MLFeatureValue featureValueWithImageAtURL:cURL pixelsWide:5 pixelsHigh:5 pixelFormatType:kCVPixelFormatType_OneComponent8 options:nil error:&localError];
                if (__c == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithA:(CVPixelBufferRef)__a.imageBufferValue b:b c:(CVPixelBufferRef)__c.imageBufferValue];
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

-(BOOL)setAWithCGImage:(CGImageRef)a error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __a = [MLFeatureValue featureValueWithCGImage:a pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
        if (__a != nil) {
            CVPixelBufferRelease(self.a);
            self.a =  (CVPixelBufferRef)__a.imageBufferValue;
            CVPixelBufferRetain(self.a);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setCWithCGImage:(CGImageRef)c error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __c = [MLFeatureValue featureValueWithCGImage:c pixelsWide:5 pixelsHigh:5 pixelFormatType:kCVPixelFormatType_OneComponent8 options:nil error:&localError];
        if (__c != nil) {
            CVPixelBufferRelease(self.c);
            self.c =  (CVPixelBufferRef)__c.imageBufferValue;
            CVPixelBufferRetain(self.c);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setAWithURL:(NSURL *)aURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __a = [MLFeatureValue featureValueWithImageAtURL:aURL pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32BGRA options:nil error:&localError];
        if (__a != nil) {
            CVPixelBufferRelease(self.a);
            self.a =  (CVPixelBufferRef)__a.imageBufferValue;
            CVPixelBufferRetain(self.a);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

-(BOOL)setCWithURL:(NSURL *)cURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __c = [MLFeatureValue featureValueWithImageAtURL:cURL pixelsWide:5 pixelsHigh:5 pixelFormatType:kCVPixelFormatType_OneComponent8 options:nil error:&localError];
        if (__c != nil) {
            CVPixelBufferRelease(self.c);
            self.c =  (CVPixelBufferRef)__c.imageBufferValue;
            CVPixelBufferRetain(self.c);
            result = YES;
        }
    }
    if (error != NULL) {
        *error = localError;
    }
    return result;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"a", @"b", @"c"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"a"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.a];
    }
    if ([featureName isEqualToString:@"b"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.b];
    }
    if ([featureName isEqualToString:@"c"]) {
        return [MLFeatureValue featureValueWithPixelBuffer:self.c];
    }
    return nil;
}

@end

@implementation ImThreeOutput

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

@implementation ImThree


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"ImThree" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load ImThree.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize ImThree instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of ImThree.
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
    Initialize ImThree instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize ImThree instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize ImThree instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImThree.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize ImThree instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImThree.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct ImThree instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImThree instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImThree * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct ImThree instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImThree instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImThree * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            ImThree *typedModel = [[ImThree alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable ImThreeOutput *)predictionFromFeatures:(ImThreeInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable ImThreeOutput *)predictionFromFeatures:(ImThreeInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[ImThreeOutput alloc] initWithReshape_0:(MLMultiArray *)[outFeatures featureValueForName:@"reshape_0"].multiArrayValue];
}

- (void)predictionFromFeatures:(ImThreeInput *)input completionHandler:(void (^)(ImThreeOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ImThreeOutput *output = [[ImThreeOutput alloc] initWithReshape_0:(MLMultiArray *)[prediction featureValueForName:@"reshape_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(ImThreeInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(ImThreeOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ImThreeOutput *output = [[ImThreeOutput alloc] initWithReshape_0:(MLMultiArray *)[prediction featureValueForName:@"reshape_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable ImThreeOutput *)predictionFromA:(CVPixelBufferRef)a b:(MLMultiArray *)b c:(CVPixelBufferRef)c error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    ImThreeInput *input_ = [[ImThreeInput alloc] initWithA:a b:b c:c];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<ImThreeOutput *> *)predictionsFromInputs:(NSArray<ImThreeInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<ImThreeOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        ImThreeOutput * result = [[ImThreeOutput alloc] initWithReshape_0:(MLMultiArray *)[resultProvider featureValueForName:@"reshape_0"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
