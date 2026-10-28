//
// FlexImgEnum.h
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
@interface FlexImgEnumInput : NSObject<MLFeatureProvider>

/// img as color (kCVPixelFormatType_32BGRA) image buffer, 8 pixels wide by 8 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef img;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithImg:(CVPixelBufferRef)img NS_DESIGNATED_INITIALIZER;

- (nullable instancetype)initWithImgFromCGImage:(CGImageRef)img error:(NSError * _Nullable __autoreleasing * _Nullable)error;

- (nullable instancetype)initWithImgAtURL:(NSURL *)imgURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

-(BOOL)setImgWithCGImage:(CGImageRef)img error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setImgWithURL:(NSURL *)imgURL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
@end

/// Model Prediction Output Type
API_AVAILABLE(macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)) __attribute__((visibility("hidden")))
@interface FlexImgEnumOutput : NSObject<MLFeatureProvider>

/// reduce_mean_1 as 1 by 3 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * reduce_mean_1;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithReduce_mean_1:(MLMultiArray *)reduce_mean_1 NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(13.0), ios(16.0), watchos(9.0), tvos(16.0)) __attribute__((visibility("hidden")))
@interface FlexImgEnum : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize FlexImgEnum instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of FlexImgEnum.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize FlexImgEnum instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize FlexImgEnum instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize FlexImgEnum instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for FlexImgEnum.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize FlexImgEnum instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for FlexImgEnum.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct FlexImgEnum instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid FlexImgEnum instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(FlexImgEnum * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct FlexImgEnum instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid FlexImgEnum instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(FlexImgEnum * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of FlexImgEnumInput to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as FlexImgEnumOutput
*/
- (nullable FlexImgEnumOutput *)predictionFromFeatures:(FlexImgEnumInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of FlexImgEnumInput to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as FlexImgEnumOutput
*/
- (nullable FlexImgEnumOutput *)predictionFromFeatures:(FlexImgEnumInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of FlexImgEnumInput to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(FlexImgEnumInput *)input completionHandler:(void (^)(FlexImgEnumOutput * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of FlexImgEnumInput to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(FlexImgEnumInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(FlexImgEnumOutput * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make a prediction using the convenience interface
    @param img color (kCVPixelFormatType_32BGRA) image buffer, 8 pixels wide by 8 pixels high
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as FlexImgEnumOutput
*/
- (nullable FlexImgEnumOutput *)predictionFromImg:(CVPixelBufferRef)img error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of FlexImgEnumInput instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<FlexImgEnumOutput *>
*/
- (nullable NSArray<FlexImgEnumOutput *> *)predictionsFromInputs:(NSArray<FlexImgEnumInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
