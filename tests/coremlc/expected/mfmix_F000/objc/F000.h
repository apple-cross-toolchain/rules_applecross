//
// F000.h
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
@interface F000Input : NSObject<MLFeatureProvider>

/// x as 1 × 5 × 5 × 3 4-dimensional array of 32-bit integers
@property (readwrite, nonatomic, strong) MLMultiArray * x;

/// tokens as color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 8 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef tokens;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithX:(MLMultiArray *)x tokens:(CVPixelBufferRef)tokens NS_DESIGNATED_INITIALIZER;

- (nullable instancetype)initWithX:(MLMultiArray *)x tokensCGImage:(CGImageRef)tokens error:(NSError * _Nullable __autoreleasing * _Nullable)error;

- (nullable instancetype)initWithX:(MLMultiArray *)x tokensAtURL:(NSURL *)tokensURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

-(BOOL)setTokensWithCGImage:(CGImageRef)tokens error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setTokensWithURL:(NSURL *)tokensURL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
@end

/// Model Prediction Output Type
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface F000Output : NSObject<MLFeatureProvider>

/// o_x as 1 × 5 × 5 × 3 4-dimensional array of 32-bit integers
@property (readwrite, nonatomic, strong) MLMultiArray * o_x;

/// mean_tokens as 1 by 3 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * mean_tokens;

/// s_image as 4 × 4 × 3 3-dimensional array of 16-bit floats
@property (readwrite, nonatomic, strong) MLMultiArray * s_image;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithO_x:(MLMultiArray *)o_x mean_tokens:(MLMultiArray *)mean_tokens s_image:(MLMultiArray *)s_image NS_DESIGNATED_INITIALIZER;

@end

/// Model Prediction State Names
typedef NS_ENUM(NSInteger, F000StateName) {
    F000StateNameImage,
} API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")));

/// Model Prediction State Type
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface F000State : NSObject
@property (readonly, strong, nonatomic) MLState *state;
- (void)getMultiArrayForState:(F000StateName)stateName
                      handler:(void (NS_NOESCAPE ^)(MLMultiArray *buffer))handler;
@end

/// Class for model loading and prediction
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface F000 : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize F000 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of F000.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize F000 instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize F000 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize F000 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for F000.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize F000 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for F000.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct F000 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid F000 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(F000 * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct F000 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid F000 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(F000 * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a new state.
*/
- (F000State *)newState;

/**
    Make a prediction using the standard interface
    @param input an instance of F000Input to predict from
    @param state prediction state
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as F000Output
*/
- (nullable F000Output *)predictionFromFeatures:(F000Input *)input usingState:(F000State *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of F000Input to predict from
    @param state prediction state
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as F000Output
*/
- (nullable F000Output *)predictionFromFeatures:(F000Input *)input usingState:(F000State *)state options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of F000Input to predict from
    @param state prediction state
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(F000Input *)input usingState:(F000State *)state completionHandler:(void (^)(F000Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of F000Input to predict from
    @param state prediction state
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(F000Input *)input usingState:(F000State *)state options:(MLPredictionOptions *)options completionHandler:(void (^)(F000Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make a prediction using the convenience interface
    @param x 1 × 5 × 5 × 3 4-dimensional array of 32-bit integers
    @param tokens color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 8 pixels high
    @param state prediction state
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as F000Output
*/
- (nullable F000Output *)predictionFromX:(MLMultiArray *)x tokens:(CVPixelBufferRef)tokens usingState:(F000State *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
