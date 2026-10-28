//
// StImage.h
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
@interface StImageInput : NSObject<MLFeatureProvider>

/// img as color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef img;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithImg:(CVPixelBufferRef)img NS_DESIGNATED_INITIALIZER;

- (nullable instancetype)initWithImgFromCGImage:(CGImageRef)img error:(NSError * _Nullable __autoreleasing * _Nullable)error;

- (nullable instancetype)initWithImgAtURL:(NSURL *)imgURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

-(BOOL)setImgWithCGImage:(CGImageRef)img error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setImgWithURL:(NSURL *)imgURL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
@end

/// Model Prediction Output Type
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface StImageOutput : NSObject<MLFeatureProvider>

/// add_0 as 1 × 3 × 4 × 4 4-dimensional array of 16-bit floats
@property (readwrite, nonatomic, strong) MLMultiArray * add_0;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithAdd_0:(MLMultiArray *)add_0 NS_DESIGNATED_INITIALIZER;

@end

/// Model Prediction State Names
typedef NS_ENUM(NSInteger, StImageStateName) {
    StImageStateNameS,
} API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")));

/// Model Prediction State Type
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface StImageState : NSObject
@property (readonly, strong, nonatomic) MLState *state;
- (void)getMultiArrayForState:(StImageStateName)stateName
                      handler:(void (NS_NOESCAPE ^)(MLMultiArray *buffer))handler;
@end

/// Class for model loading and prediction
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface StImage : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize StImage instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of StImage.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize StImage instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize StImage instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize StImage instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for StImage.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize StImage instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for StImage.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct StImage instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid StImage instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(StImage * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct StImage instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid StImage instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(StImage * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a new state.
*/
- (StImageState *)newState;

/**
    Make a prediction using the standard interface
    @param input an instance of StImageInput to predict from
    @param state prediction state
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as StImageOutput
*/
- (nullable StImageOutput *)predictionFromFeatures:(StImageInput *)input usingState:(StImageState *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of StImageInput to predict from
    @param state prediction state
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as StImageOutput
*/
- (nullable StImageOutput *)predictionFromFeatures:(StImageInput *)input usingState:(StImageState *)state options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of StImageInput to predict from
    @param state prediction state
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(StImageInput *)input usingState:(StImageState *)state completionHandler:(void (^)(StImageOutput * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of StImageInput to predict from
    @param state prediction state
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(StImageInput *)input usingState:(StImageState *)state options:(MLPredictionOptions *)options completionHandler:(void (^)(StImageOutput * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make a prediction using the convenience interface
    @param img color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
    @param state prediction state
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as StImageOutput
*/
- (nullable StImageOutput *)predictionFromImg:(CVPixelBufferRef)img usingState:(StImageState *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
