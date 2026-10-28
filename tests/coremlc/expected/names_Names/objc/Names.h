//
// Names.h
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
@interface NamesInput : NSObject<MLFeatureProvider>

/// x as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * x;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithX:(MLMultiArray *)x NS_DESIGNATED_INITIALIZER;

@end

/// Model Prediction Output Type
API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")))
@interface NamesOutput : NSObject<MLFeatureProvider>

/// out as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * out_;

/// var as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * var_;

/// in as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * in_;

/// inout as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * inout_;

/// self as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * self_;

/// class as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * class_;

/// func_workaround as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * func_workaround;

/// let as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * let_;

/// import as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * import_;

/// return as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * return_;

/// if as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * if_;

/// for as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * for_;

/// while as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * while_;

/// struct as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * struct_;

/// protocol as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * protocol_;

/// model as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * model;

/// input as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * input;

/// output as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * output;

/// features as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * features;

/// featureNames as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * featureNames;

/// provider as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * provider;

/// id as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * id_;

/// int as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * int_;

/// float as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * float_;

/// double as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * double_;

/// char as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * char_;

/// void as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * void_;

/// bool_workaround as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * bool_workaround;

/// BOOL as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * BOOL_;

/// YES as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * YES_;

/// NO as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * NO_;

/// nil as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * nil_;

/// new as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * new;

/// default as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * default_;

/// switch as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * switch_;

/// case as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * case_;

/// operator as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * operator_;

/// typealias as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * typealias_;

/// extension as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * extension_;

/// init as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * init_;

/// deinit as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * deinit_;

/// subscript as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * subscript_;

/// static as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * static_;

/// public as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * public_;

/// private as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * private_;

/// internal as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * internal_;

/// true_workaround as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * true_workaround;

/// false_workaround as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * false_workaround;

/// Type as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Type_;

/// Protocol as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Protocol_;

/// super as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * super_;

/// where as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * where_;

/// as as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * as_;

/// is as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * is_;

/// try as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * try;

/// throw as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * throw;

/// throws as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * throws;

/// rethrows as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * rethrows;

/// async as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * async;

/// await as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * await;

/// defer as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * defer;

/// guard as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * guard;

/// repeat as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * repeat;

/// break as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * break_;

/// continue as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * continue_;

/// fallthrough as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * fallthrough_;

/// associatedtype as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * associatedtype;

/// enum as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * enum_;

/// union as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * union_;

/// const as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * const_;

/// signed as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * signed_;

/// unsigned as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * unsigned_;

/// long as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * long_;

/// short as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * short_;

/// auto as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * auto_;

/// register as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * register_;

/// extern as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * extern_;

/// volatile as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * volatile_;

/// goto as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * goto_;

/// sizeof as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * sizeof_;

/// typedef as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * typedef_;

/// oneway as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * oneway_;

/// bycopy as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * bycopy_;

/// byref as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * byref_;

/// restrict as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * restrict_;

/// inline as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * inline_;

/// template as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * template;

/// namespace as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * namespace;

/// delete as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * delete;

/// this as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * this;

/// friend as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * friend;

/// virtual as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * virtual;

/// mutable as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * mutable;

/// explicit as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * explicit;

/// export as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * export;

/// Any as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Any;

/// Self as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Self_;

/// some as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * some;

/// any_workaround as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * any_workaround;

/// do as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * do_;

/// else as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * else_;

/// open as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * open;

/// fileprivate as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * fileprivate;

/// convenience as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * convenience_;

/// dynamic as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * dynamic_;

/// final as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * final_;

/// lazy as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * lazy_;

/// optional as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * optional_;

/// required as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * required_;

/// override as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * override_;

/// weak as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * weak_;

/// unowned as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * unowned_;

/// mutating as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * mutating_;

/// nonmutating as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * nonmutating_;

/// indirect as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * indirect;

/// prefix as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * prefix_;

/// postfix as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * postfix_;

/// infix as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * infix_;

/// get as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * get_;

/// set as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * set_;

/// willSet as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * willSet;

/// didSet as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * didSet_;

/// NULL as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * NULL_;

/// Class as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * Class_;

/// SEL as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * SEL_;

/// IMP as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * IMP_;

/// nonatomic as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * nonatomic_;

/// atomic as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * atomic_;

/// strong as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * strong;

/// copy as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * copy;

/// assign as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * assign;

/// readonly as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * readonly;

/// readwrite as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * readwrite;

/// instancetype as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * instancetype;

/// description as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * description;

/// hash as 1 by 2 matrix of floats
@property (readwrite, nonatomic, strong) MLMultiArray * hash;
- (instancetype)init NS_UNAVAILABLE;
- (instancetype)initWithOut:(MLMultiArray *)out_ var_:(MLMultiArray *)var_ in_:(MLMultiArray *)in_ inout_:(MLMultiArray *)inout_ self_:(MLMultiArray *)self_ class_:(MLMultiArray *)class_ func_workaround:(MLMultiArray *)func_workaround let_:(MLMultiArray *)let_ import_:(MLMultiArray *)import_ return_:(MLMultiArray *)return_ if_:(MLMultiArray *)if_ for_:(MLMultiArray *)for_ while_:(MLMultiArray *)while_ struct_:(MLMultiArray *)struct_ protocol_:(MLMultiArray *)protocol_ model:(MLMultiArray *)model input:(MLMultiArray *)input output:(MLMultiArray *)output features:(MLMultiArray *)features featureNames:(MLMultiArray *)featureNames provider:(MLMultiArray *)provider id_:(MLMultiArray *)id_ int_:(MLMultiArray *)int_ float_:(MLMultiArray *)float_ double_:(MLMultiArray *)double_ char_:(MLMultiArray *)char_ void_:(MLMultiArray *)void_ bool_workaround:(MLMultiArray *)bool_workaround BOOL_:(MLMultiArray *)BOOL_ YES_:(MLMultiArray *)YES_ NO_:(MLMultiArray *)NO_ nil_:(MLMultiArray *)nil_ new:(MLMultiArray *)new default_:(MLMultiArray *)default_ switch_:(MLMultiArray *)switch_ case_:(MLMultiArray *)case_ operator_:(MLMultiArray *)operator_ typealias_:(MLMultiArray *)typealias_ extension_:(MLMultiArray *)extension_ init_:(MLMultiArray *)init_ deinit_:(MLMultiArray *)deinit_ subscript_:(MLMultiArray *)subscript_ static_:(MLMultiArray *)static_ public_:(MLMultiArray *)public_ private_:(MLMultiArray *)private_ internal_:(MLMultiArray *)internal_ true_workaround:(MLMultiArray *)true_workaround false_workaround:(MLMultiArray *)false_workaround Type_:(MLMultiArray *)Type_ Protocol_:(MLMultiArray *)Protocol_ super_:(MLMultiArray *)super_ where_:(MLMultiArray *)where_ as_:(MLMultiArray *)as_ is_:(MLMultiArray *)is_ try:(MLMultiArray *)try throw:(MLMultiArray *)throw throws:(MLMultiArray *)throws rethrows:(MLMultiArray *)rethrows async:(MLMultiArray *)async await:(MLMultiArray *)await defer:(MLMultiArray *)defer guard:(MLMultiArray *)guard repeat:(MLMultiArray *)repeat break_:(MLMultiArray *)break_ continue_:(MLMultiArray *)continue_ fallthrough_:(MLMultiArray *)fallthrough_ associatedtype:(MLMultiArray *)associatedtype enum_:(MLMultiArray *)enum_ union_:(MLMultiArray *)union_ const_:(MLMultiArray *)const_ signed_:(MLMultiArray *)signed_ unsigned_:(MLMultiArray *)unsigned_ long_:(MLMultiArray *)long_ short_:(MLMultiArray *)short_ auto_:(MLMultiArray *)auto_ register_:(MLMultiArray *)register_ extern_:(MLMultiArray *)extern_ volatile_:(MLMultiArray *)volatile_ goto_:(MLMultiArray *)goto_ sizeof_:(MLMultiArray *)sizeof_ typedef_:(MLMultiArray *)typedef_ oneway_:(MLMultiArray *)oneway_ bycopy_:(MLMultiArray *)bycopy_ byref_:(MLMultiArray *)byref_ restrict_:(MLMultiArray *)restrict_ inline_:(MLMultiArray *)inline_ template:(MLMultiArray *)template namespace:(MLMultiArray *)namespace delete:(MLMultiArray *)delete this:(MLMultiArray *)this friend:(MLMultiArray *)friend virtual:(MLMultiArray *)virtual mutable:(MLMultiArray *)mutable explicit:(MLMultiArray *)explicit export:(MLMultiArray *)export Any:(MLMultiArray *)Any Self_:(MLMultiArray *)Self_ some:(MLMultiArray *)some any_workaround:(MLMultiArray *)any_workaround do_:(MLMultiArray *)do_ else_:(MLMultiArray *)else_ open:(MLMultiArray *)open fileprivate:(MLMultiArray *)fileprivate convenience_:(MLMultiArray *)convenience_ dynamic_:(MLMultiArray *)dynamic_ final_:(MLMultiArray *)final_ lazy_:(MLMultiArray *)lazy_ optional_:(MLMultiArray *)optional_ required_:(MLMultiArray *)required_ override_:(MLMultiArray *)override_ weak_:(MLMultiArray *)weak_ unowned_:(MLMultiArray *)unowned_ mutating_:(MLMultiArray *)mutating_ nonmutating_:(MLMultiArray *)nonmutating_ indirect:(MLMultiArray *)indirect prefix_:(MLMultiArray *)prefix_ postfix_:(MLMultiArray *)postfix_ infix_:(MLMultiArray *)infix_ get_:(MLMultiArray *)get_ set_:(MLMultiArray *)set_ willSet:(MLMultiArray *)willSet didSet_:(MLMultiArray *)didSet_ NULL_:(MLMultiArray *)NULL_ Class_:(MLMultiArray *)Class_ SEL_:(MLMultiArray *)SEL_ IMP_:(MLMultiArray *)IMP_ nonatomic_:(MLMultiArray *)nonatomic_ atomic_:(MLMultiArray *)atomic_ strong:(MLMultiArray *)strong copy:(MLMultiArray *)copy assign:(MLMultiArray *)assign readonly:(MLMultiArray *)readonly readwrite:(MLMultiArray *)readwrite instancetype:(MLMultiArray *)instancetype description:(MLMultiArray *)description hash:(MLMultiArray *)hash NS_DESIGNATED_INITIALIZER;

@end

/// Class for model loading and prediction
API_AVAILABLE(macos(14.0), ios(17.0), watchos(10.0), tvos(17.0)) __attribute__((visibility("hidden")))
@interface Names : NSObject
@property (readonly, nonatomic, nullable) MLModel * model;

/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle;

/**
    Initialize Names instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of Names.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model NS_DESIGNATED_INITIALIZER;

/**
    Initialize Names instance with the model in this bundle.
*/
- (nullable instancetype)init;

/**
    Initialize Names instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize Names instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Names.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Initialize Names instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Names.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Construct Names instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Names instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Names * _Nullable model, NSError * _Nullable error))handler;

/**
    Construct Names instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Names instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Names * _Nullable model, NSError * _Nullable error))handler;

/**
    Make a prediction using the standard interface
    @param input an instance of NamesInput to predict from
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as NamesOutput
*/
- (nullable NamesOutput *)predictionFromFeatures:(NamesInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make a prediction using the standard interface
    @param input an instance of NamesInput to predict from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as NamesOutput
*/
- (nullable NamesOutput *)predictionFromFeatures:(NamesInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of NamesInput to predict from
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(NamesInput *)input completionHandler:(void (^)(NamesOutput * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make an asynchronous prediction using the standard interface
    @param input an instance of NamesInput to predict from
    @param options prediction options
    @param completionHandler a block that will be called upon completion of the prediction. error will be nil if no error occurred.
*/
- (void)predictionFromFeatures:(NamesInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(NamesOutput * _Nullable output, NSError * _Nullable error))completionHandler;

/**
    Make a prediction using the convenience interface
    @param x 1 by 2 matrix of floats
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the prediction as NamesOutput
*/
- (nullable NamesOutput *)predictionFromX:(MLMultiArray *)x error:(NSError * _Nullable __autoreleasing * _Nullable)error;

/**
    Batch prediction
    @param inputArray array of NamesInput instances to obtain predictions from
    @param options prediction options
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
    @return the predictions as NSArray<NamesOutput *>
*/
- (nullable NSArray<NamesOutput *> *)predictionsFromInputs:(NSArray<NamesInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error;
@end

NS_ASSUME_NONNULL_END
