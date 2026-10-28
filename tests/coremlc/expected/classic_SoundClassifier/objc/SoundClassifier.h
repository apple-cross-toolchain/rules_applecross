//
// SoundClassifier.h
//
// This file was automatically generated and should not be edited.
//

#import <Foundation/Foundation.h>
#import <CoreML/CoreML.h>
#include <stdint.h>
#include <os/log.h>

NS_ASSUME_NONNULL_BEGIN

/// Model Prediction Input Type
API_AVAILABLE(macos(12.0), ios(15.0), watchos(8.0), tvos(15.0)) __attribute__((visibility("hidden")))
@interface SoundClassifierInput : NSObject<MLFeatureProvider>

/// audioSamples as 15600 element vector of floats
@property (readwrite, nonatomic, strong) MLMultiArray * audioSamples;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithAudioSamples:(MLMultiArray *)audioSamples NS_DESIGNATED_INITIALIZER;

@end

/// Model Prediction Output Type
API_AVAILABLE(macos(12.0), ios(15.0), watchos(8.0), tvos(15.0)) __attribute__((visibility("hidden")))
@interface SoundClassifierOutput : NSObject<MLFeatureProvider>

/// target as string value
@property (readwrite, nonatomic, strong) NSString * target;

/// targetProbability as dictionary of strings to doubles
@property (readwrite, nonatomic, strong) NSDictionary<NSString *, NSNumber *> * targetProbability;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithTarget:(NSString *)target targetProbability:(NSDictionary<NSString *, NSNumber *> *)targetProbability NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(12.0), ios(15.0), watchos(8.0), tvos(15.0)) __attribute__((visibility("hidden")))
@interface SoundClassifier : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize SoundClassifier instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of SoundClassifier.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize SoundClassifier instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize SoundClassifier instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize SoundClassifier instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for SoundClassifier.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize SoundClassifier instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for SoundClassifier.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct SoundClassifier instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid SoundClassifier instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(SoundClassifier * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct SoundClassifier instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid SoundClassifier instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(SoundClassifier * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of SoundClassifierInput to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as SoundClassifierOutput
*/
- (nullable SoundClassifierOutput *)predictionFromFeatures:(SoundClassifierInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of SoundClassifierInput to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as SoundClassifierOutput
*/
- (nullable SoundClassifierOutput *)predictionFromFeatures:(SoundClassifierInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of SoundClassifierInput to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(SoundClassifierInput *)input completionHandler:(void (^)(SoundClassifierOutput * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of SoundClassifierInput to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(SoundClassifierInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(SoundClassifierOutput * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make a prediction using the convenience interface
    @param audioSamples 15600 element vector of floats
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as SoundClassifierOutput
*/
- (nullable SoundClassifierOutput *)predictionFromAudioSamples:(MLMultiArray *)audioSamples error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of SoundClassifierInput instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<SoundClassifierOutput *>
*/
- (nullable NSArray<SoundClassifierOutput *> *)predictionsFromInputs:(NSArray<SoundClassifierInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
