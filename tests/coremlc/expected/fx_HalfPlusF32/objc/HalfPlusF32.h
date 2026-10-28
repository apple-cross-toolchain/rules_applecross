//
// HalfPlusF32.h
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
@interface HalfPlusF32Input : NSObject<MLFeatureProvider>

/// x as 1 by 3 matrix of 16-bit floats
@property (readwrite, nonatomic, strong) MLMultiArray * x;

/// y as 1 by 3 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * y;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithX:(MLMultiArray *)x y:(MLMultiArray *)y NS_DESIGNATED_INITIALIZER;

@end

/// Model Prediction Output Type
API_AVAILABLE(macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)) __attribute__((visibility("hidden")))
@interface HalfPlusF32Output : NSObject<MLFeatureProvider>

/// relu_0 as 1 by 3 matrix of 16-bit floats
@property (readwrite, nonatomic, strong) MLMultiArray * relu_0;

/// relu_1 as 1 by 3 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * relu_1;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithRelu_0:(MLMultiArray *)relu_0 relu_1:(MLMultiArray *)relu_1 NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)) __attribute__((visibility("hidden")))
@interface HalfPlusF32 : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize HalfPlusF32 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of HalfPlusF32.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize HalfPlusF32 instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize HalfPlusF32 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize HalfPlusF32 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for HalfPlusF32.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize HalfPlusF32 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for HalfPlusF32.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct HalfPlusF32 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid HalfPlusF32 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(HalfPlusF32 * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct HalfPlusF32 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid HalfPlusF32 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(HalfPlusF32 * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of HalfPlusF32Input to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as HalfPlusF32Output
*/
- (nullable HalfPlusF32Output *)predictionFromFeatures:(HalfPlusF32Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of HalfPlusF32Input to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as HalfPlusF32Output
*/
- (nullable HalfPlusF32Output *)predictionFromFeatures:(HalfPlusF32Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of HalfPlusF32Input to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(HalfPlusF32Input *)input completionHandler:(void (^)(HalfPlusF32Output * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of HalfPlusF32Input to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(HalfPlusF32Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(HalfPlusF32Output * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make a prediction using the convenience interface
    @param x 1 by 3 matrix of 16-bit floats
    @param y 1 by 3 matrix of floats
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as HalfPlusF32Output
*/
- (nullable HalfPlusF32Output *)predictionFromX:(MLMultiArray *)x y:(MLMultiArray *)y error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of HalfPlusF32Input instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<HalfPlusF32Output *>
*/
- (nullable NSArray<HalfPlusF32Output *> *)predictionsFromInputs:(NSArray<HalfPlusF32Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
