//
// ThreeFP1_m1.h
//
// This file was automatically generated and should not be edited.
//

#import <Foundation/Foundation.h>
#import <CoreML/CoreML.h>
#include <stdint.h>
#include <os/log.h>

NS_ASSUME_NONNULL_BEGIN

/// Model Prediction Input Type
API_AVAILABLE(macos(10.14), ios(12.0), tvos(12.0)) API_UNAVAILABLE(watchos)  __attribute__((visibility("hidden")))
@interface ThreeFP1_m1Input : NSObject<MLFeatureProvider>

/// image as color (kCVPixelFormatType_32BGRA) image buffer, 299 pixels wide by 299 pixels high
@property (readwrite, nonatomic) CVPixelBufferRef image;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithImage:(CVPixelBufferRef)image NS_DESIGNATED_INITIALIZER;

- (nullable instancetype)initWithImageFromCGImage:(CGImageRef)image error:(NSError * _Nullable __autoreleasing * _Nullable)error API_AVAILABLE(macos(10.15), ios(13.0), watchos(6.0), tvos(13.0)) __attribute__((visibility("hidden")));

- (nullable instancetype)initWithImageAtURL:(NSURL *)imageURL error:(NSError * _Nullable __autoreleasing * _Nullable)error API_AVAILABLE(macos(10.15), ios(13.0), watchos(6.0), tvos(13.0)) __attribute__((visibility("hidden")));

-(BOOL)setImageWithCGImage:(CGImageRef)image error:(NSError * _Nullable __autoreleasing * _Nullable)error  API_AVAILABLE(macos(10.15), ios(13.0), watchos(6.0), tvos(13.0)) __attribute__((visibility("hidden")));
-(BOOL)setImageWithURL:(NSURL *)imageURL error:(NSError * _Nullable __autoreleasing * _Nullable)error  API_AVAILABLE(macos(10.15), ios(13.0), watchos(6.0), tvos(13.0)) __attribute__((visibility("hidden")));
@end

/// Model Prediction Output Type
API_AVAILABLE(macos(10.14), ios(12.0), tvos(12.0)) API_UNAVAILABLE(watchos)  __attribute__((visibility("hidden")))
@interface ThreeFP1_m1Output : NSObject<MLFeatureProvider>

/// target as string value
@property (readwrite, nonatomic, strong) NSString * target;

/// targetProbability as dictionary of strings to doubles
@property (readwrite, nonatomic, strong) NSDictionary<NSString *, NSNumber *> * targetProbability;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithTarget:(NSString *)target targetProbability:(NSDictionary<NSString *, NSNumber *> *)targetProbability NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(10.14), ios(12.0), tvos(12.0)) API_UNAVAILABLE(watchos)  __attribute__((visibility("hidden")))
@interface ThreeFP1_m1 : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize ThreeFP1_m1 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of ThreeFP1_m1.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize ThreeFP1_m1 instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize ThreeFP1_m1 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize ThreeFP1_m1 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ThreeFP1_m1.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize ThreeFP1_m1 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for ThreeFP1_m1.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct ThreeFP1_m1 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ThreeFP1_m1 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ThreeFP1_m1 * _Nullable model, NSError * _Nullable error))handler API_AVAILABLE(macos(11.0), ios(14.0), watchos(7.0), tvos(14.0)) __attribute__((visibility("hidden")));

/**
    Construct ThreeFP1_m1 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid ThreeFP1_m1 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(ThreeFP1_m1 * _Nullable model, NSError * _Nullable error))handler API_AVAILABLE(macos(11.0), ios(14.0), watchos(7.0), tvos(14.0)) __attribute__((visibility("hidden")));

/**
    Make a prediction using the standard interface
    @param input an instance of ThreeFP1_m1Input to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ThreeFP1_m1Output
*/
- (nullable ThreeFP1_m1Output *)predictionFromFeatures:(ThreeFP1_m1Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of ThreeFP1_m1Input to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ThreeFP1_m1Output
*/
- (nullable ThreeFP1_m1Output *)predictionFromFeatures:(ThreeFP1_m1Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of ThreeFP1_m1Input to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(ThreeFP1_m1Input *)input completionHandler:(void (^)(ThreeFP1_m1Output * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of ThreeFP1_m1Input to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(ThreeFP1_m1Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(ThreeFP1_m1Output * _Nullable output, NSError * _Nullable error))completionHandler API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")));

/**
    Make a prediction using the convenience interface
    @param image color (kCVPixelFormatType_32BGRA) image buffer, 299 pixels wide by 299 pixels high
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as ThreeFP1_m1Output
*/
- (nullable ThreeFP1_m1Output *)predictionFromImage:(CVPixelBufferRef)image error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of ThreeFP1_m1Input instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<ThreeFP1_m1Output *>
*/
- (nullable NSArray<ThreeFP1_m1Output *> *)predictionsFromInputs:(NSArray<ThreeFP1_m1Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
