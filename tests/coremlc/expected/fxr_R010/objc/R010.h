//
// R010.h
//
// This file was automatically generated and should not be edited.
//

#import <Foundation/Foundation.h>
#import <CoreML/CoreML.h>
#include <stdint.h>
#include <os/log.h>

NS_ASSUME_NONNULL_BEGIN

/// Model Prediction Input Type
API_AVAILABLE(macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)) __attribute__((visibility("hidden")))
@interface R010Input : NSObject<MLFeatureProvider>

/// in0 as 1 × 4 × 53 × 5 4-dimensional array of floats
@property (readwrite, nonatomic, strong) MLMultiArray * in0;

/// in1 as 6 × 5 × 1 × 4 × 3 5-dimensional array of 16-bit floats
@property (readwrite, nonatomic, strong) MLMultiArray * in1;

/// in2 as 4 element vector of 32-bit integers
@property (readwrite, nonatomic, strong) MLMultiArray * in2;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithIn0:(MLMultiArray *)in0 in1:(MLMultiArray *)in1 in2:(MLMultiArray *)in2 NS_DESIGNATED_INITIALIZER;

@end

/// Model Prediction Output Type
API_AVAILABLE(macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)) __attribute__((visibility("hidden")))
@interface R010Output : NSObject<MLFeatureProvider>

/// relu_0 as multidimensional array of floats
@property (readwrite, nonatomic, strong) MLMultiArray * relu_0;

/// relu_1 as multidimensional array of 16-bit floats
@property (readwrite, nonatomic, strong) MLMultiArray * relu_1;

/// add_0 as multidimensional array of 32-bit integers
@property (readwrite, nonatomic, strong) MLMultiArray * add_0;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithRelu_0:(MLMultiArray *)relu_0 relu_1:(MLMultiArray *)relu_1 add_0:(MLMultiArray *)add_0 NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)) __attribute__((visibility("hidden")))
@interface R010 : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize R010 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of R010.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize R010 instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize R010 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize R010 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for R010.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize R010 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for R010.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct R010 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid R010 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(R010 * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct R010 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid R010 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(R010 * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of R010Input to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as R010Output
*/
- (nullable R010Output *)predictionFromFeatures:(R010Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of R010Input to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as R010Output
*/
- (nullable R010Output *)predictionFromFeatures:(R010Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of R010Input to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(R010Input *)input completionHandler:(void (^)(R010Output * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of R010Input to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(R010Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(R010Output * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make a prediction using the convenience interface
    @param in0 1 × 4 × 53 × 5 4-dimensional array of floats
    @param in1 6 × 5 × 1 × 4 × 3 5-dimensional array of 16-bit floats
    @param in2 4 element vector of 32-bit integers
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as R010Output
*/
- (nullable R010Output *)predictionFromIn0:(MLMultiArray *)in0 in1:(MLMultiArray *)in1 in2:(MLMultiArray *)in2 error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of R010Input instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<R010Output *>
*/
- (nullable NSArray<R010Output *> *)predictionsFromInputs:(NSArray<R010Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
