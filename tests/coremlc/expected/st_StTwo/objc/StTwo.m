//
// StTwo.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "StTwo.h"

@implementation StTwoInput

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

@implementation StTwoOutput

- (instancetype)initWithAdd_1:(MLMultiArray *)add_1 {
    self = [super init];
    if (self) {
        _add_1 = add_1;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"add_1"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"add_1"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.add_1];
    }
    return nil;
}

@end

@implementation StTwoState

- (instancetype)initWithState:(MLState *)state {
    self = [super init];
    if (self) {
        _state = state;
    }
    return self;
}

- (void)getMultiArrayForState:(StTwoStateName)stateName
                      handler:(void (NS_NOESCAPE ^)(MLMultiArray *buffer))handler {
    NSString *stateNameString = nil;
    switch (stateName) {
        case StTwoStateNameK:
            stateNameString = @"k";
            break;
        case StTwoStateNameV:
            stateNameString = @"v";
            break;
    }

    [self.state getMultiArrayForStateNamed:stateNameString handler:handler];
}

@end

@implementation StTwo


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"StTwo" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load StTwo.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize StTwo instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of StTwo.
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
    Initialize StTwo instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize StTwo instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize StTwo instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for StTwo.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize StTwo instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for StTwo.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct StTwo instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid StTwo instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(StTwo * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct StTwo instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid StTwo instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(StTwo * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            StTwo *typedModel = [[StTwo alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}


/**
    Make a new state.
*/
- (StTwoState *)newState {
    MLState *state = [self.model newState];
    return [[StTwoState alloc] initWithState:state];
}

- (nullable StTwoOutput *)predictionFromFeatures:(StTwoInput *)input usingState:(StTwoState *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input usingState:state options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable StTwoOutput *)predictionFromFeatures:(StTwoInput *)input usingState:(StTwoState *)state options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input usingState:state.state options:options error:error];
    if (!outFeatures) { return nil; }
    return [[StTwoOutput alloc] initWithAdd_1:(MLMultiArray *)[outFeatures featureValueForName:@"add_1"].multiArrayValue];
}

- (void)predictionFromFeatures:(StTwoInput *)input usingState:(StTwoState *)state completionHandler:(void (^)(StTwoOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input usingState:state.state options:[[MLPredictionOptions alloc] init] completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            StTwoOutput *output = [[StTwoOutput alloc] initWithAdd_1:(MLMultiArray *)[prediction featureValueForName:@"add_1"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(StTwoInput *)input usingState:(StTwoState *)state options:(MLPredictionOptions *)options completionHandler:(void (^)(StTwoOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input usingState:state.state options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            StTwoOutput *output = [[StTwoOutput alloc] initWithAdd_1:(MLMultiArray *)[prediction featureValueForName:@"add_1"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable StTwoOutput *)predictionFromX:(MLMultiArray *)x usingState:(StTwoState *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    StTwoInput *input_ = [[StTwoInput alloc] initWithX:x];
    return [self predictionFromFeatures:input_ usingState:state error:error];
}

@end
