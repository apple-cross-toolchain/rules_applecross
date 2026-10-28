//
// StRank5.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "StRank5.h"

@implementation StRank5Input

- (instancetype)initWithX:(MLMultiArray *)x {
    self = [super init];
    if (self) {
        _x = x;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"x"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"x"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.x];
    }
    return nil;
}

@end

@implementation StRank5Output

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

@implementation StRank5State

- (instancetype)initWithState:(MLState *)state {
    self = [super init];
    if (self) {
        _state = state;
    }
    return self;
}

- (void)getMultiArrayForState:(StRank5StateName)stateName
                      handler:(void (NS_NOESCAPE ^)(MLMultiArray *buffer))handler {
    NSString *stateNameString = nil;
    switch (stateName) {
        case StRank5StateNameS:
            stateNameString = @"s";
            break;
    }

    [self.state getMultiArrayForStateNamed:stateNameString handler:handler];
}

@end

@implementation StRank5


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"StRank5" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load StRank5.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize StRank5 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of StRank5.
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
    Initialize StRank5 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize StRank5 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize StRank5 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for StRank5.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize StRank5 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for StRank5.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct StRank5 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid StRank5 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(StRank5 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct StRank5 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid StRank5 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(StRank5 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            StRank5 *typedModel = [[StRank5 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}


/**
    Make a new state.
*/
- (StRank5State *)newState {
    MLState *state = [self.model newState];
    return [[StRank5State alloc] initWithState:state];
}

- (nullable StRank5Output *)predictionFromFeatures:(StRank5Input *)input usingState:(StRank5State *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input usingState:state options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable StRank5Output *)predictionFromFeatures:(StRank5Input *)input usingState:(StRank5State *)state options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input usingState:state.state options:options error:error];
    if (!outFeatures) { return nil; }
    return [[StRank5Output alloc] initWithAdd_0:(MLMultiArray *)[outFeatures featureValueForName:@"add_0"].multiArrayValue];
}

- (void)predictionFromFeatures:(StRank5Input *)input usingState:(StRank5State *)state completionHandler:(void (^)(StRank5Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input usingState:state.state options:[[MLPredictionOptions alloc] init] completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            StRank5Output *output = [[StRank5Output alloc] initWithAdd_0:(MLMultiArray *)[prediction featureValueForName:@"add_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(StRank5Input *)input usingState:(StRank5State *)state options:(MLPredictionOptions *)options completionHandler:(void (^)(StRank5Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input usingState:state.state options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            StRank5Output *output = [[StRank5Output alloc] initWithAdd_0:(MLMultiArray *)[prediction featureValueForName:@"add_0"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable StRank5Output *)predictionFromX:(MLMultiArray *)x usingState:(StRank5State *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    StRank5Input *input_ = [[StRank5Input alloc] initWithX:x];
    return [self predictionFromFeatures:input_ usingState:state error:error];
}

@end
