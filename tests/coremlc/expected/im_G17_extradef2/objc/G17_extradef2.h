//
// G17_extradef2.h
//
// This file was automatically generated and should not be edited.
//

#import <Foundation/Foundation.h>
#import <CoreML/CoreML.h>
#include <stdint.h>
#include <os/log.h>

NS_ASSUME_NONNULL_BEGIN

/// Model Prediction Input Type
API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")))
@interface G17_extradef2Input : NSObject<MLFeatureProvider>

/// img as grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 8 pixels wide by 8 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef img;

/// t as 2 element vector of 16-bit floats
@property (readwrite, nonatomic, strong) MLMultiArray * t;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithImg:(CVPixelBufferRef)img t:(MLMultiArray *)t NS_DESIGNATED_INITIALIZER;

- (nullable instancetype)initWithImgFromCGImage:(CGImageRef)img t:(MLMultiArray *)t error:(NSError * _Nullable __autoreleasing * _Nullable)error;

- (nullable instancetype)initWithImgAtURL:(NSURL *)imgURL t:(MLMultiArray *)t error:(NSError * _Nullable __autoreleasing * _Nullable)error;

-(BOOL)setImgWithCGImage:(CGImageRef)img error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setImgWithURL:(NSURL *)imgURL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
@end

/// Model Prediction Output Type
API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")))
@interface G17_extradef2Output : NSObject<MLFeatureProvider>

/// reduce_mean_0 as 1 by 1 matrix of 16-bit floats
@property (readwrite, nonatomic, strong) MLMultiArray * reduce_mean_0;

/// relu_0 as 2 element vector of 16-bit floats
@property (readwrite, nonatomic, strong) MLMultiArray * relu_0;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithReduce_mean_0:(MLMultiArray *)reduce_mean_0 relu_0:(MLMultiArray *)relu_0 NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")))
@interface G17_extradef2 : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize G17_extradef2 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of G17_extradef2.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize G17_extradef2 instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize G17_extradef2 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize G17_extradef2 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for G17_extradef2.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize G17_extradef2 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for G17_extradef2.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct G17_extradef2 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid G17_extradef2 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(G17_extradef2 * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct G17_extradef2 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid G17_extradef2 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(G17_extradef2 * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of G17_extradef2Input to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as G17_extradef2Output
*/
- (nullable G17_extradef2Output *)predictionFromFeatures:(G17_extradef2Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of G17_extradef2Input to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as G17_extradef2Output
*/
- (nullable G17_extradef2Output *)predictionFromFeatures:(G17_extradef2Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of G17_extradef2Input to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(G17_extradef2Input *)input completionHandler:(void (^)(G17_extradef2Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of G17_extradef2Input to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(G17_extradef2Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(G17_extradef2Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make a prediction using the convenience interface
    @param img grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 8 pixels wide by 8 pixels high
    @param t 2 element vector of 16-bit floats
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as G17_extradef2Output
*/
- (nullable G17_extradef2Output *)predictionFromImg:(CVPixelBufferRef)img t:(MLMultiArray *)t error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of G17_extradef2Input instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<G17_extradef2Output *>
*/
- (nullable NSArray<G17_extradef2Output *> *)predictionsFromInputs:(NSArray<G17_extradef2Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
