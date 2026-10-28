//
// ImgFP2_m0.h
//
// This file was automatically generated and should not be edited.
//

#import <Foundation/Foundation.h>
#import <CoreML/CoreML.h>
#include <stdint.h>
#include <os/log.h>

NS_ASSUME_NONNULL_BEGIN

/// Model Prediction Input Type
API_AVAILABLE(macos(14.0), ios(17.0), tvos(17.0)) API_UNAVAILABLE(watchos)  __attribute__((visibility("hidden")))
@interface ImgFP2_m0Input : NSObject<MLFeatureProvider>

/// image as color (kCVPixelFormatType_32BGRA) image buffer, 360 pixels wide by 360 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef image;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithImage:(CVPixelBufferRef)image NS_DESIGNATED_INITIALIZER;

- (nullable instancetype)initWithImageFromCGImage:(CGImageRef)image error:(NSError * _Nullable __autoreleasing * _Nullable)error;

- (nullable instancetype)initWithImageAtURL:(NSURL *)imageURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

-(BOOL)setImageWithCGImage:(CGImageRef)image error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
-(BOOL)setImageWithURL:(NSURL *)imageURL error:(NSError * _Nullable __autoreleasing * _Nullable)error ;
@end

/// Model Prediction Output Type
API_AVAILABLE(macos(14.0), ios(17.0), tvos(17.0)) API_UNAVAILABLE(watchos)  __attribute__((visibility("hidden")))
@interface ImgFP2_m0Output : NSObject<MLFeatureProvider>

/// target as string value
@property (readwrite, nonatomic, strong) NSString * target;

/// targetProbability as dictionary of strings to doubles
@property (readwrite, nonatomic, strong) NSDictionary<NSString *, NSNumber *> * targetProbability;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithTarget:(NSString *)target targetProbability:(NSDictionary<NSString *, NSNumber *> *)targetProbability NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(14.0), ios(17.0), tvos(17.0)) API_UNAVAILABLE(watchos)  __attribute__((visibility("hidden")))
@interface ImgFP2_m0 : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize ImgFP2_m0 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of ImgFP2_m0.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize ImgFP2_m0 instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize ImgFP2_m0 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize ImgFP2_m0 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImgFP2_m0.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize ImgFP2_m0 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ImgFP2_m0.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct ImgFP2_m0 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImgFP2_m0 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImgFP2_m0 * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct ImgFP2_m0 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ImgFP2_m0 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ImgFP2_m0 * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of ImgFP2_m0Input to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ImgFP2_m0Output
*/
- (nullable ImgFP2_m0Output *)predictionFromFeatures:(ImgFP2_m0Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of ImgFP2_m0Input to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ImgFP2_m0Output
*/
- (nullable ImgFP2_m0Output *)predictionFromFeatures:(ImgFP2_m0Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of ImgFP2_m0Input to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(ImgFP2_m0Input *)input completionHandler:(void (^)(ImgFP2_m0Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of ImgFP2_m0Input to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(ImgFP2_m0Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(ImgFP2_m0Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make a prediction using the convenience interface
    @param image color (kCVPixelFormatType_32BGRA) image buffer, 360 pixels wide by 360 pixels high
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ImgFP2_m0Output
*/
- (nullable ImgFP2_m0Output *)predictionFromImage:(CVPixelBufferRef)image error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of ImgFP2_m0Input instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<ImgFP2_m0Output *>
*/
- (nullable NSArray<ImgFP2_m0Output *> *)predictionsFromInputs:(NSArray<ImgFP2_m0Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
