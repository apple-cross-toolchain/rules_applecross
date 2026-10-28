//
// ThreeEnum18.h
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
@interface ThreeEnum18Input : NSObject<MLFeatureProvider>

/// a as 1 by 4 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * a;

/// b as 3 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * b;

/// c as color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef c;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithA:(MLMultiArray *)a b:(MLMultiArray *)b c:(CVPixelBufferRef)c NS_DESIGNATED_INITIALIZER;

- (nullable instancetype)initWithA:(MLMultiArray *)a b:(MLMultiArray *)b cCGImage:(CGImageRef)c error:(NSError * _Nullable __autoreleasing * _Nullable)error;

- (nullable instancetype)initWithA:(MLMultiArray *)a b:(MLMultiArray *)b cAtURL:(NSURL *)cURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

-(BOOL)setCWithCGImage:(CGImageRef)c error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setCWithURL:(NSURL *)cURL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
@end

/// Model Prediction Output Type
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface ThreeEnum18Output : NSObject<MLFeatureProvider>

/// relu_0 as multidimensional array of floats
@property (readwrite, nonatomic, strong) MLMultiArray * relu_0;

/// relu_1 as multidimensional array of floats
@property (readwrite, nonatomic, strong) MLMultiArray * relu_1;

/// reduce_mean_0 as 1 by 3 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * reduce_mean_0;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithRelu_0:(MLMultiArray *)relu_0 relu_1:(MLMultiArray *)relu_1 reduce_mean_0:(MLMultiArray *)reduce_mean_0 NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface ThreeEnum18 : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize ThreeEnum18 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of ThreeEnum18.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize ThreeEnum18 instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize ThreeEnum18 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize ThreeEnum18 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ThreeEnum18.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize ThreeEnum18 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ThreeEnum18.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct ThreeEnum18 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ThreeEnum18 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ThreeEnum18 * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct ThreeEnum18 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ThreeEnum18 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ThreeEnum18 * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of ThreeEnum18Input to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ThreeEnum18Output
*/
- (nullable ThreeEnum18Output *)predictionFromFeatures:(ThreeEnum18Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of ThreeEnum18Input to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ThreeEnum18Output
*/
- (nullable ThreeEnum18Output *)predictionFromFeatures:(ThreeEnum18Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of ThreeEnum18Input to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(ThreeEnum18Input *)input completionHandler:(void (^)(ThreeEnum18Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of ThreeEnum18Input to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(ThreeEnum18Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(ThreeEnum18Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make a prediction using the convenience interface
    @param a 1 by 4 matrix of floats
    @param b 3 by 2 matrix of floats
    @param c color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ThreeEnum18Output
*/
- (nullable ThreeEnum18Output *)predictionFromA:(MLMultiArray *)a b:(MLMultiArray *)b c:(CVPixelBufferRef)c error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of ThreeEnum18Input instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<ThreeEnum18Output *>
*/
- (nullable NSArray<ThreeEnum18Output *> *)predictionsFromInputs:(NSArray<ThreeEnum18Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
