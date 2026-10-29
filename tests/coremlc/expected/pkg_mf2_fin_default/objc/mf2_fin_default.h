//
// mf2_fin_default.h
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
@interface mf2_fin_defaultInput : NSObject<MLFeatureProvider>

/// orig relu x as 1 by 3 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * x;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithX:(MLMultiArray *)x NS_DESIGNATED_INITIALIZER;

@end

/// Model Prediction Output Type
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface mf2_fin_defaultOutput : NSObject<MLFeatureProvider>

/// orig relu relu_0 as 1 by 3 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * relu_0;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithRelu_0:(MLMultiArray *)relu_0 NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface mf2_fin_default : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize mf2_fin_default instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of mf2_fin_default.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize mf2_fin_default instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize mf2_fin_default instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize mf2_fin_default instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for mf2_fin_default.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize mf2_fin_default instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for mf2_fin_default.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct mf2_fin_default instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid mf2_fin_default instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(mf2_fin_default * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct mf2_fin_default instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid mf2_fin_default instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(mf2_fin_default * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of mf2_fin_defaultInput to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as mf2_fin_defaultOutput
*/
- (nullable mf2_fin_defaultOutput *)predictionFromFeatures:(mf2_fin_defaultInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of mf2_fin_defaultInput to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as mf2_fin_defaultOutput
*/
- (nullable mf2_fin_defaultOutput *)predictionFromFeatures:(mf2_fin_defaultInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of mf2_fin_defaultInput to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(mf2_fin_defaultInput *)input completionHandler:(void (^)(mf2_fin_defaultOutput * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of mf2_fin_defaultInput to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(mf2_fin_defaultInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(mf2_fin_defaultOutput * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make a prediction using the convenience interface
    @param x orig relu x as 1 by 3 matrix of floats
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as mf2_fin_defaultOutput
*/
- (nullable mf2_fin_defaultOutput *)predictionFromX:(MLMultiArray *)x error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of mf2_fin_defaultInput instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<mf2_fin_defaultOutput *>
*/
- (nullable NSArray<mf2_fin_defaultOutput *> *)predictionsFromInputs:(NSArray<mf2_fin_defaultInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
