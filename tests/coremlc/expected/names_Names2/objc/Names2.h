//
// Names2.h
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
@interface Names2Input : NSObject<MLFeatureProvider>

/// x as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * x;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithX:(MLMultiArray *)x NS_DESIGNATED_INITIALIZER;

@end

/// Model Prediction Output Type
API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")))
@interface Names2Output : NSObject<MLFeatureProvider>

/// catch as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * catch;

/// package as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * package;

/// macro as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * macro;

/// borrowing as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * borrowing;

/// consuming as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * consuming;

/// nonisolated as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * nonisolated;

/// isolated as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * isolated;

/// actor as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * actor;

/// each as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * each;

/// asm as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * asm;

/// typeof as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * typeof;

/// nullable as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * nullable;

/// nonnull as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * nonnull;

/// property as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * property;

/// synthesize as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * synthesize;

/// interface as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * interface;

/// implementation as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * implementation;

/// end as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * end;

/// selector as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * selector;

/// encode as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * encode;

/// autoreleasepool as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * autoreleasepool;

/// finally as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * finally;

/// Int as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Int;

/// String as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * String;

/// Float as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Float;

/// Double as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Double;

/// Array as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Array;

/// Dictionary as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Dictionary;

/// Set as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Set;

/// Optional as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Optional;

/// Error as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Error;

/// URL as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * URL;

/// MLModel as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * MLModel;

/// MLMultiArray as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * MLMultiArray;

/// CVPixelBuffer as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * CVPixelBuffer;

/// Bundle as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Bundle;

/// result as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * result;

/// results as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * results;

/// options as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * options;

/// configuration as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * configuration;

/// handler as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * handler;

/// error as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * error_;

/// batchIn as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * batchIn;

/// outFeatures as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * outFeatures;

/// localError as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * localError;

/// retVal as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * retVal;

/// label as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * label;

/// prediction as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * prediction;

/// predictions as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * predictions;

/// urlOfModelInThisBundle as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * urlOfModelInThisBundle;

/// load as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * load;

/// bundle as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * bundle;

/// precedencegroup as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * precedencegroup;

/// _Bool as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * _Bool_;

/// static_assert as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * static_assert;

/// alignas as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * alignas;

/// alignof as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * alignof;

/// decltype as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * decltype;

/// noexcept as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * noexcept;

/// nullptr as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * nullptr;

/// thread_local as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * thread_local;

/// constexpr as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * constexpr;

/// char16_t as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * char16_t;

/// char32_t as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * char32_t;

/// wchar_t as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * wchar_t;

/// dynamicType as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * dynamicType_;

/// Category as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Category;

/// Method as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Method;

/// Ivar as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Ivar;

/// unknown as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * unknown;

/// and as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * and;

/// or as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * or;

/// not as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * not;

/// xor as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * xor;

/// bitand as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * bitand;

/// bitor as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * bitor;

/// compl as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * compl;

/// not_eq as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * not_eq;

/// or_eq as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * or_eq;

/// and_eq as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * and_eq;

/// xor_eq as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * xor_eq;

/// typeid as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * typeid;

/// typename as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * typename;

/// using as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * using;

/// public_ as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * public_;

/// throws_ as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * throws_;

/// mutating_ as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * mutating_;

/// Int32 as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Int32;

/// Int64 as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Int64;

/// UInt as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * UInt;

/// NSObject as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * NSObject;

/// NSString as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * NSString;

/// NSNumber as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * NSNumber;

/// NSError as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * NSError;

/// instancetype as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * instancetype;

/// strong as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * strong;

/// copy as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * copy;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithCatch:(MLMultiArray *)catch package:(MLMultiArray *)package macro:(MLMultiArray *)macro borrowing:(MLMultiArray *)borrowing consuming:(MLMultiArray *)consuming nonisolated:(MLMultiArray *)nonisolated isolated:(MLMultiArray *)isolated actor:(MLMultiArray *)actor each:(MLMultiArray *)each asm:(MLMultiArray *)asm typeof:(MLMultiArray *)typeof nullable:(MLMultiArray *)nullable nonnull:(MLMultiArray *)nonnull property:(MLMultiArray *)property synthesize:(MLMultiArray *)synthesize interface:(MLMultiArray *)interface implementation:(MLMultiArray *)implementation end:(MLMultiArray *)end selector:(MLMultiArray *)selector encode:(MLMultiArray *)encode autoreleasepool:(MLMultiArray *)autoreleasepool finally:(MLMultiArray *)finally Int:(MLMultiArray *)Int String:(MLMultiArray *)String Float:(MLMultiArray *)Float Double:(MLMultiArray *)Double Array:(MLMultiArray *)Array Dictionary:(MLMultiArray *)Dictionary Set:(MLMultiArray *)Set Optional:(MLMultiArray *)Optional Error:(MLMultiArray *)Error URL:(MLMultiArray *)URL MLModel:(MLMultiArray *)MLModel MLMultiArray:(MLMultiArray *)MLMultiArray CVPixelBuffer:(MLMultiArray *)CVPixelBuffer Bundle:(MLMultiArray *)Bundle result:(MLMultiArray *)result results:(MLMultiArray *)results options:(MLMultiArray *)options configuration:(MLMultiArray *)configuration handler:(MLMultiArray *)handler error_:(MLMultiArray *)error_ batchIn:(MLMultiArray *)batchIn outFeatures:(MLMultiArray *)outFeatures localError:(MLMultiArray *)localError retVal:(MLMultiArray *)retVal label:(MLMultiArray *)label prediction:(MLMultiArray *)prediction predictions:(MLMultiArray *)predictions urlOfModelInThisBundle:(MLMultiArray *)urlOfModelInThisBundle load:(MLMultiArray *)load bundle:(MLMultiArray *)bundle precedencegroup:(MLMultiArray *)precedencegroup _Bool_:(MLMultiArray *)_Bool_ static_assert:(MLMultiArray *)static_assert alignas:(MLMultiArray *)alignas alignof:(MLMultiArray *)alignof decltype:(MLMultiArray *)decltype noexcept:(MLMultiArray *)noexcept nullptr:(MLMultiArray *)nullptr thread_local:(MLMultiArray *)thread_local constexpr:(MLMultiArray *)constexpr char16_t:(MLMultiArray *)char16_t char32_t:(MLMultiArray *)char32_t wchar_t:(MLMultiArray *)wchar_t dynamicType_:(MLMultiArray *)dynamicType_ Category:(MLMultiArray *)Category Method:(MLMultiArray *)Method Ivar:(MLMultiArray *)Ivar unknown:(MLMultiArray *)unknown and:(MLMultiArray *)and or:(MLMultiArray *)or not:(MLMultiArray *)not xor:(MLMultiArray *)xor bitand:(MLMultiArray *)bitand bitor:(MLMultiArray *)bitor compl:(MLMultiArray *)compl not_eq:(MLMultiArray *)not_eq or_eq:(MLMultiArray *)or_eq and_eq:(MLMultiArray *)and_eq xor_eq:(MLMultiArray *)xor_eq typeid:(MLMultiArray *)typeid typename:(MLMultiArray *)typename using:(MLMultiArray *)using public_:(MLMultiArray *)public_ throws_:(MLMultiArray *)throws_ mutating_:(MLMultiArray *)mutating_ Int32:(MLMultiArray *)Int32 Int64:(MLMultiArray *)Int64 UInt:(MLMultiArray *)UInt NSObject:(MLMultiArray *)NSObject NSString:(MLMultiArray *)NSString NSNumber:(MLMultiArray *)NSNumber NSError:(MLMultiArray *)NSError instancetype:(MLMultiArray *)instancetype strong:(MLMultiArray *)strong copy:(MLMultiArray *)copy NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")))
@interface Names2 : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize Names2 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of Names2.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize Names2 instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize Names2 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize Names2 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Names2.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize Names2 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Names2.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct Names2 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Names2 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Names2 * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct Names2 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Names2 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Names2 * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of Names2Input to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as Names2Output
*/
- (nullable Names2Output *)predictionFromFeatures:(Names2Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of Names2Input to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as Names2Output
*/
- (nullable Names2Output *)predictionFromFeatures:(Names2Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of Names2Input to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(Names2Input *)input completionHandler:(void (^)(Names2Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of Names2Input to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(Names2Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(Names2Output * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make a prediction using the convenience interface
    @param x 1 by 2 matrix of floats
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as Names2Output
*/
- (nullable Names2Output *)predictionFromX:(MLMultiArray *)x error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of Names2Input instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<Names2Output *>
*/
- (nullable NSArray<Names2Output *> *)predictionsFromInputs:(NSArray<Names2Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
