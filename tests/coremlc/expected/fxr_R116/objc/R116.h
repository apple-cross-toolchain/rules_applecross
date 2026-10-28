//
// R116.h
//
// This file was automatically generated and should not be edited.
//

#import <Foundation/Foundation.h>
#import <CoreML/CoreML.h>
#include <stdint.h>
#include <os/log.h>

NS_ASSUME_NONNULL_BEGIN

/// Model Prediction Input Type
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface R116Input : NSObject<MLFeatureProvider>

/// in0 as 2 × 4 × 5 3-dimensional array of 16-bit floats
@property (readwrite, nonatomic, strong) MLMultiArray * in0;

/// in1 as 5 × 1 × 3 × 2 × 2 5-dimensional array of floats
@property (readwrite, nonatomic, strong) MLMultiArray * in1;

/// in2 as 5 × 4 × 5 × 4 4-dimensional array of floats
@property (readwrite, nonatomic, strong) MLMultiArray * in2;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithIn0:(MLMultiArray *)in0 in1:(MLMultiArray *)in1 in2:(MLMultiArray *)in2 NS_DESIGNATED_INITIALIZER;

@end

/// Model Prediction Output Type
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface R116Output : NSObject<MLFeatureProvider>

/// relu_0 as 2 × 4 × 5 3-dimensional array of 16-bit floats
@property (readwrite, nonatomic, strong) MLMultiArray * relu_0;

/// relu_1 as 5 × 1 × 3 × 2 × 2 5-dimensional array of floats
@property (readwrite, nonatomic, strong) MLMultiArray * relu_1;

/// relu_2 as multidimensional array of floats
@property (readwrite, nonatomic, strong) MLMultiArray * relu_2;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithRelu_0:(MLMultiArray *)relu_0 relu_1:(MLMultiArray *)relu_1 relu_2:(MLMultiArray *)relu_2 NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface R116 : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize R116 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of R116.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize R116 instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize R116 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize R116 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for R116.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize R116 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for R116.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct R116 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid R116 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(R116 * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct R116 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid R116 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(R116 * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of R116Input to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as R116Output
*/
- (nullable R116Output *)predictionFromFeatures:(R116Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of R116Input to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as R116Output
*/
- (nullable R116Output *)predictionFromFeatures:(R116Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of R116Input to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(R116Input *)input completionHandler:(void (^)(R116Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of R116Input to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(R116Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(R116Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make a prediction using the convenience interface
    @param in0 2 × 4 × 5 3-dimensional array of 16-bit floats
    @param in1 5 × 1 × 3 × 2 × 2 5-dimensional array of floats
    @param in2 5 × 4 × 5 × 4 4-dimensional array of floats
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as R116Output
*/
- (nullable R116Output *)predictionFromIn0:(MLMultiArray *)in0 in1:(MLMultiArray *)in1 in2:(MLMultiArray *)in2 error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of R116Input instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<R116Output *>
*/
- (nullable NSArray<R116Output *> *)predictionsFromInputs:(NSArray<R116Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
