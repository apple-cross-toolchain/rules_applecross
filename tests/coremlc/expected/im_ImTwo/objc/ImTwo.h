//
// ImTwo.h
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
@interface ImTwoInput : NSObject<MLFeatureProvider>

/// left as color (kCVPixelFormatType_32BGRA) image buffer, 8 pixels wide by 8 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef left_;

/// right as color (kCVPixelFormatType_32BGRA) image buffer, 8 pixels wide by 8 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef right_;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithLeft:(CVPixelBufferRef)left_ right_:(CVPixelBufferRef)right_ NS_DESIGNATED_INITIALIZER;

- (nullable instancetype)initWithLeftFromCGImage:(CGImageRef)left_ right_CGImage:(CGImageRef)right_ error:(NSError * _Nullable __autoreleasing * _Nullable)error;

- (nullable instancetype)initWithLeftAtURL:(NSURL *)left_URL right_AtURL:(NSURL *)right_URL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

-(BOOL)setLeft_WithCGImage:(CGImageRef)left_ error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setRight_WithCGImage:(CGImageRef)right_ error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setLeft_WithURL:(NSURL *)left_URL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setRight_WithURL:(NSURL *)right_URL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
@end

/// Model Prediction Output Type
API_AVAILABLE(macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)) __attribute__((visibility("hidden")))
@interface ImTwoOutput : NSObject<MLFeatureProvider>

/// reshape_0 as 1 element vector of floats
@property (readwrite, nonatomic, strong) MLMultiArray * reshape_0;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithReshape_0:(MLMultiArray *)reshape_0 NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)) __attribute__((visibility("hidden")))
@interface ImTwo : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize ImTwo instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of ImTwo.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize ImTwo instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize ImTwo instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize ImTwo instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImTwo.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize ImTwo instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImTwo.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct ImTwo instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImTwo instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImTwo * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct ImTwo instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImTwo instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImTwo * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of ImTwoInput to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ImTwoOutput
*/
- (nullable ImTwoOutput *)predictionFromFeatures:(ImTwoInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of ImTwoInput to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ImTwoOutput
*/
- (nullable ImTwoOutput *)predictionFromFeatures:(ImTwoInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of ImTwoInput to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(ImTwoInput *)input completionHandler:(void (^)(ImTwoOutput * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of ImTwoInput to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(ImTwoInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(ImTwoOutput * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make a prediction using the convenience interface
    @param left_ color (kCVPixelFormatType_32BGRA) image buffer, 8 pixels wide by 8 pixels high
    @param right_ color (kCVPixelFormatType_32BGRA) image buffer, 8 pixels wide by 8 pixels high
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ImTwoOutput
*/
- (nullable ImTwoOutput *)predictionFromLeft:(CVPixelBufferRef)left_ right_:(CVPixelBufferRef)right_ error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of ImTwoInput instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<ImTwoOutput *>
*/
- (nullable NSArray<ImTwoOutput *> *)predictionsFromInputs:(NSArray<ImTwoInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
