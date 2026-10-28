//
// Recommender.h
//
// This file was automatically generated and should not be edited.
//

#import <Foundation/Foundation.h>
#import <CoreML/CoreML.h>
#include <stdint.h>
#include <os/log.h>

NS_ASSUME_NONNULL_BEGIN

/// Model Prediction Input Type
API_AVAILABLE(macos(10.15), ios(13.0), watchos(6.0), tvos(13.0)) __attribute__((visibility("hidden")))
@interface RecommenderInput : NSObject<MLFeatureProvider>

/// The list of items used to generate the recommendations. as dictionary of strings to doubles
@property (readwrite, nonatomic, strong) NSDictionary<NSString *, NSNumber *> * items;

/// The number of items to return on a recommendation. as integer value
@property (readwrite, nonatomic) int64_t k;

/// A sequence of items from which to generate recommendations. as optional sequence of strings
@property (readwrite, nonatomic, strong, nullable) NSArray<NSString *> * restrict_;

/// A sequence of items to exclude from recommendations.  Defaults to the input item list if not given. as optional sequence of strings
@property (readwrite, nonatomic, strong, nullable) NSArray<NSString *> * exclude;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithItems:(NSDictionary<NSString *, NSNumber *> *)items k:(int64_t)k;
- (instancetype)initWithItems:(NSDictionary<NSString *, NSNumber *> *)items k:(int64_t)k restrict_:(nullable NSArray<NSString *> *)restrict_ exclude:(nullable NSArray<NSString *> *)exclude NS_DESIGNATED_INITIALIZER;

@end

/// Model Prediction Output Type
API_AVAILABLE(macos(10.15), ios(13.0), watchos(6.0), tvos(13.0)) __attribute__((visibility("hidden")))
@interface RecommenderOutput : NSObject<MLFeatureProvider>

/// The recommended items in order from most relevant to least relevant. as sequence of strings
@property (readwrite, nonatomic, strong) NSArray<NSString *> * recommendations;

/// The scores for the recommended items, given as a dictionary of items and the corresponding scores. as dictionary of strings to doubles
@property (readwrite, nonatomic, strong) NSDictionary<NSString *, NSNumber *> * scores;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithRecommendations:(NSArray<NSString *> *)recommendations scores:(NSDictionary<NSString *, NSNumber *> *)scores NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(10.15), ios(13.0), watchos(6.0), tvos(13.0)) __attribute__((visibility("hidden")))
@interface Recommender : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize Recommender instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of Recommender.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize Recommender instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize Recommender instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize Recommender instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Recommender.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize Recommender instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Recommender.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct Recommender instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Recommender instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Recommender * _Nullable model, NSError * _Nullable error))handler API_AVAILABLE(macos(11.0), ios(14.0), watchos(7.0), tvos(14.0)) __attribute__((visibility("hidden")));

/**
    Construct Recommender instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Recommender instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Recommender * _Nullable model, NSError * _Nullable error))handler API_AVAILABLE(macos(11.0), ios(14.0), watchos(7.0), tvos(14.0)) __attribute__((visibility("hidden")));

/**
    Make a prediction using the standard interface
    @param input an instance of RecommenderInput to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as RecommenderOutput
*/
- (nullable RecommenderOutput *)predictionFromFeatures:(RecommenderInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of RecommenderInput to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as RecommenderOutput
*/
- (nullable RecommenderOutput *)predictionFromFeatures:(RecommenderInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of RecommenderInput to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(RecommenderInput *)input completionHandler:(void (^)(RecommenderOutput * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of RecommenderInput to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(RecommenderInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(RecommenderOutput * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make a prediction using the convenience interface
    @param items The list of items used to generate the recommendations. as dictionary of strings to doubles
    @param k The number of items to return on a recommendation. as integer value
    @param restrict_ A sequence of items from which to generate recommendations. as optional sequence of strings
    @param exclude A sequence of items to exclude from recommendations.  Defaults to the input item list if not given. as optional sequence of strings
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as RecommenderOutput
*/
- (nullable RecommenderOutput *)predictionFromItems:(NSDictionary<NSString *, NSNumber *> *)items k:(int64_t)k restrict_:(nullable NSArray<NSString *> *)restrict_ exclude:(nullable NSArray<NSString *> *)exclude error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of RecommenderInput instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<RecommenderOutput *>
*/
- (nullable NSArray<RecommenderOutput *> *)predictionsFromInputs:(NSArray<RecommenderInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
