//
// ThreeEnum18.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "ThreeEnum18.h"

@implementation ThreeEnum18Input

- (instancetype)initWithA:(MLMultiArray *)a b:(MLMultiArray *)b c:(CVPixelBufferRef)c {
    self = [super init];
    if (self) {
        _a = a;
        _b = b;
        _c = c;
        CVPixelBufferRetain(_c);
    }
    return self;
}

- (void)dealloc {
    CVPixelBufferRelease(_c);
}

- (nullable instancetype)initWithA:(MLMultiArray *)a b:(MLMultiArray *)b cCGImage:(CGImageRef)c error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __c = [MLFeatureValue featureValueWithCGImage:c pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
                if (__c == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithA:a b:b c:(CVPixelBufferRef)__c.imageBufferValue];
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

- (nullable instancetype)initWithA:(MLMultiArray *)a b:(MLMultiArray *)b cAtURL:(NSURL *)cURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    if (self) {
        NSError *localError;
        BOOL result = YES;
        id retVal = nil;
        @autoreleasepool {
            do {
                MLFeatureValue * __c = [MLFeatureValue featureValueWithImageAtURL:cURL pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
                if (__c == nil) {
                    result = NO;
                    break;
                }
                retVal = [self initWithA:a b:b c:(CVPixelBufferRef)__c.imageBufferValue];
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

-(BOOL)setCWithCGImage:(CGImageRef)c error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __c = [MLFeatureValue featureValueWithCGImage:c pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
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

-(BOOL)setCWithURL:(NSURL *)cURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NSError *localError;
    BOOL result = NO;
    @autoreleasepool {
        MLFeatureValue * __c = [MLFeatureValue featureValueWithImageAtURL:cURL pixelsWide:4 pixelsHigh:4 pixelFormatType:kCVPixelFormatType_32ARGB options:nil error:&localError];
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
        return [MLFeatureValue featureValueWithMultiArray:self.a];
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

@implementation ThreeEnum18Output

- (instancetype)initWithRelu_0:(MLMultiArray *)relu_0 relu_1:(MLMultiArray *)relu_1 reduce_mean_0:(MLMultiArray *)reduce_mean_0 {
    self = [super init];
    if (self) {
        _relu_0 = relu_0;
        _relu_1 = relu_1;
        _reduce_mean_0 = reduce_mean_0;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"relu_0", @"relu_1", @"reduce_mean_0"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"relu_0"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.relu_0];
    }
    if ([featureName isEqualToString:@"relu_1"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.relu_1];
    }
    if ([featureName isEqualToString:@"reduce_mean_0"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.reduce_mean_0];
    }
    return nil;
}

@end

@implementation ThreeEnum18


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"ThreeEnum18" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load ThreeEnum18.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize ThreeEnum18 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of ThreeEnum18.
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
    Initialize ThreeEnum18 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize ThreeEnum18 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize ThreeEnum18 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ThreeEnum18.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize ThreeEnum18 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ThreeEnum18.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct ThreeEnum18 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ThreeEnum18 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ThreeEnum18 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct ThreeEnum18 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ThreeEnum18 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ThreeEnum18 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            ThreeEnum18 *typedModel = [[ThreeEnum18 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable ThreeEnum18Output *)predictionFromFeatures:(ThreeEnum18Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable ThreeEnum18Output *)predictionFromFeatures:(ThreeEnum18Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[ThreeEnum18Output alloc] initWithRelu_0:(MLMultiArray *)[outFeatures featureValueForName:@"relu_0"].multiArrayValue relu_1:(MLMultiArray *)[outFeatures featureValueForName:@"relu_1"].multiArrayValue reduce_mean_0:(MLMultiArray *)[outFeatures featureValueForName:@"reduce_mean_0"].multiArrayValue];
}

- (void)predictionFromFeatures:(ThreeEnum18Input *)input completionHandler:(void (^)(ThreeEnum18Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ThreeEnum18Output *output = [[ThreeEnum18Output alloc] initWithRelu_0:(MLMultiArray *)[prediction featureValueForName:@"relu_0"].multiArrayValue relu_1:(MLMultiArray *)[prediction featureValueForName:@"relu_1"].multiArrayValue reduce_mean_0:(MLMultiArray *)[prediction featureValueForName:@"reduce_mean_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(ThreeEnum18Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(ThreeEnum18Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            ThreeEnum18Output *output = [[ThreeEnum18Output alloc] initWithRelu_0:(MLMultiArray *)[prediction featureValueForName:@"relu_0"].multiArrayValue relu_1:(MLMultiArray *)[prediction featureValueForName:@"relu_1"].multiArrayValue reduce_mean_0:(MLMultiArray *)[prediction featureValueForName:@"reduce_mean_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable ThreeEnum18Output *)predictionFromA:(MLMultiArray *)a b:(MLMultiArray *)b c:(CVPixelBufferRef)c error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    ThreeEnum18Input *input_ = [[ThreeEnum18Input alloc] initWithA:a b:b c:c];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<ThreeEnum18Output *> *)predictionsFromInputs:(NSArray<ThreeEnum18Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<ThreeEnum18Output*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        ThreeEnum18Output * result = [[ThreeEnum18Output alloc] initWithRelu_0:(MLMultiArray *)[resultProvider featureValueForName:@"relu_0"].multiArrayValue relu_1:(MLMultiArray *)[resultProvider featureValueForName:@"relu_1"].multiArrayValue reduce_mean_0:(MLMultiArray *)[resultProvider featureValueForName:@"reduce_mean_0"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
