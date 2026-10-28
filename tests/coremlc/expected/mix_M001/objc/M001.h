//
// M001.h
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
@interface M001Input : NSObject<MLFeatureProvider>

/// features as color (kCVPixelFormatType_32BGRA) image buffer, 10 pixels wide by 6 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef features;

/// z9 as grayscale (kCVPixelFormatType_OneComponent8) image buffer, 6 pixels wide by 8 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef z9;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithFeatures:(CVPixelBufferRef)features z9:(CVPixelBufferRef)z9 NS_DESIGNATED_INITIALIZER;

- (nullable instancetype)initWithFeaturesFromCGImage:(CGImageRef)features z9CGImage:(CGImageRef)z9 error:(NSError * _Nullable __autoreleasing * _Nullable)error;

- (nullable instancetype)initWithFeaturesAtURL:(NSURL *)featuresURL z9AtURL:(NSURL *)z9URL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

-(BOOL)setFeaturesWithCGImage:(CGImageRef)features error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setZ9WithCGImage:(CGImageRef)z9 error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setFeaturesWithURL:(NSURL *)featuresURL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setZ9WithURL:(NSURL *)z9URL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
@end

/// Model Prediction Output Type
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface M001Output : NSObject<MLFeatureProvider>

/// mean_features as 1 by 3 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * mean_features;

/// mean_z9 as 1 by 1 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * mean_z9;

/// s_image as 1 by 1 matrix of 16-bit floats
@property (readwrite, nonatomic, strong) MLMultiArray * s_image;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithMean_features:(MLMultiArray *)mean_features mean_z9:(MLMultiArray *)mean_z9 s_image:(MLMultiArray *)s_image NS_DESIGNATED_INITIALIZER;

@end

/// Model Prediction State Names
typedef NS_ENUM(NSInteger, M001StateName) {
    M001StateNameImage,
} API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")));

/// Model Prediction State Type
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface M001State : NSObject
@property (readonly, strong, nonatomic) MLState *state;
- (void)getMultiArrayForState:(M001StateName)stateName
                      handler:(void (NS_NOESCAPE ^)(MLMultiArray *buffer))handler;
@end

/// Class for model loading and prediction
API_AVAILABLE(macos(15.0), ios(18.0), watchos(11.0), tvos(18.0)) __attribute__((visibility("hidden")))
@interface M001 : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize M001 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of M001.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize M001 instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize M001 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize M001 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for M001.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize M001 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for M001.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct M001 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid M001 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(M001 * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct M001 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid M001 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(M001 * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a new state.
*/
- (M001State *)newState;

/**
    Make a prediction using the standard interface
    @param input an instance of M001Input to predict from
    @param state prediction state
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as M001Output
*/
- (nullable M001Output *)predictionFromFeatures:(M001Input *)input usingState:(M001State *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of M001Input to predict from
    @param state prediction state
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as M001Output
*/
- (nullable M001Output *)predictionFromFeatures:(M001Input *)input usingState:(M001State *)state options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of M001Input to predict from
    @param state prediction state
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(M001Input *)input usingState:(M001State *)state completionHandler:(void (^)(M001Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of M001Input to predict from
    @param state prediction state
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(M001Input *)input usingState:(M001State *)state options:(MLPredictionOptions *)options completionHandler:(void (^)(M001Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make a prediction using the convenience interface
    @param features color (kCVPixelFormatType_32BGRA) image buffer, 10 pixels wide by 6 pixels high
    @param z9 grayscale (kCVPixelFormatType_OneComponent8) image buffer, 6 pixels wide by 8 pixels high
    @param state prediction state
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as M001Output
*/
- (nullable M001Output *)predictionFromFeatures_:(CVPixelBufferRef)features z9:(CVPixelBufferRef)z9 usingState:(M001State *)state error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
