//
// ImThree.h
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
@interface ImThreeInput : NSObject<MLFeatureProvider>

/// a as color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef a;

/// b as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * b;

/// c as grayscale (kCVPixelFormatType_OneComponent8) image buffer, 5 pixels wide by 5 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef c;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithA:(CVPixelBufferRef)a b:(MLMultiArray *)b c:(CVPixelBufferRef)c NS_DESIGNATED_INITIALIZER;

- (nullable instancetype)initWithAFromCGImage:(CGImageRef)a b:(MLMultiArray *)b cCGImage:(CGImageRef)c error:(NSError * _Nullable __autoreleasing * _Nullable)error;

- (nullable instancetype)initWithAAtURL:(NSURL *)aURL b:(MLMultiArray *)b cAtURL:(NSURL *)cURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

-(BOOL)setAWithCGImage:(CGImageRef)a error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setCWithCGImage:(CGImageRef)c error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setAWithURL:(NSURL *)aURL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setCWithURL:(NSURL *)cURL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
@end

/// Model Prediction Output Type
API_AVAILABLE(macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)) __attribute__((visibility("hidden")))
@interface ImThreeOutput : NSObject<MLFeatureProvider>

/// reshape_0 as 1 element vector of floats
@property (readwrite, nonatomic, strong) MLMultiArray * reshape_0;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithReshape_0:(MLMultiArray *)reshape_0 NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)) __attribute__((visibility("hidden")))
@interface ImThree : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize ImThree instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of ImThree.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize ImThree instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize ImThree instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize ImThree instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImThree.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize ImThree instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImThree.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct ImThree instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImThree instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImThree * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct ImThree instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImThree instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImThree * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of ImThreeInput to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ImThreeOutput
*/
- (nullable ImThreeOutput *)predictionFromFeatures:(ImThreeInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of ImThreeInput to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ImThreeOutput
*/
- (nullable ImThreeOutput *)predictionFromFeatures:(ImThreeInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of ImThreeInput to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(ImThreeInput *)input completionHandler:(void (^)(ImThreeOutput * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of ImThreeInput to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(ImThreeInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(ImThreeOutput * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make a prediction using the convenience interface
    @param a color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
    @param b 1 by 2 matrix of floats
    @param c grayscale (kCVPixelFormatType_OneComponent8) image buffer, 5 pixels wide by 5 pixels high
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ImThreeOutput
*/
- (nullable ImThreeOutput *)predictionFromA:(CVPixelBufferRef)a b:(MLMultiArray *)b c:(CVPixelBufferRef)c error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of ImThreeInput instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<ImThreeOutput *>
*/
- (nullable NSArray<ImThreeOutput *> *)predictionsFromInputs:(NSArray<ImThreeInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
