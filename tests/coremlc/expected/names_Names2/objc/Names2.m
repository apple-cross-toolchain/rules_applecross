//
// Names2.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "Names2.h"

@implementation Names2Input

- (instancetype)initWithX:(MLMultiArray *)x {
    self = [super init];
    if (self) {
        _x = x;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"x"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"x"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.x];
    }
    return nil;
}

@end

@implementation Names2Output

- (instancetype)initWithCatch:(MLMultiArray *)catch package:(MLMultiArray *)package macro:(MLMultiArray *)macro borrowing:(MLMultiArray *)borrowing consuming:(MLMultiArray *)consuming nonisolated:(MLMultiArray *)nonisolated isolated:(MLMultiArray *)isolated actor:(MLMultiArray *)actor each:(MLMultiArray *)each asm:(MLMultiArray *)asm typeof:(MLMultiArray *)typeof nullable:(MLMultiArray *)nullable nonnull:(MLMultiArray *)nonnull property:(MLMultiArray *)property synthesize:(MLMultiArray *)synthesize interface:(MLMultiArray *)interface implementation:(MLMultiArray *)implementation end:(MLMultiArray *)end selector:(MLMultiArray *)selector encode:(MLMultiArray *)encode autoreleasepool:(MLMultiArray *)autoreleasepool finally:(MLMultiArray *)finally Int:(MLMultiArray *)Int String:(MLMultiArray *)String Float:(MLMultiArray *)Float Double:(MLMultiArray *)Double Array:(MLMultiArray *)Array Dictionary:(MLMultiArray *)Dictionary Set:(MLMultiArray *)Set Optional:(MLMultiArray *)Optional Error:(MLMultiArray *)Error URL:(MLMultiArray *)URL MLModel:(MLMultiArray *)MLModel MLMultiArray:(MLMultiArray *)MLMultiArray CVPixelBuffer:(MLMultiArray *)CVPixelBuffer Bundle:(MLMultiArray *)Bundle result:(MLMultiArray *)result results:(MLMultiArray *)results options:(MLMultiArray *)options configuration:(MLMultiArray *)configuration handler:(MLMultiArray *)handler error_:(MLMultiArray *)error_ batchIn:(MLMultiArray *)batchIn outFeatures:(MLMultiArray *)outFeatures localError:(MLMultiArray *)localError retVal:(MLMultiArray *)retVal label:(MLMultiArray *)label prediction:(MLMultiArray *)prediction predictions:(MLMultiArray *)predictions urlOfModelInThisBundle:(MLMultiArray *)urlOfModelInThisBundle load:(MLMultiArray *)load bundle:(MLMultiArray *)bundle precedencegroup:(MLMultiArray *)precedencegroup _Bool_:(MLMultiArray *)_Bool_ static_assert:(MLMultiArray *)static_assert alignas:(MLMultiArray *)alignas alignof:(MLMultiArray *)alignof decltype:(MLMultiArray *)decltype noexcept:(MLMultiArray *)noexcept nullptr:(MLMultiArray *)nullptr thread_local:(MLMultiArray *)thread_local constexpr:(MLMultiArray *)constexpr char16_t:(MLMultiArray *)char16_t char32_t:(MLMultiArray *)char32_t wchar_t:(MLMultiArray *)wchar_t dynamicType_:(MLMultiArray *)dynamicType_ Category:(MLMultiArray *)Category Method:(MLMultiArray *)Method Ivar:(MLMultiArray *)Ivar unknown:(MLMultiArray *)unknown and:(MLMultiArray *)and or:(MLMultiArray *)or not:(MLMultiArray *)not xor:(MLMultiArray *)xor bitand:(MLMultiArray *)bitand bitor:(MLMultiArray *)bitor compl:(MLMultiArray *)compl not_eq:(MLMultiArray *)not_eq or_eq:(MLMultiArray *)or_eq and_eq:(MLMultiArray *)and_eq xor_eq:(MLMultiArray *)xor_eq typeid:(MLMultiArray *)typeid typename:(MLMultiArray *)typename using:(MLMultiArray *)using public_:(MLMultiArray *)public_ throws_:(MLMultiArray *)throws_ mutating_:(MLMultiArray *)mutating_ Int32:(MLMultiArray *)Int32 Int64:(MLMultiArray *)Int64 UInt:(MLMultiArray *)UInt NSObject:(MLMultiArray *)NSObject NSString:(MLMultiArray *)NSString NSNumber:(MLMultiArray *)NSNumber NSError:(MLMultiArray *)NSError instancetype:(MLMultiArray *)instancetype strong:(MLMultiArray *)strong copy:(MLMultiArray *)copy {
    self = [super init];
    if (self) {
        _catch = catch;
        _package = package;
        _macro = macro;
        _borrowing = borrowing;
        _consuming = consuming;
        _nonisolated = nonisolated;
        _isolated = isolated;
        _actor = actor;
        _each = each;
        _asm = asm;
        _typeof = typeof;
        _nullable = nullable;
        _nonnull = nonnull;
        _property = property;
        _synthesize = synthesize;
        _interface = interface;
        _implementation = implementation;
        _end = end;
        _selector = selector;
        _encode = encode;
        _autoreleasepool = autoreleasepool;
        _finally = finally;
        _Int = Int;
        _String = String;
        _Float = Float;
        _Double = Double;
        _Array = Array;
        _Dictionary = Dictionary;
        _Set = Set;
        _Optional = Optional;
        _Error = Error;
        _URL = URL;
        _MLModel = MLModel;
        _MLMultiArray = MLMultiArray;
        _CVPixelBuffer = CVPixelBuffer;
        _Bundle = Bundle;
        _result = result;
        _results = results;
        _options = options;
        _configuration = configuration;
        _handler = handler;
        _error_ = error_;
        _batchIn = batchIn;
        _outFeatures = outFeatures;
        _localError = localError;
        _retVal = retVal;
        _label = label;
        _prediction = prediction;
        _predictions = predictions;
        _urlOfModelInThisBundle = urlOfModelInThisBundle;
        _load = load;
        _bundle = bundle;
        _precedencegroup = precedencegroup;
        __Bool_ = _Bool_;
        _static_assert = static_assert;
        _alignas = alignas;
        _alignof = alignof;
        _decltype = decltype;
        _noexcept = noexcept;
        _nullptr = nullptr;
        _thread_local = thread_local;
        _constexpr = constexpr;
        _char16_t = char16_t;
        _char32_t = char32_t;
        _wchar_t = wchar_t;
        _dynamicType_ = dynamicType_;
        _Category = Category;
        _Method = Method;
        _Ivar = Ivar;
        _unknown = unknown;
        _and = and;
        _or = or;
        _not = not;
        _xor = xor;
        _bitand = bitand;
        _bitor = bitor;
        _compl = compl;
        _not_eq = not_eq;
        _or_eq = or_eq;
        _and_eq = and_eq;
        _xor_eq = xor_eq;
        _typeid = typeid;
        _typename = typename;
        _using = using;
        _public_ = public_;
        _throws_ = throws_;
        _mutating_ = mutating_;
        _Int32 = Int32;
        _Int64 = Int64;
        _UInt = UInt;
        _NSObject = NSObject;
        _NSString = NSString;
        _NSNumber = NSNumber;
        _NSError = NSError;
        _instancetype = instancetype;
        _strong = strong;
        _copy = copy;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"catch", @"package", @"macro", @"borrowing", @"consuming", @"nonisolated", @"isolated", @"actor", @"each", @"asm", @"typeof", @"nullable", @"nonnull", @"property", @"synthesize", @"interface", @"implementation", @"end", @"selector", @"encode", @"autoreleasepool", @"finally", @"Int", @"String", @"Float", @"Double", @"Array", @"Dictionary", @"Set", @"Optional", @"Error", @"URL", @"MLModel", @"MLMultiArray", @"CVPixelBuffer", @"Bundle", @"result", @"results", @"options", @"configuration", @"handler", @"error", @"batchIn", @"outFeatures", @"localError", @"retVal", @"label", @"prediction", @"predictions", @"urlOfModelInThisBundle", @"load", @"bundle", @"precedencegroup", @"_Bool", @"static_assert", @"alignas", @"alignof", @"decltype", @"noexcept", @"nullptr", @"thread_local", @"constexpr", @"char16_t", @"char32_t", @"wchar_t", @"dynamicType", @"Category", @"Method", @"Ivar", @"unknown", @"and", @"or", @"not", @"xor", @"bitand", @"bitor", @"compl", @"not_eq", @"or_eq", @"and_eq", @"xor_eq", @"typeid", @"typename", @"using", @"public_", @"throws_", @"mutating_", @"Int32", @"Int64", @"UInt", @"NSObject", @"NSString", @"NSNumber", @"NSError", @"instancetype", @"strong", @"copy"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"catch"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.catch];
    }
    if ([featureName isEqualToString:@"package"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.package];
    }
    if ([featureName isEqualToString:@"macro"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.macro];
    }
    if ([featureName isEqualToString:@"borrowing"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.borrowing];
    }
    if ([featureName isEqualToString:@"consuming"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.consuming];
    }
    if ([featureName isEqualToString:@"nonisolated"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.nonisolated];
    }
    if ([featureName isEqualToString:@"isolated"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.isolated];
    }
    if ([featureName isEqualToString:@"actor"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.actor];
    }
    if ([featureName isEqualToString:@"each"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.each];
    }
    if ([featureName isEqualToString:@"asm"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.asm];
    }
    if ([featureName isEqualToString:@"typeof"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.typeof];
    }
    if ([featureName isEqualToString:@"nullable"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.nullable];
    }
    if ([featureName isEqualToString:@"nonnull"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.nonnull];
    }
    if ([featureName isEqualToString:@"property"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.property];
    }
    if ([featureName isEqualToString:@"synthesize"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.synthesize];
    }
    if ([featureName isEqualToString:@"interface"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.interface];
    }
    if ([featureName isEqualToString:@"implementation"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.implementation];
    }
    if ([featureName isEqualToString:@"end"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.end];
    }
    if ([featureName isEqualToString:@"selector"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.selector];
    }
    if ([featureName isEqualToString:@"encode"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.encode];
    }
    if ([featureName isEqualToString:@"autoreleasepool"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.autoreleasepool];
    }
    if ([featureName isEqualToString:@"finally"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.finally];
    }
    if ([featureName isEqualToString:@"Int"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Int];
    }
    if ([featureName isEqualToString:@"String"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.String];
    }
    if ([featureName isEqualToString:@"Float"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Float];
    }
    if ([featureName isEqualToString:@"Double"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Double];
    }
    if ([featureName isEqualToString:@"Array"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Array];
    }
    if ([featureName isEqualToString:@"Dictionary"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Dictionary];
    }
    if ([featureName isEqualToString:@"Set"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Set];
    }
    if ([featureName isEqualToString:@"Optional"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Optional];
    }
    if ([featureName isEqualToString:@"Error"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Error];
    }
    if ([featureName isEqualToString:@"URL"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.URL];
    }
    if ([featureName isEqualToString:@"MLModel"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.MLModel];
    }
    if ([featureName isEqualToString:@"MLMultiArray"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.MLMultiArray];
    }
    if ([featureName isEqualToString:@"CVPixelBuffer"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.CVPixelBuffer];
    }
    if ([featureName isEqualToString:@"Bundle"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Bundle];
    }
    if ([featureName isEqualToString:@"result"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.result];
    }
    if ([featureName isEqualToString:@"results"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.results];
    }
    if ([featureName isEqualToString:@"options"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.options];
    }
    if ([featureName isEqualToString:@"configuration"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.configuration];
    }
    if ([featureName isEqualToString:@"handler"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.handler];
    }
    if ([featureName isEqualToString:@"error"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.error_];
    }
    if ([featureName isEqualToString:@"batchIn"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.batchIn];
    }
    if ([featureName isEqualToString:@"outFeatures"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.outFeatures];
    }
    if ([featureName isEqualToString:@"localError"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.localError];
    }
    if ([featureName isEqualToString:@"retVal"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.retVal];
    }
    if ([featureName isEqualToString:@"label"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.label];
    }
    if ([featureName isEqualToString:@"prediction"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.prediction];
    }
    if ([featureName isEqualToString:@"predictions"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.predictions];
    }
    if ([featureName isEqualToString:@"urlOfModelInThisBundle"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.urlOfModelInThisBundle];
    }
    if ([featureName isEqualToString:@"load"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.load];
    }
    if ([featureName isEqualToString:@"bundle"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.bundle];
    }
    if ([featureName isEqualToString:@"precedencegroup"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.precedencegroup];
    }
    if ([featureName isEqualToString:@"_Bool"]) {
        return [MLFeatureValue featureValueWithMultiArray:self._Bool_];
    }
    if ([featureName isEqualToString:@"static_assert"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.static_assert];
    }
    if ([featureName isEqualToString:@"alignas"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.alignas];
    }
    if ([featureName isEqualToString:@"alignof"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.alignof];
    }
    if ([featureName isEqualToString:@"decltype"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.decltype];
    }
    if ([featureName isEqualToString:@"noexcept"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.noexcept];
    }
    if ([featureName isEqualToString:@"nullptr"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.nullptr];
    }
    if ([featureName isEqualToString:@"thread_local"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.thread_local];
    }
    if ([featureName isEqualToString:@"constexpr"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.constexpr];
    }
    if ([featureName isEqualToString:@"char16_t"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.char16_t];
    }
    if ([featureName isEqualToString:@"char32_t"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.char32_t];
    }
    if ([featureName isEqualToString:@"wchar_t"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.wchar_t];
    }
    if ([featureName isEqualToString:@"dynamicType"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.dynamicType_];
    }
    if ([featureName isEqualToString:@"Category"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Category];
    }
    if ([featureName isEqualToString:@"Method"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Method];
    }
    if ([featureName isEqualToString:@"Ivar"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Ivar];
    }
    if ([featureName isEqualToString:@"unknown"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.unknown];
    }
    if ([featureName isEqualToString:@"and"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.and];
    }
    if ([featureName isEqualToString:@"or"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.or];
    }
    if ([featureName isEqualToString:@"not"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.not];
    }
    if ([featureName isEqualToString:@"xor"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.xor];
    }
    if ([featureName isEqualToString:@"bitand"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.bitand];
    }
    if ([featureName isEqualToString:@"bitor"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.bitor];
    }
    if ([featureName isEqualToString:@"compl"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.compl];
    }
    if ([featureName isEqualToString:@"not_eq"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.not_eq];
    }
    if ([featureName isEqualToString:@"or_eq"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.or_eq];
    }
    if ([featureName isEqualToString:@"and_eq"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.and_eq];
    }
    if ([featureName isEqualToString:@"xor_eq"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.xor_eq];
    }
    if ([featureName isEqualToString:@"typeid"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.typeid];
    }
    if ([featureName isEqualToString:@"typename"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.typename];
    }
    if ([featureName isEqualToString:@"using"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.using];
    }
    if ([featureName isEqualToString:@"public_"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.public_];
    }
    if ([featureName isEqualToString:@"throws_"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.throws_];
    }
    if ([featureName isEqualToString:@"mutating_"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.mutating_];
    }
    if ([featureName isEqualToString:@"Int32"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Int32];
    }
    if ([featureName isEqualToString:@"Int64"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Int64];
    }
    if ([featureName isEqualToString:@"UInt"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.UInt];
    }
    if ([featureName isEqualToString:@"NSObject"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.NSObject];
    }
    if ([featureName isEqualToString:@"NSString"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.NSString];
    }
    if ([featureName isEqualToString:@"NSNumber"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.NSNumber];
    }
    if ([featureName isEqualToString:@"NSError"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.NSError];
    }
    if ([featureName isEqualToString:@"instancetype"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.instancetype];
    }
    if ([featureName isEqualToString:@"strong"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.strong];
    }
    if ([featureName isEqualToString:@"copy"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.copy];
    }
    return nil;
}

@end

@implementation Names2


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"Names2" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load Names2.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize Names2 instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of Names2.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model {
    if (model == nil) {
        return nil;
    }
    self = [super init];
    if (self != nil) {
        _model = model;
    }
    return self;
}


/**
    Initialize Names2 instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize Names2 instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize Names2 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Names2.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize Names2 instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Names2.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct Names2 instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Names2 instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Names2 * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct Names2 instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Names2 instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Names2 * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            Names2 *typedModel = [[Names2 alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable Names2Output *)predictionFromFeatures:(Names2Input *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable Names2Output *)predictionFromFeatures:(Names2Input *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[Names2Output alloc] initWithCatch:(MLMultiArray *)[outFeatures featureValueForName:@"catch"].multiArrayValue package:(MLMultiArray *)[outFeatures featureValueForName:@"package"].multiArrayValue macro:(MLMultiArray *)[outFeatures featureValueForName:@"macro"].multiArrayValue borrowing:(MLMultiArray *)[outFeatures featureValueForName:@"borrowing"].multiArrayValue consuming:(MLMultiArray *)[outFeatures featureValueForName:@"consuming"].multiArrayValue nonisolated:(MLMultiArray *)[outFeatures featureValueForName:@"nonisolated"].multiArrayValue isolated:(MLMultiArray *)[outFeatures featureValueForName:@"isolated"].multiArrayValue actor:(MLMultiArray *)[outFeatures featureValueForName:@"actor"].multiArrayValue each:(MLMultiArray *)[outFeatures featureValueForName:@"each"].multiArrayValue asm:(MLMultiArray *)[outFeatures featureValueForName:@"asm"].multiArrayValue typeof:(MLMultiArray *)[outFeatures featureValueForName:@"typeof"].multiArrayValue nullable:(MLMultiArray *)[outFeatures featureValueForName:@"nullable"].multiArrayValue nonnull:(MLMultiArray *)[outFeatures featureValueForName:@"nonnull"].multiArrayValue property:(MLMultiArray *)[outFeatures featureValueForName:@"property"].multiArrayValue synthesize:(MLMultiArray *)[outFeatures featureValueForName:@"synthesize"].multiArrayValue interface:(MLMultiArray *)[outFeatures featureValueForName:@"interface"].multiArrayValue implementation:(MLMultiArray *)[outFeatures featureValueForName:@"implementation"].multiArrayValue end:(MLMultiArray *)[outFeatures featureValueForName:@"end"].multiArrayValue selector:(MLMultiArray *)[outFeatures featureValueForName:@"selector"].multiArrayValue encode:(MLMultiArray *)[outFeatures featureValueForName:@"encode"].multiArrayValue autoreleasepool:(MLMultiArray *)[outFeatures featureValueForName:@"autoreleasepool"].multiArrayValue finally:(MLMultiArray *)[outFeatures featureValueForName:@"finally"].multiArrayValue Int:(MLMultiArray *)[outFeatures featureValueForName:@"Int"].multiArrayValue String:(MLMultiArray *)[outFeatures featureValueForName:@"String"].multiArrayValue Float:(MLMultiArray *)[outFeatures featureValueForName:@"Float"].multiArrayValue Double:(MLMultiArray *)[outFeatures featureValueForName:@"Double"].multiArrayValue Array:(MLMultiArray *)[outFeatures featureValueForName:@"Array"].multiArrayValue Dictionary:(MLMultiArray *)[outFeatures featureValueForName:@"Dictionary"].multiArrayValue Set:(MLMultiArray *)[outFeatures featureValueForName:@"Set"].multiArrayValue Optional:(MLMultiArray *)[outFeatures featureValueForName:@"Optional"].multiArrayValue Error:(MLMultiArray *)[outFeatures featureValueForName:@"Error"].multiArrayValue URL:(MLMultiArray *)[outFeatures featureValueForName:@"URL"].multiArrayValue MLModel:(MLMultiArray *)[outFeatures featureValueForName:@"MLModel"].multiArrayValue MLMultiArray:(MLMultiArray *)[outFeatures featureValueForName:@"MLMultiArray"].multiArrayValue CVPixelBuffer:(MLMultiArray *)[outFeatures featureValueForName:@"CVPixelBuffer"].multiArrayValue Bundle:(MLMultiArray *)[outFeatures featureValueForName:@"Bundle"].multiArrayValue result:(MLMultiArray *)[outFeatures featureValueForName:@"result"].multiArrayValue results:(MLMultiArray *)[outFeatures featureValueForName:@"results"].multiArrayValue options:(MLMultiArray *)[outFeatures featureValueForName:@"options"].multiArrayValue configuration:(MLMultiArray *)[outFeatures featureValueForName:@"configuration"].multiArrayValue handler:(MLMultiArray *)[outFeatures featureValueForName:@"handler"].multiArrayValue error_:(MLMultiArray *)[outFeatures featureValueForName:@"error"].multiArrayValue batchIn:(MLMultiArray *)[outFeatures featureValueForName:@"batchIn"].multiArrayValue outFeatures:(MLMultiArray *)[outFeatures featureValueForName:@"outFeatures"].multiArrayValue localError:(MLMultiArray *)[outFeatures featureValueForName:@"localError"].multiArrayValue retVal:(MLMultiArray *)[outFeatures featureValueForName:@"retVal"].multiArrayValue label:(MLMultiArray *)[outFeatures featureValueForName:@"label"].multiArrayValue prediction:(MLMultiArray *)[outFeatures featureValueForName:@"prediction"].multiArrayValue predictions:(MLMultiArray *)[outFeatures featureValueForName:@"predictions"].multiArrayValue urlOfModelInThisBundle:(MLMultiArray *)[outFeatures featureValueForName:@"urlOfModelInThisBundle"].multiArrayValue load:(MLMultiArray *)[outFeatures featureValueForName:@"load"].multiArrayValue bundle:(MLMultiArray *)[outFeatures featureValueForName:@"bundle"].multiArrayValue precedencegroup:(MLMultiArray *)[outFeatures featureValueForName:@"precedencegroup"].multiArrayValue _Bool_:(MLMultiArray *)[outFeatures featureValueForName:@"_Bool"].multiArrayValue static_assert:(MLMultiArray *)[outFeatures featureValueForName:@"static_assert"].multiArrayValue alignas:(MLMultiArray *)[outFeatures featureValueForName:@"alignas"].multiArrayValue alignof:(MLMultiArray *)[outFeatures featureValueForName:@"alignof"].multiArrayValue decltype:(MLMultiArray *)[outFeatures featureValueForName:@"decltype"].multiArrayValue noexcept:(MLMultiArray *)[outFeatures featureValueForName:@"noexcept"].multiArrayValue nullptr:(MLMultiArray *)[outFeatures featureValueForName:@"nullptr"].multiArrayValue thread_local:(MLMultiArray *)[outFeatures featureValueForName:@"thread_local"].multiArrayValue constexpr:(MLMultiArray *)[outFeatures featureValueForName:@"constexpr"].multiArrayValue char16_t:(MLMultiArray *)[outFeatures featureValueForName:@"char16_t"].multiArrayValue char32_t:(MLMultiArray *)[outFeatures featureValueForName:@"char32_t"].multiArrayValue wchar_t:(MLMultiArray *)[outFeatures featureValueForName:@"wchar_t"].multiArrayValue dynamicType_:(MLMultiArray *)[outFeatures featureValueForName:@"dynamicType"].multiArrayValue Category:(MLMultiArray *)[outFeatures featureValueForName:@"Category"].multiArrayValue Method:(MLMultiArray *)[outFeatures featureValueForName:@"Method"].multiArrayValue Ivar:(MLMultiArray *)[outFeatures featureValueForName:@"Ivar"].multiArrayValue unknown:(MLMultiArray *)[outFeatures featureValueForName:@"unknown"].multiArrayValue and:(MLMultiArray *)[outFeatures featureValueForName:@"and"].multiArrayValue or:(MLMultiArray *)[outFeatures featureValueForName:@"or"].multiArrayValue not:(MLMultiArray *)[outFeatures featureValueForName:@"not"].multiArrayValue xor:(MLMultiArray *)[outFeatures featureValueForName:@"xor"].multiArrayValue bitand:(MLMultiArray *)[outFeatures featureValueForName:@"bitand"].multiArrayValue bitor:(MLMultiArray *)[outFeatures featureValueForName:@"bitor"].multiArrayValue compl:(MLMultiArray *)[outFeatures featureValueForName:@"compl"].multiArrayValue not_eq:(MLMultiArray *)[outFeatures featureValueForName:@"not_eq"].multiArrayValue or_eq:(MLMultiArray *)[outFeatures featureValueForName:@"or_eq"].multiArrayValue and_eq:(MLMultiArray *)[outFeatures featureValueForName:@"and_eq"].multiArrayValue xor_eq:(MLMultiArray *)[outFeatures featureValueForName:@"xor_eq"].multiArrayValue typeid:(MLMultiArray *)[outFeatures featureValueForName:@"typeid"].multiArrayValue typename:(MLMultiArray *)[outFeatures featureValueForName:@"typename"].multiArrayValue using:(MLMultiArray *)[outFeatures featureValueForName:@"using"].multiArrayValue public_:(MLMultiArray *)[outFeatures featureValueForName:@"public_"].multiArrayValue throws_:(MLMultiArray *)[outFeatures featureValueForName:@"throws_"].multiArrayValue mutating_:(MLMultiArray *)[outFeatures featureValueForName:@"mutating_"].multiArrayValue Int32:(MLMultiArray *)[outFeatures featureValueForName:@"Int32"].multiArrayValue Int64:(MLMultiArray *)[outFeatures featureValueForName:@"Int64"].multiArrayValue UInt:(MLMultiArray *)[outFeatures featureValueForName:@"UInt"].multiArrayValue NSObject:(MLMultiArray *)[outFeatures featureValueForName:@"NSObject"].multiArrayValue NSString:(MLMultiArray *)[outFeatures featureValueForName:@"NSString"].multiArrayValue NSNumber:(MLMultiArray *)[outFeatures featureValueForName:@"NSNumber"].multiArrayValue NSError:(MLMultiArray *)[outFeatures featureValueForName:@"NSError"].multiArrayValue instancetype:(MLMultiArray *)[outFeatures featureValueForName:@"instancetype"].multiArrayValue strong:(MLMultiArray *)[outFeatures featureValueForName:@"strong"].multiArrayValue copy:(MLMultiArray *)[outFeatures featureValueForName:@"copy"].multiArrayValue];
}

- (void)predictionFromFeatures:(Names2Input *)input completionHandler:(void (^)(Names2Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            Names2Output *output = [[Names2Output alloc] initWithCatch:(MLMultiArray *)[prediction featureValueForName:@"catch"].multiArrayValue package:(MLMultiArray *)[prediction featureValueForName:@"package"].multiArrayValue macro:(MLMultiArray *)[prediction featureValueForName:@"macro"].multiArrayValue borrowing:(MLMultiArray *)[prediction featureValueForName:@"borrowing"].multiArrayValue consuming:(MLMultiArray *)[prediction featureValueForName:@"consuming"].multiArrayValue nonisolated:(MLMultiArray *)[prediction featureValueForName:@"nonisolated"].multiArrayValue isolated:(MLMultiArray *)[prediction featureValueForName:@"isolated"].multiArrayValue actor:(MLMultiArray *)[prediction featureValueForName:@"actor"].multiArrayValue each:(MLMultiArray *)[prediction featureValueForName:@"each"].multiArrayValue asm:(MLMultiArray *)[prediction featureValueForName:@"asm"].multiArrayValue typeof:(MLMultiArray *)[prediction featureValueForName:@"typeof"].multiArrayValue nullable:(MLMultiArray *)[prediction featureValueForName:@"nullable"].multiArrayValue nonnull:(MLMultiArray *)[prediction featureValueForName:@"nonnull"].multiArrayValue property:(MLMultiArray *)[prediction featureValueForName:@"property"].multiArrayValue synthesize:(MLMultiArray *)[prediction featureValueForName:@"synthesize"].multiArrayValue interface:(MLMultiArray *)[prediction featureValueForName:@"interface"].multiArrayValue implementation:(MLMultiArray *)[prediction featureValueForName:@"implementation"].multiArrayValue end:(MLMultiArray *)[prediction featureValueForName:@"end"].multiArrayValue selector:(MLMultiArray *)[prediction featureValueForName:@"selector"].multiArrayValue encode:(MLMultiArray *)[prediction featureValueForName:@"encode"].multiArrayValue autoreleasepool:(MLMultiArray *)[prediction featureValueForName:@"autoreleasepool"].multiArrayValue finally:(MLMultiArray *)[prediction featureValueForName:@"finally"].multiArrayValue Int:(MLMultiArray *)[prediction featureValueForName:@"Int"].multiArrayValue String:(MLMultiArray *)[prediction featureValueForName:@"String"].multiArrayValue Float:(MLMultiArray *)[prediction featureValueForName:@"Float"].multiArrayValue Double:(MLMultiArray *)[prediction featureValueForName:@"Double"].multiArrayValue Array:(MLMultiArray *)[prediction featureValueForName:@"Array"].multiArrayValue Dictionary:(MLMultiArray *)[prediction featureValueForName:@"Dictionary"].multiArrayValue Set:(MLMultiArray *)[prediction featureValueForName:@"Set"].multiArrayValue Optional:(MLMultiArray *)[prediction featureValueForName:@"Optional"].multiArrayValue Error:(MLMultiArray *)[prediction featureValueForName:@"Error"].multiArrayValue URL:(MLMultiArray *)[prediction featureValueForName:@"URL"].multiArrayValue MLModel:(MLMultiArray *)[prediction featureValueForName:@"MLModel"].multiArrayValue MLMultiArray:(MLMultiArray *)[prediction featureValueForName:@"MLMultiArray"].multiArrayValue CVPixelBuffer:(MLMultiArray *)[prediction featureValueForName:@"CVPixelBuffer"].multiArrayValue Bundle:(MLMultiArray *)[prediction featureValueForName:@"Bundle"].multiArrayValue result:(MLMultiArray *)[prediction featureValueForName:@"result"].multiArrayValue results:(MLMultiArray *)[prediction featureValueForName:@"results"].multiArrayValue options:(MLMultiArray *)[prediction featureValueForName:@"options"].multiArrayValue configuration:(MLMultiArray *)[prediction featureValueForName:@"configuration"].multiArrayValue handler:(MLMultiArray *)[prediction featureValueForName:@"handler"].multiArrayValue error_:(MLMultiArray *)[prediction featureValueForName:@"error"].multiArrayValue batchIn:(MLMultiArray *)[prediction featureValueForName:@"batchIn"].multiArrayValue outFeatures:(MLMultiArray *)[prediction featureValueForName:@"outFeatures"].multiArrayValue localError:(MLMultiArray *)[prediction featureValueForName:@"localError"].multiArrayValue retVal:(MLMultiArray *)[prediction featureValueForName:@"retVal"].multiArrayValue label:(MLMultiArray *)[prediction featureValueForName:@"label"].multiArrayValue prediction:(MLMultiArray *)[prediction featureValueForName:@"prediction"].multiArrayValue predictions:(MLMultiArray *)[prediction featureValueForName:@"predictions"].multiArrayValue urlOfModelInThisBundle:(MLMultiArray *)[prediction featureValueForName:@"urlOfModelInThisBundle"].multiArrayValue load:(MLMultiArray *)[prediction featureValueForName:@"load"].multiArrayValue bundle:(MLMultiArray *)[prediction featureValueForName:@"bundle"].multiArrayValue precedencegroup:(MLMultiArray *)[prediction featureValueForName:@"precedencegroup"].multiArrayValue _Bool_:(MLMultiArray *)[prediction featureValueForName:@"_Bool"].multiArrayValue static_assert:(MLMultiArray *)[prediction featureValueForName:@"static_assert"].multiArrayValue alignas:(MLMultiArray *)[prediction featureValueForName:@"alignas"].multiArrayValue alignof:(MLMultiArray *)[prediction featureValueForName:@"alignof"].multiArrayValue decltype:(MLMultiArray *)[prediction featureValueForName:@"decltype"].multiArrayValue noexcept:(MLMultiArray *)[prediction featureValueForName:@"noexcept"].multiArrayValue nullptr:(MLMultiArray *)[prediction featureValueForName:@"nullptr"].multiArrayValue thread_local:(MLMultiArray *)[prediction featureValueForName:@"thread_local"].multiArrayValue constexpr:(MLMultiArray *)[prediction featureValueForName:@"constexpr"].multiArrayValue char16_t:(MLMultiArray *)[prediction featureValueForName:@"char16_t"].multiArrayValue char32_t:(MLMultiArray *)[prediction featureValueForName:@"char32_t"].multiArrayValue wchar_t:(MLMultiArray *)[prediction featureValueForName:@"wchar_t"].multiArrayValue dynamicType_:(MLMultiArray *)[prediction featureValueForName:@"dynamicType"].multiArrayValue Category:(MLMultiArray *)[prediction featureValueForName:@"Category"].multiArrayValue Method:(MLMultiArray *)[prediction featureValueForName:@"Method"].multiArrayValue Ivar:(MLMultiArray *)[prediction featureValueForName:@"Ivar"].multiArrayValue unknown:(MLMultiArray *)[prediction featureValueForName:@"unknown"].multiArrayValue and:(MLMultiArray *)[prediction featureValueForName:@"and"].multiArrayValue or:(MLMultiArray *)[prediction featureValueForName:@"or"].multiArrayValue not:(MLMultiArray *)[prediction featureValueForName:@"not"].multiArrayValue xor:(MLMultiArray *)[prediction featureValueForName:@"xor"].multiArrayValue bitand:(MLMultiArray *)[prediction featureValueForName:@"bitand"].multiArrayValue bitor:(MLMultiArray *)[prediction featureValueForName:@"bitor"].multiArrayValue compl:(MLMultiArray *)[prediction featureValueForName:@"compl"].multiArrayValue not_eq:(MLMultiArray *)[prediction featureValueForName:@"not_eq"].multiArrayValue or_eq:(MLMultiArray *)[prediction featureValueForName:@"or_eq"].multiArrayValue and_eq:(MLMultiArray *)[prediction featureValueForName:@"and_eq"].multiArrayValue xor_eq:(MLMultiArray *)[prediction featureValueForName:@"xor_eq"].multiArrayValue typeid:(MLMultiArray *)[prediction featureValueForName:@"typeid"].multiArrayValue typename:(MLMultiArray *)[prediction featureValueForName:@"typename"].multiArrayValue using:(MLMultiArray *)[prediction featureValueForName:@"using"].multiArrayValue public_:(MLMultiArray *)[prediction featureValueForName:@"public_"].multiArrayValue throws_:(MLMultiArray *)[prediction featureValueForName:@"throws_"].multiArrayValue mutating_:(MLMultiArray *)[prediction featureValueForName:@"mutating_"].multiArrayValue Int32:(MLMultiArray *)[prediction featureValueForName:@"Int32"].multiArrayValue Int64:(MLMultiArray *)[prediction featureValueForName:@"Int64"].multiArrayValue UInt:(MLMultiArray *)[prediction featureValueForName:@"UInt"].multiArrayValue NSObject:(MLMultiArray *)[prediction featureValueForName:@"NSObject"].multiArrayValue NSString:(MLMultiArray *)[prediction featureValueForName:@"NSString"].multiArrayValue NSNumber:(MLMultiArray *)[prediction featureValueForName:@"NSNumber"].multiArrayValue NSError:(MLMultiArray *)[prediction featureValueForName:@"NSError"].multiArrayValue instancetype:(MLMultiArray *)[prediction featureValueForName:@"instancetype"].multiArrayValue strong:(MLMultiArray *)[prediction featureValueForName:@"strong"].multiArrayValue copy:(MLMultiArray *)[prediction featureValueForName:@"copy"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(Names2Input *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(Names2Output * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            Names2Output *output = [[Names2Output alloc] initWithCatch:(MLMultiArray *)[prediction featureValueForName:@"catch"].multiArrayValue package:(MLMultiArray *)[prediction featureValueForName:@"package"].multiArrayValue macro:(MLMultiArray *)[prediction featureValueForName:@"macro"].multiArrayValue borrowing:(MLMultiArray *)[prediction featureValueForName:@"borrowing"].multiArrayValue consuming:(MLMultiArray *)[prediction featureValueForName:@"consuming"].multiArrayValue nonisolated:(MLMultiArray *)[prediction featureValueForName:@"nonisolated"].multiArrayValue isolated:(MLMultiArray *)[prediction featureValueForName:@"isolated"].multiArrayValue actor:(MLMultiArray *)[prediction featureValueForName:@"actor"].multiArrayValue each:(MLMultiArray *)[prediction featureValueForName:@"each"].multiArrayValue asm:(MLMultiArray *)[prediction featureValueForName:@"asm"].multiArrayValue typeof:(MLMultiArray *)[prediction featureValueForName:@"typeof"].multiArrayValue nullable:(MLMultiArray *)[prediction featureValueForName:@"nullable"].multiArrayValue nonnull:(MLMultiArray *)[prediction featureValueForName:@"nonnull"].multiArrayValue property:(MLMultiArray *)[prediction featureValueForName:@"property"].multiArrayValue synthesize:(MLMultiArray *)[prediction featureValueForName:@"synthesize"].multiArrayValue interface:(MLMultiArray *)[prediction featureValueForName:@"interface"].multiArrayValue implementation:(MLMultiArray *)[prediction featureValueForName:@"implementation"].multiArrayValue end:(MLMultiArray *)[prediction featureValueForName:@"end"].multiArrayValue selector:(MLMultiArray *)[prediction featureValueForName:@"selector"].multiArrayValue encode:(MLMultiArray *)[prediction featureValueForName:@"encode"].multiArrayValue autoreleasepool:(MLMultiArray *)[prediction featureValueForName:@"autoreleasepool"].multiArrayValue finally:(MLMultiArray *)[prediction featureValueForName:@"finally"].multiArrayValue Int:(MLMultiArray *)[prediction featureValueForName:@"Int"].multiArrayValue String:(MLMultiArray *)[prediction featureValueForName:@"String"].multiArrayValue Float:(MLMultiArray *)[prediction featureValueForName:@"Float"].multiArrayValue Double:(MLMultiArray *)[prediction featureValueForName:@"Double"].multiArrayValue Array:(MLMultiArray *)[prediction featureValueForName:@"Array"].multiArrayValue Dictionary:(MLMultiArray *)[prediction featureValueForName:@"Dictionary"].multiArrayValue Set:(MLMultiArray *)[prediction featureValueForName:@"Set"].multiArrayValue Optional:(MLMultiArray *)[prediction featureValueForName:@"Optional"].multiArrayValue Error:(MLMultiArray *)[prediction featureValueForName:@"Error"].multiArrayValue URL:(MLMultiArray *)[prediction featureValueForName:@"URL"].multiArrayValue MLModel:(MLMultiArray *)[prediction featureValueForName:@"MLModel"].multiArrayValue MLMultiArray:(MLMultiArray *)[prediction featureValueForName:@"MLMultiArray"].multiArrayValue CVPixelBuffer:(MLMultiArray *)[prediction featureValueForName:@"CVPixelBuffer"].multiArrayValue Bundle:(MLMultiArray *)[prediction featureValueForName:@"Bundle"].multiArrayValue result:(MLMultiArray *)[prediction featureValueForName:@"result"].multiArrayValue results:(MLMultiArray *)[prediction featureValueForName:@"results"].multiArrayValue options:(MLMultiArray *)[prediction featureValueForName:@"options"].multiArrayValue configuration:(MLMultiArray *)[prediction featureValueForName:@"configuration"].multiArrayValue handler:(MLMultiArray *)[prediction featureValueForName:@"handler"].multiArrayValue error_:(MLMultiArray *)[prediction featureValueForName:@"error"].multiArrayValue batchIn:(MLMultiArray *)[prediction featureValueForName:@"batchIn"].multiArrayValue outFeatures:(MLMultiArray *)[prediction featureValueForName:@"outFeatures"].multiArrayValue localError:(MLMultiArray *)[prediction featureValueForName:@"localError"].multiArrayValue retVal:(MLMultiArray *)[prediction featureValueForName:@"retVal"].multiArrayValue label:(MLMultiArray *)[prediction featureValueForName:@"label"].multiArrayValue prediction:(MLMultiArray *)[prediction featureValueForName:@"prediction"].multiArrayValue predictions:(MLMultiArray *)[prediction featureValueForName:@"predictions"].multiArrayValue urlOfModelInThisBundle:(MLMultiArray *)[prediction featureValueForName:@"urlOfModelInThisBundle"].multiArrayValue load:(MLMultiArray *)[prediction featureValueForName:@"load"].multiArrayValue bundle:(MLMultiArray *)[prediction featureValueForName:@"bundle"].multiArrayValue precedencegroup:(MLMultiArray *)[prediction featureValueForName:@"precedencegroup"].multiArrayValue _Bool_:(MLMultiArray *)[prediction featureValueForName:@"_Bool"].multiArrayValue static_assert:(MLMultiArray *)[prediction featureValueForName:@"static_assert"].multiArrayValue alignas:(MLMultiArray *)[prediction featureValueForName:@"alignas"].multiArrayValue alignof:(MLMultiArray *)[prediction featureValueForName:@"alignof"].multiArrayValue decltype:(MLMultiArray *)[prediction featureValueForName:@"decltype"].multiArrayValue noexcept:(MLMultiArray *)[prediction featureValueForName:@"noexcept"].multiArrayValue nullptr:(MLMultiArray *)[prediction featureValueForName:@"nullptr"].multiArrayValue thread_local:(MLMultiArray *)[prediction featureValueForName:@"thread_local"].multiArrayValue constexpr:(MLMultiArray *)[prediction featureValueForName:@"constexpr"].multiArrayValue char16_t:(MLMultiArray *)[prediction featureValueForName:@"char16_t"].multiArrayValue char32_t:(MLMultiArray *)[prediction featureValueForName:@"char32_t"].multiArrayValue wchar_t:(MLMultiArray *)[prediction featureValueForName:@"wchar_t"].multiArrayValue dynamicType_:(MLMultiArray *)[prediction featureValueForName:@"dynamicType"].multiArrayValue Category:(MLMultiArray *)[prediction featureValueForName:@"Category"].multiArrayValue Method:(MLMultiArray *)[prediction featureValueForName:@"Method"].multiArrayValue Ivar:(MLMultiArray *)[prediction featureValueForName:@"Ivar"].multiArrayValue unknown:(MLMultiArray *)[prediction featureValueForName:@"unknown"].multiArrayValue and:(MLMultiArray *)[prediction featureValueForName:@"and"].multiArrayValue or:(MLMultiArray *)[prediction featureValueForName:@"or"].multiArrayValue not:(MLMultiArray *)[prediction featureValueForName:@"not"].multiArrayValue xor:(MLMultiArray *)[prediction featureValueForName:@"xor"].multiArrayValue bitand:(MLMultiArray *)[prediction featureValueForName:@"bitand"].multiArrayValue bitor:(MLMultiArray *)[prediction featureValueForName:@"bitor"].multiArrayValue compl:(MLMultiArray *)[prediction featureValueForName:@"compl"].multiArrayValue not_eq:(MLMultiArray *)[prediction featureValueForName:@"not_eq"].multiArrayValue or_eq:(MLMultiArray *)[prediction featureValueForName:@"or_eq"].multiArrayValue and_eq:(MLMultiArray *)[prediction featureValueForName:@"and_eq"].multiArrayValue xor_eq:(MLMultiArray *)[prediction featureValueForName:@"xor_eq"].multiArrayValue typeid:(MLMultiArray *)[prediction featureValueForName:@"typeid"].multiArrayValue typename:(MLMultiArray *)[prediction featureValueForName:@"typename"].multiArrayValue using:(MLMultiArray *)[prediction featureValueForName:@"using"].multiArrayValue public_:(MLMultiArray *)[prediction featureValueForName:@"public_"].multiArrayValue throws_:(MLMultiArray *)[prediction featureValueForName:@"throws_"].multiArrayValue mutating_:(MLMultiArray *)[prediction featureValueForName:@"mutating_"].multiArrayValue Int32:(MLMultiArray *)[prediction featureValueForName:@"Int32"].multiArrayValue Int64:(MLMultiArray *)[prediction featureValueForName:@"Int64"].multiArrayValue UInt:(MLMultiArray *)[prediction featureValueForName:@"UInt"].multiArrayValue NSObject:(MLMultiArray *)[prediction featureValueForName:@"NSObject"].multiArrayValue NSString:(MLMultiArray *)[prediction featureValueForName:@"NSString"].multiArrayValue NSNumber:(MLMultiArray *)[prediction featureValueForName:@"NSNumber"].multiArrayValue NSError:(MLMultiArray *)[prediction featureValueForName:@"NSError"].multiArrayValue instancetype:(MLMultiArray *)[prediction featureValueForName:@"instancetype"].multiArrayValue strong:(MLMultiArray *)[prediction featureValueForName:@"strong"].multiArrayValue copy:(MLMultiArray *)[prediction featureValueForName:@"copy"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable Names2Output *)predictionFromX:(MLMultiArray *)x error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    Names2Input *input_ = [[Names2Input alloc] initWithX:x];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<Names2Output *> *)predictionsFromInputs:(NSArray<Names2Input*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<Names2Output*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        Names2Output * result = [[Names2Output alloc] initWithCatch:(MLMultiArray *)[resultProvider featureValueForName:@"catch"].multiArrayValue package:(MLMultiArray *)[resultProvider featureValueForName:@"package"].multiArrayValue macro:(MLMultiArray *)[resultProvider featureValueForName:@"macro"].multiArrayValue borrowing:(MLMultiArray *)[resultProvider featureValueForName:@"borrowing"].multiArrayValue consuming:(MLMultiArray *)[resultProvider featureValueForName:@"consuming"].multiArrayValue nonisolated:(MLMultiArray *)[resultProvider featureValueForName:@"nonisolated"].multiArrayValue isolated:(MLMultiArray *)[resultProvider featureValueForName:@"isolated"].multiArrayValue actor:(MLMultiArray *)[resultProvider featureValueForName:@"actor"].multiArrayValue each:(MLMultiArray *)[resultProvider featureValueForName:@"each"].multiArrayValue asm:(MLMultiArray *)[resultProvider featureValueForName:@"asm"].multiArrayValue typeof:(MLMultiArray *)[resultProvider featureValueForName:@"typeof"].multiArrayValue nullable:(MLMultiArray *)[resultProvider featureValueForName:@"nullable"].multiArrayValue nonnull:(MLMultiArray *)[resultProvider featureValueForName:@"nonnull"].multiArrayValue property:(MLMultiArray *)[resultProvider featureValueForName:@"property"].multiArrayValue synthesize:(MLMultiArray *)[resultProvider featureValueForName:@"synthesize"].multiArrayValue interface:(MLMultiArray *)[resultProvider featureValueForName:@"interface"].multiArrayValue implementation:(MLMultiArray *)[resultProvider featureValueForName:@"implementation"].multiArrayValue end:(MLMultiArray *)[resultProvider featureValueForName:@"end"].multiArrayValue selector:(MLMultiArray *)[resultProvider featureValueForName:@"selector"].multiArrayValue encode:(MLMultiArray *)[resultProvider featureValueForName:@"encode"].multiArrayValue autoreleasepool:(MLMultiArray *)[resultProvider featureValueForName:@"autoreleasepool"].multiArrayValue finally:(MLMultiArray *)[resultProvider featureValueForName:@"finally"].multiArrayValue Int:(MLMultiArray *)[resultProvider featureValueForName:@"Int"].multiArrayValue String:(MLMultiArray *)[resultProvider featureValueForName:@"String"].multiArrayValue Float:(MLMultiArray *)[resultProvider featureValueForName:@"Float"].multiArrayValue Double:(MLMultiArray *)[resultProvider featureValueForName:@"Double"].multiArrayValue Array:(MLMultiArray *)[resultProvider featureValueForName:@"Array"].multiArrayValue Dictionary:(MLMultiArray *)[resultProvider featureValueForName:@"Dictionary"].multiArrayValue Set:(MLMultiArray *)[resultProvider featureValueForName:@"Set"].multiArrayValue Optional:(MLMultiArray *)[resultProvider featureValueForName:@"Optional"].multiArrayValue Error:(MLMultiArray *)[resultProvider featureValueForName:@"Error"].multiArrayValue URL:(MLMultiArray *)[resultProvider featureValueForName:@"URL"].multiArrayValue MLModel:(MLMultiArray *)[resultProvider featureValueForName:@"MLModel"].multiArrayValue MLMultiArray:(MLMultiArray *)[resultProvider featureValueForName:@"MLMultiArray"].multiArrayValue CVPixelBuffer:(MLMultiArray *)[resultProvider featureValueForName:@"CVPixelBuffer"].multiArrayValue Bundle:(MLMultiArray *)[resultProvider featureValueForName:@"Bundle"].multiArrayValue result:(MLMultiArray *)[resultProvider featureValueForName:@"result"].multiArrayValue results:(MLMultiArray *)[resultProvider featureValueForName:@"results"].multiArrayValue options:(MLMultiArray *)[resultProvider featureValueForName:@"options"].multiArrayValue configuration:(MLMultiArray *)[resultProvider featureValueForName:@"configuration"].multiArrayValue handler:(MLMultiArray *)[resultProvider featureValueForName:@"handler"].multiArrayValue error_:(MLMultiArray *)[resultProvider featureValueForName:@"error"].multiArrayValue batchIn:(MLMultiArray *)[resultProvider featureValueForName:@"batchIn"].multiArrayValue outFeatures:(MLMultiArray *)[resultProvider featureValueForName:@"outFeatures"].multiArrayValue localError:(MLMultiArray *)[resultProvider featureValueForName:@"localError"].multiArrayValue retVal:(MLMultiArray *)[resultProvider featureValueForName:@"retVal"].multiArrayValue label:(MLMultiArray *)[resultProvider featureValueForName:@"label"].multiArrayValue prediction:(MLMultiArray *)[resultProvider featureValueForName:@"prediction"].multiArrayValue predictions:(MLMultiArray *)[resultProvider featureValueForName:@"predictions"].multiArrayValue urlOfModelInThisBundle:(MLMultiArray *)[resultProvider featureValueForName:@"urlOfModelInThisBundle"].multiArrayValue load:(MLMultiArray *)[resultProvider featureValueForName:@"load"].multiArrayValue bundle:(MLMultiArray *)[resultProvider featureValueForName:@"bundle"].multiArrayValue precedencegroup:(MLMultiArray *)[resultProvider featureValueForName:@"precedencegroup"].multiArrayValue _Bool_:(MLMultiArray *)[resultProvider featureValueForName:@"_Bool"].multiArrayValue static_assert:(MLMultiArray *)[resultProvider featureValueForName:@"static_assert"].multiArrayValue alignas:(MLMultiArray *)[resultProvider featureValueForName:@"alignas"].multiArrayValue alignof:(MLMultiArray *)[resultProvider featureValueForName:@"alignof"].multiArrayValue decltype:(MLMultiArray *)[resultProvider featureValueForName:@"decltype"].multiArrayValue noexcept:(MLMultiArray *)[resultProvider featureValueForName:@"noexcept"].multiArrayValue nullptr:(MLMultiArray *)[resultProvider featureValueForName:@"nullptr"].multiArrayValue thread_local:(MLMultiArray *)[resultProvider featureValueForName:@"thread_local"].multiArrayValue constexpr:(MLMultiArray *)[resultProvider featureValueForName:@"constexpr"].multiArrayValue char16_t:(MLMultiArray *)[resultProvider featureValueForName:@"char16_t"].multiArrayValue char32_t:(MLMultiArray *)[resultProvider featureValueForName:@"char32_t"].multiArrayValue wchar_t:(MLMultiArray *)[resultProvider featureValueForName:@"wchar_t"].multiArrayValue dynamicType_:(MLMultiArray *)[resultProvider featureValueForName:@"dynamicType"].multiArrayValue Category:(MLMultiArray *)[resultProvider featureValueForName:@"Category"].multiArrayValue Method:(MLMultiArray *)[resultProvider featureValueForName:@"Method"].multiArrayValue Ivar:(MLMultiArray *)[resultProvider featureValueForName:@"Ivar"].multiArrayValue unknown:(MLMultiArray *)[resultProvider featureValueForName:@"unknown"].multiArrayValue and:(MLMultiArray *)[resultProvider featureValueForName:@"and"].multiArrayValue or:(MLMultiArray *)[resultProvider featureValueForName:@"or"].multiArrayValue not:(MLMultiArray *)[resultProvider featureValueForName:@"not"].multiArrayValue xor:(MLMultiArray *)[resultProvider featureValueForName:@"xor"].multiArrayValue bitand:(MLMultiArray *)[resultProvider featureValueForName:@"bitand"].multiArrayValue bitor:(MLMultiArray *)[resultProvider featureValueForName:@"bitor"].multiArrayValue compl:(MLMultiArray *)[resultProvider featureValueForName:@"compl"].multiArrayValue not_eq:(MLMultiArray *)[resultProvider featureValueForName:@"not_eq"].multiArrayValue or_eq:(MLMultiArray *)[resultProvider featureValueForName:@"or_eq"].multiArrayValue and_eq:(MLMultiArray *)[resultProvider featureValueForName:@"and_eq"].multiArrayValue xor_eq:(MLMultiArray *)[resultProvider featureValueForName:@"xor_eq"].multiArrayValue typeid:(MLMultiArray *)[resultProvider featureValueForName:@"typeid"].multiArrayValue typename:(MLMultiArray *)[resultProvider featureValueForName:@"typename"].multiArrayValue using:(MLMultiArray *)[resultProvider featureValueForName:@"using"].multiArrayValue public_:(MLMultiArray *)[resultProvider featureValueForName:@"public_"].multiArrayValue throws_:(MLMultiArray *)[resultProvider featureValueForName:@"throws_"].multiArrayValue mutating_:(MLMultiArray *)[resultProvider featureValueForName:@"mutating_"].multiArrayValue Int32:(MLMultiArray *)[resultProvider featureValueForName:@"Int32"].multiArrayValue Int64:(MLMultiArray *)[resultProvider featureValueForName:@"Int64"].multiArrayValue UInt:(MLMultiArray *)[resultProvider featureValueForName:@"UInt"].multiArrayValue NSObject:(MLMultiArray *)[resultProvider featureValueForName:@"NSObject"].multiArrayValue NSString:(MLMultiArray *)[resultProvider featureValueForName:@"NSString"].multiArrayValue NSNumber:(MLMultiArray *)[resultProvider featureValueForName:@"NSNumber"].multiArrayValue NSError:(MLMultiArray *)[resultProvider featureValueForName:@"NSError"].multiArrayValue instancetype:(MLMultiArray *)[resultProvider featureValueForName:@"instancetype"].multiArrayValue strong:(MLMultiArray *)[resultProvider featureValueForName:@"strong"].multiArrayValue copy:(MLMultiArray *)[resultProvider featureValueForName:@"copy"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
