//
// NSvrNoMeta.h
//
// This file was automatically generated and should not be edited.
//

#import <Foundation/Foundation.h>
#import <CoreML/CoreML.h>
#include <stdint.h>
#include <os/log.h>

NS_ASSUME_NONNULL_BEGIN

/// Model Prediction Input Type
API_AVAILABLE(macos(10.13), ios(11.0), watchos(4.0), tvos(11.0)) __attribute__((visibility("hidden")))
@interface NSvrNoMetaInput : NSObject<MLFeatureProvider>

/// x as 3 element vector of doubles
@property (readwrite, nonatomic, strong) MLMultiArray * x;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithX:(MLMultiArray *)x NS_DESIGNATED_INITIALIZER;

@end

/// Model Prediction Output Type
API_AVAILABLE(macos(10.13), ios(11.0), watchos(4.0), tvos(11.0)) __attribute__((visibility("hidden")))
@interface NSvrNoMetaOutput : NSObject<MLFeatureProvider>

/// y as double value
@property (readwrite, nonatomic) double y;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithY:(double)y NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(10.13), ios(11.0), watchos(4.0), tvos(11.0)) __attribute__((visibility("hidden")))
@interface NSvrNoMeta : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize NSvrNoMeta instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of NSvrNoMeta.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize NSvrNoMeta instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize NSvrNoMeta instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error API_AVAILABLE(macos(10.14), ios(12.0), watchos(5.0), tvos(12.0)) __attribute__((visibility("hidden")));

/**
    Initialize NSvrNoMeta instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for NSvrNoMeta.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize NSvrNoMeta instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for NSvrNoMeta.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error API_AVAILABLE(macos(10.14), ios(12.0), watchos(5.0), tvos(12.0)) __attribute__((visibility("hidden")));

/**
    Construct NSvrNoMeta instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid NSvrNoMeta instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(NSvrNoMeta * _Nullable model, NSError * _Nullable error))handler API_AVAILABLE(macos(11.0), ios(14.0), watchos(7.0), tvos(14.0)) __attribute__((visibility("hidden")));

/**
    Construct NSvrNoMeta instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid NSvrNoMeta instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(NSvrNoMeta * _Nullable model, NSError * _Nullable error))handler API_AVAILABLE(macos(11.0), ios(14.0), watchos(7.0), tvos(14.0)) __attribute__((visibility("hidden")));

/**
    Make a prediction using the standard interface
    @param input an instance of NSvrNoMetaInput to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as NSvrNoMetaOutput
*/
- (nullable NSvrNoMetaOutput *)predictionFromFeatures:(NSvrNoMetaInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of NSvrNoMetaInput to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as NSvrNoMetaOutput
*/
- (nullable NSvrNoMetaOutput *)predictionFromFeatures:(NSvrNoMetaInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of NSvrNoMetaInput to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(NSvrNoMetaInput *)input completionHandler:(void (^)(NSvrNoMetaOutput * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of NSvrNoMetaInput to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(NSvrNoMetaInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(NSvrNoMetaOutput * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make a prediction using the convenience interface
    @param x 3 element vector of doubles
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as NSvrNoMetaOutput
*/
- (nullable NSvrNoMetaOutput *)predictionFromX:(MLMultiArray *)x error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of NSvrNoMetaInput instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<NSvrNoMetaOutput *>
*/
- (nullable NSArray<NSvrNoMetaOutput *> *)predictionsFromInputs:(NSArray<NSvrNoMetaInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error API_AVAILABLE(macos(10.14), ios(12.0), watchos(5.0), tvos(12.0)) __attribute__((visibility("hidden")));
@end

NS_ASSUME_NONNULL_END
